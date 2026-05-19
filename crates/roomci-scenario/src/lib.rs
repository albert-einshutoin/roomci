//! Scenario file format and validator for roomci.
//!
//! This crate owns the wire format declared in `examples/*.yaml`: it parses a
//! [`ScenarioFile`], validates that every step, fault, and assertion refers
//! to a known device/scene/contact, and provides time helpers used by
//! `roomci-core` to evaluate scenarios on a virtual clock.

use std::{collections::BTreeMap, fs, path::Path};

use chrono::Duration;
use roomci_device_model::{ContactModel, DeviceModelError, LightingModel, ModbusModel};
use roomci_ops::{OpsError, OpsModel};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors produced while loading or validating a scenario.
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
    #[error("unsupported MQTT adapter declaration {0}")]
    UnsupportedMqttAdapter(String),
    #[error("MQTT contract {0} is missing command_topic or state_topic")]
    MissingMqttTopicMapping(String),
    #[error("ambiguous MQTT command topic mapping {0}")]
    AmbiguousMqttMapping(String),
    #[error("MQTT contract {0} uses unsupported device_id_from_topic strategy {1}")]
    UnsupportedMqttDeviceIdStrategy(String, String),
    #[error(transparent)]
    DeviceModel(#[from] DeviceModelError),
    #[error(transparent)]
    Ops(#[from] OpsError),
}

/// Top-level scenario document.
///
/// Every section other than `version` and `scenario` is optional; missing
/// sections fall back to empty maps/lists so a minimal scenario only needs
/// to declare what it actually exercises.
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
    pub sensors: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub comfort: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub inputs: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub commissioning: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub future_milestone: BTreeMap<String, serde_yaml::Value>,
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
    #[serde(default)]
    pub contracts: Vec<MqttConnectionContract>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct BrokerConfig {
    #[serde(default)]
    pub retained: bool,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MqttConnectionContract {
    pub name: String,
    #[serde(default = "default_mqtt_adapter")]
    pub adapter: String,
    pub command_topic: String,
    pub state_topic: String,
    #[serde(default = "default_device_id_strategy")]
    pub device_id_from_topic: String,
    #[serde(default)]
    pub payload: MqttPayloadExpectation,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct MqttPayloadExpectation {
    #[serde(default)]
    pub required_fields: Vec<String>,
}

fn default_mqtt_adapter() -> String {
    "mqtt_v3_qos0_subset".to_string()
}

fn default_device_id_strategy() -> String {
    "placeholder:{device_id}".to_string()
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

/// Read a scenario YAML file from disk and deserialize it.
///
/// Wraps I/O and YAML parsing errors with the file path so the caller gets
/// actionable error messages.
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

/// Validate that every step, fault, and assertion in `scenario` references
/// known devices, scenes, contacts, and alerts.
///
/// Called by `roomci-core::run_scenario` before execution; callers can also
/// invoke it directly for a `--dry-run`-style validation pass.
pub fn validate_scenario(scenario: &ScenarioFile) -> Result<(), ScenarioError> {
    let modbus = ModbusModel::from_config(&scenario.modbus);
    let scene_targets = scenario
        .scenes
        .iter()
        .map(|(name, scene)| (name.clone(), scene.fixtures.clone()))
        .collect::<BTreeMap<_, _>>();
    let lighting = LightingModel::from_config(&scenario.lighting, &scene_targets);
    let contacts = ContactModel::from_config(&scenario.contacts);
    let ops = OpsModel::try_from_config(&scenario.alerts)?;
    lighting.assert_scene_targets_exist()?;
    ops.validate_sources(|contact_id| contacts.has_contact(contact_id))?;
    validate_mqtt_contracts(&scenario.mqtt.contracts)?;

    for fault in &scenario.faults {
        if let Some(at) = &fault.at {
            resolve_time_offset(at)?;
        }
        if let Some(duration) = &fault.duration {
            parse_duration(duration)?;
        }
        validate_fault_reference(fault, &lighting)?;
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
            validate_fault_reference(fault, &lighting)?;
        }
        if let Some(command) = &step.command {
            if let Some(scene) = command.target.strip_prefix("scene.") {
                lighting.assert_scene_exists(scene)?;
            }
        }
        if let Some(modbus_write) = &step.modbus_write {
            modbus.assert_writable(&modbus_write.device, modbus_write.register)?;
        }
        if let Some(contact) = &step.contact {
            if !contacts.has_contact(&contact.id) {
                return Err(DeviceModelError::UnknownContact(contact.id.clone()).into());
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
        if let Some(modbus_assertion) = &assertion.modbus {
            if !modbus.has_register(&modbus_assertion.device, modbus_assertion.register) {
                return Err(DeviceModelError::UnknownModbusRegister {
                    device: modbus_assertion.device.clone(),
                    register: modbus_assertion.register,
                }
                .into());
            }
        }
        if let Some(inline_assert) = &assertion.inline_assert {
            if let Some(scene) = inline_assert.get("scene").and_then(|value| value.as_str()) {
                lighting.assert_scene_exists(scene)?;
            }
        }
    }

    Ok(())
}

fn validate_mqtt_contracts(contracts: &[MqttConnectionContract]) -> Result<(), ScenarioError> {
    let mut command_topics = BTreeMap::<String, String>::new();
    for contract in contracts {
        if contract.adapter != "mqtt_v3_qos0_subset" {
            return Err(ScenarioError::UnsupportedMqttAdapter(
                contract.adapter.clone(),
            ));
        }
        if contract.command_topic.trim().is_empty() || contract.state_topic.trim().is_empty() {
            return Err(ScenarioError::MissingMqttTopicMapping(
                contract.name.clone(),
            ));
        }
        if contract.device_id_from_topic != "placeholder:{device_id}" {
            return Err(ScenarioError::UnsupportedMqttDeviceIdStrategy(
                contract.name.clone(),
                contract.device_id_from_topic.clone(),
            ));
        }
        if !contract.command_topic.contains("{device_id}")
            || !contract.state_topic.contains("{device_id}")
        {
            return Err(ScenarioError::UnsupportedMqttDeviceIdStrategy(
                contract.name.clone(),
                "missing {device_id} placeholder".to_string(),
            ));
        }
        if let Some(existing) =
            command_topics.insert(contract.command_topic.clone(), contract.name.clone())
        {
            return Err(ScenarioError::AmbiguousMqttMapping(format!(
                "{} used by {} and {}",
                contract.command_topic, existing, contract.name
            )));
        }
    }
    Ok(())
}

fn validate_fault_reference(
    fault: &FaultStep,
    lighting: &LightingModel,
) -> Result<(), ScenarioError> {
    if let Some(fixture) = fault.target.strip_prefix("dali.fixture.") {
        lighting.assert_fixture_exists(fixture)?;
    }
    Ok(())
}

/// Resolve a symbolic time offset like `T`, `T+15s`, or `T-1m` into a
/// [`Duration`] relative to scenario start.
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

/// Parse a duration token like `15s`, `2m`, or `1h` into a [`Duration`].
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

/// Convert a YAML key/value map into a JSON-shaped one, replacing
/// non-convertible values with [`serde_json::Value::Null`].
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

    #[test]
    fn rejects_modbus_write_to_read_only_register() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: invalid_read_only_modbus_write
modbus:
  devices:
    - id: floor_heating
      input_registers:
        30001:
          type: decimal_0_1
          value: 228
steps:
  - at: T
    modbus_write:
      device: floor_heating
      register: 30001
      value: 230
assertions:
  - at: T+1s
    modbus:
      device: floor_heating
      register: 30001
      readable_value: 23.0
"#,
        )
        .unwrap();

        let error = validate_scenario(&scenario).unwrap_err();

        assert!(matches!(
            error,
            ScenarioError::DeviceModel(
                roomci_device_model::DeviceModelError::ReadOnlyModbusRegister { .. }
            )
        ));
    }

    #[test]
    fn rejects_unknown_scene_reference() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: invalid_scene_reference
steps:
  - at: T
    command:
      target: scene.missing
      action: activate
assertions:
  - at: T+1s
    assert:
      scene: missing
      consistency: complete
"#,
        )
        .unwrap();

        let error = validate_scenario(&scenario).unwrap_err();

        assert!(matches!(
            error,
            ScenarioError::DeviceModel(roomci_device_model::DeviceModelError::UnknownScene(_))
        ));
    }

    #[test]
    fn rejects_scene_with_unknown_fixture_reference() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: invalid_scene_fixture_reference
lighting:
  fixtures:
    - id: D411S10
      level: 0
scenes:
  welcome:
    fixtures:
      missing_fixture: 60
steps:
  - at: T
    command:
      target: scene.welcome
      action: activate
assertions:
  - at: T+1s
    assert:
      scene: welcome
      consistency: complete
"#,
        )
        .unwrap();

        let error = validate_scenario(&scenario).unwrap_err();

        assert!(matches!(
            error,
            ScenarioError::DeviceModel(roomci_device_model::DeviceModelError::UnknownFixture(_))
        ));
    }

    #[test]
    fn rejects_unknown_contact_reference() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: invalid_contact_reference
steps:
  - at: T
    contact:
      id: missing_contact
      state: on
assertions:
  - at: T+1s
    guest_experience: unaffected
"#,
        )
        .unwrap();

        let error = validate_scenario(&scenario).unwrap_err();

        assert!(matches!(
            error,
            ScenarioError::DeviceModel(roomci_device_model::DeviceModelError::UnknownContact(_))
        ));
    }

    #[test]
    fn validates_mqtt_connection_contracts() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: valid_mqtt_contract
mqtt:
  contracts:
    - name: device_state
      adapter: mqtt_v3_qos0_subset
      command_topic: fleet/demo/device/{device_id}/command
      state_topic: fleet/demo/device/{device_id}/state
      device_id_from_topic: placeholder:{device_id}
      payload:
        required_fields: [online]
steps:
  - at: T
    event: no-op
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
        )
        .unwrap();

        validate_scenario(&scenario).unwrap();
    }

    #[test]
    fn rejects_ambiguous_mqtt_connection_contracts() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: invalid_mqtt_contract
mqtt:
  contracts:
    - name: first
      command_topic: fleet/demo/device/{device_id}/command
      state_topic: fleet/demo/device/{device_id}/state
    - name: second
      command_topic: fleet/demo/device/{device_id}/command
      state_topic: fleet/demo/device/{device_id}/shadow
steps:
  - at: T
    event: no-op
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
        )
        .unwrap();

        let error = validate_scenario(&scenario).unwrap_err();

        assert!(matches!(error, ScenarioError::AmbiguousMqttMapping(_)));
    }

    #[test]
    fn rejects_alert_with_unknown_contact_source() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: invalid_alert_source
contacts:
  inputs:
    - id: known_contact
      state: off
alerts:
  - id: missing_contact_alert
    source: contact.missing_contact
    notify:
      slack: true
steps:
  - at: T
    contact:
      id: known_contact
      state: on
assertions:
  - at: T+1s
    ops:
      slack_notification_sent: true
"#,
        )
        .unwrap();

        let error = validate_scenario(&scenario).unwrap_err();

        assert!(matches!(
            error,
            ScenarioError::Ops(roomci_ops::OpsError::UnknownAlertSource(_))
        ));
    }

    #[test]
    fn rejects_malformed_alert_config() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: malformed_alert
contacts:
  inputs:
    - id: known_contact
      state: off
alerts:
  - source: contact.known_contact
steps:
  - at: T
    contact:
      id: known_contact
      state: on
assertions:
  - at: T+1s
    ops:
      slack_notification_sent: true
"#,
        )
        .unwrap();

        let error = validate_scenario(&scenario).unwrap_err();

        assert!(matches!(
            error,
            ScenarioError::Ops(roomci_ops::OpsError::InvalidAlertConfig(_))
        ));
    }

    #[test]
    fn missing_scenario_file_returns_read_error() {
        let error = load_scenario("/nonexistent/path/scenario.yaml").unwrap_err();

        assert!(matches!(error, ScenarioError::Read { .. }));
    }

    #[test]
    fn invalid_yaml_returns_parse_error() {
        let tempfile = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tempfile.path(), "this is: : not valid: yaml: : :").unwrap();

        let error = load_scenario(tempfile.path()).unwrap_err();

        assert!(matches!(error, ScenarioError::Parse { .. }));
    }

    #[test]
    fn rejects_step_with_no_action() {
        let scenario: ScenarioFile = serde_yaml::from_str(
            r#"
version: "0.1"
scenario:
  name: empty_step
steps:
  - at: T
assertions:
  - at: T+1s
    guest_experience: unaffected
"#,
        )
        .unwrap();

        let error = validate_scenario(&scenario).unwrap_err();

        assert!(matches!(error, ScenarioError::InvalidStepKind));
    }

    #[test]
    fn rejects_invalid_time_offset() {
        let error = resolve_time_offset("invalid_time").unwrap_err();

        assert!(matches!(error, ScenarioError::InvalidRelativeTime(_)));
    }

    #[test]
    fn rejects_invalid_duration() {
        let error = parse_duration("notaduration").unwrap_err();

        assert!(matches!(error, ScenarioError::InvalidDuration(_)));
    }
}
