use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use roomci_core::{run_scenario, RunResult};
use roomci_device_model::ModbusModel;
use roomci_scenario::{
    extract_mqtt_placeholder_value, load_scenario, match_mqtt_contract, ScenarioFile,
};
use serde_json::json;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn serve_state() -> Arc<Mutex<ServeState>> {
    let scenario = load_scenario(fixture("examples/generic_mqtt_retained_state.yaml")).unwrap();
    serve_state_for_scenario(scenario)
}

fn modbus_serve_state() -> Arc<Mutex<ServeState>> {
    let scenario = load_scenario(fixture("examples/modbus_floor_heating.yaml")).unwrap();
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

fn response_json(response: &str) -> serde_json::Value {
    let body = response
        .split_once("\r\n\r\n")
        .expect("HTTP response should include header/body separator")
        .1;
    serde_json::from_str(body).expect("HTTP response body should be JSON")
}

fn start_modbus_test_server() -> SocketAddr {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let state = modbus_serve_state();
    std::thread::spawn(move || serve_modbus(listener, address, state));
    address
}

#[test]
fn mqtt_helpers_parse_and_match_contracts() {
    let topic = "fleet/demo/site/lab/device/env_sensor_01/command";
    let mut packet_payload = Vec::new();
    packet_payload.extend_from_slice(&(topic.len() as u16).to_be_bytes());
    packet_payload.extend_from_slice(topic.as_bytes());
    packet_payload.extend_from_slice(br#"{"online":true,"sample_interval_seconds":15}"#);

    let publish = parse_mqtt_publish(&packet_payload).unwrap();
    assert_eq!(publish.topic, topic);
    assert_eq!(publish.payload["sample_interval_seconds"], json!(15));

    let state = serve_state();
    let contract = &state.lock().unwrap().scenario.mqtt.contracts[0];
    let matched = match_mqtt_contract(contract, topic).unwrap();
    assert_eq!(matched.device_id, "env_sensor_01");
    assert_eq!(
        matched.state_topic,
        "fleet/demo/site/lab/device/env_sensor_01/state"
    );
    assert_eq!(
        extract_mqtt_placeholder_value("a/{device_id}/b", "a/device-01/b", "{device_id}"),
        Some("device-01".to_string())
    );
    assert_eq!(
        extract_mqtt_placeholder_value("a/{device_id}/b", "a/site/device-01/b", "{device_id}"),
        None
    );
}

#[test]
fn external_mqtt_publish_updates_and_rejects_by_contract() {
    let state = serve_state();
    {
        let mut state = state.lock().unwrap();
        apply_external_mqtt_publish(
            &mut state,
            MqttPublish {
                topic: "fleet/demo/site/lab/device/env_sensor_01/command".to_string(),
                payload: BTreeMap::from([
                    ("online".to_string(), json!(true)),
                    ("sample_interval_seconds".to_string(), json!(15)),
                ]),
            },
        );
        assert_eq!(state.external_publish_count, 1);
        assert!(state
            .external_mqtt_final_state
            .contains_key("env_sensor_01"));
        assert!(state
            .external_mqtt_retained_state
            .contains_key("fleet/demo/site/lab/device/env_sensor_01/state"));

        apply_external_mqtt_publish(
            &mut state,
            MqttPublish {
                topic: "fleet/demo/site/lab/device/env_sensor_01/command".to_string(),
                payload: BTreeMap::from([("online".to_string(), json!(true))]),
            },
        );
        apply_external_mqtt_publish(
            &mut state,
            MqttPublish {
                topic: "unknown/topic".to_string(),
                payload: BTreeMap::new(),
            },
        );
        apply_external_mqtt_publish(
            &mut state,
            MqttPublish {
                topic: "fleet/demo/site/lab/device/env_sensor_01/command".to_string(),
                payload: BTreeMap::from([
                    ("online".to_string(), json!("yes")),
                    ("sample_interval_seconds".to_string(), json!(15)),
                ]),
            },
        );
    }

    let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
    assert!(timeline.contains("external_mqtt_retained_state_updated"));
    assert!(timeline.contains("payload missing required fields"));
    assert!(timeline.contains("topic did not match"));
    assert!(timeline.contains("payload field online is invalid"));
    assert!(timeline.contains("expected boolean"));
}

#[test]
fn mqtt_contract_required_fields_match_run_and_serve() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: mqtt_contract_same_decision
mqtt:
  local:
    retained: true
  contracts:
    - name: device_state
      command_topic: fleet/demo/device/{device_id}/command
      state_topic: fleet/demo/device/{device_id}/state
      payload:
        required_fields: [online, sample_interval_seconds]
devices:
  - id: env_sensor_01
    type: sensor
    protocol: mqtt
    state:
      online: false
      sample_interval_seconds: 60
steps:
  - at: T
    mqtt_publish:
      client: edge_contract_test
      topic: fleet/demo/device/env_sensor_01/command
      payload:
        online: true

assertions:
  - at: T+1s
    target: user_override
    condition: false
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();
    assert_eq!(report.result, RunResult::Failed);
    let run_message = report
        .assertions
        .iter()
        .find(|assertion| assertion.assertion_type == "mqtt_contract")
        .map(|assertion| assertion.message.clone())
        .expect("run should record MQTT contract failure");

    let state = serve_state_for_scenario(scenario);
    {
        let mut state = state.lock().unwrap();
        apply_external_mqtt_publish(
            &mut state,
            MqttPublish {
                topic: "fleet/demo/device/env_sensor_01/command".to_string(),
                payload: BTreeMap::from([("online".to_string(), json!(true))]),
            },
        );
        let serve_message = state
            .external_observation_timeline
            .last()
            .map(|event| event.message.clone())
            .expect("serve should record MQTT contract rejection");
        assert_eq!(serve_message, run_message);
    }
}

#[test]
fn external_mqtt_retained_state_is_visible_in_state_before_run() {
    let state = serve_state();
    {
        let mut state = state.lock().unwrap();
        apply_external_mqtt_publish(
            &mut state,
            MqttPublish {
                topic: "fleet/demo/site/lab/device/env_sensor_01/command".to_string(),
                payload: BTreeMap::from([
                    ("online".to_string(), json!(true)),
                    ("sample_interval_seconds".to_string(), json!(15)),
                ]),
            },
        );
    }

    let state_response = route_request(&request("GET", "/state", ""), Arc::clone(&state));
    let state_json = response_json(&state_response);
    assert_eq!(
        state_json["final_state"]["env_sensor_01"]["sample_interval_seconds"],
        json!(15)
    );
    assert_eq!(
        state_json["retained_messages"]["fleet/demo/site/lab/device/env_sensor_01/state"]
            ["sample_interval_seconds"],
        json!(15)
    );
}

#[test]
fn external_mqtt_retained_state_survives_run_boundary() {
    let state = serve_state();

    // Apply an external MQTT publish before /run
    {
        let mut state = state.lock().unwrap();
        apply_external_mqtt_publish(
            &mut state,
            MqttPublish {
                topic: "fleet/demo/site/lab/device/env_sensor_01/command".to_string(),
                payload: BTreeMap::from([
                    ("online".to_string(), json!(true)),
                    ("sample_interval_seconds".to_string(), json!(15)),
                ]),
            },
        );
        // Before /run, the message is in external_mqtt_retained_state, not latest_report.retained_messages
        assert_eq!(state.external_mqtt_final_state.len(), 1);
        assert_eq!(state.external_mqtt_retained_state.len(), 1);
    }

    // Call /run
    let run = route_request(&request("POST", "/run", ""), Arc::clone(&state));
    assert!(run.contains("HTTP/1.1 200 OK"));

    // After /run, the retained message should appear in /state
    let state_response = route_request(&request("GET", "/state", ""), Arc::clone(&state));
    let state_json = response_json(&state_response);
    assert_eq!(
        state_json["final_state"]["env_sensor_01"]["sample_interval_seconds"],
        json!(15)
    );
    assert_eq!(
        state_json["retained_messages"]["fleet/demo/site/lab/device/env_sensor_01/state"]
            ["sample_interval_seconds"],
        json!(15)
    );

    // And in /reports/latest.json
    let json_report = route_request(
        &request("GET", "/reports/latest.json", ""),
        Arc::clone(&state),
    );
    let report_json = response_json(&json_report);
    assert_eq!(
        report_json["final_state"]["env_sensor_01"]["sample_interval_seconds"],
        json!(15)
    );
    assert_eq!(
        report_json["retained_messages"]["fleet/demo/site/lab/device/env_sensor_01/state"]
            ["sample_interval_seconds"],
        json!(15)
    );

    // Timeline still shows the MQTT event
    let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
    assert!(timeline.contains("external_mqtt_retained_state_updated"));

    // After /run, the overlay is drained
    {
        let state = state.lock().unwrap();
        assert_eq!(state.external_mqtt_final_state.len(), 0);
        assert_eq!(state.external_mqtt_retained_state.len(), 0);
        assert!(state
            .latest_report
            .final_state
            .contains_key("env_sensor_01"));
        assert!(state
            .latest_report
            .retained_messages
            .contains_key("fleet/demo/site/lab/device/env_sensor_01/state"));
    }
}

#[test]
fn modbus_tcp_reads_and_writes_register_subset() {
    let state = modbus_serve_state();
    let read_holding = modbus_request(1, 1, 0x03, &[0x00, 0x00, 0x00, 0x01]);
    let response = {
        let request = parse_modbus_request_for_test(&read_holding);
        let mut state = state.lock().unwrap();
        apply_modbus_tcp_request(&mut state, request)
    };
    assert_eq!(response, modbus_response(1, 1, 0x03, &[0x02, 0x00, 0xF5]));

    let write = modbus_request(2, 1, 0x06, &[0x00, 0x00, 0x00, 0xFA]);
    let response = {
        let request = parse_modbus_request_for_test(&write);
        let mut state = state.lock().unwrap();
        apply_modbus_tcp_request(&mut state, request)
    };
    assert_eq!(
        response,
        modbus_response(2, 1, 0x06, &[0x00, 0x00, 0x00, 0xFA])
    );

    let current_state = route_request(&request("GET", "/state", ""), Arc::clone(&state));
    assert!(current_state.contains("modbus.floor_heating_01.40001"));
    assert!(current_state.contains("\"readable_value\":25.0"));
}

#[test]
fn modbus_tcp_reads_contiguous_registers() {
    let state = modbus_serve_state();
    let read_holding = modbus_request(1, 1, 0x03, &[0x00, 0x00, 0x00, 0x02]);
    let response = {
        let request = parse_modbus_request_for_test(&read_holding);
        let mut state = state.lock().unwrap();
        apply_modbus_tcp_request(&mut state, request)
    };
    assert_eq!(
        response,
        modbus_response(1, 1, 0x03, &[0x04, 0x00, 0xF5, 0x00, 0xD2])
    );

    let read_input = modbus_request(2, 1, 0x04, &[0x00, 0x00, 0x00, 0x02]);
    let response = {
        let request = parse_modbus_request_for_test(&read_input);
        let mut state = state.lock().unwrap();
        apply_modbus_tcp_request(&mut state, request)
    };
    assert_eq!(
        response,
        modbus_response(2, 1, 0x04, &[0x04, 0x00, 0xE4, 0x00, 0xDD])
    );
}

#[test]
fn modbus_tcp_returns_exceptions_for_unsupported_and_invalid_requests() {
    let state = modbus_serve_state();
    let unsupported = modbus_request(1, 1, 0x10, &[0x00, 0x00]);
    let response = {
        let request = parse_modbus_request_for_test(&unsupported);
        let mut state = state.lock().unwrap();
        apply_modbus_tcp_request(&mut state, request)
    };
    assert_eq!(
        response,
        modbus_exception_for_test(1, 1, 0x10, MODBUS_EXCEPTION_ILLEGAL_FUNCTION)
    );

    let read_missing = modbus_request(2, 1, 0x03, &[0x00, 0x63, 0x00, 0x01]);
    let response = {
        let request = parse_modbus_request_for_test(&read_missing);
        let mut state = state.lock().unwrap();
        apply_modbus_tcp_request(&mut state, request)
    };
    assert_eq!(
        response,
        modbus_exception_for_test(2, 1, 0x03, MODBUS_EXCEPTION_ILLEGAL_ADDRESS)
    );

    let read_gap = modbus_request(4, 1, 0x03, &[0x00, 0x01, 0x00, 0x02]);
    let response = {
        let request = parse_modbus_request_for_test(&read_gap);
        let mut state = state.lock().unwrap();
        apply_modbus_tcp_request(&mut state, request)
    };
    assert_eq!(
        response,
        modbus_exception_for_test(4, 1, 0x03, MODBUS_EXCEPTION_ILLEGAL_ADDRESS)
    );

    let read_zero = modbus_request(5, 1, 0x03, &[0x00, 0x00, 0x00, 0x00]);
    let response = {
        let request = parse_modbus_request_for_test(&read_zero);
        let mut state = state.lock().unwrap();
        apply_modbus_tcp_request(&mut state, request)
    };
    assert_eq!(
        response,
        modbus_exception_for_test(5, 1, 0x03, MODBUS_EXCEPTION_ILLEGAL_VALUE)
    );

    let write_read_only = modbus_request(3, 1, 0x06, &[0x75, 0x31, 0x00, 0xE5]);
    let response = {
        let request = parse_modbus_request_for_test(&write_read_only);
        let mut state = state.lock().unwrap();
        apply_modbus_tcp_request(&mut state, request)
    };
    assert_eq!(
        response,
        modbus_exception_for_test(3, 1, 0x06, MODBUS_EXCEPTION_ILLEGAL_VALUE)
    );
}

#[test]
fn modbus_tcp_server_handles_standard_mbap_request() {
    let address = start_modbus_test_server();
    let response = modbus_tcp_roundtrip(
        address,
        &modbus_request(7, 1, 0x03, &[0x00, 0x00, 0x00, 0x01]),
    );
    assert_eq!(response, modbus_response(7, 1, 0x03, &[0x02, 0x00, 0xF5]));
}

#[test]
fn mqtt_client_handler_accepts_connect_and_publish() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let state = serve_state();
    let server_state = Arc::clone(&state);
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        handle_mqtt_client(stream, server_state).unwrap();
    });

    let mut stream = TcpStream::connect(address).unwrap();
    stream.write_all(&mqtt_connect_packet("unit-test")).unwrap();
    let mut connack = [0_u8; 4];
    stream.read_exact(&mut connack).unwrap();
    assert_eq!(connack, [0x20, 0x02, 0x00, 0x00]);
    stream
        .write_all(&mqtt_publish_packet(
            "fleet/demo/site/lab/device/env_sensor_01/command",
            br#"{"online":true,"sample_interval_seconds":15}"#,
        ))
        .unwrap();
    drop(stream);
    server.join().unwrap();

    let state = state.lock().unwrap();
    assert_eq!(state.external_publish_count, 1);
}

#[test]
fn mqtt_client_handler_subscribes_and_receives_retained_replay() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let state = serve_state();
    let server_state = Arc::clone(&state);
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        handle_mqtt_client(stream, server_state).unwrap();
    });

    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(&mqtt_connect_packet("unit-test")).unwrap();
    let mut connack = [0_u8; 4];
    stream.read_exact(&mut connack).unwrap();
    assert_eq!(connack, [0x20, 0x02, 0x00, 0x00]);
    stream
        .write_all(&mqtt_subscribe_packet(
            7,
            "fleet/demo/site/lab/device/env_sensor_01/state",
        ))
        .unwrap();

    let mut suback = [0_u8; 5];
    stream.read_exact(&mut suback).unwrap();
    assert_eq!(suback, [0x90, 0x03, 0x00, 0x07, 0x00]);
    let publish = read_mqtt_packet(&mut stream).unwrap().unwrap();
    assert_eq!(publish.packet_type, 3);
    let retained = parse_mqtt_publish(&publish.payload).unwrap();
    assert_eq!(
        retained.topic,
        "fleet/demo/site/lab/device/env_sensor_01/state"
    );
    assert_eq!(retained.payload["sample_interval_seconds"], json!(30));
    drop(stream);
    server.join().unwrap();
}

#[test]
fn mqtt_subscribe_rejects_unsupported_qos() {
    let subscribe = parse_mqtt_subscribe(&mqtt_subscribe_payload(
        9,
        "fleet/demo/site/lab/device/env_sensor_01/state",
        1,
    ))
    .unwrap();
    let replay = mqtt_retained_replay_for_subscribe(&serve_state().lock().unwrap(), &subscribe);

    assert_eq!(replay.return_codes, vec![0x80]);
    assert!(replay.publishes.is_empty());
}

#[test]
fn mqtt_connect_with_legacy_protocol_name_is_rejected() {
    let connack = mqtt_connack_for(mqtt_connect_packet_with("MQIsdp", 3, "legacy"));

    assert_eq!(
        connack,
        [0x20, 0x02, 0x00, MQTT_CONNACK_UNACCEPTABLE_PROTOCOL]
    );
}

#[test]
fn mqtt_connect_with_unsupported_level_is_rejected() {
    let connack = mqtt_connack_for(mqtt_connect_packet_with("MQTT", 5, "mqtt5"));

    assert_eq!(
        connack,
        [0x20, 0x02, 0x00, MQTT_CONNACK_UNACCEPTABLE_PROTOCOL]
    );
}

#[test]
fn mqtt_connect_with_truncated_header_closes_connection() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let state = serve_state();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        assert!(handle_mqtt_client(stream, state).is_err());
    });

    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(&[0x10, 0x02, 0x00]).unwrap();
    drop(stream);

    server.join().unwrap();
}

#[test]
fn mqtt_packet_parser_reports_malformed_inputs() {
    assert!(parse_mqtt_publish(&[]).is_err());
    assert!(parse_mqtt_publish(&[0x00, 0x10, b'a']).is_err());
    let mut payload = Vec::new();
    payload.extend_from_slice(&[0x00, 0x03]);
    payload.extend_from_slice(b"a/b");
    payload.extend_from_slice(b"not-json");
    assert!(parse_mqtt_publish(&payload).is_err());
}

#[test]
fn mqtt_payload_parsers_do_not_panic_on_bounded_malformed_inputs() {
    for len in 0..32 {
        let bytes = (0..len)
            .map(|index| (index * 17 % 251) as u8)
            .collect::<Vec<_>>();

        let publish_result = std::panic::catch_unwind(|| parse_mqtt_publish(&bytes));
        assert!(
            publish_result.is_ok(),
            "publish parser panicked for len {len}"
        );

        let subscribe_result = std::panic::catch_unwind(|| parse_mqtt_subscribe(&bytes));
        assert!(
            subscribe_result.is_ok(),
            "subscribe parser panicked for len {len}"
        );
    }
}

#[test]
fn modbus_request_handler_returns_protocol_responses_for_bounded_payload_shapes() {
    for function in [0x03, 0x04, 0x06, 0x10] {
        for len in 0..8 {
            let payload = (0..len)
                .map(|index| (index * 29 % 251) as u8)
                .collect::<Vec<_>>();
            let state = modbus_serve_state();
            let request = ModbusTcpRequest {
                transaction_id: len as u16,
                unit_id: 1,
                function,
                payload,
            };

            let response = {
                let mut state = state.lock().unwrap();
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    apply_modbus_tcp_request(&mut state, request)
                }))
            };

            let response = response.expect("Modbus handler should not panic");
            assert!(
                response.len() >= 9,
                "Modbus response should include MBAP and PDU for function {function:#04x} len {len}"
            );
        }
    }
}

#[test]
fn request_size_guards_reject_large_inputs() {
    let request = format!(
        "POST /fault HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
        MAX_HTTP_BODY_BYTES + 1
    );
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let client = thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(request.as_bytes()).unwrap();
    });
    let (mut stream, _) = listener.accept().unwrap();
    assert!(read_http_request(&mut stream).is_err());
    client.join().unwrap();

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let client = thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(&[0x30, 0xff, 0xff, 0xff, 0x7f]).unwrap();
    });
    let (mut stream, _) = listener.accept().unwrap();
    assert!(read_mqtt_packet(&mut stream).is_err());
    client.join().unwrap();
}

#[test]
fn http_request_rejects_short_body_and_ignores_extra_bytes() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let client = thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .write_all(b"POST /fault HTTP/1.1\r\nContent-Length: 9\r\n\r\n{}")
            .unwrap();
    });
    let (mut stream, _) = listener.accept().unwrap();
    assert!(read_http_request(&mut stream).is_err());
    client.join().unwrap();

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let client = thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .write_all(b"POST /fault HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}EXTRA")
            .unwrap();
    });
    let (mut stream, _) = listener.accept().unwrap();
    let request = read_http_request(&mut stream).unwrap();
    assert_eq!(request.body, "{}");
    client.join().unwrap();
}

fn mqtt_connect_packet(client_id: &str) -> Vec<u8> {
    mqtt_connect_packet_with("MQTT", 0x04, client_id)
}

fn mqtt_connect_packet_with(protocol_name: &str, protocol_level: u8, client_id: &str) -> Vec<u8> {
    let mut variable = Vec::new();
    variable.extend_from_slice(&(protocol_name.len() as u16).to_be_bytes());
    variable.extend_from_slice(protocol_name.as_bytes());
    variable.push(protocol_level);
    variable.push(0x02);
    variable.extend_from_slice(&60_u16.to_be_bytes());
    variable.extend_from_slice(&(client_id.len() as u16).to_be_bytes());
    variable.extend_from_slice(client_id.as_bytes());

    let mut packet = vec![0x10];
    encode_mqtt_remaining_length(variable.len(), &mut packet);
    packet.extend(variable);
    packet
}

fn mqtt_connack_for(connect_packet: Vec<u8>) -> [u8; 4] {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let state = serve_state();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        handle_mqtt_client(stream, state).unwrap();
    });

    let mut stream = TcpStream::connect(address).unwrap();
    stream.write_all(&connect_packet).unwrap();
    let mut connack = [0_u8; 4];
    stream.read_exact(&mut connack).unwrap();
    drop(stream);
    server.join().unwrap();
    connack
}

fn mqtt_publish_packet(topic: &str, payload: &[u8]) -> Vec<u8> {
    let mut variable = Vec::new();
    variable.extend_from_slice(&(topic.len() as u16).to_be_bytes());
    variable.extend_from_slice(topic.as_bytes());
    variable.extend_from_slice(payload);

    let mut packet = vec![0x30];
    encode_mqtt_remaining_length(variable.len(), &mut packet);
    packet.extend(variable);
    packet
}

fn mqtt_subscribe_packet(packet_id: u16, topic_filter: &str) -> Vec<u8> {
    let payload = mqtt_subscribe_payload(packet_id, topic_filter, 0);
    let mut packet = vec![0x82];
    encode_mqtt_remaining_length(payload.len(), &mut packet);
    packet.extend(payload);
    packet
}

fn mqtt_subscribe_payload(packet_id: u16, topic_filter: &str, qos: u8) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&packet_id.to_be_bytes());
    payload.extend_from_slice(&(topic_filter.len() as u16).to_be_bytes());
    payload.extend_from_slice(topic_filter.as_bytes());
    payload.push(qos);
    payload
}

fn encode_mqtt_remaining_length(mut length: usize, packet: &mut Vec<u8>) {
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

fn modbus_request(transaction: u16, unit: u8, function: u8, payload: &[u8]) -> Vec<u8> {
    let mut request = Vec::with_capacity(8 + payload.len());
    request.extend_from_slice(&transaction.to_be_bytes());
    request.extend_from_slice(&0_u16.to_be_bytes());
    request.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
    request.push(unit);
    request.push(function);
    request.extend_from_slice(payload);
    request
}

fn modbus_response(transaction: u16, unit: u8, function: u8, payload: &[u8]) -> Vec<u8> {
    modbus_tcp_response(transaction, unit, &[&[function], payload].concat())
}

fn modbus_exception_for_test(transaction: u16, unit: u8, function: u8, exception: u8) -> Vec<u8> {
    modbus_tcp_response(transaction, unit, &[function | 0x80, exception])
}

fn parse_modbus_request_for_test(bytes: &[u8]) -> ModbusTcpRequest {
    let transaction_id = u16::from_be_bytes([bytes[0], bytes[1]]);
    let unit_id = bytes[6];
    let function = bytes[7];
    ModbusTcpRequest {
        transaction_id,
        unit_id,
        function,
        payload: bytes[8..].to_vec(),
    }
}

fn modbus_tcp_roundtrip(address: SocketAddr, request: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(request).unwrap();
    let mut header = [0_u8; 7];
    stream.read_exact(&mut header).unwrap();
    let length = u16::from_be_bytes([header[4], header[5]]) as usize;
    let mut pdu = vec![0_u8; length - 1];
    stream.read_exact(&mut pdu).unwrap();
    let mut response = header.to_vec();
    response.extend(pdu);
    response
}
