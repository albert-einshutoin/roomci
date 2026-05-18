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
        .arg(fixture("docs/examples/checkin_lock_offline.yaml"))
        .arg(fixture("docs/examples/ac_preheat_failed.yaml"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid:"));
}

#[test]
fn run_generates_reports_and_returns_failure_for_failed_scenario() {
    let tempdir = tempfile::tempdir().unwrap();
    let json = tempdir.path().join("roomci.json");
    let markdown = tempdir.path().join("roomci.md");
    let junit = tempdir.path().join("roomci.xml");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("docs/examples/checkin_lock_offline.yaml"))
        .arg("--json")
        .arg(&json)
        .arg("--markdown")
        .arg(&markdown)
        .arg("--junit")
        .arg(&junit)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(json.exists());
    assert!(markdown.exists());
    assert!(junit.exists());
    assert!(std::fs::read_to_string(markdown)
        .unwrap()
        .contains("Guest impact"));
    assert!(std::fs::read_to_string(junit).unwrap().contains("<failure"));
}
