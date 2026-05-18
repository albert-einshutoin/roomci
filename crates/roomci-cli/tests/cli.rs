use std::{path::PathBuf, process::Command};

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn validate_accepts_example_scenarios() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("validate")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg(fixture("examples/edge_server_failover.yaml"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid:"));
}

#[test]
fn run_generates_reports_for_latest_local_first_scenario() {
    let tempdir = tempfile::tempdir().unwrap();
    let json = tempdir.path().join("roomci.json");
    let markdown = tempdir.path().join("roomci.md");
    let junit = tempdir.path().join("roomci.xml");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg("--report-json")
        .arg(&json)
        .arg("--report-md")
        .arg(&markdown)
        .arg("--junit")
        .arg(&junit)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(json.exists());
    assert!(markdown.exists());
    assert!(junit.exists());
    assert!(std::fs::read_to_string(markdown)
        .unwrap()
        .contains("guest experience"));
    assert!(std::fs::read_to_string(junit)
        .unwrap()
        .contains("failures=\"0\""));
}
