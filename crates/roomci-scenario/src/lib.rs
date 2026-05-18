use std::{collections::BTreeMap, fs, path::Path};

use chrono::Duration;
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
    #[error("invalid relative time expression {0}")]
    InvalidRelativeTime(String),
    #[error("invalid duration {0}")]
    InvalidDuration(String),
    #[error("unknown device target {0}")]
    UnknownDevice(String),
    #[error("step must contain a supported action")]
    InvalidStepKind,
    #[error("assertion must contain a supported condition")]
    InvalidAssertionKind,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScenarioFile {
    pub version: String,
    pub scenario: ScenarioMetadata,
    #[serde(default)]
    pub environment: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub network: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub wan: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub edge: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub mqtt: MqttConfig,
    #[serde(default)]
    pub devices: Vec<DeviceDefinition>,
    #[serde(default)]
    pub modbus: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub lighting: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub scenes: BTreeMap<String, SceneDefinition>,
    #[serde(default)]
    pub contacts: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub alerts: Vec<BTreeMap<String, serde_yaml::Value>>,
    #[serde(default)]
    pub faults: Vec<FaultStep>,
    #[serde(default)]
    pub steps: Vec<ScenarioStep>,
    #[serde(default)]
    pub assertions: Vec<AssertionDefinition>,
    #[serde(default)]
    pub report: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct ScenarioMetadata {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub clock: Option<ScenarioClock>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScenarioClock {
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub guest_arrival: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct MqttConfig {
    #[serde(default)]
    pub local: BrokerConfig,
    #[serde(default)]
    pub cloud: BrokerConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct BrokerConfig {
    #[serde(default)]
    pub retained: bool,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DeviceDefinition {
    pub id: String,
    #[serde(rename = "type")]
    pub device_type: String,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub state: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScenarioStep {
    pub at: String,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub command: Option<CommandStep>,
    #[serde(default)]
    pub modbus_write: Option<ModbusWriteStep>,
    #[serde(default)]
    pub mqtt_publish: Option<MqttPublishStep>,
    #[serde(default)]
    pub fault: Option<FaultStep>,
    #[serde(default)]
    pub contact: Option<ContactStep>,
    #[serde(default)]
    pub ops: Option<BTreeMap<String, serde_yaml::Value>>,
    #[serde(default)]
    pub automation: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CommandStep {
    pub target: String,
    pub action: String,
    #[serde(default)]
    pub value: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModbusWriteStep {
    pub device: String,
    pub register: u32,
    pub value: serde_yaml::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MqttPublishStep {
    pub client: String,
    pub topic: String,
    #[serde(default)]
    pub payload: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FaultStep {
    #[serde(default)]
    pub at: Option<String>,
    pub target: String,
    #[serde(rename = "type")]
    pub fault_type: String,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ContactStep {
    pub id: String,
    pub state: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AssertionDefinition {
    pub at: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub condition: Option<serde_yaml::Value>,
    #[serde(default)]
    pub mqtt: Option<MqttAssertion>,
    #[serde(default)]
    pub modbus: Option<ModbusAssertion>,
    #[serde(default)]
    pub guest_experience: Option<String>,
    #[serde(default)]
    pub ops: Option<BTreeMap<String, serde_yaml::Value>>,
    #[serde(default, rename = "assert")]
    pub inline_assert: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SceneDefinition {
    #[serde(default)]
    pub fixtures: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MqttAssertion {
    pub topic: String,
    #[serde(default)]
    pub retained: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModbusAssertion {
    pub device: String,
    pub register: u32,
    #[serde(default)]
    pub value: Option<serde_yaml::Value>,
    #[serde(default)]
    pub readable_value: Option<f64>,
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
    for fault in &scenario.faults {
        if let Some(at) = &fault.at {
            resolve_time_offset(at)?;
        }
        if let Some(duration) = &fault.duration {
            parse_duration(duration)?;
        }
    }

    for step in &scenario.steps {
        resolve_time_offset(&step.at)?;
        let kinds = [
            step.event.is_some(),
            step.command.is_some(),
            step.modbus_write.is_some(),
            step.mqtt_publish.is_some(),
            step.fault.is_some(),
            step.contact.is_some(),
            step.ops.is_some(),
            step.automation.is_some(),
        ]
        .iter()
        .filter(|present| **present)
        .count();
        if kinds != 1 {
            return Err(ScenarioError::InvalidStepKind);
        }
        if let Some(fault) = &step.fault {
            if let Some(duration) = &fault.duration {
                parse_duration(duration)?;
            }
        }
    }

    for assertion in &scenario.assertions {
        resolve_time_offset(&assertion.at)?;
        let kinds = [
            assertion.mqtt.is_some(),
            assertion.modbus.is_some(),
            assertion.guest_experience.is_some(),
            assertion.ops.is_some(),
            assertion.target.is_some() && assertion.condition.is_some(),
            assertion.inline_assert.is_some(),
        ]
        .iter()
        .filter(|present| **present)
        .count();
        if kinds != 1 {
            return Err(ScenarioError::InvalidAssertionKind);
        }
    }

    Ok(())
}

pub fn resolve_time_offset(expression: &str) -> Result<Duration, ScenarioError> {
    if expression == "T" {
        return Ok(Duration::zero());
    }
    if !expression.starts_with('T') {
        return Err(ScenarioError::InvalidRelativeTime(expression.to_string()));
    }
    let sign = expression
        .chars()
        .nth(1)
        .ok_or_else(|| ScenarioError::InvalidRelativeTime(expression.to_string()))?;
    let duration = parse_duration(&expression[2..])?;
    match sign {
        '+' => Ok(duration),
        '-' => Ok(-duration),
        _ => Err(ScenarioError::InvalidRelativeTime(expression.to_string())),
    }
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

pub fn yaml_map_to_json(
    map: &BTreeMap<String, serde_yaml::Value>,
) -> BTreeMap<String, serde_json::Value> {
    map.iter()
        .map(|(key, value)| {
            (
                key.clone(),
                serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn fixture(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }

    #[test]
    fn validates_latest_local_first_scenario() {
        let scenario = load_scenario(fixture("examples/local_first_cloud_outage.yaml")).unwrap();

        validate_scenario(&scenario).unwrap();

        assert_eq!(scenario.scenario.name, "local_first_cloud_outage");
        assert!(scenario.scenario.tags.contains(&"local-first".to_string()));
    }

    #[test]
    fn resolves_symbolic_time_without_clock() {
        assert_eq!(resolve_time_offset("T+15s").unwrap(), Duration::seconds(15));
        assert_eq!(resolve_time_offset("T").unwrap(), Duration::zero());
    }
}
