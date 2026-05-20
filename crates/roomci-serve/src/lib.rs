//! Local `roomci serve` runtime for HTTP and MQTT-based PoC integrations.
//!
//! This crate owns the long-running localhost service used by external
//! controllers. It intentionally implements a narrow PoC surface rather than a
//! production HTTP server or MQTT broker.

use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread,
    time::Duration,
};

use roomci_core::{run_scenario, RunReport, TimelineEvent};
use roomci_report::{to_json, to_junit, to_markdown};
use roomci_scenario::{MqttConnectionContract, ScenarioFile};
use serde_json::json;
use thiserror::Error;

const MAX_HTTP_BODY_BYTES: usize = 1024 * 1024;
const MAX_MQTT_PACKET_BYTES: usize = 1024 * 1024;
const HTTP_CLIENT_TIMEOUT_SECS: u64 = 2;
const HTTP_MAX_INFLIGHT_CONNECTIONS: usize = 32;
const MQTT_PROTOCOL_NAME: &str = "MQTT";
const MQTT_PROTOCOL_LEVEL_3_1_1: u8 = 4;
const MQTT_CONNACK_ACCEPTED: u8 = 0x00;
const MQTT_CONNACK_UNACCEPTABLE_PROTOCOL: u8 = 0x01;
const EXTERNAL_BMS_STATE_PREFIX: &str = "external.bms.";

/// Configuration for a `roomci serve` process.
#[derive(Debug)]
pub struct ServeOptions {
    /// Loaded and validated scenario used as the runtime configuration.
    pub scenario: ScenarioFile,
    /// HTTP bind host. Loopback is enforced unless `allow_non_loopback` is true.
    pub host: String,
    /// HTTP bind port. Use `0` to let the OS choose a free port.
    pub port: u16,
    /// Optional MQTT PoC ingress port.
    pub mqtt_port: Option<u16>,
    /// Allow binding the HTTP and MQTT sockets to a non-loopback host.
    pub allow_non_loopback: bool,
}

/// Errors emitted by the `roomci serve` runtime.
#[derive(Debug, Error)]
pub enum ServeError {
    /// Scenario execution failed during initial state creation or `/run`.
    #[error(transparent)]
    Core(#[from] roomci_core::CoreError),
    /// JSON rendering failed while creating a serve response.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// The serve runtime rejected or failed an HTTP/MQTT operation.
    #[error("serve error: {0}")]
    Runtime(String),
}

/// Run the blocking HTTP/MQTT serve runtime.
pub fn run_serve(options: ServeOptions) -> Result<(), ServeError> {
    if !options.allow_non_loopback && !is_loopback_host(&options.host) {
        return Err(ServeError::Runtime(format!(
            "refusing to bind non-loopback host {}; pass --allow-non-loopback to override",
            options.host
        )));
    }
    serve_http(
        options.scenario,
        &options.host,
        options.port,
        options.mqtt_port,
    )
}

struct ServeState {
    scenario: ScenarioFile,
    latest_report: RunReport,
    injected_faults: Vec<serde_json::Value>,
    external_publish_count: u64,
    run_in_progress: bool,
    has_completed_run: bool,
    /// External-observation timeline events queued for the *next* `/run`
    /// boundary. Persists across `/run` so that events observed during a run
    /// are still visible in `/timeline` and the rendered reports rather than
    /// silently clobbered when `latest_report` is replaced.
    external_observation_timeline: Vec<TimelineEvent>,
    /// BMS / contact observations posted to `/external/bms/contact`. Keyed by
    /// the sanitized `source` field. Survives across `/run` and is merged into
    /// `latest_report.final_state` (with the `external.bms.` prefix) at the
    /// next `/run` boundary.
    external_observations: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
}

fn serve_http(
    scenario: ScenarioFile,
    host: &str,
    port: u16,
    mqtt_port: Option<u16>,
) -> Result<(), ServeError> {
    let latest_report = run_scenario(&scenario)?;
    let state = Arc::new(Mutex::new(ServeState {
        scenario,
        latest_report,
        injected_faults: Vec::new(),
        external_publish_count: 0,
        run_in_progress: false,
        has_completed_run: false,
        external_observation_timeline: Vec::new(),
        external_observations: BTreeMap::new(),
    }));
    let listener =
        TcpListener::bind((host, port)).map_err(|error| ServeError::Runtime(error.to_string()))?;
    let address = listener
        .local_addr()
        .map_err(|error| ServeError::Runtime(error.to_string()))?;

    println!("roomci serve listening on http://{address}");
    println!("endpoints: /health /scenario /state /timeline /reports/latest.json");
    if let Some(mqtt_port) = mqtt_port {
        let mqtt_listener = TcpListener::bind((host, mqtt_port))
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
        let mqtt_address = mqtt_listener
            .local_addr()
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
        println!("roomci mqtt listening on mqtt://{mqtt_address}");
        let mqtt_state = Arc::clone(&state);
        thread::spawn(move || serve_mqtt(mqtt_listener, mqtt_address, mqtt_state));
    }
    std::io::stdout()
        .flush()
        .map_err(|error| ServeError::Runtime(error.to_string()))?;

    serve_http_listener(listener, state);

    Ok(())
}

fn serve_http_listener(listener: TcpListener, state: Arc<Mutex<ServeState>>) {
    let inflight = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let current = inflight.fetch_add(1, Ordering::SeqCst);
                if current >= HTTP_MAX_INFLIGHT_CONNECTIONS {
                    inflight.fetch_sub(1, Ordering::SeqCst);
                    let response = raw_response(
                        503,
                        "application/json",
                        r#"{"error":"too_many_connections"}"#,
                    );
                    if let Err(error) = stream.write_all(response.as_bytes()) {
                        eprintln!("serve overload response error: {error}");
                    }
                    continue;
                }
                let state = Arc::clone(&state);
                let inflight = Arc::clone(&inflight);
                thread::spawn(move || {
                    let _guard = InflightGuard { inflight };
                    if let Err(error) = configure_http_stream(&stream)
                        .and_then(|()| handle_connection(stream, state))
                    {
                        eprintln!("serve request error: {error}");
                    }
                });
            }
            Err(error) => eprintln!("serve accept error: {error}"),
        }
    }
}

fn handle_connection(
    mut stream: TcpStream,
    state: Arc<Mutex<ServeState>>,
) -> Result<(), ServeError> {
    let request = read_http_request(&mut stream)?;
    let response = route_request(&request, state);
    stream
        .write_all(response.as_bytes())
        .map_err(|error| ServeError::Runtime(error.to_string()))
}

struct InflightGuard {
    inflight: Arc<AtomicUsize>,
}

impl Drop for InflightGuard {
    fn drop(&mut self) {
        self.inflight.fetch_sub(1, Ordering::SeqCst);
    }
}

fn configure_http_stream(stream: &TcpStream) -> Result<(), ServeError> {
    let timeout = Some(Duration::from_secs(HTTP_CLIENT_TIMEOUT_SECS));
    stream
        .set_read_timeout(timeout)
        .map_err(|error| ServeError::Runtime(error.to_string()))?;
    stream
        .set_write_timeout(timeout)
        .map_err(|error| ServeError::Runtime(error.to_string()))
}

struct HttpRequest {
    method: String,
    path: String,
    body: String,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, ServeError> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream
            .read(&mut chunk)
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > 64 * 1024 {
            return Err(ServeError::Runtime("request header too large".to_string()));
        }
    }

    let header_end = buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
        .ok_or_else(|| ServeError::Runtime("malformed HTTP request".to_string()))?;
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let mut lines = headers.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| ServeError::Runtime("missing request line".to_string()))?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| ServeError::Runtime("missing method".to_string()))?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| ServeError::Runtime("missing path".to_string()))?
        .to_string();
    let content_length = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    if content_length > MAX_HTTP_BODY_BYTES {
        return Err(ServeError::Runtime(format!(
            "request body too large: {content_length} bytes"
        )));
    }

    while buffer.len() < header_end + content_length {
        let read = stream
            .read(&mut chunk)
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }

    let body = String::from_utf8_lossy(&buffer[header_end..]).to_string();
    Ok(HttpRequest { method, path, body })
}

fn route_request(request: &HttpRequest, state: Arc<Mutex<ServeState>>) -> String {
    match (request.method.as_str(), request.path_without_query()) {
        ("GET", "/health") => {
            let state = match lock_serve_state(&state) {
                Ok(state) => state,
                Err(error) => return serve_error_response(error),
            };
            let status = state.health_status();
            let status_code = if status == "failed" { 503 } else { 200 };
            json_response(
                status_code,
                &json!({
                    "status": status,
                    "scenario": state.scenario.scenario.name,
                    "result": state.latest_report.result,
                    "latest_report_id": format!("{}:latest", state.latest_report.scenario_name),
                    "serve_version": env!("CARGO_PKG_VERSION"),
                }),
            )
        }
        ("GET", "/scenario") => {
            let state = match lock_serve_state(&state) {
                Ok(state) => state,
                Err(error) => return serve_error_response(error),
            };
            json_response(200, &state.scenario)
        }
        ("GET", "/state") => {
            let state = match lock_serve_state(&state) {
                Ok(state) => state,
                Err(error) => return serve_error_response(error),
            };
            json_response(
                200,
                &json!({
                    "scenario": state.scenario.scenario.name,
                    "result": state.latest_report.result,
                    "final_state": state.latest_report.final_state,
                    "retained_messages": state.latest_report.retained_messages,
                    "injected_faults": state.injected_faults,
                    "external_publish_count": state.external_publish_count,
                    "external_observations": state.external_observations,
                }),
            )
        }
        ("GET", "/timeline") => {
            let state = match lock_serve_state(&state) {
                Ok(state) => state,
                Err(error) => return serve_error_response(error),
            };
            let rendered = rendered_report_view(&state);
            json_response(200, &rendered.timeline)
        }
        ("POST", "/fault") => match serde_json::from_str::<serde_json::Value>(&request.body) {
            Ok(fault) => {
                let mut state = match lock_serve_state(&state) {
                    Ok(state) => state,
                    Err(error) => return serve_error_response(error),
                };
                state.injected_faults.push(fault.clone());
                let fault_index = state.injected_faults.len();
                let fault_summary = serde_json::to_string(&fault)
                    .unwrap_or_else(|_| "unrenderable fault".to_string());
                let safe_summary = sanitize_external_message_value(&fault_summary);
                state.external_observation_timeline.push(TimelineEvent {
                    at: format!("external#fault{fault_index}"),
                    event_type: "external_fault_injected".to_string(),
                    target: fault
                        .get("target")
                        .and_then(|value| value.as_str())
                        .map(sanitize_external_message_value),
                    message: format!("external fault accepted: {safe_summary}"),
                });
                json_response(202, &json!({ "accepted": true, "fault": fault }))
            }
            Err(error) => json_response(
                400,
                &json!({ "error": "invalid_json", "message": error.to_string() }),
            ),
        },
        ("POST", "/external/bms/contact") => {
            match serde_json::from_str::<serde_json::Value>(&request.body) {
                Ok(contact) => {
                    let Some(source) = contact.get("source").and_then(|value| value.as_str())
                    else {
                        return json_response(
                            400,
                            &json!({
                                "error": "missing_source",
                                "message": "external BMS contact payload must include string field source"
                            }),
                        );
                    };
                    let Some(contact_state) = contact.get("state").and_then(|value| value.as_str())
                    else {
                        return json_response(
                            400,
                            &json!({
                                "error": "missing_state",
                                "message": "external BMS contact payload must include string field state"
                            }),
                        );
                    };

                    let mut state = match lock_serve_state(&state) {
                        Ok(state) => state,
                        Err(error) => return serve_error_response(error),
                    };
                    let sanitized_source = sanitize_external_key(source);
                    let sanitized_state = sanitize_external_key(contact_state);
                    let mut state_map = BTreeMap::new();
                    state_map.insert(
                        "state".to_string(),
                        serde_json::Value::String(sanitized_state.clone()),
                    );
                    if let Some(severity) = contact.get("severity").and_then(|value| value.as_str())
                    {
                        state_map.insert(
                            "severity".to_string(),
                            serde_json::Value::String(sanitize_external_key(severity)),
                        );
                    }
                    state
                        .external_observations
                        .insert(sanitized_source.clone(), state_map);
                    let event_index = state.external_observation_timeline.len() + 1;
                    let safe_source = sanitize_external_message_value(source);
                    let safe_state = sanitize_external_message_value(contact_state);
                    state.external_observation_timeline.push(TimelineEvent {
                        at: format!("external#bms{event_index}"),
                        event_type: "external_bms_contact_observed".to_string(),
                        target: Some(sanitized_source.clone()),
                        message: format!(
                            "external BMS/contact event observed: {safe_source}={safe_state}"
                        ),
                    });
                    json_response(
                        202,
                        &json!({
                            "accepted": true,
                            "source": sanitized_source,
                            "state": sanitized_state
                        }),
                    )
                }
                Err(error) => json_response(
                    400,
                    &json!({ "error": "invalid_json", "message": error.to_string() }),
                ),
            }
        }
        ("POST", "/finish") => {
            let mut state = match lock_serve_state(&state) {
                Ok(state) => state,
                Err(error) => return serve_error_response(error),
            };
            state.has_completed_run = true;
            json_response(
                200,
                &json!({
                    "finished": true,
                    "result": state.latest_report.result,
                    "external_publish_count": state.external_publish_count,
                }),
            )
        }
        ("POST", "/run") => {
            let scenario = {
                let mut state = match lock_serve_state(&state) {
                    Ok(state) => state,
                    Err(error) => return serve_error_response(error),
                };
                if state.run_in_progress {
                    return json_response(
                        409,
                        &json!({
                            "error": "run_in_progress",
                            "message": "a scenario run is already in progress"
                        }),
                    );
                }
                state.run_in_progress = true;
                state.scenario.clone()
            };

            let run_result = run_scenario(&scenario);
            let mut state = match lock_serve_state(&state) {
                Ok(state) => state,
                Err(error) => return serve_error_response(error),
            };
            state.run_in_progress = false;
            match run_result {
                Ok(mut report) => {
                    let result = report.result;
                    drain_external_overlay_into(&mut state, &mut report);
                    state.latest_report = report;
                    state.injected_faults.clear();
                    state.external_publish_count = 0;
                    state.has_completed_run = true;
                    json_response(200, &json!({ "finished": true, "result": result }))
                }
                Err(error) => json_response(
                    500,
                    &json!({ "error": "run_failed", "message": error.to_string() }),
                ),
            }
        }
        ("GET", "/reports/latest") | ("GET", "/reports/latest.json") => {
            let state = match lock_serve_state(&state) {
                Ok(state) => state,
                Err(error) => return serve_error_response(error),
            };
            let rendered = rendered_report_view(&state);
            match to_json(&rendered) {
                Ok(report) => raw_response(200, "application/json", &report),
                Err(error) => json_response(
                    500,
                    &json!({ "error": "report_render_failed", "message": error.to_string() }),
                ),
            }
        }
        ("GET", "/reports/latest.md") => {
            let state = match lock_serve_state(&state) {
                Ok(state) => state,
                Err(error) => return serve_error_response(error),
            };
            let rendered = rendered_report_view(&state);
            raw_response(200, "text/markdown; charset=utf-8", &to_markdown(&rendered))
        }
        ("GET", "/reports/latest.junit.xml") => {
            let state = match lock_serve_state(&state) {
                Ok(state) => state,
                Err(error) => return serve_error_response(error),
            };
            let rendered = rendered_report_view(&state);
            raw_response(200, "application/xml; charset=utf-8", &to_junit(&rendered))
        }
        _ => json_response(
            404,
            &json!({ "error": "not_found", "message": "unknown endpoint" }),
        ),
    }
}

fn lock_serve_state(
    state: &Arc<Mutex<ServeState>>,
) -> Result<MutexGuard<'_, ServeState>, ServeError> {
    state
        .lock()
        .map_err(|_| ServeError::Runtime("serve_state_poisoned".to_string()))
}

fn serve_error_response(error: ServeError) -> String {
    match error {
        ServeError::Runtime(message) if message == "serve_state_poisoned" => json_response(
            500,
            &json!({
                "error": "serve_state_poisoned",
                "message": "serve state mutex was poisoned"
            }),
        ),
        error => json_response(
            500,
            &json!({ "error": "serve_error", "message": error.to_string() }),
        ),
    }
}

/// Sanitize an external-input key so it can be used in `final_state`,
/// `external_observations`, or timeline messages without breaking downstream
/// renderers. Allows alphanumerics, `.`, `_`, `-`, `:`, `/`. Replaces every
/// other character with `_`, and falls back to `unknown` if the result is
/// empty.
fn sanitize_external_key(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-' | ':' | '/')
            {
                character
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        cleaned
    }
}

/// Sanitize a free-form external-input string for safe inclusion in timeline
/// messages and rendered Markdown reports. Strips control characters
/// (newlines, carriage returns, tabs, etc.) so a malicious or malformed input
/// cannot inject report structure.
fn sanitize_external_message_value(raw: &str) -> String {
    raw.chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

/// Build a view of `latest_report` that includes external observations queued
/// since the last `/run`. The returned report is what `/timeline` and
/// `/reports/latest.*` render.
fn rendered_report_view(state: &ServeState) -> RunReport {
    let mut report = state.latest_report.clone();
    report
        .timeline
        .extend(state.external_observation_timeline.iter().cloned());
    for (key, value) in &state.external_observations {
        let prefixed = format!("{EXTERNAL_BMS_STATE_PREFIX}{key}");
        report.final_state.insert(prefixed, value.clone());
    }
    report
}

/// Merge any external-observation overlay state into a freshly produced
/// `RunReport` and drain the overlay. Called at the `/run` success boundary so
/// that events observed before / during the run are preserved across the
/// `latest_report` replacement.
fn drain_external_overlay_into(state: &mut ServeState, report: &mut RunReport) {
    report
        .timeline
        .append(&mut state.external_observation_timeline);
    for (key, value) in std::mem::take(&mut state.external_observations) {
        let prefixed = format!("{EXTERNAL_BMS_STATE_PREFIX}{key}");
        report.final_state.insert(prefixed, value);
    }
}

impl ServeState {
    fn health_status(&self) -> &'static str {
        if self.run_in_progress {
            return "running";
        }
        if !self.has_completed_run {
            return "idle";
        }
        match self.latest_report.result {
            roomci_core::RunResult::Passed => "passed",
            roomci_core::RunResult::Failed => "failed",
        }
    }
}

fn serve_mqtt(listener: TcpListener, address: SocketAddr, state: Arc<Mutex<ServeState>>) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    if let Err(error) = handle_mqtt_client(stream, state) {
                        eprintln!("mqtt request error on {address}: {error}");
                    }
                });
            }
            Err(error) => eprintln!("mqtt accept error on {address}: {error}"),
        }
    }
}

fn handle_mqtt_client(
    mut stream: TcpStream,
    state: Arc<Mutex<ServeState>>,
) -> Result<(), ServeError> {
    loop {
        let Some(packet) = read_mqtt_packet(&mut stream)? else {
            return Ok(());
        };
        match packet.packet_type {
            1 => {
                let return_code = if validate_mqtt_connect(&packet.payload) {
                    MQTT_CONNACK_ACCEPTED
                } else {
                    MQTT_CONNACK_UNACCEPTABLE_PROTOCOL
                };
                stream
                    .write_all(&[0x20, 0x02, 0x00, return_code])
                    .map_err(|error| ServeError::Runtime(error.to_string()))?;
                if return_code != MQTT_CONNACK_ACCEPTED {
                    return Ok(());
                }
            }
            3 => {
                let publish = parse_mqtt_publish(&packet.payload)?;
                let mut state = lock_serve_state(&state)?;
                apply_external_mqtt_publish(&mut state, publish);
            }
            _ => {
                return Err(ServeError::Runtime(format!(
                    "unsupported MQTT packet type {}",
                    packet.packet_type
                )));
            }
        }
    }
}

fn validate_mqtt_connect(payload: &[u8]) -> bool {
    if payload.len() < 2 {
        return false;
    }
    let protocol_name_len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    let protocol_name_end = 2 + protocol_name_len;
    if payload.len() <= protocol_name_end {
        return false;
    }
    let Ok(protocol_name) = std::str::from_utf8(&payload[2..protocol_name_end]) else {
        return false;
    };
    let protocol_level = payload[protocol_name_end];
    protocol_name == MQTT_PROTOCOL_NAME && protocol_level == MQTT_PROTOCOL_LEVEL_3_1_1
}

struct MqttPacket {
    packet_type: u8,
    payload: Vec<u8>,
}

struct MqttPublish {
    topic: String,
    payload: BTreeMap<String, serde_json::Value>,
}

fn read_mqtt_packet(stream: &mut TcpStream) -> Result<Option<MqttPacket>, ServeError> {
    let mut fixed = [0_u8; 1];
    match stream.read_exact(&mut fixed) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(ServeError::Runtime(error.to_string())),
    }
    let remaining = read_mqtt_remaining_length(stream)?;
    if remaining > MAX_MQTT_PACKET_BYTES {
        return Err(ServeError::Runtime(format!(
            "MQTT packet too large: {remaining} bytes"
        )));
    }
    let mut payload = vec![0_u8; remaining];
    stream
        .read_exact(&mut payload)
        .map_err(|error| ServeError::Runtime(error.to_string()))?;
    Ok(Some(MqttPacket {
        packet_type: fixed[0] >> 4,
        payload,
    }))
}

fn read_mqtt_remaining_length(stream: &mut TcpStream) -> Result<usize, ServeError> {
    let mut multiplier = 1_usize;
    let mut value = 0_usize;
    for _ in 0..4 {
        let mut encoded = [0_u8; 1];
        stream
            .read_exact(&mut encoded)
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
        value += ((encoded[0] & 127) as usize) * multiplier;
        if encoded[0] & 128 == 0 {
            return Ok(value);
        }
        multiplier *= 128;
    }
    Err(ServeError::Runtime(
        "malformed MQTT remaining length".to_string(),
    ))
}

fn parse_mqtt_publish(payload: &[u8]) -> Result<MqttPublish, ServeError> {
    if payload.len() < 2 {
        return Err(ServeError::Runtime("malformed MQTT publish".to_string()));
    }
    let topic_len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + topic_len {
        return Err(ServeError::Runtime("malformed MQTT topic".to_string()));
    }
    let topic = String::from_utf8(payload[2..2 + topic_len].to_vec())
        .map_err(|error| ServeError::Runtime(error.to_string()))?;
    let payload_bytes = &payload[2 + topic_len..];
    let payload = serde_json::from_slice::<BTreeMap<String, serde_json::Value>>(payload_bytes)
        .map_err(|error| {
            ServeError::Runtime(format!("MQTT payload must be JSON object: {error}"))
        })?;
    Ok(MqttPublish { topic, payload })
}

fn apply_external_mqtt_publish(state: &mut ServeState, publish: MqttPublish) {
    state.external_publish_count += 1;
    let event_index = state.external_publish_count;
    let matched_contract = state
        .scenario
        .mqtt
        .contracts
        .iter()
        .find_map(|contract| match_contract(contract, &publish.topic));

    let Some((contract, device_id, state_topic)) = matched_contract else {
        state.external_observation_timeline.push(TimelineEvent {
            at: format!("external#{event_index}"),
            event_type: "external_mqtt_publish_rejected".to_string(),
            target: Some(sanitize_external_message_value(&publish.topic)),
            message: "topic did not match any configured MQTT contract".to_string(),
        });
        return;
    };

    let missing_fields = contract
        .payload
        .required_fields
        .iter()
        .filter(|field| !publish.payload.contains_key(*field))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_fields.is_empty() {
        state.external_observation_timeline.push(TimelineEvent {
            at: format!("external#{event_index}"),
            event_type: "external_mqtt_publish_rejected".to_string(),
            target: Some(sanitize_external_message_value(&publish.topic)),
            message: format!(
                "payload missing required fields for {}: {}",
                sanitize_external_message_value(&contract.name),
                missing_fields.join(", ")
            ),
        });
        return;
    }

    state
        .latest_report
        .final_state
        .insert(device_id.clone(), publish.payload.clone());
    state
        .latest_report
        .retained_messages
        .insert(state_topic.clone(), publish.payload);
    state.external_observation_timeline.push(TimelineEvent {
        at: format!("external#{event_index}"),
        event_type: "external_mqtt_retained_state_updated".to_string(),
        target: Some(sanitize_external_message_value(&device_id)),
        message: format!(
            "external MQTT publish matched {} and updated retained state at {}",
            sanitize_external_message_value(&contract.name),
            sanitize_external_message_value(&state_topic)
        ),
    });
}

fn match_contract<'a>(
    contract: &'a MqttConnectionContract,
    topic: &str,
) -> Option<(&'a MqttConnectionContract, String, String)> {
    let device_id = extract_placeholder_value(&contract.command_topic, topic, "{device_id}")?;
    let state_topic = contract.state_topic.replace("{device_id}", &device_id);
    Some((contract, device_id, state_topic))
}

fn extract_placeholder_value(template: &str, value: &str, placeholder: &str) -> Option<String> {
    let (prefix, suffix) = template.split_once(placeholder)?;
    let rest = value.strip_prefix(prefix)?;
    let extracted = rest.strip_suffix(suffix)?;
    if extracted.is_empty() || extracted.contains('/') {
        return None;
    }
    Some(extracted.to_string())
}

impl HttpRequest {
    fn path_without_query(&self) -> &str {
        self.path
            .split_once('?')
            .map_or(&self.path, |(path, _)| path)
    }
}

fn json_response<T: serde::Serialize>(status: u16, value: &T) -> String {
    match serde_json::to_string(value) {
        Ok(body) => raw_response(status, "application/json", &body),
        Err(error) => raw_response(
            500,
            "application/json",
            &format!(
                "{{\"error\":\"json_render_failed\",\"message\":\"{}\"}}",
                escape_json_string(&error.to_string())
            ),
        ),
    }
}

fn raw_response(status: u16, content_type: &str, body: &str) -> String {
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        409 => "Conflict",
        404 => "Not Found",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n{body}",
        length = body.len()
    )
}

fn escape_json_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
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
        let latest_report = run_scenario(&scenario).unwrap();
        Arc::new(Mutex::new(ServeState {
            scenario,
            latest_report,
            injected_faults: Vec::new(),
            external_publish_count: 0,
            run_in_progress: false,
            has_completed_run: false,
            external_observation_timeline: Vec::new(),
            external_observations: BTreeMap::new(),
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
    }

    #[test]
    fn health_reports_idle_running_passed_and_failed_states() {
        let state = serve_state();

        let idle = route_request(&request("GET", "/health", ""), Arc::clone(&state));
        assert!(idle.contains("HTTP/1.1 200 OK"));
        assert!(idle.contains("\"status\":\"idle\""));
        assert!(idle.contains("\"serve_version\""));

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

        let invalid_json =
            route_request(&request("POST", "/fault", "not json"), Arc::clone(&state));
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
        let (_, device_id, state_topic) = match_contract(contract, topic).unwrap();
        assert_eq!(device_id, "env_sensor_01");
        assert_eq!(
            state_topic,
            "fleet/demo/site/lab/device/env_sensor_01/state"
        );
        assert_eq!(
            extract_placeholder_value("a/{device_id}/b", "a/device-01/b", "{device_id}"),
            Some("device-01".to_string())
        );
        assert_eq!(
            extract_placeholder_value("a/{device_id}/b", "a/site/device-01/b", "{device_id}"),
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
                .latest_report
                .retained_messages
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
        }

        let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
        assert!(timeline.contains("external_mqtt_retained_state_updated"));
        assert!(timeline.contains("payload missing required fields"));
        assert!(timeline.contains("topic did not match"));
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

    fn mqtt_connect_packet(client_id: &str) -> Vec<u8> {
        mqtt_connect_packet_with("MQTT", 0x04, client_id)
    }

    fn mqtt_connect_packet_with(
        protocol_name: &str,
        protocol_level: u8,
        client_id: &str,
    ) -> Vec<u8> {
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
}
