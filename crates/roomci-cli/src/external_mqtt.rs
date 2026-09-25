//! Real broker recovery check. The embedded broker and virtual-time runner are not used here.

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use chrono::Utc;
use roomci_core::{AssertionResult, RunReport, RunResult, TimelineEvent};
use rumqttc::{Client, ConnectionError, Event, MqttOptions, Packet, QoS, RecvTimeoutError};
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
    events: mpsc::Receiver<(Instant, Result<Event, ConnectionError>)>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    timeline: Vec<TimelineEvent>,
    run_id: String,
    device_id: String,
    reported_topic: String,
    status_topic: String,
    last_report: Option<(Instant, Value)>,
    last_seen: Option<Value>,
    status: Option<String>,
    pubacks: usize,
    subacks: usize,
    ready: bool,
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
        let (client, mut connection) = Client::new(options, 20);
        let (sender, events) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = std::thread::spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                match connection.recv_timeout(Duration::from_millis(50)) {
                    Ok(event) => {
                        let failed = event.is_err();
                        if sender.send((Instant::now(), event)).is_err() {
                            break;
                        }
                        if failed {
                            std::thread::sleep(Duration::from_millis(50));
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });
        let prefix = format!("roomci/{run_id}/{}", contract.device_id);
        let reported_topic = format!("{prefix}/reported");
        let status_topic = format!("{prefix}/status");
        Ok(Self {
            client,
            events,
            stop,
            worker: Some(worker),
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
            ready: false,
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
            .events
            .recv_timeout(remaining.min(Duration::from_millis(250)))
        {
            Ok((received_at, event)) => self.process_event(event, received_at),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(()),
            Err(error) => Err(format!("observer receive: {error:?}")),
        }
    }

    fn poll_ready(&mut self) -> Result<bool, String> {
        match self.events.try_recv() {
            Ok((received_at, event)) => {
                self.process_event(event, received_at)?;
                Ok(true)
            }
            Err(mpsc::TryRecvError::Empty) => Ok(false),
            Err(error) => Err(format!("observer receive: {error:?}")),
        }
    }

    fn process_event(
        &mut self,
        event: Result<Event, ConnectionError>,
        received_at: Instant,
    ) -> Result<(), String> {
        match event {
            Ok(Event::Incoming(Packet::ConnAck(_))) if !self.ready => {
                self.subacks = 0;
                self.client
                    .subscribe(&self.reported_topic, QoS::AtLeastOnce)
                    .map_err(|e| e.to_string())?;
                self.client
                    .subscribe(&self.status_topic, QoS::AtLeastOnce)
                    .map_err(|e| e.to_string())?;
            }
            Ok(Event::Incoming(Packet::SubAck(_))) => {
                self.subacks += 1;
                if self.subacks >= 2 {
                    self.ready = true;
                }
            }
            Ok(Event::Incoming(Packet::PubAck(_))) => self.pubacks += 1,
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                if publish.topic == self.reported_topic {
                    if publish.retain {
                        let message = serde_json::from_slice::<Value>(&publish.payload)
                            .map(|value| value.to_string())
                            .unwrap_or_else(|_| "invalid retained report".into());
                        self.record("ignored_retained_report", Some(publish.topic), message);
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
                            self.last_report = Some((received_at, value));
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
            Err(error)
                if !self.ready
                    && matches!(
                        error,
                        ConnectionError::Io(_)
                            | ConnectionError::NetworkTimeout
                            | ConnectionError::FlushTimeout
                    ) =>
            {
                self.record("observer_connect_retry", None, error.to_string());
            }
            Err(error) => return Err(format!("observer connection: {error}")),
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
            if predicate(self) {
                return Ok(true);
            }
            self.poll(deadline)?;
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

impl Drop for Observer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn resolved_addresses(host: &str, port: u16) -> Result<Vec<SocketAddr>, String> {
    let addresses: Vec<_> = (host, port)
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .collect();
    if addresses.is_empty() {
        return Err(format!("no addresses for {host}:{port}"));
    }
    Ok(addresses)
}

fn connect_candidates(
    addresses: &[SocketAddr],
    deadline: Instant,
) -> Result<Option<TcpStream>, String> {
    for (index, address) in addresses.iter().enumerate() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("connection deadline exceeded before all addresses were tried".into());
        }
        // Share the remaining connection time so an unreachable first address cannot starve later ones.
        let allowance = (remaining / (addresses.len() - index) as u32)
            .max(Duration::from_millis(1))
            .min(remaining);
        if let Ok(stream) = TcpStream::connect_timeout(address, allowance) {
            if Instant::now() <= deadline {
                return Ok(Some(stream));
            }
            return Err("connection completed after deadline".into());
        }
    }
    Ok(None)
}

fn write_before(stream: &mut TcpStream, mut bytes: &[u8], deadline: Instant) -> Result<(), String> {
    while !bytes.is_empty() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("write deadline exceeded".into());
        }
        stream
            .set_write_timeout(Some(remaining))
            .map_err(|e| e.to_string())?;
        let count = stream.write(bytes).map_err(|e| e.to_string())?;
        if count == 0 {
            return Err("connection closed during write".into());
        }
        bytes = &bytes[count..];
    }
    if Instant::now() > deadline {
        return Err("write completed after deadline".into());
    }
    Ok(())
}

fn proxy_request(
    contract: &Contract,
    method: &str,
    path: &str,
    body: Option<&str>,
    deadline: Instant,
) -> Result<Value, String> {
    let addresses = resolved_addresses(&contract.proxy_host, contract.proxy_api_port)?;
    let mut stream = connect_candidates(&addresses, deadline)?
        .ok_or("proxy API unavailable at every resolved address")?;
    let body = body.unwrap_or("");
    let request = format!("{method} {path} HTTP/1.0\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", contract.proxy_host, body.len());
    write_before(&mut stream, request.as_bytes(), deadline)?;
    let mut response = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("proxy API response deadline exceeded".into());
        }
        stream
            .set_read_timeout(Some(remaining))
            .map_err(|e| e.to_string())?;
        let mut chunk = [0u8; 4096];
        let count = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        if response.len() + count > 64 * 1024 {
            return Err("proxy API response too large".into());
        }
        response.extend_from_slice(&chunk[..count]);
    }
    if Instant::now() > deadline {
        return Err("proxy API response completed after deadline".into());
    }
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

fn proxy_state(contract: &Contract, enabled: bool, deadline: Instant) -> Result<(), String> {
    let path = format!("/proxies/{}", contract.proxy_name);
    let response = proxy_request(
        contract,
        "POST",
        &path,
        Some(&json!({"enabled":enabled}).to_string()),
        deadline,
    )?;
    let observed = proxy_request(contract, "GET", &path, None, deadline)?;
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

fn proxy_mqtt_reachable(
    contract: &Contract,
    run_id: &str,
    deadline: Instant,
) -> Result<bool, String> {
    let addresses = resolved_addresses(&contract.proxy_host, contract.proxy_port)?;
    mqtt_reachable_at(&addresses, run_id, deadline)
}

fn mqtt_reachable_at(
    addresses: &[SocketAddr],
    run_id: &str,
    deadline: Instant,
) -> Result<bool, String> {
    if addresses.is_empty() {
        return Ok(false);
    }
    let cancelled = AtomicBool::new(false);
    let (sender, receiver) = mpsc::channel();
    let reachable = std::thread::scope(|scope| {
        for (index, address) in addresses.iter().copied().enumerate() {
            let sender = sender.clone();
            let cancelled = &cancelled;
            scope.spawn(move || {
                let _ = sender.send(mqtt_probe_address(
                    address,
                    &format!("roomci-probe-{run_id}-{index}"),
                    deadline,
                    cancelled,
                ));
            });
        }
        drop(sender);
        let mut reachable = false;
        for _ in addresses {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            match receiver.recv_timeout(remaining) {
                Ok(Some(received_at)) if received_at <= deadline => {
                    reachable = true;
                    break;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        cancelled.store(true, Ordering::Relaxed);
        // Scoped workers own their sockets and exit after at most one short I/O wait.
        reachable
    });
    Ok(reachable
        || receiver
            .try_iter()
            .any(|received_at| received_at.is_some_and(|at| at <= deadline)))
}

fn mqtt_probe_address(
    address: SocketAddr,
    client_id: &str,
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Option<Instant> {
    let mut stream = loop {
        if cancelled.load(Ordering::Relaxed) {
            return None;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        match TcpStream::connect_timeout(&address, remaining.min(Duration::from_millis(25))) {
            Ok(stream) => break stream,
            Err(_) => std::thread::sleep(remaining.min(Duration::from_millis(10))),
        }
    };
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
    if write_before(&mut stream, &connect, deadline).is_err() {
        return None;
    }
    let mut connack = [0; 4];
    let mut received = 0;
    while received < connack.len() && !cancelled.load(Ordering::Relaxed) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        if stream
            .set_read_timeout(Some(remaining.min(Duration::from_millis(25))))
            .is_err()
        {
            return None;
        }
        match stream.read(&mut connack[received..]) {
            Ok(0) => return None,
            Ok(count) => received += count,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) => {}
            Err(_) => return None,
        }
    }
    let received_at = Instant::now();
    (connack == [0x20, 0x02, 0x00, 0x00] && received_at <= deadline).then_some(received_at)
}

fn proxy_is_blocked(contract: &Contract, deadline: Instant) -> Result<bool, String> {
    let addresses = resolved_addresses(&contract.proxy_host, contract.proxy_port)?;
    all_candidates_blocked(&addresses, deadline)
}

fn all_candidates_blocked(addresses: &[SocketAddr], deadline: Instant) -> Result<bool, String> {
    let mut unverified = Vec::new();
    for (index, address) in addresses.iter().enumerate() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(
                "block verification deadline exceeded before all addresses were checked".into(),
            );
        }
        let allowance = (remaining / (addresses.len() - index) as u32)
            .max(Duration::from_millis(1))
            .min(remaining);
        match TcpStream::connect_timeout(address, allowance) {
            Ok(_) => return Ok(false),
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {}
            Err(error) => unverified.push(format!("{address}: {error}")),
        }
    }
    if Instant::now() > deadline {
        return Err("block verification completed after deadline".into());
    }
    if unverified.is_empty() {
        Ok(true)
    } else {
        Err(format!(
            "block verification unavailable: {}",
            unverified.join(", ")
        ))
    }
}

fn wait_for_proxy_mqtt(contract: &Contract, run_id: &str, deadline: Instant) -> Result<(), String> {
    while Instant::now() < deadline {
        if proxy_mqtt_reachable(contract, run_id, deadline)? && Instant::now() <= deadline {
            return Ok(());
        }
        std::thread::sleep(
            deadline
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(50)),
        );
    }
    Err("MQTT path unavailable before recovery deadline".into())
}

fn matches_report(value: &Value, revision: u64, expected: &str) -> bool {
    value.get("revision").and_then(Value::as_u64) == Some(revision)
        && value.get("value").and_then(Value::as_str) == Some(expected)
}

fn assess_recovery_report(
    observer: &mut Observer,
    contract: &Contract,
    deadline: Instant,
    stable_until: &mut Option<Instant>,
    failure: &mut Option<&'static str>,
) {
    let Some((received_at, value)) = observer.last_report.take() else {
        return;
    };
    if (stable_until.is_none() && received_at > deadline)
        || stable_until.is_some_and(|until| received_at > until)
    {
        observer.record("late_report", None, value.to_string());
    } else if value.get("revision").and_then(Value::as_u64) == Some(2) {
        if !matches_report(&value, 2, &contract.latest_value) {
            *failure = Some(if stable_until.is_some() {
                "rollback_observed"
            } else {
                "wrong_latest_value"
            });
        } else if stable_until.is_none() {
            *stable_until = Some(received_at + Duration::from_millis(contract.stability_ms));
            observer.record("latest_reached", None, value.to_string());
        }
    } else if stable_until.is_some() {
        *failure = Some("rollback_observed");
    } else {
        observer.record("stale_report", None, value.to_string());
    }
}

fn observe_recovery(
    observer: &mut Observer,
    contract: &Contract,
    deadline: Instant,
) -> Result<Outcome, String> {
    std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel();
        let run_id = observer.run_id.clone();
        scope.spawn(move || {
            let _ = sender.send(wait_for_proxy_mqtt(contract, &run_id, deadline));
        });
        let mut probe_result = None;
        let mut path_recovered = false;
        let mut stable_until = None;
        let mut failure = None;
        loop {
            if probe_result.is_none() {
                if let Ok(result) = receiver.try_recv() {
                    probe_result = Some(result);
                }
            }
            if !path_recovered {
                if let Some(result) = &probe_result {
                    if let Err(error) = result {
                        return Ok(Outcome::Inconclusive(format!(
                            "fault_release_failed: {error}"
                        )));
                    }
                    path_recovered = true;
                    observer.record(
                        "fault_released",
                        Some(contract.proxy_name.clone()),
                        "proxy enabled and GET confirmed; MQTT CONNACK through proxy confirmed",
                    );
                }
            }
            if path_recovered {
                if let Some(reason) = failure {
                    return Ok(Outcome::Failed(reason));
                }
                if stable_until.is_some_and(|until| Instant::now() >= until) {
                    // A queued rollback must be considered before the verdict is frozen.
                    for _ in 0..1024 {
                        if !observer.poll_ready()? {
                            return Ok(Outcome::Passed);
                        }
                        assess_recovery_report(
                            observer,
                            contract,
                            deadline,
                            &mut stable_until,
                            &mut failure,
                        );
                        if let Some(reason) = failure {
                            return Ok(Outcome::Failed(reason));
                        }
                    }
                    return Ok(Outcome::Inconclusive("observer_queue_not_drained".into()));
                }
            }
            let now = Instant::now();
            if now >= deadline && probe_result.is_none() {
                probe_result = Some(
                    receiver
                        .recv()
                        .map_err(|_| "MQTT probe ended without a result".to_string())?,
                );
                continue;
            }
            if now >= deadline && stable_until.is_none() {
                return Ok(Outcome::Failed("missing_latest_report"));
            }
            let poll_until = stable_until.unwrap_or(deadline);
            if now < poll_until {
                observer.poll(poll_until)?;
                assess_recovery_report(
                    observer,
                    contract,
                    deadline,
                    &mut stable_until,
                    &mut failure,
                );
            } else if !path_recovered {
                std::thread::sleep(
                    deadline
                        .saturating_duration_since(now)
                        .min(Duration::from_millis(50)),
                );
            }
        }
    })
}

fn run_steps(observer: &mut Observer, contract: &Contract) -> Outcome {
    let result = (|| -> Result<Outcome, String> {
        if !observer.wait_for(
            Instant::now() + Duration::from_millis(contract.ready_timeout_ms),
            |s| s.ready,
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
        let initial_deadline = Instant::now() + Duration::from_millis(contract.ready_timeout_ms);
        if !observer.wait_for(initial_deadline, |s| {
            s.last_report.as_ref().is_some_and(|(received_at, value)| {
                *received_at <= initial_deadline
                    && matches_report(value, 1, &contract.initial_value)
            })
        })? {
            return Ok(Outcome::Inconclusive("initial_report_missing".into()));
        }
        observer.record("initial_ready", None, "SUT reported initial revision 1");
        observer.status = None;
        observer.record(
            "fault_requested",
            Some(contract.proxy_name.clone()),
            "disable SUT-only TCP proxy",
        );
        let fault_deadline = Instant::now() + Duration::from_millis(contract.fault_timeout_ms);
        proxy_state(contract, false, fault_deadline)
            .map_err(|e| format!("fault_not_applied: {e}"))?;
        observer.record(
            "fault_applied",
            Some(contract.proxy_name.clone()),
            "proxy disabled and GET confirmed",
        );
        if !proxy_is_blocked(contract, fault_deadline)
            .map_err(|e| format!("fault_not_applied: {e}"))?
        {
            return Ok(Outcome::Inconclusive(
                "fault_not_applied: proxy_still_accepts_connections".into(),
            ));
        }
        if !observer.wait_for(fault_deadline, |s| s.status.as_deref() == Some("offline"))? {
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
        let deadline = Instant::now() + Duration::from_millis(contract.recovery_timeout_ms);
        proxy_state(contract, true, deadline).map_err(|e| format!("fault_release_failed: {e}"))?;
        observe_recovery(observer, contract, deadline)
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
            let cleanup = proxy_state(
                &contract,
                true,
                Instant::now() + Duration::from_millis(contract.fault_timeout_ms),
            );
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
            if let Some(value) = observer.last_seen.take() {
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
            (std::mem::take(&mut observer.timeline), state, outcome)
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

    fn test_contract(proxy_port: u16) -> Contract {
        serde_json::from_value(json!({
            "scenario":"recovery","broker_host":"localhost","broker_port":1883,
            "proxy_host":"127.0.0.1","proxy_api_port":8474,"proxy_port":proxy_port,"proxy_name":"sut",
            "device_id":"device-1","initial_value":"old","latest_value":"new",
            "ready_timeout_ms":2000,"fault_timeout_ms":1000,"recovery_timeout_ms":5000,"stability_ms":100
        }))
        .unwrap()
    }

    fn read_mqtt_packet(stream: &mut TcpStream) -> Vec<u8> {
        let mut header = [0u8; 1];
        stream.read_exact(&mut header).unwrap();
        let mut remaining = 0usize;
        for shift in (0..28).step_by(7) {
            let mut digit = [0u8; 1];
            stream.read_exact(&mut digit).unwrap();
            remaining += ((digit[0] & 0x7f) as usize) << shift;
            if digit[0] & 0x80 == 0 {
                break;
            }
        }
        let mut payload = vec![0; remaining];
        stream.read_exact(&mut payload).unwrap();
        [vec![header[0]], payload].concat()
    }

    fn read_probe_client_id(stream: &mut TcpStream) -> String {
        let packet = read_mqtt_packet(stream);
        assert_eq!(packet[0], 0x10);
        let length = u16::from_be_bytes([packet[11], packet[12]]) as usize;
        String::from_utf8(packet[13..13 + length].to_vec()).unwrap()
    }

    #[test]
    fn initial_report_wait_handles_early_late_and_absent_reports() {
        for report_first in [Some(true), Some(false), None] {
            let broker = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = broker.local_addr().unwrap().port();
            let responder = std::thread::spawn(move || {
                let (mut stream, _) = broker.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                assert_eq!(read_mqtt_packet(&mut stream)[0], 0x10);
                stream.write_all(&[0x20, 0x02, 0x00, 0x00]).unwrap();
                for _ in 0..2 {
                    let packet = read_mqtt_packet(&mut stream);
                    stream
                        .write_all(&[0x90, 0x03, packet[1], packet[2], 0x01])
                        .unwrap();
                }
                let desired = read_mqtt_packet(&mut stream);
                assert_eq!(desired[0] & 0xf0, 0x30);
                let topic_len = u16::from_be_bytes([desired[1], desired[2]]) as usize;
                let puback = [0x40, 0x02, desired[3 + topic_len], desired[4 + topic_len]];
                let topic = b"roomci/initial/device-1/reported";
                let payload =
                    br#"{"run_id":"initial","device_id":"device-1","revision":1,"value":"old"}"#;
                let length = 2 + topic.len() + payload.len();
                assert!(length < 128);
                let mut report = vec![0x30, length as u8];
                report.extend_from_slice(&(topic.len() as u16).to_be_bytes());
                report.extend_from_slice(topic);
                report.extend_from_slice(payload);
                match report_first {
                    Some(true) => {
                        stream.write_all(&report).unwrap();
                        std::thread::sleep(Duration::from_millis(20));
                        stream.write_all(&puback).unwrap();
                    }
                    Some(false) => {
                        stream.write_all(&puback).unwrap();
                        std::thread::sleep(Duration::from_millis(20));
                        stream.write_all(&report).unwrap();
                    }
                    None => stream.write_all(&puback).unwrap(),
                }
                std::thread::sleep(Duration::from_millis(200));
            });
            let mut contract = test_contract(1884);
            contract.broker_host = "127.0.0.1".into();
            contract.broker_port = port;
            contract.ready_timeout_ms = 100;
            let mut observer = Observer::new(&contract, "initial").unwrap();
            assert!(observer
                .wait_for(Instant::now() + Duration::from_secs(1), |s| s.ready)
                .unwrap());
            observer
                .publish_desired(
                    &contract,
                    1,
                    &contract.initial_value,
                    Instant::now() + Duration::from_millis(contract.ready_timeout_ms),
                )
                .unwrap();
            let deadline = Instant::now() + Duration::from_millis(contract.ready_timeout_ms);
            assert_eq!(
                observer
                    .wait_for(deadline, |s| {
                        s.last_report.as_ref().is_some_and(|(received_at, value)| {
                            *received_at <= deadline
                                && matches_report(value, 1, &contract.initial_value)
                        })
                    })
                    .unwrap(),
                report_first.is_some(),
                "wrong initial report verdict when report_first={report_first:?}"
            );
            responder.join().unwrap();
        }
    }

    #[test]
    fn mqtt_probe_accepts_400ms_connack_with_one_or_two_addresses_in_600ms() {
        for two_addresses in [false, true] {
            let ipv4 = TcpListener::bind("127.0.0.1:0").unwrap();
            let ipv6 = TcpListener::bind("[::1]:0").unwrap();
            let addresses = if two_addresses {
                vec![ipv6.local_addr().unwrap(), ipv4.local_addr().unwrap()]
            } else {
                vec![ipv4.local_addr().unwrap()]
            };
            let (ids, received_ids) = mpsc::channel();
            let ipv4_ids = ids.clone();
            let ipv4_responder = std::thread::spawn(move || {
                let (mut stream, _) = ipv4.accept().unwrap();
                ipv4_ids.send(read_probe_client_id(&mut stream)).unwrap();
                std::thread::sleep(Duration::from_millis(400));
                let _ = stream.write_all(&[0x20, 0x02, 0x00, 0x00]);
            });
            let ipv6_responder = two_addresses.then(|| {
                std::thread::spawn(move || {
                    let (mut stream, _) = ipv6.accept().unwrap();
                    ids.send(read_probe_client_id(&mut stream)).unwrap();
                    std::thread::sleep(Duration::from_millis(400));
                    let _ = stream.write_all(&[0x20, 0x02, 0x00, 0x00]);
                })
            });
            assert!(mqtt_reachable_at(
                &addresses,
                "dual",
                Instant::now() + Duration::from_millis(600)
            )
            .unwrap());
            ipv4_responder.join().unwrap();
            if let Some(responder) = ipv6_responder {
                responder.join().unwrap();
            }
            let ids: Vec<_> = received_ids.try_iter().collect();
            assert_eq!(ids.len(), if two_addresses { 2 } else { 1 });
            if two_addresses {
                assert_ne!(ids[0], ids[1], "probe candidates reused a client ID");
            }
        }
    }

    #[test]
    fn mqtt_probe_rejects_unavailable_candidates_and_late_connack() {
        let ipv4 = TcpListener::bind("127.0.0.1:0").unwrap();
        let ipv6 = TcpListener::bind("[::1]:0").unwrap();
        let addresses = [ipv6.local_addr().unwrap(), ipv4.local_addr().unwrap()];
        drop(ipv4);
        drop(ipv6);
        let start = Instant::now();
        assert!(
            !mqtt_reachable_at(&addresses, "absent", start + Duration::from_millis(250)).unwrap()
        );
        assert!(start.elapsed() < Duration::from_millis(500));
        let contract = test_contract(addresses[1].port());
        assert!(wait_for_proxy_mqtt(
            &contract,
            "absent",
            Instant::now() + Duration::from_millis(250)
        )
        .unwrap_err()
        .contains("recovery deadline"));

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let responder = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            assert_eq!(read_mqtt_packet(&mut stream)[0], 0x10);
            std::thread::sleep(Duration::from_millis(350));
            let _ = stream.write_all(&[0x20, 0x02, 0x00, 0x00]);
        });
        let start = Instant::now();
        assert!(
            !mqtt_reachable_at(&[address], "late", start + Duration::from_millis(250)).unwrap()
        );
        assert!(start.elapsed() < Duration::from_millis(500));
        responder.join().unwrap();
    }

    #[test]
    fn observer_waits_for_broker_start_within_ready_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let responder = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            assert_eq!(read_mqtt_packet(&mut stream)[0], 0x10);
            stream.write_all(&[0x20, 0x02, 0x00, 0x00]).unwrap();
            for _ in 0..2 {
                let packet = read_mqtt_packet(&mut stream);
                assert_eq!(packet[0], 0x82);
                stream
                    .write_all(&[0x90, 0x03, packet[1], packet[2], 0x01])
                    .unwrap();
            }
            std::thread::sleep(Duration::from_millis(500));
        });
        let contract: Contract = serde_json::from_value(json!({
            "scenario":"recovery","broker_host":"127.0.0.1","broker_port":port,
            "proxy_host":"127.0.0.1","proxy_api_port":8474,"proxy_port":1884,"proxy_name":"sut",
            "device_id":"device-1","initial_value":"old","latest_value":"new",
            "ready_timeout_ms":2000,"fault_timeout_ms":1000,"recovery_timeout_ms":1000,"stability_ms":100
        })).unwrap();
        let mut observer = Observer::new(&contract, "delayed").unwrap();
        assert!(observer
            .wait_for(Instant::now() + Duration::from_secs(2), |s| s.subacks >= 2)
            .unwrap());
        responder.join().unwrap();
        assert!(observer
            .poll(Instant::now() + Duration::from_secs(1))
            .is_err());
    }

    #[test]
    fn observer_readiness_expires_when_broker_never_starts() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let mut contract = test_contract(1884);
        contract.broker_host = "127.0.0.1".into();
        contract.broker_port = port;
        let mut observer = Observer::new(&contract, "absent").unwrap();
        let start = Instant::now();
        assert!(!observer
            .wait_for(start + Duration::from_millis(350), |s| s.ready)
            .unwrap());
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn later_address_can_connect_but_one_refusal_does_not_prove_blocking() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let live = listener.local_addr().unwrap();
        let refused_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let refused = refused_listener.local_addr().unwrap();
        drop(refused_listener);
        let addresses = [refused, live];
        assert!(
            connect_candidates(&addresses, Instant::now() + Duration::from_secs(1))
                .unwrap()
                .is_some()
        );
        assert!(
            !all_candidates_blocked(&addresses, Instant::now() + Duration::from_secs(1)).unwrap()
        );
        assert!(
            all_candidates_blocked(&[refused], Instant::now() + Duration::from_secs(1)).unwrap()
        );
    }

    #[test]
    fn mqtt_probe_tries_next_address_after_missing_connack() {
        let first = TcpListener::bind("127.0.0.1:0").unwrap();
        let second = TcpListener::bind("127.0.0.1:0").unwrap();
        let addresses = [first.local_addr().unwrap(), second.local_addr().unwrap()];
        let first_thread = std::thread::spawn(move || {
            let (mut stream, _) = first.accept().unwrap();
            let mut packet = [0u8; 128];
            assert!(stream.read(&mut packet).unwrap() > 0);
            std::thread::sleep(Duration::from_millis(1100));
        });
        let second_thread = std::thread::spawn(move || {
            let (mut stream, _) = second.accept().unwrap();
            let mut packet = [0u8; 128];
            assert!(stream.read(&mut packet).unwrap() > 0);
            stream.write_all(&[0x20, 0x02, 0x00, 0x00]).unwrap();
        });
        assert!(
            mqtt_reachable_at(&addresses, "multi", Instant::now() + Duration::from_secs(2))
                .unwrap()
        );
        first_thread.join().unwrap();
        second_thread.join().unwrap();
    }

    #[test]
    fn probe_waits_for_path_after_one_second_without_extending_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let responder = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1200));
            let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            let mut packet = [0u8; 128];
            assert!(stream.read(&mut packet).unwrap() > 0);
            stream.write_all(&[0x20, 0x02, 0x00, 0x00]).unwrap();
        });
        let contract = test_contract(port);
        assert!(
            wait_for_proxy_mqtt(&contract, "late", Instant::now() + Duration::from_secs(3)).is_ok()
        );
        responder.join().unwrap();
        let absent = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = absent.local_addr().unwrap().port();
        drop(absent);
        let contract = test_contract(port);
        let start = Instant::now();
        assert!(
            wait_for_proxy_mqtt(&contract, "timeout", start + Duration::from_millis(350))
                .unwrap_err()
                .contains("recovery deadline")
        );
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn observer_records_latest_report_while_proxy_probe_is_waiting() {
        let broker = TcpListener::bind("127.0.0.1:0").unwrap();
        let broker_port = broker.local_addr().unwrap().port();
        let broker_thread = std::thread::spawn(move || {
            let (mut stream, _) = broker.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            assert_eq!(read_mqtt_packet(&mut stream)[0], 0x10);
            stream.write_all(&[0x20, 0x02, 0x00, 0x00]).unwrap();
            for _ in 0..2 {
                let packet = read_mqtt_packet(&mut stream);
                stream
                    .write_all(&[0x90, 0x03, packet[1], packet[2], 0x01])
                    .unwrap();
            }
            std::thread::sleep(Duration::from_millis(100));
            let topic = b"roomci/concurrent/device-1/reported";
            let payload =
                br#"{"run_id":"concurrent","device_id":"device-1","revision":2,"value":"new"}"#;
            let length = 2 + topic.len() + payload.len();
            assert!(length < 128);
            let mut packet = vec![0x30, length as u8];
            packet.extend_from_slice(&(topic.len() as u16).to_be_bytes());
            packet.extend_from_slice(topic);
            packet.extend_from_slice(payload);
            stream.write_all(&packet).unwrap();
            std::thread::sleep(Duration::from_millis(1500));
        });
        let probe = TcpListener::bind("127.0.0.1:0").unwrap();
        let probe_port = probe.local_addr().unwrap().port();
        let probe_thread = std::thread::spawn(move || {
            let (mut stream, _) = probe.accept().unwrap();
            let mut packet = [0u8; 128];
            assert!(stream.read(&mut packet).unwrap() > 0);
            std::thread::sleep(Duration::from_millis(800));
            stream.write_all(&[0x20, 0x02, 0x00, 0x00]).unwrap();
        });
        let mut contract = test_contract(probe_port);
        contract.broker_host = "127.0.0.1".into();
        contract.broker_port = broker_port;
        let mut observer = Observer::new(&contract, "concurrent").unwrap();
        assert!(observer
            .wait_for(Instant::now() + Duration::from_secs(2), |s| s.ready)
            .unwrap());
        let outcome = observe_recovery(
            &mut observer,
            &contract,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
        assert!(matches!(outcome, Outcome::Passed));
        let latest = observer
            .timeline
            .iter()
            .position(|event| event.event_type == "latest_reached")
            .unwrap();
        let released = observer
            .timeline
            .iter()
            .position(|event| event.event_type == "fault_released")
            .unwrap();
        assert!(latest < released, "observer did not progress during probe");
        broker_thread.join().unwrap();
        probe_thread.join().unwrap();
    }

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
    fn report_receipt_time_sets_recovery_and_stability_boundaries() {
        let contract = test_contract(1884);
        let mut observer = Observer::new(&contract, "timed").unwrap();
        let now = Instant::now();
        let deadline = now + Duration::from_millis(50);
        let stable_end = now + Duration::from_millis(100);
        let mut stable_until = None;
        let mut failure = None;
        observer.last_report = Some((
            deadline + Duration::from_millis(1),
            json!({"revision":2,"value":"new"}),
        ));
        assess_recovery_report(
            &mut observer,
            &contract,
            deadline,
            &mut stable_until,
            &mut failure,
        );
        assert!(stable_until.is_none());
        assert!(failure.is_none());

        stable_until = Some(stable_end);
        observer.last_report = Some((
            stable_end + Duration::from_millis(1),
            json!({"revision":1,"value":"old"}),
        ));
        assess_recovery_report(
            &mut observer,
            &contract,
            deadline,
            &mut stable_until,
            &mut failure,
        );
        assert!(
            failure.is_none(),
            "post-window rollback changed the verdict"
        );

        observer.last_report = Some((
            stable_end - Duration::from_millis(1),
            json!({"revision":1,"value":"old"}),
        ));
        assess_recovery_report(
            &mut observer,
            &contract,
            deadline,
            &mut stable_until,
            &mut failure,
        );
        assert_eq!(failure, Some("rollback_observed"));
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
                proxy_state(
                    &contract,
                    requested,
                    Instant::now() + Duration::from_secs(2)
                )
                .unwrap_err(),
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
        assert!(!proxy_mqtt_reachable(
            &contract,
            "probe",
            Instant::now() + Duration::from_millis(300)
        )
        .unwrap());
        responder.join().unwrap();
    }

    #[test]
    fn probe_accepts_connack_within_recovery_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let responder = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut packet = [0u8; 128];
            assert!(stream.read(&mut packet).unwrap() > 0);
            std::thread::sleep(Duration::from_millis(400));
            stream.write_all(&[0x20, 0x02, 0x00, 0x00]).unwrap();
        });
        let contract: Contract = serde_json::from_value(json!({
            "scenario":"recovery","broker_host":"localhost","broker_port":1883,
            "proxy_host":"127.0.0.1","proxy_api_port":8474,"proxy_port":port,"proxy_name":"sut",
            "device_id":"device-1","initial_value":"old","latest_value":"new",
            "ready_timeout_ms":1000,"fault_timeout_ms":1000,"recovery_timeout_ms":5000,"stability_ms":100
        })).unwrap();
        assert!(
            proxy_mqtt_reachable(&contract, "probe", Instant::now() + Duration::from_secs(5))
                .unwrap()
        );
        responder.join().unwrap();
    }
}
