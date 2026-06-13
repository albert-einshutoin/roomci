//! Golden snapshot of the report *render* output contracts (Markdown, JUnit
//! XML, timeline NDJSON, observability JSON) for a representative set of
//! scenarios.
//!
//! Lives in `roomci-report` because `roomci-core` cannot depend on
//! `roomci-report` (the dependency points the other way). Three scenarios cover
//! the meaningful states: a passing local-first run, a failing/alerting BMS
//! run, and an intentionally-failing DALI scene.
//!
//! Regenerate after an *intentional* render change with:
//! `UPDATE_GOLDEN=1 cargo test -p roomci-report --test golden_renders`

use std::{fs, path::PathBuf};

use roomci_core::run_scenario;
use roomci_report::{to_junit, to_markdown, to_observability_json, to_timeline_ndjson};
use roomci_scenario::load_scenario;

/// Representative scenarios: passing, alerting, and intentionally-failing.
const SCENARIOS: &[&str] = &[
    "local_first_cloud_outage",
    "bms_sauna_emergency_alert",
    "dali_scene_partial_failure",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn check_or_update(name: &str, actual: &str, update: bool, failures: &mut Vec<String>) {
    let golden_path = golden_dir().join(name);

    if update {
        fs::create_dir_all(golden_dir()).unwrap();
        fs::write(&golden_path, format!("{actual}\n")).unwrap();
        return;
    }

    let expected = fs::read_to_string(&golden_path).unwrap_or_else(|_| {
        panic!(
            "missing golden file {golden_path:?}; regenerate with \
             `UPDATE_GOLDEN=1 cargo test -p roomci-report --test golden_renders`"
        )
    });
    if actual.trim_end_matches('\n') != expected.trim_end_matches('\n') {
        failures.push(name.to_string());
    }
}

#[test]
fn report_renders_match_golden_files() {
    let update = std::env::var("UPDATE_GOLDEN").is_ok();
    let mut failures = Vec::new();

    for stem in SCENARIOS {
        let path = repo_root().join("examples").join(format!("{stem}.yaml"));
        let scenario = load_scenario(&path).expect("example must parse");
        let report = run_scenario(&scenario).expect("example must run");

        check_or_update(
            &format!("{stem}.md"),
            &to_markdown(&report),
            update,
            &mut failures,
        );
        check_or_update(
            &format!("{stem}.junit.xml"),
            &to_junit(&report),
            update,
            &mut failures,
        );
        check_or_update(
            &format!("{stem}.ndjson"),
            &to_timeline_ndjson(&report).unwrap(),
            update,
            &mut failures,
        );
        check_or_update(
            &format!("{stem}.observability.json"),
            &to_observability_json(&report).unwrap(),
            update,
            &mut failures,
        );
    }

    assert!(
        failures.is_empty(),
        "report renders diverged from golden files {failures:?}; if intentional, \
         regenerate with `UPDATE_GOLDEN=1 cargo test -p roomci-report --test golden_renders`"
    );
}
