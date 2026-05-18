use std::collections::BTreeMap;

use chrono::Duration;
use roomci_mqtt::{BrokerModel, MqttError};
use roomci_scenario::{
    resolve_time_offset, validate_scenario, yaml_map_to_json, MqttPublishStep, ScenarioError,
    ScenarioFile,
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
    broker: BrokerModel,
    duplicate_delivery_by_topic: BTreeMap<String, u32>,
    timeline: Vec<TimelineEvent>,
}

pub fn run_scenario(scenario: &ScenarioFile) -> Result<RunReport, CoreError> {
    validate_scenario(scenario)?;

    let mut runtime = RuntimeState::new(scenario);
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
    fn new(scenario: &ScenarioFile) -> Self {
        let states = scenario
            .devices
            .iter()
            .map(|device| (device.id.clone(), yaml_map_to_json(&device.state)))
            .collect::<BTreeMap<_, _>>();

        Self {
            states,
            broker: BrokerModel::new(true, scenario.mqtt.cloud.enabled),
            duplicate_delivery_by_topic: BTreeMap::new(),
            timeline: Vec::new(),
        }
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
                    self.push(
                        at,
                        "command_received",
                        Some(command.target),
                        format!("{} command received", command.action),
                    );
                }
                if let Some(mqtt_publish) = step.mqtt_publish {
                    self.apply_mqtt_publish(at, &mqtt_publish);
                }
                if let Some(contact) = step.contact {
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
        if target == "mqtt.local" && fault_type == "duplicate_delivery" {
            if let Some(topic) = topic {
                self.duplicate_delivery_by_topic
                    .insert(topic.to_string(), count.unwrap_or(2).max(2));
            }
        }
        self.push(
            at,
            "fault_activated",
            Some(target.to_string()),
            format!("{} fault activated", fault_type),
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

        AssertionResult {
            name: "unsupported_assertion".to_string(),
            assertion_type: "unsupported".to_string(),
            passed: false,
            message: "unsupported assertion type".to_string(),
            impact_level: Some("unknown".to_string()),
            impact_message: Some("The runner does not support this assertion yet.".to_string()),
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
}
