use std::collections::{BTreeMap, BTreeSet};

use chrono::Duration;
use roomci_edge::{EdgeError, EdgeModel, EdgeStatus};
use roomci_mqtt::{device_id_from_command_topic, BrokerModel, MqttError};
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
    modbus_registers: BTreeMap<String, BTreeMap<u32, serde_json::Value>>,
    dali_levels: BTreeMap<String, i64>,
    scene_targets: BTreeMap<String, BTreeMap<String, i64>>,
    dali_command_drops: BTreeSet<String>,
    contact_states: BTreeMap<String, String>,
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
            modbus_registers: modbus_registers(&scenario.modbus),
            dali_levels: dali_levels(&scenario.lighting),
            scene_targets: scenario
                .scenes
                .iter()
                .map(|(name, scene)| (name.clone(), scene.fixtures.clone()))
                .collect(),
            dali_command_drops: BTreeSet::new(),
            contact_states: contact_states(&scenario.contacts),
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
                    self.contact_states
                        .insert(contact.id.clone(), contact.state.clone());
                    self.push(
                        at,
                        "contact_changed",
                        Some(contact.id),
                        format!("contact state changed to {}", contact.state),
                    );
                }
                if step.ops.is_some() {
                    self.push(at, "ops_action", None, "ops action applied");
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
                self.dali_command_drops.insert(fixture.to_string());
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
        let json_value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
        self.modbus_registers
            .entry(device.to_string())
            .or_default()
            .insert(register, json_value.clone());
        self.push(
            at,
            "modbus_write",
            Some(device.to_string()),
            format!("register {} set to {}", register, json_value),
        );
    }

    fn activate_scene(&mut self, at: Duration, scene: &str) {
        if let Some(targets) = self.scene_targets.get(scene).cloned() {
            for (fixture, level) in targets {
                if self.dali_command_drops.contains(&fixture) {
                    self.push(
                        at,
                        "dali_command_dropped",
                        Some(fixture),
                        "fixture command dropped".to_string(),
                    );
                } else {
                    self.dali_levels.insert(fixture.clone(), level);
                    self.push(
                        at,
                        "dali_level_changed",
                        Some(fixture),
                        format!("fixture level changed to {level}"),
                    );
                }
            }
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
            let actual = self
                .modbus_registers
                .get(&modbus.device)
                .and_then(|registers| registers.get(&modbus.register));
            let passed = if let Some(expected_readable) = modbus.readable_value {
                actual
                    .and_then(|value| value.as_f64().or_else(|| value.as_i64().map(|v| v as f64)))
                    .map(|value| ((value / 10.0) - expected_readable).abs() < f64::EPSILON)
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
        let Some(targets) = self.scene_targets.get(scene) else {
            return AssertionResult {
                name: format!("scene_consistency:{scene}"),
                assertion_type: "scene_consistency".to_string(),
                passed: false,
                message: format!("scene {scene} is not defined"),
                impact_level: Some("medium".to_string()),
                impact_message: Some("Scene mapping is missing from the scenario.".to_string()),
            };
        };
        let mut failures = Vec::new();
        for (fixture, expected) in targets {
            let actual = self.dali_levels.get(fixture).copied().unwrap_or(0);
            if actual != *expected {
                failures.push(format!(
                    "{fixture} expected level {expected}, actual {actual}"
                ));
            }
        }
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

fn modbus_registers(
    modbus: &BTreeMap<String, serde_yaml::Value>,
) -> BTreeMap<String, BTreeMap<u32, serde_json::Value>> {
    let mut devices = BTreeMap::new();
    let Some(device_values) = modbus.get("devices").and_then(|value| value.as_sequence()) else {
        return devices;
    };
    for device_value in device_values {
        let Some(device_map) = device_value.as_mapping() else {
            continue;
        };
        let Some(device_id) = yaml_mapping_get(device_map, "id").and_then(|value| value.as_str())
        else {
            continue;
        };
        let registers = devices
            .entry(device_id.to_string())
            .or_insert_with(BTreeMap::new);
        for section in [
            "holding_registers",
            "input_registers",
            "discrete_inputs",
            "coils",
        ] {
            let Some(section_map) =
                yaml_mapping_get(device_map, section).and_then(|value| value.as_mapping())
            else {
                continue;
            };
            for (address, definition) in section_map {
                let Some(address) = yaml_key_u32(address) else {
                    continue;
                };
                let Some(value) = definition
                    .as_mapping()
                    .and_then(|mapping| yaml_mapping_get(mapping, "value"))
                else {
                    continue;
                };
                registers.insert(
                    address,
                    serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
                );
            }
        }
    }
    devices
}

fn dali_levels(lighting: &BTreeMap<String, serde_yaml::Value>) -> BTreeMap<String, i64> {
    let mut levels = BTreeMap::new();
    let Some(fixtures) = lighting
        .get("fixtures")
        .and_then(|value| value.as_sequence())
    else {
        return levels;
    };
    for fixture in fixtures {
        let Some(mapping) = fixture.as_mapping() else {
            continue;
        };
        let Some(id) = yaml_mapping_get(mapping, "id").and_then(|value| value.as_str()) else {
            continue;
        };
        let level = yaml_mapping_get(mapping, "level")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        levels.insert(id.to_string(), level);
    }
    levels
}

fn contact_states(contacts: &BTreeMap<String, serde_yaml::Value>) -> BTreeMap<String, String> {
    let mut states = BTreeMap::new();
    let Some(inputs) = contacts.get("inputs").and_then(|value| value.as_sequence()) else {
        return states;
    };
    for input in inputs {
        let Some(mapping) = input.as_mapping() else {
            continue;
        };
        let Some(id) = yaml_mapping_get(mapping, "id").and_then(|value| value.as_str()) else {
            continue;
        };
        let state = yaml_mapping_get(mapping, "state")
            .and_then(|value| value.as_str())
            .unwrap_or("off");
        states.insert(id.to_string(), state.to_string());
    }
    states
}

fn yaml_mapping_get<'a>(
    mapping: &'a serde_yaml::Mapping,
    key: &str,
) -> Option<&'a serde_yaml::Value> {
    mapping.get(serde_yaml::Value::String(key.to_string()))
}

fn yaml_key_u32(value: &serde_yaml::Value) -> Option<u32> {
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| value.as_str().and_then(|value| value.parse::<u32>().ok()))
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

        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "contact_changed"
                && event.target.as_deref() == Some("sauna_emergency_button")));
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
