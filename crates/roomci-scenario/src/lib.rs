use std::{collections::BTreeMap, fs, path::Path};

use chrono::{DateTime, Duration, FixedOffset};
use roomci_device_model::RoomDefinition;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScenarioError {
    #[error("failed to read scenario {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse scenario {path}: {source}")]
    Parse {
        path: String,
        source: serde_yaml::Error,
    },
    #[error("scenario clock is missing guest_arrival required for relative time expression {0}")]
    MissingGuestArrival(String),
    #[error("invalid timestamp {0}")]
    InvalidTimestamp(String),
    #[error("invalid relative time expression {0}")]
    InvalidRelativeTime(String),
    #[error("invalid duration {0}")]
    InvalidDuration(String),
    #[error("unknown device target {0}")]
    UnknownDevice(String),
    #[error("step must contain exactly one of event, command, fault, state, or assert")]
    InvalidStepKind,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScenarioFile {
    pub version: String,
    pub scenario: ScenarioMetadata,
    pub room: RoomDefinition,
    #[serde(default)]
    pub steps: Vec<ScenarioStep>,
    #[serde(default)]
    pub assertions: Vec<AssertionDefinition>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScenarioMetadata {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub clock: ScenarioClock,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScenarioClock {
    pub start: String,
    #[serde(default)]
    pub guest_arrival: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScenarioStep {
    pub at: String,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub command: Option<CommandStep>,
    #[serde(default)]
    pub fault: Option<FaultStep>,
    #[serde(default)]
    pub state: Option<StateStep>,
    #[serde(default, rename = "assert")]
    pub assertion: Option<AssertionDefinition>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CommandStep {
    pub target: String,
    pub action: String,
    #[serde(default)]
    pub value: Option<serde_yaml::Value>,
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FaultStep {
    pub target: String,
    #[serde(rename = "type")]
    pub fault_type: String,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub probability: Option<f64>,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StateStep {
    pub target: String,
    pub patch: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AssertionDefinition {
    pub at: String,
    #[serde(rename = "type")]
    pub assertion_type: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub expect: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub within: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub scene: Option<String>,
    #[serde(default)]
    pub impact: Option<Impact>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Impact {
    pub level: String,
    pub message: String,
}

pub fn load_scenario(path: impl AsRef<Path>) -> Result<ScenarioFile, ScenarioError> {
    let path_ref = path.as_ref();
    let path_display = path_ref.display().to_string();
    let contents = fs::read_to_string(path_ref).map_err(|source| ScenarioError::Read {
        path: path_display.clone(),
        source,
    })?;
    serde_yaml::from_str(&contents).map_err(|source| ScenarioError::Parse {
        path: path_display,
        source,
    })
}

pub fn validate_scenario(scenario: &ScenarioFile) -> Result<(), ScenarioError> {
    let device_ids = scenario
        .room
        .devices
        .iter()
        .map(|device| device.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    for step in &scenario.steps {
        let kinds = [
            step.event.is_some(),
            step.command.is_some(),
            step.fault.is_some(),
            step.state.is_some(),
            step.assertion.is_some(),
        ]
        .iter()
        .filter(|present| **present)
        .count();
        if kinds != 1 {
            return Err(ScenarioError::InvalidStepKind);
        }
        resolve_time(&scenario.scenario.clock, &step.at)?;
        if let Some(command) = &step.command {
            ensure_target(&device_ids, &command.target)?;
        }
        if let Some(fault) = &step.fault {
            ensure_target(&device_ids, &fault.target)?;
            if let Some(duration) = &fault.duration {
                parse_duration(duration)?;
            }
            if fault.probability.is_some() && fault.seed.is_none() {
                return Err(ScenarioError::InvalidDuration(
                    "probabilistic faults require seed".to_string(),
                ));
            }
        }
        if let Some(state) = &step.state {
            ensure_target(&device_ids, &state.target)?;
        }
    }

    for assertion in &scenario.assertions {
        resolve_time(&scenario.scenario.clock, &assertion.at)?;
        if let Some(target) = &assertion.target {
            ensure_target(&device_ids, target)?;
        }
        if let Some(within) = &assertion.within {
            parse_duration(within)?;
        }
    }

    Ok(())
}

fn ensure_target(
    device_ids: &std::collections::BTreeSet<&str>,
    target: &str,
) -> Result<(), ScenarioError> {
    if device_ids.contains(target) {
        Ok(())
    } else {
        Err(ScenarioError::UnknownDevice(target.to_string()))
    }
}

pub fn resolve_time(
    clock: &ScenarioClock,
    expression: &str,
) -> Result<DateTime<FixedOffset>, ScenarioError> {
    if expression.starts_with('T') {
        let guest_arrival = clock
            .guest_arrival
            .as_ref()
            .ok_or_else(|| ScenarioError::MissingGuestArrival(expression.to_string()))?;
        let base = parse_timestamp(guest_arrival)?;
        if expression == "T" {
            return Ok(base);
        }
        let sign = expression
            .chars()
            .nth(1)
            .ok_or_else(|| ScenarioError::InvalidRelativeTime(expression.to_string()))?;
        let duration = parse_duration(&expression[2..])?;
        return match sign {
            '+' => Ok(base + duration),
            '-' => Ok(base - duration),
            _ => Err(ScenarioError::InvalidRelativeTime(expression.to_string())),
        };
    }

    parse_timestamp(expression)
}

pub fn parse_timestamp(value: &str) -> Result<DateTime<FixedOffset>, ScenarioError> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| ScenarioError::InvalidTimestamp(value.to_string()))
}

pub fn parse_duration(value: &str) -> Result<Duration, ScenarioError> {
    let split_at = value
        .find(|char: char| !char.is_ascii_digit())
        .ok_or_else(|| ScenarioError::InvalidDuration(value.to_string()))?;
    let amount: i64 = value[..split_at]
        .parse()
        .map_err(|_| ScenarioError::InvalidDuration(value.to_string()))?;
    let unit = &value[split_at..];
    match unit {
        "s" => Ok(Duration::seconds(amount)),
        "m" => Ok(Duration::minutes(amount)),
        "h" => Ok(Duration::hours(amount)),
        _ => Err(ScenarioError::InvalidDuration(value.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_time_from_guest_arrival() {
        let clock = ScenarioClock {
            start: "2026-08-10T14:30:00+09:00".to_string(),
            guest_arrival: Some("2026-08-10T15:00:00+09:00".to_string()),
            mode: Some("simulated".to_string()),
        };

        let resolved = resolve_time(&clock, "T-5m").unwrap();

        assert_eq!(resolved.to_rfc3339(), "2026-08-10T14:55:00+09:00");
    }

    #[test]
    fn rejects_unknown_device_target() {
        let yaml = r#"
version: "0.1"
scenario:
  name: invalid_target
  clock:
    start: "2026-08-10T14:30:00+09:00"
    guest_arrival: "2026-08-10T15:00:00+09:00"
room:
  id: room_nasu_001
  devices: []
steps:
  - at: "T"
    command:
      target: lock_missing
      action: unlock
"#;
        let scenario: ScenarioFile = serde_yaml::from_str(yaml).unwrap();

        let error = validate_scenario(&scenario).unwrap_err();

        assert!(matches!(error, ScenarioError::UnknownDevice(target) if target == "lock_missing"));
    }
}
