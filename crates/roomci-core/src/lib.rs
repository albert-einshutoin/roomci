use std::collections::BTreeMap;

use chrono::Duration;
use roomci_device_model::{
    ContactModel, DeviceModelError, LightingEvent, LightingModel, ModbusModel,
};
use roomci_edge::{EdgeError, EdgeModel, EdgeStatus};
use roomci_mqtt::{device_id_from_command_topic, BrokerModel, MqttError};
use roomci_ops::{OpsError, OpsEvent, OpsModel};
use roomci_scenario::{
    parse_duration, resolve_time_offset, validate_scenario, yaml_map_to_json, MqttPublishStep,
    ScenarioError, ScenarioFile,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type StateMap = BTreeMap<String, serde_json::Value>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
    #[error(transparent)]
    Mqtt(#[from] MqttError),
    #[error(transparent)]
    Edge(#[from] EdgeError),
    #[error(transparent)]
    DeviceModel(#[from] DeviceModelError),
    #[error(transparent)]
    Ops(#[from] OpsError),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RunReport {
    pub scenario_name: String,
    pub result: RunResult,
    pub timeline: Vec<TimelineEvent>,
    pub assertions: Vec<AssertionResult>,
    pub final_state: BTreeMap<String, StateMap>,
    pub retained_messages: BTreeMap<String, StateMap>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunResult {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TimelineEvent {
    pub at: String,
    pub event_type: String,
    pub target: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AssertionResult {
    pub name: String,
    pub assertion_type: String,
    pub passed: bool,
    pub message: String,
    pub impact_level: Option<String>,
    pub impact_message: Option<String>,
}

#[derive(Debug)]
struct RuntimeState {
    states: BTreeMap<String, StateMap>,
    protocols: BTreeMap<String, Option<String>>,
    broker: BrokerModel,
    edge: EdgeModel,
    edge_primary_failed_at: Option<Duration>,
    edge_failover_at: Option<Duration>,
    edge_expected_within: Option<Duration>,
    duplicate_delivery_by_topic: BTreeMap<String, u32>,
    modbus: ModbusModel,
    lighting: LightingModel,
    contacts: ContactModel,
    ops: OpsModel,
    timeline: Vec<TimelineEvent>,
}

pub fn run_scenario(scenario: &ScenarioFile) -> Result<RunReport, CoreError> {
    validate_scenario(scenario)?;

    let mut runtime = RuntimeState::new(scenario)?;
    let mut events = Vec::new();
    for fault in &scenario.faults {
        let at = fault
            .at
            .as_deref()
            .map(resolve_time_offset)
            .transpose()?
            .unwrap_or_else(Duration::zero);
        events.push(ScheduledEvent::GlobalFault(at, fault.clone()));
    }
    for step in &scenario.steps {
        events.push(ScheduledEvent::Step(
            resolve_time_offset(&step.at)?,
            step.clone(),
        ));
    }
    events.sort_by_key(|event| event.at().num_milliseconds());

    for event in events {
        runtime.apply_event(event)?;
    }

    let mut assertions = Vec::new();
    for assertion in &scenario.assertions {
        assertions.push(runtime.evaluate_assertion(assertion));
    }

    let result = if assertions.iter().all(|assertion| assertion.passed) {
        RunResult::Passed
    } else {
        RunResult::Failed
    };

    Ok(RunReport {
        scenario_name: scenario.scenario.name.clone(),
        result,
        timeline: runtime.timeline,
        assertions,
        final_state: runtime.states,
        retained_messages: runtime.broker.retained().clone(),
    })
}

#[derive(Debug)]
enum ScheduledEvent {
    GlobalFault(Duration, roomci_scenario::FaultStep),
    Step(Duration, roomci_scenario::ScenarioStep),
}

impl ScheduledEvent {
    fn at(&self) -> Duration {
        match self {
            ScheduledEvent::GlobalFault(at, _) | ScheduledEvent::Step(at, _) => *at,
        }
    }
}

impl RuntimeState {
    fn new(scenario: &ScenarioFile) -> Result<Self, CoreError> {
        let states = scenario
            .devices
            .iter()
            .map(|device| (device.id.clone(), yaml_map_to_json(&device.state)))
            .collect::<BTreeMap<_, _>>();
        let protocols = scenario
            .devices
            .iter()
            .map(|device| (device.id.clone(), device.protocol.clone()))
            .collect::<BTreeMap<_, _>>();

        Ok(Self {
            states,
            protocols,
            broker: BrokerModel::new(true, scenario.mqtt.cloud.enabled),
            edge: EdgeModel::from_config(&scenario.edge),
            edge_primary_failed_at: None,
            edge_failover_at: None,
            edge_expected_within: edge_expected_within(&scenario.edge)?,
            duplicate_delivery_by_topic: BTreeMap::new(),
            modbus: ModbusModel::from_config(&scenario.modbus),
            lighting: LightingModel::from_config(
                &scenario.lighting,
                &scenario
                    .scenes
                    .iter()
                    .map(|(name, scene)| (name.clone(), scene.fixtures.clone()))
                    .collect(),
            ),
            contacts: ContactModel::from_config(&scenario.contacts),
            ops: OpsModel::from_config(&scenario.alerts),
            timeline: Vec::new(),
        })
    }

    fn apply_event(&mut self, event: ScheduledEvent) -> Result<(), CoreError> {
        match event {
            ScheduledEvent::GlobalFault(at, fault) => {
                self.apply_fault(
                    at,
                    &fault.target,
                    &fault.fault_type,
                    fault.topic.as_deref(),
                    fault.count,
                );
            }
            ScheduledEvent::Step(at, step) => {
                if let Some(event) = step.event {
                    self.push(at, "event", None, event);
                }
                if let Some(fault) = step.fault {
                    self.apply_fault(
                        at,
                        &fault.target,
                        &fault.fault_type,
                        fault.topic.as_deref(),
                        fault.count,
                    );
                }
                if let Some(command) = step.command {
                    self.apply_command(at, &command.target, &command.action);
                }
                if let Some(modbus_write) = step.modbus_write {
                    self.apply_modbus_write(
                        at,
                        &modbus_write.device,
                        modbus_write.register,
                        modbus_write.value,
                    );
                }
                if let Some(mqtt_publish) = step.mqtt_publish {
                    self.apply_mqtt_publish(at, &mqtt_publish);
                }
                if let Some(contact) = step.contact {
                    let contact_id = contact.id.clone();
                    let contact_state = contact.state.clone();
                    self.contacts.set_state(&contact_id, &contact_state)?;
                    self.push(
                        at,
                        "contact_changed",
                        Some(contact_id.clone()),
                        format!("contact state changed to {contact_state}"),
                    );
                    for event in self.ops.apply_contact_change(&contact_id, &contact_state) {
                        self.push_ops_event(at, event);
                    }
                }
                if let Some(ops) = step.ops {
                    self.apply_ops_step(at, &ops);
                }
                if step.automation.is_some() {
                    self.push(at, "automation_started", None, "automation started");
                }
            }
        }
        Ok(())
    }

    fn apply_fault(
        &mut self,
        at: Duration,
        target: &str,
        fault_type: &str,
        topic: Option<&str>,
        count: Option<u32>,
    ) {
        if matches!(target, "mqtt.cloud" | "mqtt.local") && fault_type == "offline" {
            self.broker.set_online(target, false);
        }
        if target == "edge.primary" && fault_type == "power_lost" {
            self.edge_primary_failed_at = Some(at);
            if let Some(outcome) = self.edge.apply_power_lost_to_primary() {
                self.edge_failover_at = Some(at);
                self.push(
                    at,
                    "edge_failover",
                    Some(outcome.to),
                    format!("edge failover completed from {}", outcome.from),
                );
            }
        }
        if target == "mqtt.local" && fault_type == "duplicate_delivery" {
            if let Some(topic) = topic {
                self.duplicate_delivery_by_topic
                    .insert(topic.to_string(), count.unwrap_or(2).max(2));
            }
        }
        if let Some(fixture) = target.strip_prefix("dali.fixture.") {
            if fault_type == "command_drop" {
                if let Err(error) = self.lighting.drop_command_for_fixture(fixture) {
                    self.push(
                        at,
                        "fault_rejected",
                        Some(target.to_string()),
                        error.to_string(),
                    );
                    return;
                }
            }
        }
        self.push(
            at,
            "fault_activated",
            Some(target.to_string()),
            format!("{} fault activated", fault_type),
        );
    }

    fn apply_command(&mut self, at: Duration, target: &str, action: &str) {
        self.push(
            at,
            "command_received",
            Some(target.to_string()),
            format!("{} command received", action),
        );
        if action == "activate" {
            if let Some(scene) = target.strip_prefix("scene.") {
                self.activate_scene(at, scene);
            }
        }
    }

    fn apply_modbus_write(
        &mut self,
        at: Duration,
        device: &str,
        register: u32,
        value: serde_yaml::Value,
    ) {
        match self.modbus.write(device, register, value) {
            Ok(json_value) => self.push(
                at,
                "modbus_write",
                Some(device.to_string()),
                format!("register {} set to {}", register, json_value),
            ),
            Err(error) => self.push(
                at,
                "modbus_write_rejected",
                Some(device.to_string()),
                error.to_string(),
            ),
        }
    }

    fn activate_scene(&mut self, at: Duration, scene: &str) {
        match self.lighting.activate_scene(scene) {
            Ok(events) => {
                for event in events {
                    match event {
                        LightingEvent::LevelChanged { fixture, level } => self.push(
                            at,
                            "dali_level_changed",
                            Some(fixture),
                            format!("fixture level changed to {level}"),
                        ),
                        LightingEvent::CommandDropped { fixture } => self.push(
                            at,
                            "dali_command_dropped",
                            Some(fixture),
                            "fixture command dropped".to_string(),
                        ),
                    }
                }
            }
            Err(error) => self.push(
                at,
                "scene_activation_failed",
                Some(scene.to_string()),
                error.to_string(),
            ),
        }
        self.push(
            at,
            "scene_activation_requested",
            Some(scene.to_string()),
            "scene activation requested".to_string(),
        );
    }

    fn apply_mqtt_publish(&mut self, at: Duration, publish: &MqttPublishStep) {
        self.push(
            at,
            "mqtt_publish",
            Some(publish.client.clone()),
            format!("published {}", publish.topic),
        );

        let deliveries = self
            .duplicate_delivery_by_topic
            .get(&publish.topic)
            .copied()
            .unwrap_or(1);
        let Some(device_id) = device_id_from_command_topic(&publish.topic) else {
            self.push(
                at,
                "mqtt_publish_failed",
                Some("mqtt.local".to_string()),
                format!("topic is not a device command topic: {}", publish.topic),
            );
            return;
        };
        match self.edge.route_mqtt_command(
            &publish.client,
            &device_id,
            self.protocols.get(&device_id).cloned().flatten(),
        ) {
            Ok(routed) => self.push(
                at,
                "edge_command_routed",
                Some(routed.edge_id),
                format!(
                    "{} routed command from {} to {}",
                    routed.action, routed.source_client, routed.target_device
                ),
            ),
            Err(error) => {
                self.push(
                    at,
                    "edge_command_failed",
                    Some(device_id),
                    error.to_string(),
                );
                return;
            }
        }

        let payload = yaml_map_to_json(&publish.payload);
        match self.broker.publish_device_command(
            "mqtt.local",
            &publish.client,
            &publish.topic,
            payload,
            deliveries,
        ) {
            Ok(outcome) => {
                if let (Some(device_id), Some(retained_payload)) =
                    (outcome.device_id.clone(), outcome.retained_payload)
                {
                    self.states.insert(device_id.clone(), retained_payload);
                    let state_topic = outcome.state_topic.unwrap_or_default();
                    let delivery_word = if outcome.deliveries == 1 {
                        "delivery"
                    } else {
                        "deliveries"
                    };
                    self.push(
                        at,
                        "mqtt_retained_state_updated",
                        Some(device_id),
                        format!(
                            "retained state updated at {} after {} {}",
                            state_topic, outcome.deliveries, delivery_word
                        ),
                    );
                }
            }
            Err(error) => {
                self.push(
                    at,
                    "mqtt_publish_failed",
                    Some("mqtt.local".to_string()),
                    error.to_string(),
                );
            }
        }
    }

    fn apply_ops_step(&mut self, at: Duration, ops: &BTreeMap<String, serde_yaml::Value>) {
        let action = ops.get("action").and_then(|value| value.as_str());
        if action == Some("acknowledge") {
            let assignee = ops
                .get("assignee")
                .and_then(|value| value.as_str())
                .map(str::to_string);
            for event in self.ops.acknowledge(assignee) {
                self.push_ops_event(at, event);
            }
        }
        self.push(at, "ops_action", None, "ops action applied");
    }

    fn push_ops_event(&mut self, at: Duration, event: OpsEvent) {
        match event {
            OpsEvent::SlackNotificationSent {
                alert_id,
                runbook_url,
            } => self.push(
                at,
                "ops_slack_notification_sent",
                Some(alert_id),
                match runbook_url {
                    Some(url) => format!("Slack notification sent with runbook {url}"),
                    None => "Slack notification sent".to_string(),
                },
            ),
            OpsEvent::PhoneCallTriggered { alert_id } => self.push(
                at,
                "ops_phone_call_triggered",
                Some(alert_id),
                "Phone escalation triggered".to_string(),
            ),
            OpsEvent::TicketOpened { alert_id, status } => self.push(
                at,
                "ops_ticket_opened",
                Some(alert_id),
                format!("Ops ticket opened with status {status}"),
            ),
            OpsEvent::TicketAcknowledged { alert_id, assignee } => self.push(
                at,
                "ops_ticket_acknowledged",
                Some(alert_id),
                match assignee {
                    Some(assignee) => format!("Ops ticket acknowledged by {assignee}"),
                    None => "Ops ticket acknowledged".to_string(),
                },
            ),
            OpsEvent::RunbookUrlIncluded {
                alert_id,
                runbook_url,
            } => self.push(
                at,
                "ops_runbook_url_included",
                Some(alert_id),
                format!("Runbook URL included: {runbook_url}"),
            ),
        }
    }

    fn evaluate_assertion(
        &self,
        assertion: &roomci_scenario::AssertionDefinition,
    ) -> AssertionResult {
        if let Some(mqtt) = &assertion.mqtt {
            let expected = yaml_map_to_json(&mqtt.retained);
            let actual = self.broker.retained().get(&mqtt.topic);
            let passed = actual == Some(&expected);
            return AssertionResult {
                name: format!("mqtt_retained:{}", mqtt.topic),
                assertion_type: "mqtt_retained".to_string(),
                passed,
                message: if passed {
                    "retained MQTT state matched".to_string()
                } else {
                    format!("retained MQTT state mismatch: expected {expected:?}, got {actual:?}")
                },
                impact_level: if passed {
                    None
                } else {
                    Some("high".to_string())
                },
                impact_message: if passed {
                    None
                } else {
                    Some(
                        "Local controller state did not synchronize through retained MQTT state."
                            .to_string(),
                    )
                },
            };
        }

        if let Some(modbus) = &assertion.modbus {
            let actual = self.modbus.value(&modbus.device, modbus.register);
            let passed = if let Some(expected_readable) = modbus.readable_value {
                self.modbus
                    .readable_value(&modbus.device, modbus.register)
                    .map(|value| (value - expected_readable).abs() < f64::EPSILON)
                    .unwrap_or(false)
            } else if let Some(expected) = &modbus.value {
                let expected_json =
                    serde_json::to_value(expected).unwrap_or(serde_json::Value::Null);
                actual == Some(&expected_json)
            } else {
                false
            };
            return AssertionResult {
                name: format!("modbus:{}:{}", modbus.device, modbus.register),
                assertion_type: "modbus_register".to_string(),
                passed,
                message: if passed {
                    "Modbus register matched expected value".to_string()
                } else {
                    format!("Modbus register mismatch: got {actual:?}")
                },
                impact_level: if passed {
                    None
                } else {
                    Some("medium".to_string())
                },
                impact_message: if passed {
                    None
                } else {
                    Some(
                        "Register-map behavior did not match commissioning expectation."
                            .to_string(),
                    )
                },
            };
        }

        if let Some(expected) = &assertion.guest_experience {
            let local_ok = self.broker.is_online("mqtt.local");
            let passed = expected == "unaffected" && local_ok;
            return AssertionResult {
                name: "guest_experience".to_string(),
                assertion_type: "guest_experience".to_string(),
                passed,
                message: if passed {
                    "guest experience remained unaffected by upstream outage".to_string()
                } else {
                    "guest experience was affected".to_string()
                },
                impact_level: if passed {
                    None
                } else {
                    Some("high".to_string())
                },
                impact_message: if passed {
                    None
                } else {
                    Some("Local-first control did not preserve guest experience.".to_string())
                },
            };
        }

        if let Some(ops) = &assertion.ops {
            return match self.ops.evaluate_assertion(ops) {
                Ok(outcome) => AssertionResult {
                    name: "ops".to_string(),
                    assertion_type: "ops".to_string(),
                    passed: outcome.passed,
                    message: if outcome.passed {
                        "ops escalation matched expected state".to_string()
                    } else {
                        format!("ops escalation mismatch: {}", outcome.failures.join("; "))
                    },
                    impact_level: if outcome.passed {
                        None
                    } else {
                        Some("high".to_string())
                    },
                    impact_message: if outcome.passed {
                        None
                    } else {
                        Some(
                            "Operations response did not meet the emergency alert contract."
                                .to_string(),
                        )
                    },
                },
                Err(error) => AssertionResult {
                    name: "ops".to_string(),
                    assertion_type: "ops".to_string(),
                    passed: false,
                    message: error.to_string(),
                    impact_level: Some("unknown".to_string()),
                    impact_message: Some(
                        "The ops assertion is not supported by the runner.".to_string(),
                    ),
                },
            };
        }

        if let (Some(target), Some(condition)) = (&assertion.target, &assertion.condition) {
            let condition_text = condition.as_str().unwrap_or_default();
            if target == "edge.secondary" {
                let status_passed = condition_text == "active"
                    && self.edge.secondary_status() == Some(EdgeStatus::Active);
                let timing_passed = self
                    .edge_expected_within
                    .map(|expected_within| {
                        match (self.edge_primary_failed_at, self.edge_failover_at) {
                            (Some(failed_at), Some(failover_at)) => {
                                failover_at - failed_at <= expected_within
                            }
                            _ => false,
                        }
                    })
                    .unwrap_or(true);
                let passed = status_passed && timing_passed;
                return AssertionResult {
                    name: "edge.secondary".to_string(),
                    assertion_type: "edge_state".to_string(),
                    passed,
                    message: if passed {
                        if timing_passed {
                            "secondary edge server is active".to_string()
                        } else {
                            "secondary edge server did not activate within expected failover window"
                                .to_string()
                        }
                    } else {
                        "secondary edge server is not active".to_string()
                    },
                    impact_level: if passed {
                        None
                    } else {
                        Some("high".to_string())
                    },
                    impact_message: if passed {
                        None
                    } else {
                        Some("Edge failover did not preserve local control.".to_string())
                    },
                };
            }
            if target == "mqtt.local" {
                let passed = condition_text == "available" && self.broker.is_online("mqtt.local");
                return AssertionResult {
                    name: "mqtt.local".to_string(),
                    assertion_type: "broker_state".to_string(),
                    passed,
                    message: if passed {
                        "local MQTT broker is available".to_string()
                    } else {
                        "local MQTT broker is unavailable".to_string()
                    },
                    impact_level: if passed {
                        None
                    } else {
                        Some("high".to_string())
                    },
                    impact_message: if passed {
                        None
                    } else {
                        Some("Local MQTT broker is unavailable during failover.".to_string())
                    },
                };
            }
            if target == "guest_experience" {
                let passed = condition_text == "unaffected"
                    && self.broker.is_online("mqtt.local")
                    && self.edge.active_id().is_some();
                return AssertionResult {
                    name: "guest_experience".to_string(),
                    assertion_type: "guest_experience".to_string(),
                    passed,
                    message: if passed {
                        "guest experience remained unaffected by local edge availability"
                            .to_string()
                    } else {
                        "guest experience was affected".to_string()
                    },
                    impact_level: if passed {
                        None
                    } else {
                        Some("high".to_string())
                    },
                    impact_message: if passed {
                        None
                    } else {
                        Some(
                            "Local edge or MQTT availability did not preserve guest experience."
                                .to_string(),
                        )
                    },
                };
            }
        }

        if let Some(inline_assert) = &assertion.inline_assert {
            if let Some(scene) = inline_assert.get("scene").and_then(|value| value.as_str()) {
                let complete = inline_assert
                    .get("consistency")
                    .and_then(|value| value.as_str())
                    == Some("complete");
                if complete {
                    return self.evaluate_scene_consistency(scene);
                }
            }
        }

        AssertionResult {
            name: "unsupported_assertion".to_string(),
            assertion_type: "unsupported".to_string(),
            passed: false,
            message: "unsupported assertion type".to_string(),
            impact_level: Some("unknown".to_string()),
            impact_message: Some("The runner does not support this assertion yet.".to_string()),
        }
    }

    fn evaluate_scene_consistency(&self, scene: &str) -> AssertionResult {
        let failures = match self.lighting.scene_consistency_failures(scene) {
            Ok(failures) => failures,
            Err(_) => {
                return AssertionResult {
                    name: format!("scene_consistency:{scene}"),
                    assertion_type: "scene_consistency".to_string(),
                    passed: false,
                    message: format!("scene {scene} is not defined"),
                    impact_level: Some("medium".to_string()),
                    impact_message: Some("Scene mapping is missing from the scenario.".to_string()),
                };
            }
        };
        let passed = failures.is_empty();
        AssertionResult {
            name: format!("scene_consistency:{scene}"),
            assertion_type: "scene_consistency".to_string(),
            passed,
            message: if passed {
                "DALI-like scene reached expected levels".to_string()
            } else {
                format!(
                    "DALI-like scene consistency violation: {}",
                    failures.join("; ")
                )
            },
            impact_level: if passed {
                None
            } else {
                Some("medium".to_string())
            },
            impact_message: if passed {
                None
            } else {
                Some("Lighting scene did not match intended guest ambience.".to_string())
            },
        }
    }

    fn push(
        &mut self,
        at: Duration,
        event_type: impl Into<String>,
        target: Option<String>,
        message: impl Into<String>,
    ) {
        self.timeline.push(TimelineEvent {
            at: format_duration(at),
            event_type: event_type.into(),
            target,
            message: message.into(),
        });
    }
}

fn edge_expected_within(
    edge: &BTreeMap<String, serde_yaml::Value>,
) -> Result<Option<Duration>, ScenarioError> {
    let Some(failover) = edge.get("failover").and_then(|value| value.as_mapping()) else {
        return Ok(None);
    };
    let Some(expected_within) = failover
        .get(serde_yaml::Value::String("expected_within".to_string()))
        .and_then(|value| value.as_str())
    else {
        return Ok(None);
    };
    parse_duration(expected_within).map(Some)
}

fn format_duration(duration: Duration) -> String {
    if duration == Duration::zero() {
        return "T".to_string();
    }
    let milliseconds = duration.num_milliseconds();
    let sign = if milliseconds >= 0 { "+" } else { "-" };
    let seconds = milliseconds.abs() / 1000;
    if seconds % 60 == 0 && seconds >= 60 {
        format!("T{}{}m", sign, seconds / 60)
    } else {
        format!("T{}{}s", sign, seconds)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use roomci_scenario::load_scenario;

    use super::*;

    fn fixture(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }

    #[test]
    fn local_first_cloud_outage_passes() {
        let scenario = load_scenario(fixture("examples/local_first_cloud_outage.yaml")).unwrap();

        let report = run_scenario(&scenario).unwrap();

        assert_eq!(report.result, RunResult::Passed);
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "fault_activated"
                && event.target.as_deref() == Some("mqtt.cloud")));
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "mqtt_retained_state_updated"));
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "edge_command_routed"));
    }

    #[test]
    fn duplicate_delivery_keeps_single_retained_state() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: duplicate_delivery
  tags: [mqtt]
mqtt:
  local:
    retained: true
devices:
  - id: living_light
    type: light
    protocol: dali
    state:
      power: false
faults:
  - at: T
    target: mqtt.local
    type: duplicate_delivery
    topic: house/minakami/room/living/device/living_light/command
    count: 3
steps:
  - at: T+1s
    mqtt_publish:
      client: ipad_controller
      topic: house/minakami/room/living/device/living_light/command
      payload:
        power: true
assertions:
  - at: T+2s
    mqtt:
      topic: house/minakami/room/living/device/living_light/state
      retained:
        power: true
"#,
        )
        .unwrap();

        let report = run_scenario(&scenario).unwrap();

        assert_eq!(report.result, RunResult::Passed);
        assert_eq!(report.retained_messages.len(), 1);
        assert!(report
            .timeline
            .iter()
            .any(|event| event.message.contains("3 deliveries")));
    }

    #[test]
    fn edge_server_failover_passes() {
        let scenario = load_scenario(fixture("examples/edge_server_failover.yaml")).unwrap();

        let report = run_scenario(&scenario).unwrap();

        assert_eq!(report.result, RunResult::Passed);
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "edge_failover"));
    }

    #[test]
    fn modbus_floor_heating_passes() {
        let scenario = load_scenario(fixture("examples/modbus_floor_heating.yaml")).unwrap();

        let report = run_scenario(&scenario).unwrap();

        assert_eq!(report.result, RunResult::Passed);
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "modbus_write"));
    }

    #[test]
    fn dali_scene_partial_failure_is_detected() {
        let scenario = load_scenario(fixture("examples/dali_scene_partial_failure.yaml")).unwrap();

        let report = run_scenario(&scenario).unwrap();

        assert_eq!(report.result, RunResult::Failed);
        assert!(report
            .assertions
            .iter()
            .any(|assertion| assertion.assertion_type == "scene_consistency" && !assertion.passed));
    }

    #[test]
    fn contact_input_changes_are_recorded_for_ops_phase() {
        let scenario = load_scenario(fixture("examples/bms_sauna_emergency_alert.yaml")).unwrap();

        let report = run_scenario(&scenario).unwrap();

        assert_eq!(report.result, RunResult::Passed);
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "contact_changed"
                && event.target.as_deref() == Some("sauna_emergency_button")));
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "ops_slack_notification_sent"));
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "ops_phone_call_triggered"));
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "ops_runbook_url_included"
                && event.message.contains("sauna-emergency")));
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "ops_ticket_acknowledged"
                && event.message.contains("ops_member_01")));
    }

    #[test]
    fn mqtt_command_does_not_update_state_when_edge_is_unavailable() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: no_active_edge
edge:
  primary:
    id: edge_primary
    status: failed
mqtt:
  local:
    retained: true
devices:
  - id: living_light
    type: light
    protocol: dali
    state:
      power: false
steps:
  - at: T
    mqtt_publish:
      client: ipad_controller
      topic: house/minakami/room/living/device/living_light/command
      payload:
        power: true
assertions:
  - at: T+1s
    mqtt:
      topic: house/minakami/room/living/device/living_light/state
      retained:
        power: true
"#,
        )
        .unwrap();

        let report = run_scenario(&scenario).unwrap();

        assert_eq!(report.result, RunResult::Failed);
        assert!(report.retained_messages.is_empty());
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "edge_command_failed"));
    }
}
