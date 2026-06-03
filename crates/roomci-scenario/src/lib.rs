//! Scenario file format and validator for roomci.
//!
//! This crate owns the wire format declared in `examples/*.yaml`: it parses a
//! [`ScenarioFile`], validates that every step, fault, and assertion refers
//! to a known device/scene/contact, and provides time helpers used by
//! `roomci-core` to evaluate scenarios on a virtual clock.

use std::{collections::BTreeMap, fs, path::Path};

use chrono::Duration;
use roomci_device_model::DeviceModelError;
use roomci_edge::EdgeError;
use roomci_ops::OpsError;
use thiserror::Error;

mod schema;
mod validated;

pub use schema::*;
pub use validated::*;

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
    #[error("unsupported fault target/type combination: {target} {fault_type}")]
    InvalidFaultKind { target: String, fault_type: String },
    #[error("assertion must contain a supported condition")]
    InvalidAssertionKind,
    #[error("invalid sensor reading: {0}")]
    InvalidSensorReading(String),
    #[error("invalid identifier {field}: {value}")]
    InvalidIdentifier { field: String, value: String },
    #[error("invalid MQTT topic {field}: {value} ({reason})")]
    InvalidMqttTopic {
        field: String,
        value: String,
        reason: String,
    },
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
    Edge(#[from] EdgeError),
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
        || !contract.edge.commands.is_empty()
        || contract.protocol_profiles.has_profiles();
    if !has_surface {
        return Err(ScenarioError::InvalidAdapterContract(
            "at least one device, MQTT contract, Modbus device, BMS alert, edge command, or protocol profile is required"
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

    validate_protocol_profiles(&contract.protocol_profiles)?;

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

fn validate_protocol_profiles(profiles: &AdapterProtocolProfiles) -> Result<(), ScenarioError> {
    for profile in &profiles.matter {
        require_non_empty("protocol_profiles.matter[].name", &profile.name)?;
        require_non_empty("protocol_profiles.matter[].gateway", &profile.gateway)?;
        require_non_empty("protocol_profiles.matter[].device_id", &profile.device_id)?;
        require_non_empty("protocol_profiles.matter[].cluster", &profile.cluster)?;
        require_non_empty("protocol_profiles.matter[].attribute", &profile.attribute)?;
        require_non_empty("protocol_profiles.matter[].command", &profile.command)?;
        if profile.expected_state.is_empty() {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "Matter profile {} must declare expected_state",
                profile.name
            )));
        }
    }

    for profile in &profiles.bacnet {
        require_non_empty("protocol_profiles.bacnet[].name", &profile.name)?;
        require_non_empty("protocol_profiles.bacnet[].device_id", &profile.device_id)?;
        require_non_empty(
            "protocol_profiles.bacnet[].object_type",
            &profile.object_type,
        )?;
        require_non_empty("protocol_profiles.bacnet[].property", &profile.property)?;
        if profile.expected_value.is_null() {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "BACnet profile {} must declare expected_value",
                profile.name
            )));
        }
    }

    for profile in &profiles.knx {
        require_non_empty("protocol_profiles.knx[].name", &profile.name)?;
        require_non_empty("protocol_profiles.knx[].gateway", &profile.gateway)?;
        require_non_empty(
            "protocol_profiles.knx[].group_address",
            &profile.group_address,
        )?;
        require_non_empty(
            "protocol_profiles.knx[].datapoint_type",
            &profile.datapoint_type,
        )?;
        match profile.direction.as_str() {
            "read" | "write" | "read_write" => {}
            _ => {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "KNX profile {} uses unsupported direction {}; expected read, write, or read_write",
                    profile.name, profile.direction
                )));
            }
        }
        require_non_empty("protocol_profiles.knx[].function", &profile.function)?;
        if profile.expected_value.is_null() {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "KNX profile {} must declare expected_value",
                profile.name
            )));
        }
    }

    for profile in &profiles.opcua {
        require_non_empty("protocol_profiles.opcua[].name", &profile.name)?;
        require_non_empty("protocol_profiles.opcua[].endpoint", &profile.endpoint)?;
        require_non_empty("protocol_profiles.opcua[].namespace", &profile.namespace)?;
        require_non_empty("protocol_profiles.opcua[].node_id", &profile.node_id)?;
        require_non_empty(
            "protocol_profiles.opcua[].browse_name",
            &profile.browse_name,
        )?;
        require_non_empty("protocol_profiles.opcua[].attribute", &profile.attribute)?;
        if profile.expected_value.is_null() {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "OPC UA profile {} must declare expected_value",
                profile.name
            )));
        }
    }

    Ok(())
}

impl AdapterProtocolProfiles {
    fn has_profiles(&self) -> bool {
        !self.matter.is_empty()
            || !self.bacnet.is_empty()
            || !self.knx.is_empty()
            || !self.opcua.is_empty()
    }
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
    ValidatedScenario::try_from(scenario).map(|_| ())
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TypedStepKind<'a> {
    Event(&'a str),
    Command(&'a CommandStep),
    ModbusWrite(&'a ModbusWriteStep),
    MqttPublish(&'a MqttPublishStep),
    Fault(&'a FaultStep),
    Contact(&'a ContactStep),
    Intercom(&'a IntercomStep),
    SensorReading(&'a SensorReadingStep),
    Ops(&'a BTreeMap<String, serde_yaml::Value>),
    Automation(&'a BTreeMap<String, serde_yaml::Value>),
}

pub(crate) fn typed_step_kind(step: &ScenarioStep) -> Result<TypedStepKind<'_>, ScenarioError> {
    let mut kind = None;
    if let Some(event) = step.event.as_deref() {
        set_step_kind(&mut kind, TypedStepKind::Event(event))?;
    }
    if let Some(command) = &step.command {
        set_step_kind(&mut kind, TypedStepKind::Command(command))?;
    }
    if let Some(modbus_write) = &step.modbus_write {
        set_step_kind(&mut kind, TypedStepKind::ModbusWrite(modbus_write))?;
    }
    if let Some(mqtt_publish) = &step.mqtt_publish {
        set_step_kind(&mut kind, TypedStepKind::MqttPublish(mqtt_publish))?;
    }
    if let Some(fault) = &step.fault {
        set_step_kind(&mut kind, TypedStepKind::Fault(fault))?;
    }
    if let Some(contact) = &step.contact {
        set_step_kind(&mut kind, TypedStepKind::Contact(contact))?;
    }
    if let Some(intercom) = &step.intercom {
        set_step_kind(&mut kind, TypedStepKind::Intercom(intercom))?;
    }
    if let Some(sensor_reading) = &step.sensor_reading {
        set_step_kind(&mut kind, TypedStepKind::SensorReading(sensor_reading))?;
    }
    if let Some(ops) = &step.ops {
        set_step_kind(&mut kind, TypedStepKind::Ops(ops))?;
    }
    if let Some(automation) = &step.automation {
        set_step_kind(&mut kind, TypedStepKind::Automation(automation))?;
    }
    kind.ok_or(ScenarioError::InvalidStepKind)
}

fn set_step_kind<'a>(
    slot: &mut Option<TypedStepKind<'a>>,
    kind: TypedStepKind<'a>,
) -> Result<(), ScenarioError> {
    if slot.replace(kind).is_some() {
        return Err(ScenarioError::InvalidStepKind);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypedFaultKind<'a> {
    BrokerOffline {
        target: &'a str,
    },
    EdgePrimaryPowerLost,
    WanPrimaryDown,
    MqttDuplicateDelivery {
        topic: Option<&'a str>,
        count: Option<u32>,
    },
    NetworkSegmentIsolated {
        target: &'a str,
        segment: &'a str,
    },
    FirewallPolicyDrift {
        target: &'a str,
        policy: &'a str,
    },
    MqttLocalUnreachable,
    ControlPanelUpsBatteryDegraded,
    ControlPanelCircuitProtectorTripped {
        target: &'a str,
    },
    ControlPanelPsuDegraded {
        target: &'a str,
    },
    EdgeSecondaryTakeoverFailed,
    DaliFixtureCommandDrop {
        target: &'a str,
        fixture: &'a str,
    },
}

pub(crate) fn typed_fault_kind(fault: &FaultStep) -> Result<TypedFaultKind<'_>, ScenarioError> {
    match (fault.target.as_str(), fault.fault_type.as_str()) {
        (target @ ("mqtt.cloud" | "mqtt.local"), "offline") => {
            Ok(TypedFaultKind::BrokerOffline { target })
        }
        ("edge.primary", "power_lost") => Ok(TypedFaultKind::EdgePrimaryPowerLost),
        ("wan.primary", "down") => Ok(TypedFaultKind::WanPrimaryDown),
        ("mqtt.local", "duplicate_delivery") => Ok(TypedFaultKind::MqttDuplicateDelivery {
            topic: fault.topic.as_deref(),
            count: fault.count,
        }),
        ("mqtt.local", "unreachable") => Ok(TypedFaultKind::MqttLocalUnreachable),
        ("control_panel.ups", "battery_degraded") => {
            Ok(TypedFaultKind::ControlPanelUpsBatteryDegraded)
        }
        ("edge.secondary", "takeover_failed") => Ok(TypedFaultKind::EdgeSecondaryTakeoverFailed),
        (target, "isolated") => {
            let Some(segment) = target.strip_prefix("network.segment.") else {
                return invalid_fault_kind(fault);
            };
            Ok(TypedFaultKind::NetworkSegmentIsolated { target, segment })
        }
        (target, "drift") => {
            let Some(policy) = target.strip_prefix("firewall.policy.") else {
                return invalid_fault_kind(fault);
            };
            Ok(TypedFaultKind::FirewallPolicyDrift { target, policy })
        }
        (target, "tripped") => {
            if !target.starts_with("control_panel.circuit_protector.") {
                return invalid_fault_kind(fault);
            }
            Ok(TypedFaultKind::ControlPanelCircuitProtectorTripped { target })
        }
        (target, "degraded") => {
            if !target.starts_with("control_panel.psu.") {
                return invalid_fault_kind(fault);
            }
            Ok(TypedFaultKind::ControlPanelPsuDegraded { target })
        }
        (target, "command_drop") => {
            let Some(fixture) = target.strip_prefix("dali.fixture.") else {
                return invalid_fault_kind(fault);
            };
            Ok(TypedFaultKind::DaliFixtureCommandDrop { target, fixture })
        }
        _ => invalid_fault_kind(fault),
    }
}

fn invalid_fault_kind<T>(fault: &FaultStep) -> Result<T, ScenarioError> {
    Err(ScenarioError::InvalidFaultKind {
        target: fault.target.clone(),
        fault_type: fault.fault_type.clone(),
    })
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TypedAssertionKind<'a> {
    MqttRetained(&'a MqttAssertion),
    ModbusRegister(&'a ModbusAssertion),
    GuestExperienceField,
    Ops(&'a BTreeMap<String, serde_yaml::Value>),
    TargetCondition(TargetConditionAssertion),
    Inline(InlineAssertionKind<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetConditionAssertion {
    EdgeSecondaryActive,
    MqttLocalAvailable,
    WanBackupActive,
    LivingAreaDiscomfortIndex,
    UserOverrideFalse,
    GuestExperienceUnaffected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineAssertionKind<'a> {
    SceneConsistencyComplete(&'a str),
    AccessControlDriftDetected,
    CommissioningChecklistGenerated,
    IntercomRelaySafeEvidence,
    NetworkControlPanelFaultsObserved,
    ComfortTimeseriesObserved,
}

pub(crate) fn typed_assertion_kind<'a>(
    assertion: &'a AssertionDefinition,
) -> Result<TypedAssertionKind<'a>, ScenarioError> {
    if let Some(mqtt) = &assertion.mqtt {
        return Ok(TypedAssertionKind::MqttRetained(mqtt));
    }
    if let Some(modbus) = &assertion.modbus {
        return Ok(TypedAssertionKind::ModbusRegister(modbus));
    }
    if let Some(expected) = assertion.guest_experience.as_deref() {
        if expected == "unaffected" {
            return Ok(TypedAssertionKind::GuestExperienceField);
        }
        return Err(ScenarioError::InvalidAssertionKind);
    }
    if let Some(ops) = &assertion.ops {
        return Ok(TypedAssertionKind::Ops(ops));
    }
    if let (Some(target), Some(condition)) = (&assertion.target, &assertion.condition) {
        return typed_target_condition_assertion(target, condition)
            .map(TypedAssertionKind::TargetCondition);
    }
    if let Some(inline_assert) = &assertion.inline_assert {
        return typed_inline_assertion(inline_assert).map(TypedAssertionKind::Inline);
    }
    Err(ScenarioError::InvalidAssertionKind)
}

fn typed_target_condition_assertion(
    target: &str,
    condition: &serde_yaml::Value,
) -> Result<TargetConditionAssertion, ScenarioError> {
    let condition_text = condition.as_str().unwrap_or_default();
    match target {
        "edge.secondary" if condition_text == "active" => {
            Ok(TargetConditionAssertion::EdgeSecondaryActive)
        }
        "mqtt.local" if condition_text == "available" => {
            Ok(TargetConditionAssertion::MqttLocalAvailable)
        }
        "wan.backup" if condition_text == "active" => Ok(TargetConditionAssertion::WanBackupActive),
        "living_area.discomfort_index" if condition_text.starts_with("between ") => {
            Ok(TargetConditionAssertion::LivingAreaDiscomfortIndex)
        }
        "user_override" if condition.as_bool() == Some(false) || condition_text == "false" => {
            Ok(TargetConditionAssertion::UserOverrideFalse)
        }
        "guest_experience" if condition_text == "unaffected" => {
            Ok(TargetConditionAssertion::GuestExperienceUnaffected)
        }
        _ => Err(ScenarioError::InvalidAssertionKind),
    }
}

fn typed_inline_assertion<'a>(
    inline_assert: &'a BTreeMap<String, serde_yaml::Value>,
) -> Result<InlineAssertionKind<'a>, ScenarioError> {
    if let Some(scene) = inline_assert.get("scene").and_then(|value| value.as_str()) {
        let complete = inline_assert
            .get("consistency")
            .and_then(|value| value.as_str())
            == Some("complete");
        if complete {
            return Ok(InlineAssertionKind::SceneConsistencyComplete(scene));
        }
    }
    if inline_assert
        .get("access_control_drift")
        .and_then(|value| value.as_str())
        == Some("detected")
    {
        return Ok(InlineAssertionKind::AccessControlDriftDetected);
    }
    if inline_assert
        .get("commissioning_checklist")
        .and_then(|value| value.as_str())
        == Some("generated")
    {
        return Ok(InlineAssertionKind::CommissioningChecklistGenerated);
    }
    if inline_assert
        .get("intercom_relay")
        .and_then(|value| value.as_str())
        == Some("safe_evidence")
    {
        return Ok(InlineAssertionKind::IntercomRelaySafeEvidence);
    }
    if inline_assert
        .get("network_control_panel_faults")
        .and_then(|value| value.as_str())
        == Some("observed")
    {
        return Ok(InlineAssertionKind::NetworkControlPanelFaultsObserved);
    }
    if inline_assert
        .get("comfort_timeseries")
        .and_then(|value| value.as_str())
        == Some("observed")
    {
        return Ok(InlineAssertionKind::ComfortTimeseriesObserved);
    }
    Err(ScenarioError::InvalidAssertionKind)
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
