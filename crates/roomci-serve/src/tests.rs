use super::*;
use std::{
    io::Read,
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    time::{Duration, Instant},
};

use roomci_scenario::load_scenario;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn serve_state() -> Arc<Mutex<ServeState>> {
    let scenario = load_scenario(fixture("examples/generic_mqtt_retained_state.yaml")).unwrap();
    serve_state_for_scenario(scenario)
}

fn serve_state_for_scenario(scenario: ScenarioFile) -> Arc<Mutex<ServeState>> {
    let latest_report = run_scenario(&scenario).unwrap();
    let mut modbus = ModbusModel::try_from_config(&scenario.modbus).unwrap();
    apply_scenario_modbus_steps(&scenario, &mut modbus);
    let modbus_units = modbus_unit_map(&scenario);
    Arc::new(Mutex::new(ServeState {
        scenario,
        latest_report,
        injected_faults: Vec::new(),
        external_publish_count: 0,
        run_in_progress: false,
        has_completed_run: false,
        external_observation_timeline: Vec::new(),
        external_observations: BTreeMap::new(),
        bms_replay_ids: BTreeSet::new(),
        external_mqtt_final_state: BTreeMap::new(),
        external_mqtt_retained_state: BTreeMap::new(),
        modbus,
        modbus_units,
        external_modbus_registers: BTreeMap::new(),
    }))
}

fn request(method: &str, path: &str, body: &str) -> HttpRequest {
    HttpRequest {
        method: method.to_string(),
        path: path.to_string(),
        body: body.to_string(),
    }
}

fn start_http_test_server() -> SocketAddr {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let state = serve_state();
    thread::spawn(move || serve_http_listener(listener, state));
    address
}

fn http_get(address: SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[test]
fn http_router_serves_core_observation_endpoints() {
    let state = serve_state();

    let health = route_request(&request("GET", "/health", ""), Arc::clone(&state));
    assert!(health.contains("HTTP/1.1 200 OK"));
    assert!(health.contains("\"status\":\"idle\""));

    let scenario = route_request(&request("GET", "/scenario", ""), Arc::clone(&state));
    assert!(scenario.contains("generic_mqtt_retained_state"));

    let current_state = route_request(&request("GET", "/state", ""), Arc::clone(&state));
    assert!(current_state.contains("retained_messages"));

    let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
    assert!(timeline.contains("mqtt_publish"));

    let timeline_export = route_request(
        &request("GET", "/timeline.export.json", ""),
        Arc::clone(&state),
    );
    assert!(timeline_export.contains("roomci.timeline.v1"));
    assert!(timeline_export.contains("\"sequence\": 0"));
    assert!(timeline_export.contains("\"trace_id\""));

    let timeline_ndjson =
        route_request(&request("GET", "/timeline.ndjson", ""), Arc::clone(&state));
    assert!(timeline_ndjson.contains("application/x-ndjson"));
    assert!(timeline_ndjson.contains("roomci.timeline.v1"));

    let observability = route_request(
        &request("GET", "/observability/latest.json", ""),
        Arc::clone(&state),
    );
    assert!(observability.contains("roomci.observability.v1"));
    assert!(observability.contains("\"events_by_type\""));
}

#[test]
fn health_reports_idle_running_passed_and_failed_states() {
    let state = serve_state();

    let idle = route_request(&request("GET", "/health", ""), Arc::clone(&state));
    assert!(idle.contains("HTTP/1.1 200 OK"));
    assert!(idle.contains("\"status\":\"idle\""));
    assert!(idle.contains("\"serve_version\""));
    assert!(idle.contains("\"latest_report_id\":\"generic_mqtt_retained_state\""));

    {
        let mut state_guard = state.lock().unwrap();
        state_guard.run_in_progress = true;
    }
    let running = route_request(&request("GET", "/health", ""), Arc::clone(&state));
    assert!(running.contains("HTTP/1.1 200 OK"));
    assert!(running.contains("\"status\":\"running\""));

    {
        let mut state_guard = state.lock().unwrap();
        state_guard.run_in_progress = false;
        state_guard.has_completed_run = true;
    }
    let passed = route_request(&request("GET", "/health", ""), Arc::clone(&state));
    assert!(passed.contains("HTTP/1.1 200 OK"));
    assert!(passed.contains("\"status\":\"passed\""));

    let failed_scenario =
        load_scenario(fixture("examples/dali_scene_partial_failure.yaml")).unwrap();
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.latest_report = run_scenario(&failed_scenario).unwrap();
        state_guard.has_completed_run = true;
    }
    let failed = route_request(&request("GET", "/health", ""), Arc::clone(&state));
    assert!(failed.contains("HTTP/1.1 503 Service Unavailable"));
    assert!(failed.contains("\"status\":\"failed\""));
}

#[test]
fn slow_http_client_does_not_block_fast_client() {
    let address = start_http_test_server();
    let _slow_client = TcpStream::connect(address).unwrap();

    let started = Instant::now();
    let response = http_get(address, "/health");

    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("\"status\":\"idle\""));
}

#[test]
fn concurrent_health_requests_do_not_serialize() {
    let address = start_http_test_server();
    let started = Instant::now();
    let clients = (0..3)
        .map(|_| thread::spawn(move || http_get(address, "/health")))
        .collect::<Vec<_>>();

    let responses = clients
        .into_iter()
        .map(|client| client.join().unwrap())
        .collect::<Vec<_>>();

    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(responses
        .iter()
        .all(|response| response.contains("HTTP/1.1 200 OK")));
}

#[test]
fn raw_response_renders_service_unavailable() {
    let response = raw_response(503, "application/json", r#"{"error":"x"}"#);

    assert!(response.starts_with("HTTP/1.1 503 Service Unavailable"));
}

#[test]
fn slow_http_client_is_closed_by_read_timeout() {
    let address = start_http_test_server();
    let mut slow_client = TcpStream::connect(address).unwrap();
    slow_client
        .set_read_timeout(Some(Duration::from_secs(HTTP_CLIENT_TIMEOUT_SECS + 3)))
        .unwrap();

    let started = Instant::now();
    let mut buffer = [0_u8; 1];
    let read = slow_client.read(&mut buffer);

    assert!(started.elapsed() < Duration::from_secs(HTTP_CLIENT_TIMEOUT_SECS + 3));
    assert!(matches!(read, Ok(0) | Err(_)));
}

#[test]
fn http_router_handles_fault_finish_and_reports() {
    let state = serve_state();

    let fault = route_request(
        &request(
            "POST",
            "/fault",
            r#"{"target":"mqtt.local","type":"offline"}"#,
        ),
        Arc::clone(&state),
    );
    assert!(fault.contains("HTTP/1.1 202 Accepted"));
    assert!(fault.contains("\"accepted\":true"));

    let finish = route_request(&request("POST", "/finish", ""), Arc::clone(&state));
    assert!(finish.contains("HTTP/1.1 200 OK"));
    assert!(finish.contains("\"finished\":true"));
    let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
    assert!(timeline.contains("external_fault_injected"));
    assert!(timeline.contains("mqtt.local"));

    let run = route_request(&request("POST", "/run", ""), Arc::clone(&state));
    assert!(run.contains("HTTP/1.1 200 OK"));

    let json = route_request(
        &request("GET", "/reports/latest.json", ""),
        Arc::clone(&state),
    );
    assert!(json.contains("\"scenario_name\""));

    let markdown = route_request(
        &request("GET", "/reports/latest.md", ""),
        Arc::clone(&state),
    );
    assert!(markdown.contains("# roomci Report"));

    let junit = route_request(
        &request("GET", "/reports/latest.junit.xml", ""),
        Arc::clone(&state),
    );
    assert!(junit.contains("<testsuite"));
}

#[test]
fn external_bms_contact_updates_state_and_timeline() {
    let state = serve_state();

    let response = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            r#"{"source":"contact.sauna_emergency_button","state":"on","severity":"critical"}"#,
        ),
        Arc::clone(&state),
    );

    assert!(response.contains("HTTP/1.1 202 Accepted"));
    assert!(response.contains("\"accepted\":true"));

    let current_state = route_request(&request("GET", "/state", ""), Arc::clone(&state));
    // The observation lives in its own `external_observations` bucket so
    // it does not pollute device-state in `final_state`.
    assert!(current_state.contains("\"external_observations\""));
    assert!(current_state.contains("contact.sauna_emergency_button"));
    assert!(current_state.contains("\"severity\":\"critical\""));

    let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
    assert!(timeline.contains("external_bms_contact_observed"));
    assert!(timeline.contains("contact.sauna_emergency_button"));
}

#[test]
fn external_events_survive_run_boundary() {
    let state = serve_state();

    let bms = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            r#"{"source":"contact.sauna_emergency_button","state":"on","severity":"critical"}"#,
        ),
        Arc::clone(&state),
    );
    assert!(bms.contains("HTTP/1.1 202 Accepted"));

    let fault = route_request(
        &request(
            "POST",
            "/fault",
            r#"{"target":"mqtt.local","type":"offline"}"#,
        ),
        Arc::clone(&state),
    );
    assert!(fault.contains("HTTP/1.1 202 Accepted"));

    let run = route_request(&request("POST", "/run", ""), Arc::clone(&state));
    assert!(run.contains("HTTP/1.1 200 OK"));

    // After /run, the rendered report (latest.md, latest.json, /timeline)
    // must still include the BMS and fault observations that were
    // recorded before the run started.
    let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
    assert!(timeline.contains("external_bms_contact_observed"));
    assert!(timeline.contains("external_fault_injected"));

    // BMS observation is merged into final_state with the documented
    // prefix. The JSON report serializes final_state, so the prefix-merged
    // key must be visible to a CI report consumer.
    let json_report = route_request(
        &request("GET", "/reports/latest.json", ""),
        Arc::clone(&state),
    );
    assert!(json_report.contains("external.bms.contact.sauna_emergency_button"));

    // Markdown rendering renders the timeline, so the BMS event's
    // sanitized target still appears in latest.md even though
    // `final_state` is not rendered there.
    let markdown = route_request(
        &request("GET", "/reports/latest.md", ""),
        Arc::clone(&state),
    );
    assert!(markdown.contains("external_bms_contact_observed"));
    assert!(markdown.contains("contact.sauna_emergency_button"));

    // After /run, the overlays are drained so a follow-up /state no
    // longer reports them under external_observations.
    let after_state = route_request(&request("GET", "/state", ""), Arc::clone(&state));
    assert!(after_state.contains("\"external_observations\":{}"));
    // But the prefix-merged observation lives in final_state now.
    assert!(after_state.contains("external.bms.contact.sauna_emergency_button"));
}

#[test]
fn overlapped_run_state_bms_and_report_requests_leave_consistent_reports() {
    let state = serve_state();
    let requests = [
        request("POST", "/run", ""),
        request("GET", "/state", ""),
        request(
            "POST",
            "/external/bms/contact",
            r#"{"source":"contact.sauna_emergency_button","state":"on","severity":"critical","replay_id":"phase24-overlap"}"#,
        ),
        request("GET", "/reports/latest.json", ""),
    ];

    let responses = requests
        .into_iter()
        .map(|http_request| {
            let state = Arc::clone(&state);
            thread::spawn(move || route_request(&http_request, state))
        })
        .map(|handle| {
            handle
                .join()
                .expect("serve request thread should not panic")
        })
        .collect::<Vec<_>>();

    assert!(responses
        .iter()
        .all(|response| !response.contains("serve_state_poisoned")));
    assert!(responses.iter().any(|response| {
        response.contains("HTTP/1.1 200 OK") && response.contains("\"finished\":true")
    }));
    assert!(responses
        .iter()
        .any(|response| response.contains("HTTP/1.1 202 Accepted")));

    let report = route_request(
        &request("GET", "/reports/latest.json", ""),
        Arc::clone(&state),
    );
    assert!(report.contains("HTTP/1.1 200 OK"));
    assert!(report.contains("\"scenario_name\""));
    assert!(report.contains("generic_mqtt_retained_state"));

    let current_state = route_request(&request("GET", "/state", ""), Arc::clone(&state));
    assert!(current_state.contains("HTTP/1.1 200 OK"));
    assert!(current_state.contains("generic_mqtt_retained_state"));
}

#[test]
fn external_bms_contact_sanitizes_source_and_message() {
    let state = serve_state();

    let response = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            "{\"source\":\"weird;value\\ninjected\",\"state\":\"on\\rmore\"}",
        ),
        Arc::clone(&state),
    );
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    // Response uses sanitized source/state values.
    assert!(response.contains("\"source\":\"weird_value_injected\""));
    assert!(response.contains("\"state\":\"on_more\""));

    let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
    // Newline and carriage return are replaced with spaces in messages so
    // Markdown rendering cannot be hijacked by an external client.
    assert!(!timeline.contains("\\n"));
    assert!(!timeline.contains("\\r"));
}

#[test]
fn external_bms_contact_rejects_invalid_payloads() {
    let state = serve_state();

    // Missing `source` returns a documented 400 with a stable error code.
    let missing_source = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            r#"{"state":"on","severity":"critical"}"#,
        ),
        Arc::clone(&state),
    );
    assert!(missing_source.contains("HTTP/1.1 400 Bad Request"));
    assert!(missing_source.contains("missing_source"));

    // Missing `state` returns a separate, distinguishable error code so
    // controllers can branch on the failure shape.
    let missing_state = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            r#"{"source":"contact.sauna_emergency_button"}"#,
        ),
        Arc::clone(&state),
    );
    assert!(missing_state.contains("HTTP/1.1 400 Bad Request"));
    assert!(missing_state.contains("missing_state"));

    // A non-string `source` is treated the same as missing — the handler
    // only accepts string source identifiers.
    let non_string_source = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            r#"{"source":42,"state":"on"}"#,
        ),
        Arc::clone(&state),
    );
    assert!(non_string_source.contains("HTTP/1.1 400 Bad Request"));
    assert!(non_string_source.contains("missing_source"));

    // Malformed JSON is reported with the `invalid_json` error code so
    // the controller can distinguish parse failures from validation
    // failures.
    let bad_json = route_request(
        &request("POST", "/external/bms/contact", "not-json{"),
        Arc::clone(&state),
    );
    assert!(bad_json.contains("HTTP/1.1 400 Bad Request"));
    assert!(bad_json.contains("invalid_json"));

    // None of the rejected payloads should have created any observation
    // state or timeline events.
    let after_state = route_request(&request("GET", "/state", ""), Arc::clone(&state));
    assert!(after_state.contains("\"external_observations\":{}"));
    let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
    assert!(!timeline.contains("external_bms_contact_observed"));
}

#[test]
fn external_bms_contact_enforces_severity_schema_and_replay_id() {
    let state = serve_state();

    let invalid_severity = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            r#"{"source":"contact.sauna_emergency_button","state":"on","severity":"page-now"}"#,
        ),
        Arc::clone(&state),
    );
    assert!(invalid_severity.contains("HTTP/1.1 400 Bad Request"));
    assert!(invalid_severity.contains("invalid_severity"));

    let invalid_schema = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            r#"{"source":"contact.sauna_emergency_button","state":"on","schema_version":1}"#,
        ),
        Arc::clone(&state),
    );
    assert!(invalid_schema.contains("HTTP/1.1 400 Bad Request"));
    assert!(invalid_schema.contains("invalid_schema_version"));

    let accepted = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            r#"{"source":"contact.sauna_emergency_button","state":"on","severity":"critical","schema_version":"bms.alert.v1","replay_id":"event-001"}"#,
        ),
        Arc::clone(&state),
    );
    assert!(accepted.contains("HTTP/1.1 202 Accepted"));

    let replayed = route_request(
        &request(
            "POST",
            "/external/bms/contact",
            r#"{"source":"contact.sauna_emergency_button","state":"on","severity":"critical","schema_version":"bms.alert.v1","replay_id":"event-001"}"#,
        ),
        Arc::clone(&state),
    );
    assert!(replayed.contains("HTTP/1.1 409 Conflict"));
    assert!(replayed.contains("duplicate_replay_id"));
}

#[test]
fn poisoned_mutex_returns_500_response() {
    let state = serve_state();
    let poison_state = Arc::clone(&state);
    let _ = std::panic::catch_unwind(move || {
        let _guard = poison_state.lock().unwrap();
        panic!("test-only poison");
    });

    let response = route_request(&request("GET", "/health", ""), Arc::clone(&state));
    assert!(response.contains("HTTP/1.1 500 Internal Server Error"));
    assert!(response.contains("serve_state_poisoned"));

    let second_response = route_request(&request("GET", "/state", ""), Arc::clone(&state));
    assert!(second_response.contains("HTTP/1.1 500 Internal Server Error"));
    assert!(second_response.contains("serve_state_poisoned"));
}

#[test]
fn second_run_while_first_in_flight_returns_409() {
    let state = serve_state();
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.run_in_progress = true;
    }

    let response = route_request(&request("POST", "/run", ""), Arc::clone(&state));

    assert!(response.contains("HTTP/1.1 409 Conflict"));
    assert!(response.contains("run_in_progress"));
}

#[test]
fn run_clears_in_progress_flag_after_success() {
    let state = serve_state();

    let response = route_request(&request("POST", "/run", ""), Arc::clone(&state));

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(!state.lock().unwrap().run_in_progress);
}

#[test]
fn http_router_reports_client_errors() {
    let state = serve_state();

    let invalid_json = route_request(&request("POST", "/fault", "not json"), Arc::clone(&state));
    assert!(invalid_json.contains("HTTP/1.1 400 Bad Request"));
    assert!(invalid_json.contains("invalid_json"));

    let not_found = route_request(&request("GET", "/missing", ""), Arc::clone(&state));
    assert!(not_found.contains("HTTP/1.1 404 Not Found"));

    let latest = route_request(
        &request("GET", "/reports/latest?format=json", ""),
        Arc::clone(&state),
    );
    assert!(latest.contains("\"scenario_name\""));
}

#[test]
fn loopback_host_policy_is_local_by_default() {
    assert!(is_loopback_host("127.0.0.1"));
    assert!(is_loopback_host("localhost"));
    assert!(is_loopback_host("::1"));
    assert!(!is_loopback_host("0.0.0.0"));
}
