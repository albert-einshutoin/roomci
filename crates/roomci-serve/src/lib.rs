//! Local `roomci serve` runtime for HTTP and MQTT-based PoC integrations.
//!
//! This crate owns the long-running localhost service used by external
//! controllers. It intentionally implements a narrow PoC surface rather than a
//! production HTTP server or MQTT broker.

use std::{
    collections::BTreeMap,
    io::Write,
    net::TcpListener,
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

use roomci_core::{run_scenario, RunReport, TimelineEvent};
use roomci_device_model::ModbusModel;
use roomci_scenario::ScenarioFile;
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
const MODBUS_EXCEPTION_ILLEGAL_FUNCTION: u8 = 0x01;
const MODBUS_EXCEPTION_ILLEGAL_ADDRESS: u8 = 0x02;
const MODBUS_EXCEPTION_ILLEGAL_VALUE: u8 = 0x03;

mod http;
mod modbus;
mod mqtt;

pub(crate) use http::{json_response, serve_http_listener};
#[cfg(test)]
pub(crate) use http::{raw_response, read_http_request, route_request, HttpRequest};
pub(crate) use modbus::serve_modbus;
#[cfg(test)]
pub(crate) use modbus::{apply_modbus_tcp_request, modbus_tcp_response, ModbusTcpRequest};
pub(crate) use mqtt::serve_mqtt;
#[cfg(test)]
pub(crate) use mqtt::{
    apply_external_mqtt_publish, extract_placeholder_value, handle_mqtt_client, match_contract,
    mqtt_retained_replay_for_subscribe, parse_mqtt_publish, parse_mqtt_subscribe, read_mqtt_packet,
    MqttPublish,
};

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
    /// Optional Modbus TCP PoC ingress port.
    pub modbus_port: Option<u16>,
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
        options.modbus_port,
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
    /// Device final-state updates derived from external MQTT publishes. Kept
    /// separate from `latest_report` until a successful `/run` drains the
    /// overlay.
    external_mqtt_final_state: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    /// MQTT retained messages published by external MQTT clients. Keyed by the
    /// retained topic. Survives across `/run` and is merged into
    /// `latest_report.retained_messages` at the next `/run` boundary, so
    /// external MQTT controllers see stable evidence across run boundaries.
    external_mqtt_retained_state: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    modbus: ModbusModel,
    modbus_units: BTreeMap<u8, String>,
    external_modbus_registers: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
}

fn serve_http(
    scenario: ScenarioFile,
    host: &str,
    port: u16,
    mqtt_port: Option<u16>,
    modbus_port: Option<u16>,
) -> Result<(), ServeError> {
    let latest_report = run_scenario(&scenario)?;
    let mut modbus = ModbusModel::from_config(&scenario.modbus);
    apply_scenario_modbus_steps(&scenario, &mut modbus);
    let modbus_units = modbus_unit_map(&scenario);
    let state = Arc::new(Mutex::new(ServeState {
        scenario,
        latest_report,
        injected_faults: Vec::new(),
        external_publish_count: 0,
        run_in_progress: false,
        has_completed_run: false,
        external_observation_timeline: Vec::new(),
        external_observations: BTreeMap::new(),
        external_mqtt_final_state: BTreeMap::new(),
        external_mqtt_retained_state: BTreeMap::new(),
        modbus,
        modbus_units,
        external_modbus_registers: BTreeMap::new(),
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
    if let Some(modbus_port) = modbus_port {
        let modbus_listener = TcpListener::bind((host, modbus_port))
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
        let modbus_address = modbus_listener
            .local_addr()
            .map_err(|error| ServeError::Runtime(error.to_string()))?;
        println!("roomci modbus listening on tcp://{modbus_address}");
        let modbus_state = Arc::clone(&state);
        thread::spawn(move || serve_modbus(modbus_listener, modbus_address, modbus_state));
    }
    std::io::stdout()
        .flush()
        .map_err(|error| ServeError::Runtime(error.to_string()))?;

    serve_http_listener(listener, state);

    Ok(())
}

fn modbus_unit_map(scenario: &ScenarioFile) -> BTreeMap<u8, String> {
    let mut units = BTreeMap::new();
    let Some(devices) = scenario
        .modbus
        .get("devices")
        .and_then(|value| value.as_sequence())
    else {
        return units;
    };
    for device in devices {
        let Some(mapping) = device.as_mapping() else {
            continue;
        };
        let Some(id) = mapping
            .get(serde_yaml::Value::String("id".to_string()))
            .and_then(|value| value.as_str())
        else {
            continue;
        };
        let unit_id = mapping
            .get(serde_yaml::Value::String("unit_id".to_string()))
            .and_then(|value| value.as_i64())
            .and_then(|value| u8::try_from(value).ok())
            .unwrap_or_else(|| (units.len() + 1).min(u8::MAX as usize) as u8);
        units.insert(unit_id, id.to_string());
    }
    units
}

fn apply_scenario_modbus_steps(scenario: &ScenarioFile, modbus: &mut ModbusModel) {
    for step in &scenario.steps {
        let Some(write) = &step.modbus_write else {
            continue;
        };
        if modbus
            .write(&write.device, write.register, write.value.clone())
            .is_err()
        {
            continue;
        }
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
    report.final_state = state_final_state_view(state);
    report.retained_messages = state_retained_messages_view(state);
    for (key, value) in &state.external_observations {
        let prefixed = format!("{EXTERNAL_BMS_STATE_PREFIX}{key}");
        report.final_state.insert(prefixed, value.clone());
    }
    for (key, value) in &state.external_modbus_registers {
        report.final_state.insert(key.clone(), value.clone());
    }
    report
}

fn state_final_state_view(
    state: &ServeState,
) -> BTreeMap<String, BTreeMap<String, serde_json::Value>> {
    let mut final_state = state.latest_report.final_state.clone();
    for (device_id, payload) in &state.external_mqtt_final_state {
        final_state.insert(device_id.clone(), payload.clone());
    }
    final_state
}

fn state_retained_messages_view(
    state: &ServeState,
) -> BTreeMap<String, BTreeMap<String, serde_json::Value>> {
    let mut retained_messages = state.latest_report.retained_messages.clone();
    for (topic, payload) in &state.external_mqtt_retained_state {
        retained_messages.insert(topic.clone(), payload.clone());
    }
    retained_messages
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
    for (device_id, payload) in std::mem::take(&mut state.external_mqtt_final_state) {
        report.final_state.insert(device_id, payload);
    }
    for (topic, payload) in std::mem::take(&mut state.external_mqtt_retained_state) {
        report.retained_messages.insert(topic, payload);
    }
    for (key, value) in std::mem::take(&mut state.external_modbus_registers) {
        report.final_state.insert(key, value);
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

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod tests;
