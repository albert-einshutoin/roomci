use roomci_core::{AssertionResult, RunReport, RunResult};

pub fn to_json(report: &RunReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

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
    if failed
        .iter()
        .any(|assertion| assertion.name.contains("fallback_access_issued"))
    {
        output.push_str("- Issue fallback access before the guest is blocked.\n");
    }
    if failed
        .iter()
        .any(|assertion| assertion.name.contains("staff_notification_sent"))
    {
        output.push_str("- Notify staff when fallback access is required.\n");
    }
    if failed
        .iter()
        .any(|assertion| assertion.assertion_type == "sensor_threshold")
    {
        output.push_str("- Verify pre-arrival climate control and alert on comfort drift.\n");
    }
    if failed.is_empty() {
        output.push_str("None.\n");
    }

    output
}

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
}
