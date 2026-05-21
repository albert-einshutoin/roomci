use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use roomci_core::{run_scenario, TimelineEvent};
use roomci_report::{to_json, to_junit, to_markdown};
use serde_json::json;

use crate::{
    drain_external_overlay_into, lock_serve_state, rendered_report_view, sanitize_external_key,
    sanitize_external_message_value, serve_error_response, state_final_state_view,
    state_retained_messages_view, ServeError, ServeState, HTTP_CLIENT_TIMEOUT_SECS,
    HTTP_MAX_INFLIGHT_CONNECTIONS, MAX_HTTP_BODY_BYTES,
};

pub(crate) fn serve_http_listener(listener: TcpListener, state: Arc<Mutex<ServeState>>) {
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

pub(crate) struct HttpRequest {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) body: String,
}

pub(crate) fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, ServeError> {
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
    if buffer.len() < header_end + content_length {
        return Err(ServeError::Runtime(format!(
            "incomplete request body: expected {content_length} bytes, got {}",
            buffer.len().saturating_sub(header_end)
        )));
    }

    let body =
        String::from_utf8_lossy(&buffer[header_end..header_end + content_length]).to_string();
    Ok(HttpRequest { method, path, body })
}

pub(crate) fn route_request(request: &HttpRequest, state: Arc<Mutex<ServeState>>) -> String {
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
                    "final_state": state_final_state_view(&state),
                    "retained_messages": state_retained_messages_view(&state),
                    "injected_faults": state.injected_faults,
                    "external_publish_count": state.external_publish_count,
                    "external_observations": state.external_observations,
                    "external_modbus_registers": state.external_modbus_registers,
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

impl HttpRequest {
    fn path_without_query(&self) -> &str {
        self.path
            .split_once('?')
            .map_or(&self.path, |(path, _)| path)
    }
}

pub(crate) fn json_response<T: serde::Serialize>(status: u16, value: &T) -> String {
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

pub(crate) fn raw_response(status: u16, content_type: &str, body: &str) -> String {
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
