//! `roomci` command-line entry point.
//!
//! Two subcommands:
//!
//! - `roomci run <scenarios...>` — load, validate, and execute one or more
//!   scenarios. Optionally emits JSON/Markdown/JUnit reports for the *last*
//!   scenario and exits non-zero if any scenario fails. With `--dry-run` only
//!   validates without executing. With `--verbose` prints the timeline; with
//!   `--quiet` suppresses per-scenario detail.
//! - `roomci validate <scenarios...>` — load and validate one or more scenarios
//!   without executing them.
//! - `roomci serve --config <scenario>` — start a localhost-bound HTTP control
//!   and report API. With `--check`, only validates the service-mode config
//!   without starting a long-running process.

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{Arc, Mutex},
    thread,
};

use clap::{ArgGroup, Parser, Subcommand};
use roomci_core::{run_scenario, RunReport, RunResult, TimelineEvent};
use roomci_report::{to_json, to_junit, to_markdown};
use roomci_scenario::{load_scenario, validate_scenario, MqttConnectionContract, ScenarioFile};
use serde_json::json;
use thiserror::Error;

const MAX_HTTP_BODY_BYTES: usize = 1024 * 1024;
const MAX_MQTT_PACKET_BYTES: usize = 1024 * 1024;

#[derive(Debug, Parser)]
#[command(
    name = "roomci",
    version,
    about = "MQTT / edge / device QA contract emulator for CI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run one or more scenarios and emit reports for the last one.
    #[command(group(ArgGroup::new("verbosity").args(["verbose", "quiet"])))]
    Run {
        /// Scenario YAML files to execute, in order.
        #[arg(required = true)]
        scenarios: Vec<PathBuf>,
        /// Write a JUnit XML report for the last scenario.
        #[arg(long)]
        junit: Option<PathBuf>,
        /// Write a Markdown report for the last scenario.
        #[arg(long, alias = "report-md")]
        markdown: Option<PathBuf>,
        /// Write a JSON report for the last scenario.
        #[arg(long, alias = "report-json")]
        json: Option<PathBuf>,
        /// Print every timeline event for each scenario.
        #[arg(long)]
        verbose: bool,
        /// Suppress per-scenario detail; only print the aggregate summary.
        #[arg(long)]
        quiet: bool,
        /// Validate only; do not execute scenarios.
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate scenario files.
    Validate {
        #[arg(required = true)]
        scenarios: Vec<PathBuf>,
    },
    /// Validate service-mode configuration.
    Serve {
        /// Scenario YAML file used as service-mode configuration.
        #[arg(long)]
        config: PathBuf,
        /// Validate service-mode config and exit without blocking.
        #[arg(long)]
        check: bool,
        /// HTTP bind host. Defaults to loopback for local and CI safety.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// HTTP bind port. Use 0 to let the OS choose a free port.
        #[arg(long, default_value_t = 8080)]
        port: u16,
        /// Optional embedded MQTT PoC port. Supports CONNECT + QoS0 PUBLISH.
        #[arg(long)]
        mqtt_port: Option<u16>,
        /// Allow binding to a non-loopback host.
        #[arg(long)]
        allow_non_loopback: bool,
    },
}

#[derive(Debug, Error)]
enum CliError {
    #[error(transparent)]
    Scenario(#[from] roomci_scenario::ScenarioError),
    #[error(transparent)]
    Core(#[from] roomci_core::CoreError),
    #[error("failed to render JSON report: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to write {path}: {source}")]
    Write {
        path: String,
        source: std::io::Error,
    },
    #[error("serve error: {0}")]
    Serve(String),
}

fn main() -> ExitCode {
    match run_cli(Cli::parse()) {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run_cli(cli: Cli) -> Result<ExitCode, CliError> {
    match cli.command {
        Command::Run {
            scenarios,
            junit,
            markdown,
            json,
            verbose,
            quiet,
            dry_run,
        } => run_scenarios(RunOptions {
            scenarios,
            junit,
            markdown,
            json,
            verbose,
            quiet,
            dry_run,
        }),
        Command::Validate { scenarios } => {
            for scenario in scenarios {
                let scenario_file = load_scenario(&scenario)?;
                validate_scenario(&scenario_file)?;
                println!("valid: {}", scenario.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Serve {
            config,
            check,
            host,
            port,
            mqtt_port,
            allow_non_loopback,
        } => serve_config(&config, check, &host, port, mqtt_port, allow_non_loopback),
    }
}

fn serve_config(
    config: &Path,
    check: bool,
    host: &str,
    port: u16,
    mqtt_port: Option<u16>,
    allow_non_loopback: bool,
) -> Result<ExitCode, CliError> {
    let scenario_file = load_scenario(config)?;
    validate_scenario(&scenario_file)?;
    if check {
        println!("serve config valid: {}", config.display());
        return Ok(ExitCode::SUCCESS);
    }
    if !allow_non_loopback && !is_loopback_host(host) {
        return Err(CliError::Serve(format!(
            "refusing to bind non-loopback host {host}; pass --allow-non-loopback to override"
        )));
    }
    serve_http(scenario_file, host, port, mqtt_port)
}

struct RunOptions {
    scenarios: Vec<PathBuf>,
    junit: Option<PathBuf>,
    markdown: Option<PathBuf>,
    json: Option<PathBuf>,
    verbose: bool,
    quiet: bool,
    dry_run: bool,
}

fn run_scenarios(options: RunOptions) -> Result<ExitCode, CliError> {
    let total = options.scenarios.len();
    let mut passed = 0_usize;
    let mut failed = 0_usize;
    let mut last_report: Option<RunReport> = None;

    for (index, path) in options.scenarios.iter().enumerate() {
        let scenario_file = load_scenario(path)?;
        validate_scenario(&scenario_file)?;

        if options.dry_run {
            if !options.quiet {
                println!(
                    "[{n}/{total}] dry-run valid: {path}",
                    n = index + 1,
                    total = total,
                    path = path.display()
                );
            }
            passed += 1;
            continue;
        }

        let report = run_scenario(&scenario_file)?;
        match report.result {
            RunResult::Passed => passed += 1,
            RunResult::Failed => failed += 1,
        }

        if !options.quiet {
            print_scenario_summary(index + 1, total, path, &report, options.verbose);
        }

        last_report = Some(report);
    }

    if let Some(report) = last_report.as_ref() {
        if let Some(path) = &options.json {
            write_file(path, &to_json(report)?)?;
        }
        if let Some(path) = &options.markdown {
            write_file(path, &to_markdown(report))?;
        }
        if let Some(path) = &options.junit {
            write_file(path, &to_junit(report))?;
        }
    }

    println!(
        "summary: {passed} passed, {failed} failed (of {total})",
        passed = passed,
        failed = failed,
        total = total
    );

    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn print_scenario_summary(
    index: usize,
    total: usize,
    path: &Path,
    report: &RunReport,
    verbose: bool,
) {
    println!(
        "[{index}/{total}] scenario: {name} ({path})",
        index = index,
        total = total,
        name = report.scenario_name,
        path = path.display()
    );
    println!("  result: {:?}", report.result);
    println!("  assertions: {}", report.assertions.len());

    if verbose {
        for event in &report.timeline {
            let target = event.target.as_deref().unwrap_or("-");
            println!(
                "    {at} [{event_type}] {target}: {message}",
                at = event.at,
                event_type = event.event_type,
                target = target,
                message = event.message
            );
        }
    }
}

fn write_file(path: &Path, contents: &str) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CliError::Write {
            path: parent.display().to_string(),
            source,
        })?;
    }
    fs::write(path, contents).map_err(|source| CliError::Write {
        path: path.display().to_string(),
        source,
    })
}

struct ServeState {
    scenario: ScenarioFile,
    latest_report: RunReport,
    injected_faults: Vec<serde_json::Value>,
    external_publish_count: u64,
}

fn serve_http(
    scenario: ScenarioFile,
    host: &str,
    port: u16,
    mqtt_port: Option<u16>,
) -> Result<ExitCode, CliError> {
    let latest_report = run_scenario(&scenario)?;
    let state = Arc::new(Mutex::new(ServeState {
        scenario,
        latest_report,
        injected_faults: Vec::new(),
        external_publish_count: 0,
    }));
    let listener =
        TcpListener::bind((host, port)).map_err(|error| CliError::Serve(error.to_string()))?;
    let address = listener
        .local_addr()
        .map_err(|error| CliError::Serve(error.to_string()))?;

    println!("roomci serve listening on http://{address}");
    println!("endpoints: /health /scenario /state /timeline /reports/latest.json");
    if let Some(mqtt_port) = mqtt_port {
        let mqtt_listener = TcpListener::bind((host, mqtt_port))
            .map_err(|error| CliError::Serve(error.to_string()))?;
        let mqtt_address = mqtt_listener
            .local_addr()
            .map_err(|error| CliError::Serve(error.to_string()))?;
        println!("roomci mqtt listening on mqtt://{mqtt_address}");
        let mqtt_state = Arc::clone(&state);
        thread::spawn(move || serve_mqtt(mqtt_listener, mqtt_address, mqtt_state));
    }
    std::io::stdout()
        .flush()
        .map_err(|error| CliError::Serve(error.to_string()))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_connection(stream, Arc::clone(&state)) {
                    eprintln!("serve request error: {error}");
                }
            }
            Err(error) => eprintln!("serve accept error: {error}"),
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<ServeState>>) -> Result<(), CliError> {
    let request = read_http_request(&mut stream)?;
    let response = route_request(&request, state);
    stream
        .write_all(response.as_bytes())
        .map_err(|error| CliError::Serve(error.to_string()))
}

struct HttpRequest {
    method: String,
    path: String,
    body: String,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, CliError> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream
            .read(&mut chunk)
            .map_err(|error| CliError::Serve(error.to_string()))?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > 64 * 1024 {
            return Err(CliError::Serve("request header too large".to_string()));
        }
    }

    let header_end = buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
        .ok_or_else(|| CliError::Serve("malformed HTTP request".to_string()))?;
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let mut lines = headers.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| CliError::Serve("missing request line".to_string()))?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| CliError::Serve("missing method".to_string()))?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| CliError::Serve("missing path".to_string()))?
        .to_string();
    let content_length = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    if content_length > MAX_HTTP_BODY_BYTES {
        return Err(CliError::Serve(format!(
            "request body too large: {content_length} bytes"
        )));
    }

    while buffer.len() < header_end + content_length {
        let read = stream
            .read(&mut chunk)
            .map_err(|error| CliError::Serve(error.to_string()))?;
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
            let state = state.lock().expect("serve state mutex poisoned");
            json_response(
                200,
                &json!({
                    "status": "ok",
                    "scenario": state.scenario.scenario.name,
                    "result": state.latest_report.result,
                }),
            )
        }
        ("GET", "/scenario") => {
            let state = state.lock().expect("serve state mutex poisoned");
            json_response(200, &state.scenario)
        }
        ("GET", "/state") => {
            let state = state.lock().expect("serve state mutex poisoned");
            json_response(
                200,
                &json!({
                    "scenario": state.scenario.scenario.name,
                    "result": state.latest_report.result,
                    "final_state": state.latest_report.final_state,
                    "retained_messages": state.latest_report.retained_messages,
                    "injected_faults": state.injected_faults,
                    "external_publish_count": state.external_publish_count,
                }),
            )
        }
        ("GET", "/timeline") => {
            let state = state.lock().expect("serve state mutex poisoned");
            json_response(200, &state.latest_report.timeline)
        }
        ("POST", "/fault") => match serde_json::from_str::<serde_json::Value>(&request.body) {
            Ok(fault) => {
                let mut state = state.lock().expect("serve state mutex poisoned");
                state.injected_faults.push(fault.clone());
                let fault_index = state.injected_faults.len();
                let fault_summary = serde_json::to_string(&fault)
                    .unwrap_or_else(|_| "unrenderable fault".to_string());
                state.latest_report.timeline.push(TimelineEvent {
                    at: format!("external#fault{fault_index}"),
                    event_type: "external_fault_injected".to_string(),
                    target: fault
                        .get("target")
                        .and_then(|value| value.as_str())
                        .map(str::to_string),
                    message: format!("external fault accepted: {fault_summary}"),
                });
                json_response(202, &json!({ "accepted": true, "fault": fault }))
            }
            Err(error) => json_response(
                400,
                &json!({ "error": "invalid_json", "message": error.to_string() }),
            ),
        },
        ("POST", "/finish") => {
            let state = state.lock().expect("serve state mutex poisoned");
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
            let mut state = state.lock().expect("serve state mutex poisoned");
            match run_scenario(&state.scenario) {
                Ok(report) => {
                    let result = report.result;
                    state.latest_report = report;
                    state.injected_faults.clear();
                    state.external_publish_count = 0;
                    json_response(200, &json!({ "finished": true, "result": result }))
                }
                Err(error) => json_response(
                    500,
                    &json!({ "error": "run_failed", "message": error.to_string() }),
                ),
            }
        }
        ("GET", "/reports/latest") | ("GET", "/reports/latest.json") => {
            let state = state.lock().expect("serve state mutex poisoned");
            match to_json(&state.latest_report) {
                Ok(report) => raw_response(200, "application/json", &report),
                Err(error) => json_response(
                    500,
                    &json!({ "error": "report_render_failed", "message": error.to_string() }),
                ),
            }
        }
        ("GET", "/reports/latest.md") => {
            let state = state.lock().expect("serve state mutex poisoned");
            raw_response(
                200,
                "text/markdown; charset=utf-8",
                &to_markdown(&state.latest_report),
            )
        }
        ("GET", "/reports/latest.junit.xml") => {
            let state = state.lock().expect("serve state mutex poisoned");
            raw_response(
                200,
                "application/xml; charset=utf-8",
                &to_junit(&state.latest_report),
            )
        }
        _ => json_response(
            404,
            &json!({ "error": "not_found", "message": "unknown endpoint" }),
        ),
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
) -> Result<(), CliError> {
    loop {
        let Some(packet) = read_mqtt_packet(&mut stream)? else {
            return Ok(());
        };
        match packet.packet_type {
            1 => {
                stream
                    .write_all(&[0x20, 0x02, 0x00, 0x00])
                    .map_err(|error| CliError::Serve(error.to_string()))?;
            }
            3 => {
                let publish = parse_mqtt_publish(&packet.payload)?;
                let mut state = state.lock().expect("serve state mutex poisoned");
                apply_external_mqtt_publish(&mut state, publish);
            }
            _ => {
                return Err(CliError::Serve(format!(
                    "unsupported MQTT packet type {}",
                    packet.packet_type
                )));
            }
        }
    }
}

struct MqttPacket {
    packet_type: u8,
    payload: Vec<u8>,
}

struct MqttPublish {
    topic: String,
    payload: BTreeMap<String, serde_json::Value>,
}

fn read_mqtt_packet(stream: &mut TcpStream) -> Result<Option<MqttPacket>, CliError> {
    let mut fixed = [0_u8; 1];
    match stream.read_exact(&mut fixed) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(CliError::Serve(error.to_string())),
    }
    let remaining = read_mqtt_remaining_length(stream)?;
    if remaining > MAX_MQTT_PACKET_BYTES {
        return Err(CliError::Serve(format!(
            "MQTT packet too large: {remaining} bytes"
        )));
    }
    let mut payload = vec![0_u8; remaining];
    stream
        .read_exact(&mut payload)
        .map_err(|error| CliError::Serve(error.to_string()))?;
    Ok(Some(MqttPacket {
        packet_type: fixed[0] >> 4,
        payload,
    }))
}

fn read_mqtt_remaining_length(stream: &mut TcpStream) -> Result<usize, CliError> {
    let mut multiplier = 1_usize;
    let mut value = 0_usize;
    for _ in 0..4 {
        let mut encoded = [0_u8; 1];
        stream
            .read_exact(&mut encoded)
            .map_err(|error| CliError::Serve(error.to_string()))?;
        value += ((encoded[0] & 127) as usize) * multiplier;
        if encoded[0] & 128 == 0 {
            return Ok(value);
        }
        multiplier *= 128;
    }
    Err(CliError::Serve(
        "malformed MQTT remaining length".to_string(),
    ))
}

fn parse_mqtt_publish(payload: &[u8]) -> Result<MqttPublish, CliError> {
    if payload.len() < 2 {
        return Err(CliError::Serve("malformed MQTT publish".to_string()));
    }
    let topic_len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + topic_len {
        return Err(CliError::Serve("malformed MQTT topic".to_string()));
    }
    let topic = String::from_utf8(payload[2..2 + topic_len].to_vec())
        .map_err(|error| CliError::Serve(error.to_string()))?;
    let payload_bytes = &payload[2 + topic_len..];
    let payload = serde_json::from_slice::<BTreeMap<String, serde_json::Value>>(payload_bytes)
        .map_err(|error| CliError::Serve(format!("MQTT payload must be JSON object: {error}")))?;
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
        state.latest_report.timeline.push(TimelineEvent {
            at: format!("external#{event_index}"),
            event_type: "external_mqtt_publish_rejected".to_string(),
            target: Some(publish.topic),
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
        state.latest_report.timeline.push(TimelineEvent {
            at: format!("external#{event_index}"),
            event_type: "external_mqtt_publish_rejected".to_string(),
            target: Some(publish.topic),
            message: format!(
                "payload missing required fields for {}: {}",
                contract.name,
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
    state.latest_report.timeline.push(TimelineEvent {
        at: format!("external#{event_index}"),
        event_type: "external_mqtt_retained_state_updated".to_string(),
        target: Some(device_id),
        message: format!(
            "external MQTT publish matched {} and updated retained state at {}",
            contract.name, state_topic
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
        404 => "Not Found",
        500 => "Internal Server Error",
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
        }))
    }

    fn request(method: &str, path: &str, body: &str) -> HttpRequest {
        HttpRequest {
            method: method.to_string(),
            path: path.to_string(),
            body: body.to_string(),
        }
    }

    #[test]
    fn http_router_serves_core_observation_endpoints() {
        let state = serve_state();

        let health = route_request(&request("GET", "/health", ""), Arc::clone(&state));
        assert!(health.contains("HTTP/1.1 200 OK"));
        assert!(health.contains("\"status\":\"ok\""));

        let scenario = route_request(&request("GET", "/scenario", ""), Arc::clone(&state));
        assert!(scenario.contains("generic_mqtt_retained_state"));

        let current_state = route_request(&request("GET", "/state", ""), Arc::clone(&state));
        assert!(current_state.contains("retained_messages"));

        let timeline = route_request(&request("GET", "/timeline", ""), Arc::clone(&state));
        assert!(timeline.contains("mqtt_publish"));
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
        let mut variable = Vec::new();
        variable.extend_from_slice(&[0x00, 0x04]);
        variable.extend_from_slice(b"MQTT");
        variable.push(0x04);
        variable.push(0x02);
        variable.extend_from_slice(&60_u16.to_be_bytes());
        variable.extend_from_slice(&(client_id.len() as u16).to_be_bytes());
        variable.extend_from_slice(client_id.as_bytes());

        let mut packet = vec![0x10];
        encode_mqtt_remaining_length(variable.len(), &mut packet);
        packet.extend(variable);
        packet
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
