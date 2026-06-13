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
            output.push_str(&format!("- `{}`: {}\n", assertion.name, assertion.message));
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
            assertion.name,
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
        for assertion in &failed {
            if let Some(message) = &assertion.impact_message {
                output.push_str(&format!("- {message}\n"));
            }
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
        escape_xml(&assertion.name)
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
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use roomci_core::run_scenario;
    use roomci_scenario::load_scenario;

    use super::*;

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
}
