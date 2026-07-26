use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn version_matches_the_workspace_release_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("--version")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("roomci {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn init_scaffolds_a_runnable_scenario() {
    let tempdir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("init")
        .current_dir(tempdir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("roomci validate roomci/smoke.yaml"));
    for path in [
        "roomci/smoke.yaml",
        "roomci/README.md",
        ".vscode/settings.json",
    ] {
        assert!(tempdir.path().join(path).is_file(), "missing {path}");
    }
    assert!(
        !std::fs::read_to_string(tempdir.path().join("roomci/smoke.yaml"))
            .unwrap()
            .contains("enabled:"),
        "the #29 compatibility field must not be scaffolded"
    );

    let run = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .args(["run", "roomci/smoke.yaml"])
        .current_dir(tempdir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn init_path_prints_copy_pasteable_next_steps_for_that_path() {
    let tempdir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .args(["init", "project"])
        .current_dir(tempdir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("roomci validate project/roomci/smoke.yaml"));
    assert!(stdout.contains("roomci run project/roomci/smoke.yaml --verbose"));
}

#[test]
fn init_allows_a_parent_directory_path() {
    let tempdir = tempfile::tempdir().unwrap();
    let current = tempdir.path().join("current");
    std::fs::create_dir(&current).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .args(["init", "../project"])
        .current_dir(&current)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(tempdir.path().join("project/roomci/smoke.yaml").is_file());
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("roomci validate ../project/roomci/smoke.yaml"));
}

#[test]
fn init_quotes_next_steps_for_paths_with_spaces_and_single_quotes() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = "project with space/owner's room";
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .args(["init", path])
        .current_dir(tempdir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout
        .contains("roomci validate 'project with space/owner'\"'\"'s room/roomci/smoke.yaml'"));
    assert!(stdout.contains(
        "roomci run 'project with space/owner'\"'\"'s room/roomci/smoke.yaml' --verbose"
    ));
}

#[test]
fn init_refuses_existing_target_without_writing_any_other_files() {
    let tempdir = tempfile::tempdir().unwrap();
    let roomci_dir = tempdir.path().join("roomci");
    std::fs::create_dir(&roomci_dir).unwrap();
    let smoke = roomci_dir.join("smoke.yaml");
    std::fs::write(&smoke, "keep this scenario unchanged\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("init")
        .current_dir(tempdir.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("roomci/smoke.yaml"));
    assert_eq!(
        std::fs::read_to_string(&smoke).unwrap(),
        "keep this scenario unchanged\n"
    );
    assert!(!tempdir.path().join("roomci/README.md").exists());
    assert!(!tempdir.path().join(".vscode/settings.json").exists());
}

#[test]
fn init_force_replaces_all_scaffold_files() {
    let tempdir = tempfile::tempdir().unwrap();
    let first = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("init")
        .current_dir(tempdir.path())
        .output()
        .unwrap();
    assert!(first.status.success());
    std::fs::write(tempdir.path().join("roomci/smoke.yaml"), "outdated\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .args(["init", "--force"])
        .current_dir(tempdir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        std::fs::read_to_string(tempdir.path().join("roomci/smoke.yaml"))
            .unwrap()
            .contains("my_first_smoke")
    );
}

#[test]
fn init_force_refuses_a_directory_where_a_generated_file_belongs() {
    let tempdir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tempdir.path().join("roomci/smoke.yaml")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .args(["init", "--force"])
        .current_dir(tempdir.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(tempdir.path().join("roomci/smoke.yaml").is_dir());
    assert!(!tempdir.path().join(".vscode/settings.json").exists());
}

#[test]
fn init_with_github_ci_emits_pinned_release_action_workflow() {
    let tempdir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .args(["init", "--ci", "github"])
        .current_dir(tempdir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let workflow =
        std::fs::read_to_string(tempdir.path().join(".github/workflows/roomci.yml")).unwrap();
    assert!(workflow.contains("actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09"));
    assert!(workflow.contains("persist-credentials: false"));
    assert!(workflow.contains("uses: albert-einshutoin/roomci@v0.1.1"));
}

#[cfg(unix)]
#[test]
fn init_never_follows_a_symlinked_target_even_with_force() {
    use std::os::unix::fs::symlink;

    let tempdir = tempfile::tempdir().unwrap();
    let outside = tempdir.path().join("outside-settings.json");
    std::fs::write(&outside, "do not overwrite\n").unwrap();
    let vscode = tempdir.path().join(".vscode");
    std::fs::create_dir(&vscode).unwrap();
    symlink(&outside, vscode.join("settings.json")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .args(["init", "--force"])
        .current_dir(tempdir.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("symbolic link"));
    assert_eq!(
        std::fs::read_to_string(outside).unwrap(),
        "do not overwrite\n"
    );
    assert!(!tempdir.path().join("roomci/smoke.yaml").exists());
}

#[cfg(unix)]
#[test]
fn init_rejects_a_path_whose_existing_ancestor_is_a_symlink() {
    use std::os::unix::fs::symlink;

    let tempdir = tempfile::tempdir().unwrap();
    let outside = tempdir.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    symlink(&outside, tempdir.path().join("linked-parent")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .args(["init", "linked-parent/project"])
        .current_dir(tempdir.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("symbolic link"));
    assert!(!outside.join("project/roomci/smoke.yaml").exists());
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
        .arg(fixture("examples/generic_mqtt_retained_state.yaml"))
        .arg(fixture("examples/generic_mqtt_duplicate_delivery.yaml"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid:"));
}

#[test]
fn adapter_validate_accepts_shipped_contracts() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("adapter")
        .arg("validate")
        .arg(fixture(
            "adapter-contracts/templates/company_adapter_contract.yaml",
        ))
        .arg(fixture(
            "adapter-contracts/examples/generic_mqtt_edge_device.yaml",
        ))
        .arg(fixture(
            "adapter-contracts/examples/hospitality_local_first_room.yaml",
        ))
        .arg(fixture(
            "adapter-contracts/examples/building_automation_bms.yaml",
        ))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("adapter contract valid:"));
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
fn run_warns_when_multiple_scenarios_share_single_report_output() {
    let tempdir = tempfile::tempdir().unwrap();
    let json = tempdir.path().join("roomci.json");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg(fixture("examples/generic_mqtt_retained_state.yaml"))
        .arg("--report-json")
        .arg(&json)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("single report output flags write only the last scenario"));
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(json).unwrap()).unwrap();
    assert_eq!(report["scenario_name"], "generic_mqtt_retained_state");
}

#[test]
fn report_dir_emits_per_scenario_artifacts() {
    let tempdir = tempfile::tempdir().unwrap();
    let report_dir = tempdir.path().join("reports");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg(fixture("examples/edge_server_failover.yaml"))
        .arg("--report-dir")
        .arg(&report_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stderr)
        .contains("single report output flags write only the last scenario"));

    for scenario_dir in ["01_local_first_cloud_outage", "02_edge_server_failover"] {
        for artifact in [
            "report.json",
            "report.md",
            "report.junit.xml",
            "timeline.json",
            "timeline.ndjson",
            "observability.json",
        ] {
            assert!(
                report_dir.join(scenario_dir).join(artifact).exists(),
                "missing {scenario_dir}/{artifact}"
            );
        }
    }

    let summary: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(report_dir.join("summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["schema_version"], "roomci.summary.v1");
    assert_eq!(summary["total"], 2);
    assert_eq!(summary["passed"], 2);
    assert_eq!(summary["failed"], 0);
    assert!(summary["run_id"]
        .as_str()
        .is_some_and(|run_id| run_id.starts_with("batch-")));
    assert_eq!(
        summary["scenarios"][0]["report_dir"],
        "01_local_first_cloud_outage"
    );
    assert_eq!(
        summary["scenarios"][1]["report_dir"],
        "02_edge_server_failover"
    );
}

#[test]
fn report_dir_with_failing_scenario_exits_one_and_records_failure() {
    let tempdir = tempfile::tempdir().unwrap();
    let report_dir = tempdir.path().join("reports");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg(fixture("examples/dali_scene_partial_failure.yaml"))
        .arg("--report-dir")
        .arg(&report_dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let summary: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(report_dir.join("summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["total"], 2);
    assert_eq!(summary["passed"], 1);
    assert_eq!(summary["failed"], 1);
    assert_eq!(summary["scenarios"][1]["result"], "failed");
    assert_eq!(summary["scenarios"][1]["assertions_failed"], 1);
}

#[test]
fn report_dir_handles_duplicate_scenario_names() {
    let tempdir = tempfile::tempdir().unwrap();
    let first = tempdir.path().join("first.yaml");
    let second = tempdir.path().join("second.yaml");
    let source =
        std::fs::read_to_string(fixture("examples/local_first_cloud_outage.yaml")).unwrap();
    std::fs::write(&first, &source).unwrap();
    std::fs::write(&second, source).unwrap();
    let report_dir = tempdir.path().join("reports");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(&first)
        .arg(&second)
        .arg("--report-dir")
        .arg(&report_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(report_dir.join("01_first/report.json").exists());
    assert!(report_dir.join("02_second/report.json").exists());
}

#[test]
fn report_dir_sanitizes_unusual_file_stems() {
    let tempdir = tempfile::tempdir().unwrap();
    let scenario = tempdir.path().join("unsafe scenario!.yaml");
    std::fs::write(
        &scenario,
        std::fs::read_to_string(fixture("examples/local_first_cloud_outage.yaml")).unwrap(),
    )
    .unwrap();
    let report_dir = tempdir.path().join("reports");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(&scenario)
        .arg("--report-dir")
        .arg(&report_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(report_dir.join("01_unsafe_scenario_/report.json").exists());
}

#[test]
fn report_dir_keeps_single_report_flags_for_the_last_scenario() {
    let tempdir = tempfile::tempdir().unwrap();
    let report_dir = tempdir.path().join("reports");
    let last_report = tempdir.path().join("last.json");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg(fixture("examples/edge_server_failover.yaml"))
        .arg("--report-dir")
        .arg(&report_dir)
        .arg("--report-json")
        .arg(&last_report)
        .arg("--run-id")
        .arg("ci-batch-42")
        .output()
        .unwrap();

    assert!(output.status.success());
    let summary: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(report_dir.join("summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["run_id"], "ci-batch-42");
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(last_report).unwrap()).unwrap();
    assert_eq!(report["scenario_name"], "edge_server_failover");
    assert_eq!(report["run_id"], "ci-batch-42");
    assert!(!String::from_utf8_lossy(&output.stderr)
        .contains("single report output flags write only the last scenario"));
}

#[test]
fn report_dir_dry_run_writes_only_a_dry_run_summary() {
    let tempdir = tempfile::tempdir().unwrap();
    let report_dir = tempdir.path().join("reports");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg("--report-dir")
        .arg(&report_dir)
        .arg("--dry-run")
        .output()
        .unwrap();

    assert!(output.status.success());
    let summary: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(report_dir.join("summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["scenarios"][0]["dry_run"], true);
    assert!(!report_dir.join("01_local_first_cloud_outage").exists());
}

#[test]
fn run_writes_explicit_github_step_summary_and_appends_to_existing_content() {
    let tempdir = tempfile::tempdir().unwrap();
    let summary = tempdir.path().join("step-summary.md");
    let automatic_summary = tempdir.path().join("automatic-summary.md");
    std::fs::write(&summary, "existing summary\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg(fixture("examples/dali_scene_partial_failure.yaml"))
        .arg("--github-summary")
        .arg(&summary)
        .env("GITHUB_STEP_SUMMARY", &automatic_summary)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let markdown = std::fs::read_to_string(summary).unwrap();
    assert!(markdown.starts_with("existing summary\n"));
    assert!(markdown.contains("## roomci: 1 passed, 1 failed (of 2)"));
    assert!(markdown.contains("| 1 | local_first_cloud_outage | ✅ passed | 2/2 |"));
    assert!(markdown.contains("| 2 | welcome_scene_partial_failure | ❌ failed | 0/1 |"));
    assert!(markdown.contains("### Failed assertions"));
    assert!(
        !automatic_summary.exists(),
        "--github-summary must take precedence over GITHUB_STEP_SUMMARY"
    );
}

#[test]
fn github_step_summary_inserts_a_newline_after_existing_content_without_one() {
    let tempdir = tempfile::tempdir().unwrap();
    let summary = tempdir.path().join("step-summary.md");
    std::fs::write(&summary, "existing summary without newline").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg("--github-summary")
        .arg(&summary)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(std::fs::read_to_string(summary)
        .unwrap()
        .contains("existing summary without newline\n## roomci:"));
}

#[test]
fn github_step_summary_warns_and_does_not_exceed_github_file_limit() {
    const GITHUB_STEP_SUMMARY_MAX_BYTES: usize = 1024 * 1024;

    let tempdir = tempfile::tempdir().unwrap();
    let summary = tempdir.path().join("step-summary.md");
    std::fs::write(&summary, vec![b'x'; GITHUB_STEP_SUMMARY_MAX_BYTES]).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg("--github-summary")
        .arg(&summary)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("warning: failed to append GitHub step summary"));
    assert_eq!(
        std::fs::metadata(summary).unwrap().len(),
        GITHUB_STEP_SUMMARY_MAX_BYTES as u64
    );
}

#[test]
fn run_uses_github_step_summary_when_no_explicit_path_is_given() {
    let tempdir = tempfile::tempdir().unwrap();
    let summary = tempdir.path().join("step-summary.md");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .env("GITHUB_STEP_SUMMARY", &summary)
        .output()
        .unwrap();

    assert!(output.status.success());
    let markdown = std::fs::read_to_string(summary).unwrap();
    assert!(markdown.contains("## roomci: 1 passed, 0 failed (of 1)"));
    assert!(!markdown.contains("### Failed assertions"));
}

#[test]
fn run_warns_but_succeeds_when_automatic_github_summary_cannot_be_written() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .env("GITHUB_STEP_SUMMARY", "/dev/null/roomci-summary.md")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("warning: failed to append GitHub step summary"));
}

#[test]
fn run_warns_but_preserves_result_when_explicit_github_summary_cannot_be_written() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg("--github-summary")
        .arg("/dev/null/roomci-summary.md")
        .env_remove("GITHUB_STEP_SUMMARY")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("warning: failed to append GitHub step summary"));
}

#[test]
fn dry_run_github_summary_is_reported_as_validated_not_passed() {
    let tempdir = tempfile::tempdir().unwrap();
    let summary = tempdir.path().join("step-summary.md");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg("--dry-run")
        .arg("--github-summary")
        .arg(&summary)
        .output()
        .unwrap();

    assert!(output.status.success());
    let markdown = std::fs::read_to_string(summary).unwrap();
    assert!(markdown.contains("1 validated (dry run; not executed)"));
    assert!(markdown.contains("🟦 validated"));
    assert!(!markdown.contains("✅ passed"));
}

#[test]
fn run_generates_timeline_and_observability_exports_with_run_id() {
    let tempdir = tempfile::tempdir().unwrap();
    let json = tempdir.path().join("roomci.json");
    let timeline_json = tempdir.path().join("roomci.timeline.json");
    let timeline_ndjson = tempdir.path().join("roomci.timeline.ndjson");
    let observability_json = tempdir.path().join("roomci.observability.json");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("run")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg("--run-id")
        .arg("phase19-smoke")
        .arg("--report-json")
        .arg(&json)
        .arg("--timeline-json")
        .arg(&timeline_json)
        .arg("--timeline-ndjson")
        .arg(&timeline_ndjson)
        .arg("--observability-json")
        .arg(&observability_json)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(json).unwrap()).unwrap();
    assert_eq!(report["schema_version"], "roomci.report.v1");
    assert_eq!(report["run_id"], "phase19-smoke");

    let timeline: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&timeline_json).unwrap()).unwrap();
    let timeline = timeline.as_array().unwrap();
    assert!(!timeline.is_empty());
    assert_eq!(timeline[0]["schema_version"], "roomci.timeline.v1");
    assert_eq!(timeline[0]["run_id"], "phase19-smoke");
    assert_eq!(timeline[0]["sequence"], 0);
    assert!(timeline[0].get("trace_id").is_some());
    assert!(timeline[0].get("span_id").is_some());

    let ndjson = std::fs::read_to_string(timeline_ndjson).unwrap();
    assert_eq!(ndjson.lines().count(), timeline.len());

    let observability: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(observability_json).unwrap()).unwrap();
    assert_eq!(observability["schema_version"], "roomci.observability.v1");
    assert_eq!(observability["run_id"], "phase19-smoke");
    assert!(observability["timeline_event_count"].as_u64().unwrap() > 0);
    assert!(observability.get("events_by_type").is_some());
    assert!(observability.get("assertions_by_status").is_some());
}

#[test]
fn debug_generates_json_and_markdown_for_failing_scenario() {
    let tempdir = tempfile::tempdir().unwrap();
    let json = tempdir.path().join("dali.debug.json");
    let markdown = tempdir.path().join("dali.debug.md");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("debug")
        .arg(fixture("examples/dali_scene_partial_failure.yaml"))
        .arg("--debug-json")
        .arg(&json)
        .arg("--debug-md")
        .arg(&markdown)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let debug: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(json).unwrap()).unwrap();
    assert_eq!(debug["schema_version"], "roomci.debug.v1");
    assert_eq!(debug["scenario_name"], "welcome_scene_partial_failure");
    assert_eq!(debug["result"], "failed");
    assert!(!debug["execution_order"].as_array().unwrap().is_empty());
    assert!(!debug["failure_causes"].as_array().unwrap().is_empty());
    assert!(std::fs::read_to_string(markdown)
        .unwrap()
        .contains("Failure Causes"));
}

#[test]
fn debug_passes_for_passing_scenario() {
    let tempdir = tempfile::tempdir().unwrap();
    let json = tempdir.path().join("local.debug.json");

    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("debug")
        .arg(fixture("examples/local_first_cloud_outage.yaml"))
        .arg("--debug-json")
        .arg(&json)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("failures=0"));

    let debug: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(json).unwrap()).unwrap();
    assert_eq!(debug["result"], "passed");
    assert!(!debug["state_diffs"].as_array().unwrap().is_empty());
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
fn serve_starts_http_runtime_and_exposes_reports() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("serve")
        .arg("--config")
        .arg(fixture("examples/generic_mqtt_retained_state.yaml"))
        .arg("--port")
        .arg("0")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut stdout = BufReader::new(stdout);
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    let address = line
        .trim()
        .strip_prefix("roomci serve listening on http://")
        .expect("serve should print listening address")
        .to_string();

    let health = http_request(&address, "GET", "/health", "");
    assert!(health.contains("HTTP/1.1 200 OK"));
    assert!(health.contains("\"status\":\"idle\""));
    assert!(health.contains("generic_mqtt_retained_state"));

    let finish = http_request(&address, "POST", "/finish", "");
    assert!(finish.contains("HTTP/1.1 200 OK"));
    assert!(finish.contains("\"finished\":true"));

    let health_after_finish = http_request(&address, "GET", "/health", "");
    assert!(health_after_finish.contains("HTTP/1.1 200 OK"));
    assert!(health_after_finish.contains("\"status\":\"passed\""));

    let report = http_request(&address, "GET", "/reports/latest.md", "");
    assert!(report.contains("HTTP/1.1 200 OK"));
    assert!(report.contains("# roomci Report"));

    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn serve_rejects_non_loopback_without_explicit_override() {
    let output = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("serve")
        .arg("--config")
        .arg(fixture("examples/generic_mqtt_retained_state.yaml"))
        .arg("--host")
        .arg("0.0.0.0")
        .arg("--port")
        .arg("0")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("refusing to bind non-loopback host"));
}

#[test]
fn external_http_controller_script_drives_serve_black_box() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("serve")
        .arg("--config")
        .arg(fixture("examples/generic_mqtt_retained_state.yaml"))
        .arg("--port")
        .arg("0")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut stdout = BufReader::new(stdout);
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    let address = line
        .trim()
        .strip_prefix("roomci serve listening on http://")
        .expect("serve should print listening address")
        .to_string();
    let report_dir = tempfile::tempdir().unwrap();

    let output = Command::new(fixture("examples/controllers/http_poc_controller.sh"))
        .env("ROOMCI_URL", format!("http://{address}"))
        .env("REPORT_DIR", report_dir.path())
        .output()
        .unwrap();

    child.kill().unwrap();
    child.wait().unwrap();

    assert!(
        output.status.success(),
        "controller failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(report_dir
        .path()
        .join("external_controller_latest.json")
        .exists());
    assert!(report_dir
        .path()
        .join("external_controller_latest.md")
        .exists());
    assert!(report_dir
        .path()
        .join("external_controller_latest.xml")
        .exists());
}

#[test]
fn external_mqtt_publish_updates_retained_state_through_serve() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("serve")
        .arg("--config")
        .arg(fixture("examples/generic_mqtt_retained_state.yaml"))
        .arg("--port")
        .arg("0")
        .arg("--mqtt-port")
        .arg("0")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut stdout = BufReader::new(stdout);
    let mut http_line = String::new();
    let mut endpoints_line = String::new();
    let mut mqtt_line = String::new();
    stdout.read_line(&mut http_line).unwrap();
    stdout.read_line(&mut endpoints_line).unwrap();
    stdout.read_line(&mut mqtt_line).unwrap();
    let http_address = http_line
        .trim()
        .strip_prefix("roomci serve listening on http://")
        .expect("serve should print HTTP listening address")
        .to_string();
    let mqtt_address = mqtt_line
        .trim()
        .strip_prefix("roomci mqtt listening on mqtt://")
        .expect("serve should print MQTT listening address")
        .to_string();

    publish_mqtt_json(
        &mqtt_address,
        "fleet/demo/site/lab/device/env_sensor_01/command",
        r#"{"online":true,"sample_interval_seconds":15}"#,
    );
    wait_for_http_contains(
        &http_address,
        "GET",
        "/state",
        "\"sample_interval_seconds\":15",
    );
    let finish = http_request(&http_address, "POST", "/finish", "");
    assert!(finish.contains("\"finished\":true"));
    assert!(finish.contains("\"external_publish_count\":1"));

    let timeline = http_request(&http_address, "GET", "/timeline", "");
    assert!(timeline.contains("external_mqtt_retained_state_updated"));
    let state = http_request(&http_address, "GET", "/state", "");
    assert!(state.contains("fleet/demo/site/lab/device/env_sensor_01/state"));
    assert!(state.contains("\"sample_interval_seconds\":15"));
    let report = http_request(&http_address, "GET", "/reports/latest.json", "");
    assert!(report.contains("external_mqtt_retained_state_updated"));
    assert!(report.contains("\"sample_interval_seconds\""));
    assert!(report.contains("15"));

    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn standard_mqtt_client_publishes_retained_state_through_serve() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_roomci"))
        .arg("serve")
        .arg("--config")
        .arg(fixture("examples/generic_mqtt_retained_state.yaml"))
        .arg("--port")
        .arg("0")
        .arg("--mqtt-port")
        .arg("0")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut stdout = BufReader::new(stdout);
    let mut http_line = String::new();
    let mut endpoints_line = String::new();
    let mut mqtt_line = String::new();
    stdout.read_line(&mut http_line).unwrap();
    stdout.read_line(&mut endpoints_line).unwrap();
    stdout.read_line(&mut mqtt_line).unwrap();
    let http_address = http_line
        .trim()
        .strip_prefix("roomci serve listening on http://")
        .expect("serve should print HTTP listening address")
        .to_string();
    let mqtt_address = mqtt_line
        .trim()
        .strip_prefix("roomci mqtt listening on mqtt://")
        .expect("serve should print MQTT listening address")
        .to_string();
    let (mqtt_host, mqtt_port) = mqtt_address
        .rsplit_once(':')
        .map(|(host, port)| (host.to_string(), port.parse::<u16>().unwrap()))
        .expect("MQTT address should include host:port");

    let mut client = MqttSmokeClient::connect(&mqtt_host, mqtt_port, "roomci-std-mqtt-client");
    client.subscribe("fleet/demo/site/lab/device/env_sensor_01/state");
    let initial_replay = client.wait_for_publish_payload("sample_interval_seconds");
    assert!(initial_replay.contains("30"));
    client.publish(
        "fleet/demo/site/lab/device/env_sensor_01/command",
        r#"{"online":true,"sample_interval_seconds":20}"#,
    );
    std::thread::sleep(Duration::from_millis(150));

    let state = http_request(&http_address, "GET", "/state", "");
    assert!(state.contains("fleet/demo/site/lab/device/env_sensor_01/state"));
    assert!(state.contains("\"sample_interval_seconds\":20"));
    client.subscribe("fleet/demo/site/lab/device/env_sensor_01/state");
    let updated_replay = client.wait_for_publish_payload("sample_interval_seconds");
    assert!(updated_replay.contains("20"));

    child.kill().unwrap();
    child.wait().unwrap();
}

struct MqttSmokeClient {
    stream: TcpStream,
    next_packet_id: u16,
}

impl MqttSmokeClient {
    fn connect(host: &str, port: u16, client_id: &str) -> Self {
        let address = format!("{host}:{port}");
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match TcpStream::connect(&address) {
                Ok(stream) => break stream,
                Err(error) if Instant::now() < deadline => {
                    let _ = error;
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(error) => panic!("failed to connect to MQTT {address}: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut body = Vec::new();
        push_mqtt_string(&mut body, "MQTT");
        body.push(4);
        body.push(0b0000_0010);
        body.extend_from_slice(&5u16.to_be_bytes());
        push_mqtt_string(&mut body, client_id);
        write_mqtt_packet(&mut stream, 0x10, &body);
        let (packet_type, packet) = read_mqtt_packet(&mut stream);
        assert_eq!(packet_type, 0x20, "expected MQTT CONNACK");
        assert_eq!(packet.as_slice(), &[0, 0], "MQTT CONNACK rejected client");
        Self {
            stream,
            next_packet_id: 1,
        }
    }

    fn subscribe(&mut self, topic: &str) {
        let packet_id = self.next_packet_id;
        self.next_packet_id = self.next_packet_id.wrapping_add(1).max(1);
        let mut body = Vec::new();
        body.extend_from_slice(&packet_id.to_be_bytes());
        push_mqtt_string(&mut body, topic);
        body.push(0);
        write_mqtt_packet(&mut self.stream, 0x82, &body);
    }

    fn publish(&mut self, topic: &str, payload: &str) {
        let mut body = Vec::new();
        push_mqtt_string(&mut body, topic);
        body.extend_from_slice(payload.as_bytes());
        write_mqtt_packet(&mut self.stream, 0x30, &body);
    }

    fn wait_for_publish_payload(&mut self, expected: &str) -> String {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let (packet_type, packet) = read_mqtt_packet(&mut self.stream);
            if packet_type != 0x30 {
                continue;
            }
            let Some(payload) = mqtt_publish_payload(&packet) else {
                continue;
            };
            if payload.contains(expected) {
                return payload;
            }
        }
        panic!("timed out waiting for MQTT publish payload containing {expected}");
    }
}

fn push_mqtt_string(buffer: &mut Vec<u8>, value: &str) {
    let len = u16::try_from(value.len()).expect("MQTT test string should fit u16");
    buffer.extend_from_slice(&len.to_be_bytes());
    buffer.extend_from_slice(value.as_bytes());
}

fn write_mqtt_packet(stream: &mut TcpStream, packet_type: u8, body: &[u8]) {
    let mut packet = vec![packet_type];
    push_remaining_length(&mut packet, body.len());
    packet.extend_from_slice(body);
    stream.write_all(&packet).unwrap();
}

fn push_remaining_length(buffer: &mut Vec<u8>, mut len: usize) {
    loop {
        let mut byte = u8::try_from(len % 128).unwrap();
        len /= 128;
        if len > 0 {
            byte |= 0x80;
        }
        buffer.push(byte);
        if len == 0 {
            break;
        }
    }
}

fn read_mqtt_packet(stream: &mut TcpStream) -> (u8, Vec<u8>) {
    let mut fixed = [0u8; 1];
    stream.read_exact(&mut fixed).unwrap();
    let len = read_remaining_length(stream);
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body).unwrap();
    (fixed[0] & 0xF0, body)
}

fn read_remaining_length(stream: &mut TcpStream) -> usize {
    let mut multiplier = 1usize;
    let mut value = 0usize;
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).unwrap();
        value += usize::from(byte[0] & 0x7F) * multiplier;
        if byte[0] & 0x80 == 0 {
            return value;
        }
        multiplier *= 128;
    }
}

fn mqtt_publish_payload(packet: &[u8]) -> Option<String> {
    let topic_len = u16::from_be_bytes([*packet.first()?, *packet.get(1)?]) as usize;
    let payload_start = 2 + topic_len;
    let payload = packet.get(payload_start..)?;
    Some(String::from_utf8_lossy(payload).to_string())
}

fn http_request(address: &str, method: &str, path: &str, body: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match TcpStream::connect(address) {
            Ok(mut stream) => {
                let request = format!(
                    "{method} {path} HTTP/1.1\r\nHost: {address}\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n{body}",
                    length = body.len()
                );
                stream.write_all(request.as_bytes()).unwrap();
                let mut response = String::new();
                stream.read_to_string(&mut response).unwrap();
                return response;
            }
            Err(error) if Instant::now() < deadline => {
                let _ = error;
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("failed to connect to {address}: {error}"),
        }
    }
}

fn wait_for_http_contains(address: &str, method: &str, path: &str, expected: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut last_response = String::new();
    while Instant::now() < deadline {
        last_response = http_request(address, method, path, "");
        if last_response.contains(expected) {
            return last_response;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for {path} to contain {expected}; last response: {last_response}");
}

fn publish_mqtt_json(address: &str, topic: &str, payload: &str) {
    let mut stream = TcpStream::connect(address).unwrap();
    let connect = mqtt_connect_packet("roomci-test-client");
    stream.write_all(&connect).unwrap();
    let mut connack = [0_u8; 4];
    stream.read_exact(&mut connack).unwrap();
    assert_eq!(connack, [0x20, 0x02, 0x00, 0x00]);
    let publish = mqtt_publish_packet(topic, payload.as_bytes());
    stream.write_all(&publish).unwrap();
}

fn mqtt_connect_packet(client_id: &str) -> Vec<u8> {
    let mut variable = Vec::new();
    variable.extend_from_slice(&[0x00, 0x04]);
    variable.extend_from_slice(b"MQTT");
    variable.push(0x04);
    variable.push(0x02);
    variable.extend_from_slice(&60_u16.to_be_bytes());
    variable.extend_from_slice(&(client_id.len() as u16).to_be_bytes());
    variable.extend_from_slice(client_id.as_bytes());

    let mut packet = vec![0x10];
    encode_remaining_length(variable.len(), &mut packet);
    packet.extend(variable);
    packet
}

fn mqtt_publish_packet(topic: &str, payload: &[u8]) -> Vec<u8> {
    let mut variable = Vec::new();
    variable.extend_from_slice(&(topic.len() as u16).to_be_bytes());
    variable.extend_from_slice(topic.as_bytes());
    variable.extend_from_slice(payload);

    let mut packet = vec![0x30];
    encode_remaining_length(variable.len(), &mut packet);
    packet.extend(variable);
    packet
}

fn encode_remaining_length(mut length: usize, packet: &mut Vec<u8>) {
    loop {
        let mut encoded = (length % 128) as u8;
        length /= 128;
        if length > 0 {
            encoded |= 128;
        }
        packet.push(encoded);
        if length == 0 {
            break;
        }
    }
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
