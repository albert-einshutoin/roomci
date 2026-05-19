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
        .arg(fixture("examples/modbus_floor_heating.yaml"))
        .arg(fixture("examples/dali_scene_partial_failure.yaml"))
        .arg(fixture("examples/bms_sauna_emergency_alert.yaml"))
        .arg(fixture("examples/starlink_failover.yaml"))
        .arg(fixture("examples/comfort_auto_mode.yaml"))
        .arg(fixture("examples/access_permission_drift.yaml"))
        .arg(fixture("examples/commissioning_checklist.yaml"))
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

#[test]
fn run_with_missing_scenario_file_exits_with_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg("/tmp/roomci-test-does-not-exist.yaml")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("error"),
        "expected error message on stderr, got: {stderr}"
    );
}

#[test]
fn validate_without_arguments_exits_with_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("validate")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("error") || stderr.to_lowercase().contains("at least one"),
        "expected error message on stderr, got: {stderr}"
    );
}

#[test]
fn unknown_subcommand_exits_with_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("frobnicate")
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn serve_check_validates_config_without_blocking() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("serve")
        .arg("--config")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg("--check")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("serve config valid:"));
}

#[test]
fn run_aggregates_multiple_scenarios() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg(fixture("examples/modbus_floor_heating.yaml"))
        .arg(fixture("examples/bms_sauna_emergency_alert.yaml"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[1/3]"));
    assert!(stdout.contains("[3/3]"));
    assert!(stdout.contains("summary: 3 passed, 0 failed (of 3)"));
}

#[test]
fn run_dry_run_skips_execution() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg("--dry-run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dry-run valid:"));
    assert!(!stdout.contains("result:"));
}

#[test]
fn run_verbose_emits_timeline() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg("--verbose")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[fault_activated]") || stdout.contains("[mqtt_publish]"));
}

#[test]
fn run_quiet_suppresses_per_scenario_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg("--quiet")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("summary:"));
    assert!(!stdout.contains("scenario:"));
}

#[test]
fn run_rejects_verbose_and_quiet_together() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg("--verbose")
        .arg("--quiet")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .output()
        .unwrap();

    assert!(!output.status.success());
}
