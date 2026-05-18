use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset};
use roomci_device_model::{
    apply_command_state, command_is_supported, yaml_state_to_json, DeviceDefinition, StateMap,
};
use roomci_fault::ActiveFault;
use roomci_scenario::{
    parse_duration, resolve_time, validate_scenario, AssertionDefinition, ScenarioError,
    ScenarioFile, ScenarioStep,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RunReport {
    pub scenario_name: String,
    pub result: RunResult,
    pub timeline: Vec<TimelineEvent>,
    pub assertions: Vec<AssertionResult>,
    pub final_state: BTreeMap<String, StateMap>,
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
    devices: BTreeMap<String, DeviceDefinition>,
    states: BTreeMap<String, StateMap>,
    faults: Vec<ActiveFault>,
    timeline: Vec<TimelineEvent>,
}

pub fn run_scenario(scenario: &ScenarioFile) -> Result<RunReport, CoreError> {
    validate_scenario(scenario)?;

    let mut runtime = RuntimeState::new(scenario);
    let mut steps = scenario.steps.clone();
    steps.sort_by_key(|step| {
        resolve_time(&scenario.scenario.clock, &step.at)
            .map(|time| time.timestamp_millis())
            .unwrap_or_default()
    });

    for step in &steps {
        let at = resolve_time(&scenario.scenario.clock, &step.at)?;
        runtime.apply_step(scenario, step, at)?;
    }

    let mut assertions = Vec::new();
    for assertion in &scenario.assertions {
        let at = resolve_time(&scenario.scenario.clock, &assertion.at)?;
        assertions.push(runtime.evaluate_assertion(assertion, at));
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
    })
}

impl RuntimeState {
    fn new(scenario: &ScenarioFile) -> Self {
        let devices = scenario
            .room
            .devices
            .iter()
            .map(|device| (device.id.clone(), device.clone()))
            .collect::<BTreeMap<_, _>>();
        let states = scenario
            .room
            .devices
            .iter()
            .map(|device| (device.id.clone(), yaml_state_to_json(&device.initial_state)))
            .collect::<BTreeMap<_, _>>();

        Self {
            devices,
            states,
            faults: Vec::new(),
            timeline: Vec::new(),
        }
    }

    fn apply_step(
        &mut self,
        scenario: &ScenarioFile,
        step: &ScenarioStep,
        at: DateTime<FixedOffset>,
    ) -> Result<(), CoreError> {
        if let Some(event) = &step.event {
            self.push(at, "event", None, event.clone());
        }

        if let Some(fault) = &step.fault {
            let ends_at = fault
                .duration
                .as_ref()
                .map(|duration| parse_duration(duration).map(|duration| at + duration))
                .transpose()?;
            self.faults.push(ActiveFault {
                target: fault.target.clone(),
                fault_type: fault.fault_type.clone(),
                starts_at: at,
                ends_at,
            });
            self.push(
                at,
                "fault_activated",
                Some(fault.target.clone()),
                format!("{} fault activated", fault.fault_type),
            );
        }

        if let Some(state) = &step.state {
            let target_state = self.states.entry(state.target.clone()).or_default();
            for (key, value) in &state.patch {
                target_state.insert(
                    key.clone(),
                    serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
                );
            }
            self.push(
                at,
                "state_changed",
                Some(state.target.clone()),
                "state patch applied".to_string(),
            );
        }

        if let Some(command) = &step.command {
            let Some(device) = self.devices.get(&command.target) else {
                return Err(ScenarioError::UnknownDevice(command.target.clone()).into());
            };

            if !command_is_supported(&device.device_type, &command.action) {
                self.push(
                    at,
                    "command_rejected",
                    Some(command.target.clone()),
                    format!("unsupported command {}", command.action),
                );
                return Ok(());
            }

            if self.has_active_fault(&command.target, "offline", at) {
                self.push(
                    at,
                    "command_failed",
                    Some(command.target.clone()),
                    format!("{} failed because device is offline", command.action),
                );
                return Ok(());
            }

            let json_value = command
                .value
                .as_ref()
                .map(|value| serde_json::to_value(value).unwrap_or(serde_json::Value::Null));
            let state = self.states.entry(command.target.clone()).or_default();
            apply_command_state(
                &device.device_type,
                &command.action,
                json_value.as_ref(),
                state,
            );
            self.push(
                at,
                "command_accepted",
                Some(command.target.clone()),
                format!("{} accepted", command.action),
            );
            self.push(
                at,
                "state_changed",
                Some(command.target.clone()),
                format!("{} applied", command.action),
            );
        }

        if let Some(assertion) = &step.assertion {
            let result = self.evaluate_assertion(assertion, at);
            self.push(
                at,
                if result.passed {
                    "assertion_passed"
                } else {
                    "assertion_failed"
                },
                assertion.target.clone(),
                result.message,
            );
        }

        let _ = scenario;
        Ok(())
    }

    fn has_active_fault(&self, target: &str, fault_type: &str, at: DateTime<FixedOffset>) -> bool {
        self.faults.iter().any(|fault| {
            fault.target == target && fault.fault_type == fault_type && fault.is_active_at(at)
        })
    }

    fn evaluate_assertion(
        &self,
        assertion: &AssertionDefinition,
        at: DateTime<FixedOffset>,
    ) -> AssertionResult {
        let (passed, message) = match assertion.assertion_type.as_str() {
            "event_emitted" => self.evaluate_event_assertion(assertion, at),
            "sensor_threshold" => self.evaluate_sensor_threshold(assertion),
            "device_state" => self.evaluate_device_state(assertion),
            "scene_consistency" => (
                false,
                "scene consistency assertions are not implemented in Phase 0".to_string(),
            ),
            other => (false, format!("unsupported assertion type {}", other)),
        };

        AssertionResult {
            name: assertion_name(assertion),
            assertion_type: assertion.assertion_type.clone(),
            passed,
            message,
            impact_level: assertion.impact.as_ref().map(|impact| impact.level.clone()),
            impact_message: assertion
                .impact
                .as_ref()
                .map(|impact| impact.message.clone()),
        }
    }

    fn evaluate_event_assertion(
        &self,
        assertion: &AssertionDefinition,
        at: DateTime<FixedOffset>,
    ) -> (bool, String) {
        let Some(expected_event) = &assertion.event else {
            return (false, "event assertion is missing event".to_string());
        };
        let window_start = assertion
            .within
            .as_ref()
            .and_then(|within| parse_duration(within).ok())
            .map(|duration| at - duration)
            .unwrap_or(at);
        let found = self.timeline.iter().any(|event| {
            event.event_type == "event"
                && event.message == *expected_event
                && DateTime::parse_from_rfc3339(&event.at)
                    .map(|event_at| event_at >= window_start && event_at <= at)
                    .unwrap_or(false)
        });

        if found {
            (true, format!("event {} was emitted", expected_event))
        } else {
            (
                false,
                format!(
                    "event {} was not emitted within expected window",
                    expected_event
                ),
            )
        }
    }

    fn evaluate_sensor_threshold(&self, assertion: &AssertionDefinition) -> (bool, String) {
        let Some(target) = &assertion.target else {
            return (
                false,
                "sensor threshold assertion is missing target".to_string(),
            );
        };
        let Some(condition) = &assertion.condition else {
            return (
                false,
                "sensor threshold assertion is missing condition".to_string(),
            );
        };
        let Some(state) = self.states.get(target) else {
            return (false, format!("target {} has no state", target));
        };
        let Some(value) = state
            .get("temperature_celsius")
            .or_else(|| state.get("humidity_measurement"))
            .and_then(|value| value.as_f64())
        else {
            return (
                false,
                format!("target {} has no numeric sensor value", target),
            );
        };

        let Some((operator, threshold)) = parse_numeric_condition(condition) else {
            return (false, format!("invalid condition {}", condition));
        };
        let passed = match operator {
            "<=" => value <= threshold,
            "<" => value < threshold,
            ">=" => value >= threshold,
            ">" => value > threshold,
            "==" => (value - threshold).abs() < f64::EPSILON,
            _ => false,
        };

        if passed {
            (
                true,
                format!("{} satisfied with value {}", condition, value),
            )
        } else {
            (false, format!("{} failed with value {}", condition, value))
        }
    }

    fn evaluate_device_state(&self, assertion: &AssertionDefinition) -> (bool, String) {
        let Some(target) = &assertion.target else {
            return (
                false,
                "device state assertion is missing target".to_string(),
            );
        };
        let Some(state) = self.states.get(target) else {
            return (false, format!("target {} has no state", target));
        };
        for (key, expected) in &assertion.expect {
            let expected_json = serde_json::to_value(expected).unwrap_or(serde_json::Value::Null);
            if state.get(key) != Some(&expected_json) {
                return (
                    false,
                    format!(
                        "{} expected {:?}, got {:?}",
                        key,
                        expected_json,
                        state.get(key)
                    ),
                );
            }
        }
        (true, "device state matched".to_string())
    }

    fn push(
        &mut self,
        at: DateTime<FixedOffset>,
        event_type: impl Into<String>,
        target: Option<String>,
        message: impl Into<String>,
    ) {
        self.timeline.push(TimelineEvent {
            at: at.to_rfc3339(),
            event_type: event_type.into(),
            target,
            message: message.into(),
        });
    }
}

fn assertion_name(assertion: &AssertionDefinition) -> String {
    if let Some(event) = &assertion.event {
        return format!("event_emitted:{}", event);
    }
    if let Some(target) = &assertion.target {
        return format!("{}:{}", assertion.assertion_type, target);
    }
    assertion.assertion_type.clone()
}

fn parse_numeric_condition(condition: &str) -> Option<(&str, f64)> {
    for operator in ["<=", ">=", "==", "<", ">"] {
        if let Some(rest) = condition.strip_prefix(operator) {
            return rest
                .trim()
                .parse::<f64>()
                .ok()
                .map(|value| (operator, value));
        }
    }
    None
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
    fn lock_offline_scenario_fails_recovery_assertions() {
        let scenario = load_scenario(fixture("docs/examples/checkin_lock_offline.yaml")).unwrap();

        let report = run_scenario(&scenario).unwrap();

        assert_eq!(report.result, RunResult::Failed);
        assert!(report
            .timeline
            .iter()
            .any(|event| event.event_type == "command_failed"));
        assert_eq!(report.assertions.len(), 2);
        assert!(report.assertions.iter().all(|assertion| !assertion.passed));
    }

    #[test]
    fn ac_preheat_scenario_fails_threshold_assertion() {
        let scenario = load_scenario(fixture("docs/examples/ac_preheat_failed.yaml")).unwrap();

        let report = run_scenario(&scenario).unwrap();

        assert_eq!(report.result, RunResult::Failed);
        assert_eq!(report.assertions[0].assertion_type, "sensor_threshold");
        assert!(!report.assertions[0].passed);
    }
}
