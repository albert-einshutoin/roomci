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
