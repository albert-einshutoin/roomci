//! Report renderers for roomci run results.
//!
//! Each renderer consumes a [`RunReport`] from `roomci-core` and produces a
//! single string: JSON for machine consumption, Markdown for humans, and
//! JUnit XML for CI systems such as GitHub Actions test reporting.

use std::collections::{BTreeMap, BTreeSet};

use roomci_core::{AssertionResult, RunReport, RunResult, TimelineEvent};
use serde::Serialize;

/// Render a run report as pretty-printed JSON.
pub fn to_json(report: &RunReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

/// Stable aggregate artifact for one `roomci run --report-dir` invocation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RunSummary {
    pub schema_version: &'static str,
    pub run_id: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub scenarios: Vec<ScenarioSummaryEntry>,
}

/// One scenario's entry in a [`RunSummary`].
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ScenarioSummaryEntry {
    pub sequence: usize,
    pub path: String,
    pub scenario_name: String,
    pub result: RunResult,
    pub assertions_total: usize,
    pub assertions_failed: usize,
    pub report_dir: String,
    pub dry_run: bool,
}

/// Render a run summary as pretty-printed JSON.
pub fn to_summary_json(summary: &RunSummary) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(summary)
}

pub const GITHUB_SUMMARY_MAX_FAILED_ASSERTIONS: usize = 20;
const GITHUB_SUMMARY_MAX_BYTES: usize = 900 * 1024;
const GITHUB_SUMMARY_TRUNCATION_SUFFIX: &str =
    "\n\n_… summary truncated; full evidence is available in the report artifacts._\n";

/// Bounded failure detail used by the GitHub summary renderer.
///
/// Keeping only these fields avoids retaining or cloning every complete report
/// in a large batch solely for the step summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubFailureDetail {
    pub scenario_name: String,
    pub assertion_name: String,
    pub message: String,
    pub impact_message: Option<String>,
}

/// Render the compact, append-safe Markdown shown in a GitHub Actions step summary.
///
/// GitHub limits step-summary files to 1 MiB. Limiting failure details keeps a
/// large batch useful while the complete evidence remains available in artifacts.
pub fn to_github_step_summary(
    summary: &RunSummary,
    failed_assertions: &[GithubFailureDetail],
    total_failed_assertions: usize,
) -> String {
    let mut output = GithubSummaryWriter::new();
    let is_dry_run =
        !summary.scenarios.is_empty() && summary.scenarios.iter().all(|scenario| scenario.dry_run);
    if is_dry_run {
        output.push(&format!(
            "## roomci: {} validated (dry run; not executed)\n\n",
            summary.total
        ));
    } else {
        output.push(&format!(
            "## roomci: {} passed, {} failed (of {})\n\n",
            summary.passed, summary.failed, summary.total
        ));
    }
    output.push("| # | Scenario | Result | Assertions |\n");
    output.push("|---|----------|--------|------------|\n");
    for scenario in &summary.scenarios {
        let passed = scenario
            .assertions_total
            .saturating_sub(scenario.assertions_failed);
        output.push("| ");
        output.push(&scenario.sequence.to_string());
        output.push(" | ");
        output.push_escaped(&scenario.scenario_name, true);
        output.push(" | ");
        if scenario.dry_run {
            output.push("🟦 validated");
        } else {
            output.push(if scenario.result == RunResult::Passed {
                "✅ passed"
            } else {
                "❌ failed"
            });
        }
        output.push(" | ");
        if scenario.dry_run {
            output.push("not executed");
        } else {
            output.push(&format!("{passed}/{}", scenario.assertions_total));
        }
        output.push(" |\n");
    }

    if failed_assertions.is_empty() {
        return output.finish();
    }

    output.push("\n### Failed assertions\n\n");
    for failure in failed_assertions
        .iter()
        .take(GITHUB_SUMMARY_MAX_FAILED_ASSERTIONS)
    {
        output.push("- `");
        output.push_escaped(&failure.scenario_name, false);
        output.push("` / `");
        output.push_escaped(&failure.assertion_name, false);
        output.push("`: ");
        output.push_escaped(&failure.message, false);
        output.push("\n");
        if let Some(impact) = &failure.impact_message {
            output.push("  - Guest impact: ");
            output.push_escaped(impact, false);
            output.push("\n");
        }
    }
    let remaining = total_failed_assertions.saturating_sub(GITHUB_SUMMARY_MAX_FAILED_ASSERTIONS);
    if remaining > 0 {
        output.push(&format!("\n…and {remaining} more\n"));
    }
    output.finish()
}

/// Writes summary Markdown while reserving room for the truncation disclosure.
/// This avoids allocating a multi-megabyte intermediate String from untrusted
/// scenario text before truncating it for GitHub's step-summary limit.
struct GithubSummaryWriter {
    output: String,
    truncated: bool,
}

impl GithubSummaryWriter {
    fn new() -> Self {
        Self {
            output: String::new(),
            truncated: false,
        }
    }

    fn push(&mut self, value: &str) {
        if self.truncated {
            return;
        }
        let content_limit = GITHUB_SUMMARY_MAX_BYTES - GITHUB_SUMMARY_TRUNCATION_SUFFIX.len();
        let remaining = content_limit.saturating_sub(self.output.len());
        if value.len() <= remaining {
            self.output.push_str(value);
            return;
        }

        let mut boundary = remaining;
        while boundary > 0 && !value.is_char_boundary(boundary) {
            boundary -= 1;
        }
        self.output.push_str(&value[..boundary]);
        self.truncated = true;
    }

    fn push_escaped(&mut self, value: &str, escape_pipe: bool) {
        for character in value.chars() {
            match character {
                '|' if escape_pipe => self.push("\\|"),
                '\n' | '\r' => self.push(" "),
                '&' => self.push("&amp;"),
                '<' => self.push("&lt;"),
                '>' => self.push("&gt;"),
                '`' => self.push("&#96;"),
                '[' => self.push("&#91;"),
                ']' => self.push("&#93;"),
                '(' => self.push("&#40;"),
                ')' => self.push("&#41;"),
                '!' => self.push("&#33;"),
                '*' => self.push("&#42;"),
                '#' => self.push("&#35;"),
                '\\' => self.push("&#92;"),
                ':' => self.push("&#58;"),
                '.' => self.push("&#46;"),
                '@' => self.push("&#64;"),
                _ => {
                    let mut encoded = [0; 4];
                    self.push(character.encode_utf8(&mut encoded));
                }
            }
            if self.truncated {
                return;
            }
        }
    }

    fn finish(mut self) -> String {
        if self.truncated {
            self.output.push_str(GITHUB_SUMMARY_TRUNCATION_SUFFIX);
        }
        self.output
    }
}

/// One event in the stable timeline export contract.
///
/// This is intentionally smaller than a full [`RunReport`] so CI, log search,
/// and artifact consumers can ingest timeline evidence without depending on the
/// full report shape.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TimelineExportEvent {
    pub schema_version: &'static str,
    pub run_id: String,
    pub scenario_name: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub sequence: usize,
    pub at: String,
    pub event_type: String,
    pub target: Option<String>,
    pub message: String,
}

/// Stable observability summary for artifact ingestion.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObservabilityExport {
    pub schema_version: &'static str,
    pub run_id: String,
    pub scenario_name: String,
    pub result: RunResult,
    pub timeline_event_count: usize,
    pub assertion_count: usize,
    pub failed_assertion_count: usize,
    pub events_by_type: BTreeMap<String, usize>,
    pub assertions_by_status: BTreeMap<String, usize>,
    pub impact_levels: Vec<String>,
}

/// Convert a report timeline into stable export records.
pub fn timeline_export_events(report: &RunReport) -> Vec<TimelineExportEvent> {
    report
        .timeline
        .iter()
        .enumerate()
        .map(|(sequence, event)| timeline_export_event(report, event, sequence))
        .collect()
}

/// Render the stable timeline export as pretty-printed JSON.
pub fn to_timeline_json(report: &RunReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&timeline_export_events(report))
}

/// Render the stable timeline export as newline-delimited JSON.
pub fn to_timeline_ndjson(report: &RunReport) -> Result<String, serde_json::Error> {
    let mut output = String::new();
    for event in timeline_export_events(report) {
        output.push_str(&serde_json::to_string(&event)?);
        output.push('\n');
    }
    Ok(output)
}

/// Build a deterministic observability artifact from a run report.
pub fn observability_export(report: &RunReport) -> ObservabilityExport {
    let mut events_by_type = BTreeMap::new();
    for event in &report.timeline {
        *events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
    }

    let failed_assertion_count = report
        .assertions
        .iter()
        .filter(|assertion| !assertion.passed)
        .count();
    let mut assertions_by_status = BTreeMap::new();
    assertions_by_status.insert(
        "passed".to_string(),
        report
            .assertions
            .len()
            .saturating_sub(failed_assertion_count),
    );
    assertions_by_status.insert("failed".to_string(), failed_assertion_count);

    let impact_levels = report
        .assertions
        .iter()
        .filter_map(|assertion| assertion.impact_level.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    ObservabilityExport {
        schema_version: "roomci.observability.v1",
        run_id: report.run_id.clone(),
        scenario_name: report.scenario_name.clone(),
        result: report.result,
        timeline_event_count: report.timeline.len(),
        assertion_count: report.assertions.len(),
        failed_assertion_count,
        events_by_type,
        assertions_by_status,
        impact_levels,
    }
}

/// Render the observability artifact as pretty-printed JSON.
pub fn to_observability_json(report: &RunReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&observability_export(report))
}

/// Render a run report as a Markdown summary suitable for PR comments or
/// `report.md` artifacts.
///
/// The output includes failed assertions, the full timeline, and a suggested
/// recovery section listing the guest-impact message of each failed assertion.
pub fn to_markdown(report: &RunReport) -> String {
    let mut output = String::new();
    output.push_str(&format!("# roomci Report — {}\n\n", report.scenario_name));
    output.push_str(&format!("Result: `{}`\n\n", result_label(report.result)));

    output.push_str("## Failed Assertions\n\n");
    let failed = report
        .assertions
        .iter()
        .filter(|assertion| !assertion.passed)
        .collect::<Vec<_>>();
    if failed.is_empty() {
        output.push_str("None.\n\n");
    } else {
        for assertion in &failed {
            output.push_str(&format!(
                "- `{}`: {}\n",
                evidence_assertion_name(assertion),
                assertion.message
            ));
            if let Some(message) = &assertion.impact_message {
                output.push_str(&format!("  Guest impact: {}\n", message));
            }
        }
        output.push('\n');
    }

    output.push_str("## Assertions\n\n");
    for assertion in &report.assertions {
        output.push_str(&format!(
            "- [{}] `{}` — {}\n",
            if assertion.passed { "pass" } else { "fail" },
            evidence_assertion_name(assertion),
            assertion.message
        ));
    }
    output.push('\n');

    output.push_str("## Timeline\n\n");
    for event in &report.timeline {
        let target = event
            .target
            .as_ref()
            .map(|target| format!(" `{}`", target))
            .unwrap_or_default();
        output.push_str(&format!(
            "- `{}` `{}`{}: {}\n",
            event.at, event.event_type, target, event.message
        ));
    }
    output.push('\n');

    output.push_str("## Suggested Recovery\n\n");
    if failed.is_empty() {
        output.push_str("None.\n");
    } else {
        let mut emitted_recovery = false;
        for assertion in &failed {
            let message = assertion
                .impact_message
                .as_deref()
                .or(Some(assertion.message.as_str()));
            if let Some(message) = message {
                output.push_str(&format!("- {message}\n"));
                emitted_recovery = true;
            }
        }
        if !emitted_recovery {
            output.push_str("None.\n");
        }
    }

    output
}

/// Render a run report as JUnit XML for CI systems that consume test results
/// (GitHub Actions test reporters, Jenkins, GitLab, etc.).
///
/// Each assertion becomes a `<testcase>`, and failed assertions emit a
/// `<failure>` element whose body carries the guest-impact message when
/// available.
pub fn to_junit(report: &RunReport) -> String {
    let failures = report
        .assertions
        .iter()
        .filter(|assertion| !assertion.passed)
        .count();
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str(&format!(
        "<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\">\n",
        escape_xml(&report.scenario_name),
        report.assertions.len(),
        failures
    ));
    for assertion in &report.assertions {
        output.push_str(&testcase_xml(assertion));
    }
    output.push_str("</testsuite>\n");
    output
}

fn testcase_xml(assertion: &AssertionResult) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "  <testcase classname=\"roomci\" name=\"{}\">",
        escape_xml(&evidence_assertion_name(assertion))
    ));
    if assertion.passed {
        output.push_str("</testcase>\n");
    } else {
        output.push_str(&format!(
            "<failure message=\"{}\">{}</failure></testcase>\n",
            escape_xml(&assertion.message),
            escape_xml(
                assertion
                    .impact_message
                    .as_deref()
                    .unwrap_or(&assertion.message)
            )
        ));
    }
    output
}

fn evidence_assertion_name(assertion: &AssertionResult) -> String {
    assertion
        .reference_id
        .as_deref()
        .map(|reference_id| format!("{reference_id} ({})", assertion.name))
        .unwrap_or_else(|| assertion.name.clone())
}

fn timeline_export_event(
    report: &RunReport,
    event: &TimelineEvent,
    sequence: usize,
) -> TimelineExportEvent {
    TimelineExportEvent {
        schema_version: "roomci.timeline.v1",
        run_id: report.run_id.clone(),
        scenario_name: report.scenario_name.clone(),
        trace_id: stable_trace_id(&report.run_id),
        span_id: stable_span_id(&report.run_id, sequence),
        parent_span_id: if sequence == 0 {
            None
        } else {
            Some(stable_span_id(&report.run_id, sequence - 1))
        },
        sequence,
        at: event.at.clone(),
        event_type: event.event_type.clone(),
        target: event.target.clone(),
        message: event.message.clone(),
    }
}

fn stable_trace_id(run_id: &str) -> String {
    format!("roomci-trace-{run_id}")
}

fn stable_span_id(run_id: &str, sequence: usize) -> String {
    format!("roomci-span-{run_id}-{sequence:06}")
}

fn result_label(result: RunResult) -> &'static str {
    match result {
        RunResult::Passed => "passed",
        RunResult::Failed => "failed",
    }
}

fn escape_xml(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .filter(|ch| match *ch {
            '\u{9}' | '\u{A}' | '\u{D}' => true,
            '\u{FEFF}' => false,
            '\u{20}'..='\u{D7FF}' => true,
            '\u{E000}'..='\u{FFFD}' => *ch != '\u{FFFE}' && *ch != '\u{FFFF}',
            '\u{10000}'..='\u{10FFFF}' => true,
            _ => false,
        })
        .collect();
    sanitized
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use roomci_core::run_scenario;
    use roomci_scenario::load_scenario;

    use super::*;

    #[test]
    fn summary_json_uses_stable_schema_and_scenario_fields() {
        let summary = RunSummary {
            schema_version: "roomci.summary.v1",
            run_id: "batch-42".to_string(),
            total: 1,
            passed: 1,
            failed: 0,
            scenarios: vec![ScenarioSummaryEntry {
                sequence: 1,
                path: "examples/local_first_cloud_outage.yaml".to_string(),
                scenario_name: "local_first_cloud_outage".to_string(),
                result: RunResult::Passed,
                assertions_total: 4,
                assertions_failed: 0,
                report_dir: "01_local_first_cloud_outage".to_string(),
                dry_run: false,
            }],
        };

        let json: serde_json::Value =
            serde_json::from_str(&to_summary_json(&summary).unwrap()).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "schema_version": "roomci.summary.v1",
                "run_id": "batch-42",
                "total": 1,
                "passed": 1,
                "failed": 0,
                "scenarios": [{
                    "sequence": 1,
                    "path": "examples/local_first_cloud_outage.yaml",
                    "scenario_name": "local_first_cloud_outage",
                    "result": "passed",
                    "assertions_total": 4,
                    "assertions_failed": 0,
                    "report_dir": "01_local_first_cloud_outage",
                    "dry_run": false
                }]
            })
        );
    }

    #[test]
    fn renders_github_step_summary_and_limits_failed_assertions() {
        let mut failed_assertions = Vec::new();
        for index in 0..21 {
            failed_assertions.push(AssertionResult {
                reference_id: None,
                name: format!("failure-{index}"),
                assertion_type: "state_equals".to_string(),
                passed: false,
                message: format!("message-{index}"),
                impact_level: None,
                impact_message: Some(format!("impact-{index}")),
            });
        }
        let report = RunReport {
            schema_version: "roomci.report.v1".to_string(),
            run_id: "summary-test".to_string(),
            generated_by: "roomci".to_string(),
            scenario_name: "failing scenario".to_string(),
            result: RunResult::Failed,
            timeline: vec![],
            assertions: failed_assertions,
            final_state: BTreeMap::new(),
            retained_messages: BTreeMap::new(),
        };
        let summary = RunSummary {
            schema_version: "roomci.summary.v1",
            run_id: "batch-42".to_string(),
            total: 1,
            passed: 0,
            failed: 1,
            scenarios: vec![ScenarioSummaryEntry {
                sequence: 1,
                path: "examples/failing.yaml".to_string(),
                scenario_name: "failing scenario".to_string(),
                result: RunResult::Failed,
                assertions_total: 21,
                assertions_failed: 21,
                report_dir: "01_failing".to_string(),
                dry_run: false,
            }],
        };

        let failures = report
            .assertions
            .iter()
            .map(|assertion| GithubFailureDetail {
                scenario_name: report.scenario_name.clone(),
                assertion_name: assertion.name.clone(),
                message: assertion.message.clone(),
                impact_message: assertion.impact_message.clone(),
            })
            .collect::<Vec<_>>();
        let markdown = to_github_step_summary(&summary, &failures, failures.len());

        assert!(markdown.contains("## roomci: 0 passed, 1 failed (of 1)"));
        assert!(markdown.contains("| 1 | failing scenario | ❌ failed | 0/21 |"));
        assert!(markdown.contains("### Failed assertions"));
        assert!(markdown.contains("`failing scenario` / `failure-19`: message-19"));
        assert!(!markdown.contains("failure-20"));
        assert!(markdown.contains("…and 1 more"));
    }

    #[test]
    fn github_step_summary_escapes_table_and_code_injection_characters() {
        let report = RunReport {
            schema_version: "roomci.report.v1".to_string(),
            run_id: "summary-escape".to_string(),
            generated_by: "roomci".to_string(),
            scenario_name: "scenario|name\nnext".to_string(),
            result: RunResult::Failed,
            timeline: vec![],
            assertions: vec![AssertionResult {
                reference_id: None,
                name: "failure`name\nnext".to_string(),
                assertion_type: "state_equals".to_string(),
                passed: false,
                message: "message|with\nnewline".to_string(),
                impact_level: None,
                impact_message: None,
            }],
            final_state: BTreeMap::new(),
            retained_messages: BTreeMap::new(),
        };
        let summary = RunSummary {
            schema_version: "roomci.summary.v1",
            run_id: "batch-escape".to_string(),
            total: 1,
            passed: 0,
            failed: 1,
            scenarios: vec![ScenarioSummaryEntry {
                sequence: 1,
                path: "examples/failing.yaml".to_string(),
                scenario_name: report.scenario_name.clone(),
                result: RunResult::Failed,
                assertions_total: 1,
                assertions_failed: 1,
                report_dir: "01_failing".to_string(),
                dry_run: false,
            }],
        };

        let failures = [GithubFailureDetail {
            scenario_name: report.scenario_name.clone(),
            assertion_name: report.assertions[0].name.clone(),
            message: report.assertions[0].message.clone(),
            impact_message: None,
        }];
        let markdown = to_github_step_summary(&summary, &failures, failures.len());

        assert!(markdown.contains("| 1 | scenario\\|name next | ❌ failed | 0/1 |"));
        assert!(markdown
            .contains("`scenario|name next` / `failure&#96;name next`: message|with newline"));
        assert!(!markdown.contains("scenario|name\nnext"));
        assert!(!markdown.contains("failure`name\nnext"));
    }

    #[test]
    fn github_step_summary_neutralizes_links_html_and_oversized_content() {
        let unsafe_text = format!(
            "[click](https://example.invalid) ![pixel](https://example.invalid/x) www.example.invalid @octocat <details>{}</details>",
            "x".repeat(1_100_000)
        );
        let report = RunReport {
            schema_version: "roomci.report.v1".to_string(),
            run_id: "summary-budget".to_string(),
            generated_by: "roomci".to_string(),
            scenario_name: unsafe_text.clone(),
            result: RunResult::Failed,
            timeline: vec![],
            assertions: vec![AssertionResult {
                reference_id: None,
                name: unsafe_text.clone(),
                assertion_type: "state_equals".to_string(),
                passed: false,
                message: unsafe_text,
                impact_level: None,
                impact_message: None,
            }],
            final_state: BTreeMap::new(),
            retained_messages: BTreeMap::new(),
        };
        let summary = RunSummary {
            schema_version: "roomci.summary.v1",
            run_id: "batch-budget".to_string(),
            total: 1,
            passed: 0,
            failed: 1,
            scenarios: vec![ScenarioSummaryEntry {
                sequence: 1,
                path: "examples/failing.yaml".to_string(),
                scenario_name: report.scenario_name.clone(),
                result: RunResult::Failed,
                assertions_total: 1,
                assertions_failed: 1,
                report_dir: "01_failing".to_string(),
                dry_run: false,
            }],
        };

        let failures = [GithubFailureDetail {
            scenario_name: report.scenario_name.clone(),
            assertion_name: report.assertions[0].name.clone(),
            message: report.assertions[0].message.clone(),
            impact_message: None,
        }];
        let markdown = to_github_step_summary(&summary, &failures, failures.len());

        assert!(!markdown.contains("[click]("));
        assert!(!markdown.contains("![pixel]("));
        assert!(!markdown.contains("<details>"));
        assert!(!markdown.contains("https://"));
        assert!(!markdown.contains("www.example.invalid"));
        assert!(!markdown.contains("@octocat"));
        assert!(markdown.len() <= 900 * 1024);
        assert!(markdown.contains("summary truncated"));
    }

    fn fixture(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }

    #[test]
    fn renders_junit_success_for_latest_scenario() {
        let scenario = load_scenario(fixture("examples/local_first_cloud_outage.yaml")).unwrap();
        let report = run_scenario(&scenario).unwrap();

        let junit = to_junit(&report);

        assert!(junit.contains("<testsuite"));
        assert!(junit.contains("failures=\"0\""));
    }

    #[test]
    fn renders_stable_assertion_reference_without_losing_diagnostic_name() {
        let scenario = load_scenario(fixture("examples/generic_mqtt_retained_state.yaml")).unwrap();
        let report = run_scenario(&scenario).unwrap();

        let markdown = to_markdown(&report);
        let junit = to_junit(&report);

        assert!(markdown.contains("`retained_state_updated (mqtt_retained:"));
        assert!(junit.contains("name=\"retained_state_updated (mqtt_retained:"));
    }

    #[test]
    fn renders_markdown_timeline_for_latest_scenario() {
        let scenario = load_scenario(fixture("examples/local_first_cloud_outage.yaml")).unwrap();
        let report = run_scenario(&scenario).unwrap();

        let markdown = to_markdown(&report);

        assert!(markdown.contains("guest experience"));
        assert!(markdown.contains("Timeline"));
    }

    #[test]
    fn renders_stable_timeline_export() {
        let scenario = load_scenario(fixture("examples/local_first_cloud_outage.yaml")).unwrap();
        let report = run_scenario(&scenario).unwrap();

        let events = timeline_export_events(&report);

        assert!(!events.is_empty());
        assert_eq!(events[0].schema_version, "roomci.timeline.v1");
        assert_eq!(events[0].run_id, "local_first_cloud_outage");
        assert_eq!(events[0].scenario_name, "local_first_cloud_outage");
        assert_eq!(events[0].sequence, 0);
        assert!(events[0].trace_id.starts_with("roomci-trace-"));
        assert!(events[0].span_id.starts_with("roomci-span-"));

        let json = to_timeline_json(&report).unwrap();
        assert!(json.contains("roomci.timeline.v1"));

        let ndjson = to_timeline_ndjson(&report).unwrap();
        assert_eq!(ndjson.lines().count(), events.len());
    }

    #[test]
    fn renders_observability_export() {
        let scenario = load_scenario(fixture("examples/bms_sauna_emergency_alert.yaml")).unwrap();
        let report = run_scenario(&scenario).unwrap();

        let export = observability_export(&report);

        assert_eq!(export.schema_version, "roomci.observability.v1");
        assert_eq!(export.run_id, "bms_sauna_emergency_alert");
        assert!(export.timeline_event_count > 0);
        assert_eq!(export.assertion_count, report.assertions.len());
        assert!(export.events_by_type.contains_key("contact_changed"));

        let json = to_observability_json(&report).unwrap();
        assert!(json.contains("roomci.observability.v1"));
    }

    #[test]
    fn renders_suggested_recovery_with_message_fallback() {
        let report = RunReport {
            schema_version: "roomci.report.v1".to_string(),
            run_id: "fallback-message".to_string(),
            generated_by: "roomci".to_string(),
            scenario_name: "fallback-message".to_string(),
            result: RunResult::Failed,
            timeline: vec![],
            assertions: vec![AssertionResult {
                reference_id: None,
                name: "room-temperature".to_string(),
                assertion_type: "guest_visibility".to_string(),
                passed: false,
                message: "Guest comfort issue occurred".to_string(),
                impact_level: None,
                impact_message: None,
            }],
            final_state: BTreeMap::new(),
            retained_messages: BTreeMap::new(),
        };

        let markdown = to_markdown(&report);

        assert!(markdown.contains("## Suggested Recovery\n\n- Guest comfort issue occurred"));
    }

    #[test]
    fn escapes_xml_reserved_entities_and_removes_control_characters() {
        let report = RunReport {
            schema_version: "roomci.report.v1".to_string(),
            run_id: "xml-control".to_string(),
            generated_by: "roomci".to_string(),
            scenario_name: "xml-control".to_string(),
            result: RunResult::Failed,
            timeline: vec![],
            assertions: vec![AssertionResult {
                reference_id: None,
                name: "bad&name\nx\0y\tz\rq<'\">".to_string(),
                assertion_type: "guest_visibility".to_string(),
                passed: true,
                message: "ok\u{FFFE}\u{FFFF}".to_string(),
                impact_level: None,
                impact_message: Some("hello\u{001f}world".to_string()),
            }],
            final_state: BTreeMap::new(),
            retained_messages: BTreeMap::new(),
        };

        let junit = to_junit(&report);

        assert!(junit.contains("&lt;"));
        assert!(junit.contains("&gt;"));
        assert!(junit.contains("&quot;"));
        assert!(junit.contains("&apos;"));
        assert!(junit.contains("&amp;"));
        assert!(junit.contains("bad&amp;name\nx"));
        assert!(!junit.contains('\u{0000}'));
        assert!(!junit.contains('\u{FFFE}'));
        assert!(!junit.contains('\u{FFFF}'));
        assert!(!junit.contains('\u{001f}'));
        assert!(!junit.contains('\u{001b}'));
    }

    #[test]
    fn escape_xml_replaces_reserved_and_removes_c0_controls() {
        let escaped = escape_xml("x\x00\x08<&>\"'\ntest\r\ta\x1f");

        assert_eq!(escaped, "x&lt;&amp;&gt;&quot;&apos;\ntest\r\ta");
    }

    #[test]
    fn escape_xml_keeps_allowed_controls() {
        let escaped = escape_xml("line1\nline2\tline3\rline4");

        assert_eq!(escaped, "line1\nline2\tline3\rline4");
    }

    #[test]
    fn escape_xml_removes_non_characters() {
        let escaped = escape_xml("safe\u{FFFE}\u{FFFF}\x00 text");

        assert_eq!(escaped, "safe text");
    }
}
