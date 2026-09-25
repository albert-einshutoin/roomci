//! Real broker recovery check. The embedded broker and virtual-time runner are not used here.

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::Path,
    time::{Duration, Instant},
};

use chrono::Utc;
use roomci_core::{AssertionResult, RunReport, RunResult, TimelineEvent};
use rumqttc::{Client, Connection, Event, MqttOptions, Packet, QoS, RecvTimeoutError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{write_file, CliError};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Contract {
    pub scenario: String,
    pub broker_host: String,
    pub broker_port: u16,
    pub proxy_host: String,
    pub proxy_api_port: u16,
    pub proxy_port: u16,
    pub proxy_name: String,
    pub device_id: String,
    pub initial_value: String,
    pub latest_value: String,
    pub ready_timeout_ms: u64,
    pub fault_timeout_ms: u64,
    pub recovery_timeout_ms: u64,
    pub stability_ms: u64,
}

impl Contract {
    fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("scenario", &self.scenario),
            ("broker_host", &self.broker_host),
            ("proxy_host", &self.proxy_host),
            ("proxy_name", &self.proxy_name),
            ("device_id", &self.device_id),
            ("initial_value", &self.initial_value),
            ("latest_value", &self.latest_value),
        ] {
            if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
                return Err(format!("{name} must contain 1..128 printable characters"));
            }
        }
        if !self
            .device_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err("device_id must use ASCII letters, digits or hyphens".into());
        }
        if !self
            .proxy_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("proxy_name must use ASCII letters, digits, hyphens or underscores".into());
        }
        for (name, host) in [
            ("broker_host", &self.broker_host),
            ("proxy_host", &self.proxy_host),
        ] {
            if !host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
            {
                return Err(format!("{name} must be a DNS name or IP address"));
            }
        }
        if self.initial_value == self.latest_value {
            return Err("initial_value and latest_value must differ".into());
        }
        if self.broker_port == 0 || self.proxy_api_port == 0 || self.proxy_port == 0 {
            return Err("network ports must be nonzero".into());
        }
        for (name, value) in [
            ("ready_timeout_ms", self.ready_timeout_ms),
            ("fault_timeout_ms", self.fault_timeout_ms),
            ("recovery_timeout_ms", self.recovery_timeout_ms),
            ("stability_ms", self.stability_ms),
        ] {
            if !(100..=120_000).contains(&value) {
                return Err(format!("{name} must be 100..120000"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub(super) struct ExternalReport {
    #[serde(flatten)]
    pub report: RunReport,
    pub evaluation_target: &'static str,
    pub observation_source: &'static str,
    pub sut_version: String,
    pub contract: Contract,
    pub verdict: &'static str,
    pub reason: String,
}

#[derive(Debug)]
enum Outcome {
    Passed,
    Failed(&'static str),
    Inconclusive(String),
}

impl Outcome {
    fn reason(&self) -> String {
        match self {
            Self::Passed => "latest_report_stable".into(),
            Self::Failed(reason) => (*reason).into(),
            Self::Inconclusive(reason) => reason.clone(),
        }
    }
    fn verdict(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed(_) => "failed",
            Self::Inconclusive(_) => "inconclusive",
        }
    }
}

struct Observer {
    client: Client,
    connection: Connection,
    timeline: Vec<TimelineEvent>,
    run_id: String,
    device_id: String,
    reported_topic: String,
    status_topic: String,
    last_report: Option<Value>,
    last_seen: Option<Value>,
    status: Option<String>,
    pubacks: usize,
    subacks: usize,
}

impl Observer {
    fn new(contract: &Contract, run_id: &str) -> Result<Self, String> {
        let mut options = MqttOptions::new(
            format!("roomci-observer-{run_id}"),
            &contract.broker_host,
            contract.broker_port,
        );
        options.set_keep_alive(Duration::from_secs(5));
        options.set_clean_session(true);
        let (client, connection) = Client::new(options, 20);
        let prefix = format!("roomci/{run_id}/{}", contract.device_id);
        let reported_topic = format!("{prefix}/reported");
        let status_topic = format!("{prefix}/status");
        client
            .subscribe(&reported_topic, QoS::AtLeastOnce)
            .map_err(|e| e.to_string())?;
        client
            .subscribe(&status_topic, QoS::AtLeastOnce)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            client,
            connection,
            timeline: Vec::new(),
            run_id: run_id.into(),
            device_id: contract.device_id.clone(),
            reported_topic,
            status_topic,
            last_report: None,
            last_seen: None,
            status: None,
            pubacks: 0,
            subacks: 0,
        })
    }

    fn record(&mut self, event_type: &str, target: Option<String>, message: impl Into<String>) {
        self.timeline.push(TimelineEvent {
            at: Utc::now().to_rfc3339(),
            event_type: event_type.into(),
            target,
            message: message.into(),
        });
    }

    fn poll(&mut self, deadline: Instant) -> Result<(), String> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(());
        }
        match self
            .connection
            .recv_timeout(remaining.min(Duration::from_millis(250)))
        {
            Ok(Ok(Event::Incoming(Packet::SubAck(_)))) => self.subacks += 1,
            Ok(Ok(Event::Incoming(Packet::PubAck(_)))) => self.pubacks += 1,
            Ok(Ok(Event::Incoming(Packet::Publish(publish)))) => {
                if publish.topic == self.reported_topic {
                    if publish.retain {
                        self.record(
                            "ignored_retained_report",
                            Some(publish.topic),
                            "preexisting retained report cannot prove SUT activity",
                        );
                        return Ok(());
                    }
                    match serde_json::from_slice::<Value>(&publish.payload) {
                        Ok(value)
                            if value.get("run_id").and_then(Value::as_str)
                                == Some(&self.run_id)
                                && value.get("device_id").and_then(Value::as_str)
                                    == Some(&self.device_id) =>
                        {
                            self.record("sut_reported", Some(publish.topic), value.to_string());
                            self.last_seen = Some(value.clone());
                            self.last_report = Some(value);
                        }
                        _ => self.record(
                            "ignored_report",
                            Some(publish.topic),
                            "invalid payload or mismatched run/device",
                        ),
                    }
                } else if publish.topic == self.status_topic {
                    match serde_json::from_slice::<Value>(&publish.payload) {
                        Ok(value)
                            if value.get("run_id").and_then(Value::as_str)
                                == Some(&self.run_id)
                                && value.get("device_id").and_then(Value::as_str)
                                    == Some(&self.device_id) =>
                        {
                            if let Some(status) = value.get("status").and_then(Value::as_str) {
                                self.status = Some(status.into());
                                self.record("sut_status", Some(publish.topic), status);
                            }
                        }
                        _ => self.record(
                            "ignored_status",
                            Some(publish.topic),
                            "mismatched run/device",
                        ),
                    }
                }
            }
            Ok(Err(error)) => return Err(format!("observer connection: {error}")),
            Err(RecvTimeoutError::Timeout) => {}
            Err(error) => return Err(format!("observer receive: {error:?}")),
            _ => {}
        }
        Ok(())
    }

    fn wait_for(
        &mut self,
        deadline: Instant,
        predicate: impl Fn(&Self) -> bool,
    ) -> Result<bool, String> {
        while Instant::now() < deadline {
            self.poll(deadline)?;
            if predicate(self) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn publish_desired(
        &mut self,
        contract: &Contract,
        revision: u64,
        value: &str,
        deadline: Instant,
    ) -> Result<(), String> {
        let prior_acks = self.pubacks;
        let topic = format!("roomci/{}/{}/desired", self.run_id, contract.device_id);
        let payload = json!({"run_id":self.run_id,"device_id":self.device_id,"revision":revision,"value":value}).to_string();
        self.client
            .publish(&topic, QoS::AtLeastOnce, true, payload.clone())
            .map_err(|e| e.to_string())?;
        if !self.wait_for(deadline, |s| s.pubacks > prior_acks)? {
            return Err("desired publish not acknowledged by broker".into());
        }
        self.record("desired_accepted", Some(topic), payload);
        Ok(())
    }
}

fn proxy_request(
    contract: &Contract,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<Value, String> {
    let addr = (contract.proxy_host.as_str(), contract.proxy_api_port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("proxy address missing")?;
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    let body = body.unwrap_or("");
    let request = format!("{method} {path} HTTP/1.0\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", contract.proxy_host, body.len());
    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut response = Vec::new();
    stream
        .take(64 * 1024)
        .read_to_end(&mut response)
        .map_err(|e| e.to_string())?;
    let response = String::from_utf8(response).map_err(|e| e.to_string())?;
    let (head, payload) = response
        .split_once("\r\n\r\n")
        .ok_or("malformed proxy HTTP response")?;
    if !head.starts_with("HTTP/1.0 200 ") && !head.starts_with("HTTP/1.1 200 ") {
        return Err(format!(
            "proxy API returned {}",
            head.lines().next().unwrap_or("unknown")
        ));
    }
    serde_json::from_str(payload).map_err(|e| e.to_string())
}

fn proxy_state(contract: &Contract, enabled: bool) -> Result<(), String> {
    let path = format!("/proxies/{}", contract.proxy_name);
    let response = proxy_request(
        contract,
        "POST",
        &path,
        Some(&json!({"enabled":enabled}).to_string()),
    )?;
    let observed = proxy_request(contract, "GET", &path, None)?;
    for value in [&response, &observed] {
        if value.get("name").and_then(Value::as_str) != Some(&contract.proxy_name)
            || value.get("enabled").and_then(Value::as_bool) != Some(enabled)
            || !value
                .get("listen")
                .and_then(Value::as_str)
                .is_some_and(|listen| listen.ends_with(&format!(":{}", contract.proxy_port)))
            || value.get("upstream").and_then(Value::as_str)
                != Some(format!("{}:{}", contract.broker_host, contract.broker_port).as_str())
        {
            return Err("proxy state verification failed".into());
        }
    }
    Ok(())
}

fn proxy_mqtt_reachable(contract: &Contract, run_id: &str) -> Result<bool, String> {
    let addr = (contract.proxy_host.as_str(), contract.proxy_port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("proxy listen address missing")?;
    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(300)) {
        Ok(stream) => stream,
        Err(_) => return Ok(false),
    };
    stream
        .set_read_timeout(Some(Duration::from_millis(300)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_millis(300)))
        .map_err(|e| e.to_string())?;
    let client_id = format!("roomci-probe-{run_id}");
    let remaining = 10 + 2 + client_id.len();
    let mut connect = vec![
        0x10,
        remaining as u8,
        0,
        4,
        b'M',
        b'Q',
        b'T',
        b'T',
        4,
        2,
        0,
        5,
    ];
    connect.extend_from_slice(&(client_id.len() as u16).to_be_bytes());
    connect.extend_from_slice(client_id.as_bytes());
    if stream.write_all(&connect).is_err() {
        return Ok(false);
    }
    let mut connack = [0; 4];
    Ok(stream.read_exact(&mut connack).is_ok() && connack == [0x20, 0x02, 0x00, 0x00])
}

fn proxy_is_blocked(contract: &Contract) -> Result<bool, String> {
    let addr = (contract.proxy_host.as_str(), contract.proxy_port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("proxy listen address missing")?;
    Ok(TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_err())
}

fn matches_report(value: &Value, revision: u64, expected: &str) -> bool {
    value.get("revision").and_then(Value::as_u64) == Some(revision)
        && value.get("value").and_then(Value::as_str) == Some(expected)
}

fn run_steps(observer: &mut Observer, contract: &Contract) -> Outcome {
    let result = (|| -> Result<Outcome, String> {
        if !observer.wait_for(
            Instant::now() + Duration::from_millis(contract.ready_timeout_ms),
            |s| s.subacks >= 2,
        )? {
            return Ok(Outcome::Inconclusive("observer_not_ready".into()));
        }
        observer.record(
            "observer_ready",
            None,
            "reported and status SUBACK received",
        );
        observer.publish_desired(
            contract,
            1,
            &contract.initial_value,
            Instant::now() + Duration::from_millis(contract.ready_timeout_ms),
        )?;
        if !observer.wait_for(
            Instant::now() + Duration::from_millis(contract.ready_timeout_ms),
            |s| {
                s.last_report
                    .as_ref()
                    .is_some_and(|v| matches_report(v, 1, &contract.initial_value))
            },
        )? {
            return Ok(Outcome::Inconclusive("initial_report_missing".into()));
        }
        observer.record("initial_ready", None, "SUT reported initial revision 1");
        observer.status = None;
        observer.record(
            "fault_requested",
            Some(contract.proxy_name.clone()),
            "disable SUT-only TCP proxy",
        );
        proxy_state(contract, false).map_err(|e| format!("fault_not_applied: {e}"))?;
        observer.record(
            "fault_applied",
            Some(contract.proxy_name.clone()),
            "proxy disabled and GET confirmed",
        );
        if !proxy_is_blocked(contract)? {
            return Ok(Outcome::Inconclusive(
                "fault_not_applied: proxy_still_accepts_connections".into(),
            ));
        }
        if !observer.wait_for(
            Instant::now() + Duration::from_millis(contract.fault_timeout_ms),
            |s| s.status.as_deref() == Some("offline"),
        )? {
            return Ok(Outcome::Inconclusive(
                "fault_not_affected: sut_disconnect_not_observed".into(),
            ));
        }
        observer.record(
            "fault_affected_sut",
            None,
            "broker emitted SUT offline will; proxy rejects new TCP connections",
        );
        observer.last_report = None;
        observer.publish_desired(
            contract,
            2,
            &contract.latest_value,
            Instant::now() + Duration::from_millis(contract.fault_timeout_ms),
        )?;
        proxy_state(contract, true).map_err(|e| format!("fault_release_failed: {e}"))?;
        let deadline = Instant::now() + Duration::from_millis(contract.recovery_timeout_ms);
        let probe_deadline = deadline.min(Instant::now() + Duration::from_secs(1));
        let mut path_recovered = false;
        while Instant::now() < probe_deadline {
            if proxy_mqtt_reachable(contract, &observer.run_id)? {
                path_recovered = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if !path_recovered {
            return Ok(Outcome::Inconclusive(
                "fault_release_failed: MQTT path unavailable".into(),
            ));
        }
        observer.record(
            "fault_released",
            Some(contract.proxy_name.clone()),
            "proxy enabled and GET confirmed; MQTT CONNACK through proxy confirmed",
        );
        let mut reached = false;
        while Instant::now() < deadline {
            observer.poll(deadline)?;
            if let Some(value) = observer.last_report.take() {
                if value.get("revision").and_then(Value::as_u64) == Some(2) {
                    if !matches_report(&value, 2, &contract.latest_value) {
                        return Ok(Outcome::Failed("wrong_latest_value"));
                    }
                    reached = true;
                    observer.record("latest_reached", None, value.to_string());
                    break;
                }
                observer.record("stale_report", None, value.to_string());
            }
        }
        if !reached {
            return Ok(Outcome::Failed("missing_latest_report"));
        }
        let stable_until = Instant::now() + Duration::from_millis(contract.stability_ms);
        while Instant::now() < stable_until {
            observer.poll(stable_until)?;
            if let Some(value) = observer.last_report.take() {
                if !matches_report(&value, 2, &contract.latest_value) {
                    return Ok(Outcome::Failed("rollback_observed"));
                }
            }
        }
        Ok(Outcome::Passed)
    })();
    match result {
        Ok(outcome) => outcome,
        Err(error) => Outcome::Inconclusive(error),
    }
}

fn validate_run_id(run_id: &str) -> Result<(), String> {
    if run_id.is_empty()
        || run_id.len() > 64
        || !run_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err("run_id must use 1..64 ASCII letters, digits or hyphens".into());
    }
    Ok(())
}

pub(super) fn execute(
    path: &Path,
    run_id: &str,
    sut_version: &str,
    json_path: &Path,
    junit_path: &Path,
    check: bool,
) -> Result<RunResult, CliError> {
    let source = fs::read_to_string(path)
        .map_err(|e| CliError::External(format!("read {}: {e}", path.display())))?;
    let contract: Contract = serde_yaml::from_str(&source)
        .map_err(|e| CliError::External(format!("parse {}: {e}", path.display())))?;
    contract.validate().map_err(CliError::External)?;
    validate_run_id(run_id).map_err(CliError::External)?;
    if sut_version.is_empty() || sut_version.len() > 128 {
        return Err(CliError::External(
            "sut_version must contain 1..128 characters".into(),
        ));
    }
    if check {
        println!("external MQTT contract valid: {}", path.display());
        return Ok(RunResult::Passed);
    }

    let (mut timeline, mut final_state, outcome) = match Observer::new(&contract, run_id) {
        Ok(mut observer) => {
            let outcome = run_steps(&mut observer, &contract);
            // Always attempt to restore the test path, including after a failed precondition.
            let cleanup = proxy_state(&contract, true);
            let outcome = match cleanup {
                Ok(()) => {
                    observer.record("proxy_cleanup", None, "proxy enabled");
                    outcome
                }
                Err(error) => {
                    observer.record("proxy_cleanup_failed", None, error.to_string());
                    Outcome::Inconclusive(format!(
                        "{}; proxy_cleanup_failed: {error}",
                        outcome.reason()
                    ))
                }
            };
            let mut state = BTreeMap::new();
            if let Some(value) = observer.last_seen {
                state.insert("observed_report".into(), value);
            }
            state.insert(
                "expected_initial".into(),
                json!({"revision":1,"value":contract.initial_value}),
            );
            state.insert(
                "expected_latest".into(),
                json!({"revision":2,"value":contract.latest_value}),
            );
            (observer.timeline, state, outcome)
        }
        Err(error) => (
            Vec::new(),
            BTreeMap::new(),
            Outcome::Inconclusive(format!("observer_setup_failed: {error}")),
        ),
    };
    let reason = outcome.reason();
    timeline.push(TimelineEvent {
        at: Utc::now().to_rfc3339(),
        event_type: "verdict".into(),
        target: None,
        message: reason.clone(),
    });
    final_state.insert("verdict".into(), json!(outcome.verdict()));
    let passed = matches!(outcome, Outcome::Passed);
    let report = RunReport {
        schema_version: "roomci.report.v1".into(),
        run_id: run_id.into(),
        generated_by: format!("roomci {}", env!("CARGO_PKG_VERSION")),
        scenario_name: contract.scenario.clone(),
        result: if passed {
            RunResult::Passed
        } else {
            RunResult::Failed
        },
        timeline,
        assertions: vec![AssertionResult {
            name: "latest_desired_state_after_recovery".into(),
            reference_id: Some("external_mqtt_recovery".into()),
            assertion_type: "external_mqtt_recovery".into(),
            passed,
            message: reason.clone(),
            impact_level: None,
            impact_message: None,
        }],
        final_state: BTreeMap::from([(contract.device_id.clone(), final_state)]),
        retained_messages: BTreeMap::new(),
    };
    let external = ExternalReport {
        report,
        evaluation_target: "external_sut",
        observation_source: "real_mqtt_broker",
        sut_version: sut_version.into(),
        contract,
        verdict: outcome.verdict(),
        reason,
    };
    write_file(json_path, &serde_json::to_string_pretty(&external)?)?;
    write_file(junit_path, &roomci_report::to_junit(&external.report))?;
    println!(
        "external MQTT recovery: {} ({})",
        external.verdict, external.reason
    );
    Ok(external.report.result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn report_requires_exact_revision_and_value() {
        assert!(!matches_report(
            &json!({"revision":1,"value":"new"}),
            2,
            "new"
        ));
        assert!(!matches_report(
            &json!({"revision":2,"value":"old"}),
            2,
            "new"
        ));
        assert!(matches_report(
            &json!({"revision":2,"value":"new"}),
            2,
            "new"
        ));
    }

    #[test]
    fn external_contract_rejects_unsupported_fields_and_empty_assertion_window() {
        let mut value = json!({"scenario":"recovery","broker_host":"localhost","broker_port":1883,
            "proxy_host":"localhost","proxy_api_port":8474,"proxy_port":1884,"proxy_name":"sut",
            "device_id":"device-1","initial_value":"old","latest_value":"new",
            "ready_timeout_ms":1000,"fault_timeout_ms":1000,"recovery_timeout_ms":1000,"stability_ms":100});
        value["unknown"] = json!(true);
        assert!(serde_json::from_value::<Contract>(value.clone()).is_err());
        value.as_object_mut().unwrap().remove("unknown");
        value["stability_ms"] = json!(0);
        assert!(serde_json::from_value::<Contract>(value)
            .unwrap()
            .validate()
            .is_err());
    }

    #[test]
    fn fault_apply_and_release_require_confirmed_proxy_state() {
        for requested in [false, true] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let responder = std::thread::spawn(move || {
                for _ in 0..2 {
                    let (mut stream, _) = listener.accept().unwrap();
                    let mut request = [0u8; 1024];
                    let _ = stream.read(&mut request).unwrap();
                    let body = json!({"name":"sut","enabled":!requested,"listen":"0.0.0.0:1884","upstream":"localhost:1883"}).to_string();
                    write!(
                        stream,
                        "HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                    .unwrap();
                }
            });
            let contract: Contract = serde_json::from_value(json!({
                "scenario":"recovery","broker_host":"localhost","broker_port":1883,
                "proxy_host":"127.0.0.1","proxy_api_port":port,"proxy_port":1884,"proxy_name":"sut",
                "device_id":"device-1","initial_value":"old","latest_value":"new",
                "ready_timeout_ms":1000,"fault_timeout_ms":1000,"recovery_timeout_ms":1000,"stability_ms":100
            })).unwrap();
            assert_eq!(
                proxy_state(&contract, requested).unwrap_err(),
                "proxy state verification failed"
            );
            responder.join().unwrap();
        }
    }

    #[test]
    fn enabled_tcp_listener_without_mqtt_connack_is_not_released_path() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let responder = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut packet = [0u8; 128];
            assert!(stream.read(&mut packet).unwrap() > 0);
            // The proxy TCP socket exists, but its upstream never returns CONNACK.
            std::thread::sleep(Duration::from_millis(400));
        });
        let contract: Contract = serde_json::from_value(json!({
            "scenario":"recovery","broker_host":"localhost","broker_port":1883,
            "proxy_host":"127.0.0.1","proxy_api_port":8474,"proxy_port":port,"proxy_name":"sut",
            "device_id":"device-1","initial_value":"old","latest_value":"new",
            "ready_timeout_ms":1000,"fault_timeout_ms":1000,"recovery_timeout_ms":1000,"stability_ms":100
        })).unwrap();
        assert!(!proxy_mqtt_reachable(&contract, "probe").unwrap());
        responder.join().unwrap();
    }
}
