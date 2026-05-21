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
use thiserror::Error;

mod schema;

pub use schema::*;

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
    #[error("invalid sensor reading: {0}")]
    InvalidSensorReading(String),
    #[error("unsupported MQTT adapter declaration {0}")]
    UnsupportedMqttAdapter(String),
    #[error("MQTT contract {0} is missing command_topic or state_topic")]
    MissingMqttTopicMapping(String),
    #[error("ambiguous MQTT command topic mapping {0}")]
    AmbiguousMqttMapping(String),
    #[error("MQTT contract {0} uses unsupported device_id_from_topic strategy {1}")]
    UnsupportedMqttDeviceIdStrategy(String, String),
    #[error("invalid adapter contract: {0}")]
    InvalidAdapterContract(String),
    #[error(transparent)]
    DeviceModel(#[from] DeviceModelError),
    #[error(transparent)]
    Ops(#[from] OpsError),
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

/// Read a company adapter contract YAML file from disk and deserialize it.
pub fn load_adapter_contract(path: impl AsRef<Path>) -> Result<AdapterContract, ScenarioError> {
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

/// Validate a company adapter contract before it is used to map private specs.
pub fn validate_adapter_contract(contract: &AdapterContract) -> Result<(), ScenarioError> {
    require_non_empty("version", &contract.version)?;
    require_non_empty("adapter.name", &contract.adapter.name)?;

    let has_surface = !contract.devices.is_empty()
        || !contract.mqtt.contracts.is_empty()
        || !contract.modbus.devices.is_empty()
        || !contract.bms.alerts.is_empty()
        || !contract.edge.commands.is_empty();
    if !has_surface {
        return Err(ScenarioError::InvalidAdapterContract(
            "at least one device, MQTT contract, Modbus device, BMS alert, or edge command is required"
                .to_string(),
        ));
    }

    for device in &contract.devices {
        require_non_empty("devices[].id", &device.id)?;
        require_non_empty("devices[].type", &device.device_type)?;
        require_non_empty("devices[].protocol", &device.protocol)?;
    }

    validate_mqtt_contracts(&contract.mqtt.contracts)?;

    for device in &contract.modbus.devices {
        require_non_empty("modbus.devices[].id", &device.id)?;
        if device.registers.is_empty() {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "modbus device {} must declare at least one register",
                device.id
            )));
        }
        for register in &device.registers {
            require_non_empty("modbus.registers[].name", &register.name)?;
            require_non_empty("modbus.registers[].type", &register.register_type)?;
            match register.access.as_str() {
                "read" | "write" | "read_write" => {}
                _ => {
                    return Err(ScenarioError::InvalidAdapterContract(format!(
                        "modbus register {} uses unsupported access {}; expected read, write, or read_write",
                        register.address, register.access
                    )));
                }
            }
        }
    }

    for alert in &contract.bms.alerts {
        require_non_empty("bms.alerts[].id", &alert.id)?;
        require_non_empty("bms.alerts[].source", &alert.source)?;
        require_non_empty("bms.alerts[].severity", &alert.severity)?;
        if let Some(schema_version) = &alert.schema_version {
            require_non_empty("bms.alerts[].schema_version", schema_version)?;
        }
        if let Some(content_type) = &alert.content_type {
            if content_type != "application/json" {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "BMS alert {} uses unsupported content_type {}; expected application/json",
                    alert.id, content_type
                )));
            }
        }
        if !alert.severity_enum.is_empty()
            && !alert
                .severity_enum
                .iter()
                .any(|severity| severity == &alert.severity)
        {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "BMS alert {} severity {} is not declared in severity_enum",
                alert.id, alert.severity
            )));
        }
        if let Some(hmac) = &alert.hmac {
            require_non_empty("bms.alerts[].hmac.header", &hmac.header)?;
            require_non_empty("bms.alerts[].hmac.algorithm", &hmac.algorithm)?;
            require_non_empty("bms.alerts[].hmac.secret_ref", &hmac.secret_ref)?;
            if hmac.algorithm != "hmac-sha256" {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "BMS alert {} uses unsupported HMAC algorithm {}; expected hmac-sha256",
                    alert.id, hmac.algorithm
                )));
            }
        }
        if alert.replay_window_seconds == Some(0) {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "BMS alert {} replay_window_seconds must be greater than zero",
                alert.id
            )));
        }
        if alert.channels.is_empty() {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "BMS alert {} must declare at least one notification channel",
                alert.id
            )));
        }
    }

    for command in &contract.edge.commands {
        require_non_empty("edge.commands[].name", &command.name)?;
        require_non_empty("edge.commands[].source", &command.source)?;
        require_non_empty("edge.commands[].target", &command.target)?;
        require_non_empty("edge.commands[].expected_state", &command.expected_state)?;
    }

    if contract.acceptance.criteria.is_empty() {
        return Err(ScenarioError::InvalidAdapterContract(
            "acceptance.criteria must declare at least one pass/fail criterion".to_string(),
        ));
    }
    if contract.acceptance.report_formats.is_empty() {
        return Err(ScenarioError::InvalidAdapterContract(
            "acceptance.report_formats must declare at least one report format".to_string(),
        ));
    }

    Ok(())
}

fn require_non_empty(field: &str, value: &str) -> Result<(), ScenarioError> {
    if value.trim().is_empty() {
        return Err(ScenarioError::InvalidAdapterContract(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
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
            step.intercom.is_some(),
            step.sensor_reading.is_some(),
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
        if let Some(intercom) = &step.intercom {
            require_non_empty("steps[].intercom.id", &intercom.id)?;
            require_non_empty("steps[].intercom.event", &intercom.event)?;
            require_non_empty("steps[].intercom.outcome", &intercom.outcome)?;
        }
        if let Some(sensor_reading) = &step.sensor_reading {
            require_non_empty("steps[].sensor_reading.target", &sensor_reading.target)?;
            if !(0.0..=100.0).contains(&sensor_reading.humidity) {
                return Err(ScenarioError::InvalidSensorReading(
                    "steps[].sensor_reading.humidity must be between 0 and 100".to_string(),
                ));
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
mod tests;
