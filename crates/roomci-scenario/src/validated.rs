use std::collections::BTreeMap;

use chrono::Duration;
use roomci_device_model::{ContactModel, LightingModel, ModbusModel};
use roomci_edge::EdgeModel;
use roomci_ops::OpsModel;

use crate::{
    parse_duration, resolve_time_offset, typed_assertion_kind, typed_fault_kind, typed_step_kind,
    validate_scenario_version, yaml_map_to_json, AssertionDefinition, CommandStep, FaultStep,
    InlineAssertionKind, IntercomStep, ModbusAssertion, ModbusWriteStep, MqttAssertion,
    MqttPublishStep, ScenarioError, ScenarioFile, SensorReadingStep, TargetConditionAssertion,
    TypedAssertionKind, TypedFaultKind, TypedStepKind,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub fn parse(
        field: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, ScenarioError> {
        let field = field.into();
        let value = value.into();
        if value.trim().is_empty() || value.chars().any(char::is_whitespace) {
            return Err(ScenarioError::InvalidIdentifier { field, value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Identifier);

        impl $name {
            pub fn parse(
                field: impl Into<String>,
                value: impl Into<String>,
            ) -> Result<Self, ScenarioError> {
                Identifier::parse(field, value).map(Self)
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

id_newtype!(DeviceId);
id_newtype!(SceneId);
id_newtype!(FixtureId);
id_newtype!(ContactId);
id_newtype!(AlertId);
id_newtype!(BrokerId);
id_newtype!(EdgeId);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MqttTopic(String);

impl MqttTopic {
    pub fn parse(
        field: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, ScenarioError> {
        let field = field.into();
        let value = value.into();
        validate_mqtt_topic_text(&field, &value, false)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MqttTopic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MqttTopicTemplate(String);

impl MqttTopicTemplate {
    pub fn parse(
        field: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, ScenarioError> {
        let field = field.into();
        let value = value.into();
        validate_mqtt_topic_text(&field, &value, true)?;
        Ok(Self(value))
    }
}

fn validate_mqtt_topic_text(
    field: &str,
    value: &str,
    allow_device_placeholder: bool,
) -> Result<(), ScenarioError> {
    let trimmed = value.trim();
    let valid_placeholder = allow_device_placeholder && trimmed.contains("{device_id}");
    if trimmed.is_empty() {
        return invalid_mqtt_topic(field, value, "must not be empty");
    }
    if trimmed != value {
        return invalid_mqtt_topic(
            field,
            value,
            "must not contain leading or trailing whitespace",
        );
    }
    if value.chars().any(char::is_whitespace) {
        return invalid_mqtt_topic(field, value, "must not contain whitespace");
    }
    if value.contains('#') || value.contains('+') {
        return invalid_mqtt_topic(
            field,
            value,
            "must be a concrete publish topic, not a wildcard subscription",
        );
    }
    if allow_device_placeholder && !valid_placeholder {
        return invalid_mqtt_topic(field, value, "must include {device_id}");
    }
    Ok(())
}

fn invalid_mqtt_topic<T>(field: &str, value: &str, reason: &str) -> Result<T, ScenarioError> {
    Err(ScenarioError::InvalidMqttTopic {
        field: field.to_string(),
        value: value.to_string(),
        reason: reason.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedScenario {
    raw: ScenarioFile,
    devices: Vec<ValidatedDevice>,
    scheduled_events: Vec<ValidatedScheduledEvent>,
    domain_config: DomainRuntimeConfig,
    runtime_inputs: ValidatedRuntimeInputs,
}

impl ValidatedScenario {
    pub fn raw(&self) -> &ScenarioFile {
        &self.raw
    }

    pub fn scenario_name(&self) -> &str {
        &self.raw.scenario.name
    }

    pub fn run_id(&self) -> &str {
        self.scenario_name()
    }

    pub fn devices(&self) -> &[ValidatedDevice] {
        &self.devices
    }

    pub fn scheduled_events(&self) -> &[ValidatedScheduledEvent] {
        &self.scheduled_events
    }

    pub fn domain_config(&self) -> &DomainRuntimeConfig {
        &self.domain_config
    }

    pub fn runtime_inputs(&self) -> &ValidatedRuntimeInputs {
        &self.runtime_inputs
    }
}

impl TryFrom<&ScenarioFile> for ValidatedScenario {
    type Error = ScenarioError;

    fn try_from(scenario: &ScenarioFile) -> Result<Self, Self::Error> {
        validate_scenario_version(&scenario.version)?;

        let modbus = ModbusModel::try_from_config(&scenario.modbus)?;
        let scene_targets = scenario
            .scenes
            .iter()
            .map(|(name, scene)| (name.clone(), scene.fixtures.clone()))
            .collect::<BTreeMap<_, _>>();
        let lighting = LightingModel::try_from_config(&scenario.lighting, &scene_targets)?;
        let contacts = ContactModel::try_from_config(&scenario.contacts)?;
        let edge = EdgeModel::try_from_config(&scenario.edge)?;
        let ops = OpsModel::try_from_config(&scenario.alerts)?;
        lighting.assert_scene_targets_exist()?;
        ops.validate_sources(|contact_id| contacts.has_contact(contact_id))?;
        validate_mqtt_contract_topics(scenario)?;

        let devices = scenario
            .devices
            .iter()
            .map(|device| {
                Ok(ValidatedDevice {
                    id: DeviceId::parse("devices[].id", device.id.clone())?,
                    device_type: Identifier::parse("devices[].type", device.device_type.clone())?,
                    protocol: device
                        .protocol
                        .as_ref()
                        .map(|protocol| Identifier::parse("devices[].protocol", protocol.clone()))
                        .transpose()?,
                    state: yaml_map_to_json(&device.state),
                })
            })
            .collect::<Result<Vec<_>, ScenarioError>>()?;

        validate_scene_and_alert_ids(scenario)?;

        let mut scheduled_events = Vec::new();
        for fault in &scenario.faults {
            let at = fault
                .at
                .as_deref()
                .map(resolve_time_offset)
                .transpose()?
                .unwrap_or_else(Duration::zero);
            if let Some(duration) = &fault.duration {
                parse_duration(duration)?;
            }
            let kind = ValidatedFaultKind::try_from_fault(fault)?;
            validate_fault_reference(fault, &lighting)?;
            scheduled_events.push(ValidatedScheduledEvent::global_fault(at, kind));
        }

        for step in &scenario.steps {
            let at = resolve_time_offset(&step.at)?;
            let kind = ValidatedStepKind::try_from_step(step, &modbus, &lighting, &contacts)?;
            scheduled_events.push(ValidatedScheduledEvent::step(at, kind));
        }

        for assertion in &scenario.assertions {
            let at = resolve_time_offset(&assertion.at)?;
            let kind = ValidatedAssertionKind::try_from_assertion(assertion, &modbus, &lighting)?;
            scheduled_events.push(ValidatedScheduledEvent::assertion(at, kind));
        }

        scheduled_events.sort_by_key(|event| (event.at.num_milliseconds(), event.order));

        Ok(Self {
            raw: scenario.clone(),
            devices,
            scheduled_events,
            domain_config: DomainRuntimeConfig::try_from_scenario(scenario)?,
            runtime_inputs: ValidatedRuntimeInputs {
                broker_cloud_enabled: scenario.mqtt.cloud.enabled,
                edge,
                modbus,
                lighting,
                contacts,
                ops,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedRuntimeInputs {
    pub broker_cloud_enabled: bool,
    pub edge: EdgeModel,
    pub modbus: ModbusModel,
    pub lighting: LightingModel,
    pub contacts: ContactModel,
    pub ops: OpsModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedDevice {
    pub id: DeviceId,
    pub device_type: Identifier,
    pub protocol: Option<Identifier>,
    pub state: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedScheduledEvent {
    at: Duration,
    order: u8,
    kind: ValidatedEventKind,
}

impl ValidatedScheduledEvent {
    fn global_fault(at: Duration, fault: ValidatedFaultKind) -> Self {
        Self {
            at,
            order: 0,
            kind: ValidatedEventKind::GlobalFault(fault),
        }
    }

    fn step(at: Duration, step: ValidatedStepKind) -> Self {
        Self {
            at,
            order: 0,
            kind: ValidatedEventKind::Step(step),
        }
    }

    fn assertion(at: Duration, assertion: ValidatedAssertionKind) -> Self {
        Self {
            at,
            order: 1,
            kind: ValidatedEventKind::Assertion(assertion),
        }
    }

    pub fn at(&self) -> Duration {
        self.at
    }

    pub fn kind(&self) -> &ValidatedEventKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidatedEventKind {
    GlobalFault(ValidatedFaultKind),
    Step(ValidatedStepKind),
    Assertion(ValidatedAssertionKind),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidatedStepKind {
    Event(String),
    Command(ValidatedCommandStep),
    ModbusWrite(ValidatedModbusWriteStep),
    MqttPublish(ValidatedMqttPublishStep),
    Fault(ValidatedFaultKind),
    Contact(ValidatedContactStep),
    Intercom(ValidatedIntercomStep),
    SensorReading(ValidatedSensorReadingStep),
    Ops(ValidatedOpsStep),
    Automation(ValidatedAutomationStep),
}

impl ValidatedStepKind {
    fn try_from_step(
        step: &crate::ScenarioStep,
        modbus: &ModbusModel,
        lighting: &LightingModel,
        contacts: &ContactModel,
    ) -> Result<Self, ScenarioError> {
        match typed_step_kind(step)? {
            TypedStepKind::Event(event) => Ok(Self::Event(event.to_string())),
            TypedStepKind::Command(command) => {
                if let Some(scene) = command.target.strip_prefix("scene.") {
                    SceneId::parse("steps[].command.target", scene.to_string())?;
                    lighting.assert_scene_exists(scene)?;
                }
                Ok(Self::Command(ValidatedCommandStep::try_from(command)?))
            }
            TypedStepKind::ModbusWrite(modbus_write) => {
                let step = ValidatedModbusWriteStep::try_from(modbus_write)?;
                modbus.assert_writable(step.device.as_str(), step.register)?;
                Ok(Self::ModbusWrite(step))
            }
            TypedStepKind::MqttPublish(mqtt_publish) => Ok(Self::MqttPublish(
                ValidatedMqttPublishStep::try_from(mqtt_publish)?,
            )),
            TypedStepKind::Fault(fault) => {
                if let Some(duration) = &fault.duration {
                    parse_duration(duration)?;
                }
                let kind = ValidatedFaultKind::try_from_fault(fault)?;
                validate_fault_reference(fault, lighting)?;
                Ok(Self::Fault(kind))
            }
            TypedStepKind::Contact(contact) => {
                let step = ValidatedContactStep::try_from(contact)?;
                if !contacts.has_contact(step.id.as_str()) {
                    return Err(roomci_device_model::DeviceModelError::UnknownContact(
                        step.id.to_string(),
                    )
                    .into());
                }
                Ok(Self::Contact(step))
            }
            TypedStepKind::Intercom(intercom) => {
                Ok(Self::Intercom(ValidatedIntercomStep::try_from(intercom)?))
            }
            TypedStepKind::SensorReading(sensor_reading) => Ok(Self::SensorReading(
                ValidatedSensorReadingStep::try_from(sensor_reading)?,
            )),
            TypedStepKind::Ops(ops) => Ok(Self::Ops(ValidatedOpsStep::try_from_map(ops)?)),
            TypedStepKind::Automation(automation) => Ok(Self::Automation(
                ValidatedAutomationStep::try_from_map(automation)?,
            )),
        }
    }
}

// `Eq` is intentionally omitted: `serde_json::Value` is not `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedCommandStep {
    pub target: CommandTarget,
    pub action: Identifier,
    /// Payload for value-bearing commands (`set_brightness`, `set_temperature`,
    /// ...). Normalized to JSON so the runtime can apply it directly to device
    /// state.
    pub value: Option<serde_json::Value>,
}

impl TryFrom<&CommandStep> for ValidatedCommandStep {
    type Error = ScenarioError;

    fn try_from(command: &CommandStep) -> Result<Self, Self::Error> {
        Ok(Self {
            target: CommandTarget::parse(&command.target)?,
            action: Identifier::parse("steps[].command.action", command.action.clone())?,
            value: command
                .value
                .as_ref()
                .map(|value| serde_json::to_value(value).unwrap_or(serde_json::Value::Null)),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandTarget {
    Scene(SceneId),
    Other(Identifier),
}

impl CommandTarget {
    fn parse(target: &str) -> Result<Self, ScenarioError> {
        if let Some(scene) = target.strip_prefix("scene.") {
            return Ok(Self::Scene(SceneId::parse(
                "steps[].command.target",
                scene.to_string(),
            )?));
        }
        Ok(Self::Other(Identifier::parse(
            "steps[].command.target",
            target.to_string(),
        )?))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Scene(scene) => scene.as_str(),
            Self::Other(target) => target.as_str(),
        }
    }

    pub fn display_target(&self) -> String {
        match self {
            Self::Scene(scene) => format!("scene.{scene}"),
            Self::Other(target) => target.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedModbusWriteStep {
    pub device: DeviceId,
    pub register: u32,
    /// Adapter payload boundary: Modbus register values intentionally preserve
    /// YAML shape until the device model converts them to runtime JSON.
    pub value: serde_yaml::Value,
}

impl TryFrom<&ModbusWriteStep> for ValidatedModbusWriteStep {
    type Error = ScenarioError;

    fn try_from(step: &ModbusWriteStep) -> Result<Self, Self::Error> {
        Ok(Self {
            device: DeviceId::parse("steps[].modbus_write.device", step.device.clone())?,
            register: step.register,
            value: step.value.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedMqttPublishStep {
    pub client: Identifier,
    pub topic: MqttTopic,
    /// Adapter payload boundary: scenario MQTT payloads stay schema-compatible
    /// maps; runtime contract validation specializes required fields per topic.
    pub payload: BTreeMap<String, serde_yaml::Value>,
}

impl TryFrom<&MqttPublishStep> for ValidatedMqttPublishStep {
    type Error = ScenarioError;

    fn try_from(step: &MqttPublishStep) -> Result<Self, Self::Error> {
        Ok(Self {
            client: Identifier::parse("steps[].mqtt_publish.client", step.client.clone())?,
            topic: MqttTopic::parse("steps[].mqtt_publish.topic", step.topic.clone())?,
            payload: step.payload.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedContactStep {
    pub id: ContactId,
    pub state: Identifier,
}

impl TryFrom<&crate::ContactStep> for ValidatedContactStep {
    type Error = ScenarioError;

    fn try_from(step: &crate::ContactStep) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ContactId::parse("steps[].contact.id", step.id.clone())?,
            state: Identifier::parse("steps[].contact.state", step.state.clone())?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedIntercomStep {
    pub id: Identifier,
    pub event: Identifier,
    pub outcome: Identifier,
    pub fallback: Option<String>,
}

impl TryFrom<&IntercomStep> for ValidatedIntercomStep {
    type Error = ScenarioError;

    fn try_from(step: &IntercomStep) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Identifier::parse("steps[].intercom.id", step.id.clone())?,
            event: Identifier::parse("steps[].intercom.event", step.event.clone())?,
            outcome: Identifier::parse("steps[].intercom.outcome", step.outcome.clone())?,
            fallback: step.fallback.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedSensorReadingStep {
    pub target: SensorTarget,
    pub temperature: f64,
    pub humidity: f64,
    pub occupancy: Option<bool>,
    pub zone: Option<String>,
}

impl TryFrom<&SensorReadingStep> for ValidatedSensorReadingStep {
    type Error = ScenarioError;

    fn try_from(step: &SensorReadingStep) -> Result<Self, Self::Error> {
        if !(0.0..=100.0).contains(&step.humidity) {
            return Err(ScenarioError::InvalidSensorReading(
                "steps[].sensor_reading.humidity must be between 0 and 100".to_string(),
            ));
        }
        Ok(Self {
            target: SensorTarget::parse(step.target.clone())?,
            temperature: step.temperature,
            humidity: step.humidity,
            occupancy: step.occupancy,
            zone: step.zone.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorTarget(Identifier);

impl SensorTarget {
    fn parse(value: impl Into<String>) -> Result<Self, ScenarioError> {
        Identifier::parse("steps[].sensor_reading.target", value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for SensorTarget {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatedFaultKind {
    BrokerOffline {
        target: BrokerId,
    },
    EdgePrimaryPowerLost,
    WanPrimaryDown,
    MqttDuplicateDelivery {
        topic: Option<MqttTopic>,
        count: Option<u32>,
    },
    NetworkSegmentIsolated {
        segment: Identifier,
    },
    FirewallPolicyDrift {
        policy: Identifier,
    },
    MqttLocalUnreachable,
    ControlPanelUpsBatteryDegraded,
    ControlPanelCircuitProtectorTripped {
        protector: Identifier,
    },
    ControlPanelPsuDegraded {
        psu: Identifier,
    },
    EdgeSecondaryTakeoverFailed,
    DaliFixtureCommandDrop {
        fixture: FixtureId,
    },
}

impl ValidatedFaultKind {
    fn try_from_fault(fault: &FaultStep) -> Result<Self, ScenarioError> {
        match typed_fault_kind(fault)? {
            TypedFaultKind::BrokerOffline { target } => Ok(Self::BrokerOffline {
                target: BrokerId::parse("faults[].target", target.to_string())?,
            }),
            TypedFaultKind::EdgePrimaryPowerLost => Ok(Self::EdgePrimaryPowerLost),
            TypedFaultKind::WanPrimaryDown => Ok(Self::WanPrimaryDown),
            TypedFaultKind::MqttDuplicateDelivery { topic, count } => {
                Ok(Self::MqttDuplicateDelivery {
                    topic: topic
                        .map(|topic| MqttTopic::parse("faults[].topic", topic.to_string()))
                        .transpose()?,
                    count,
                })
            }
            TypedFaultKind::NetworkSegmentIsolated { target, segment } => {
                let _ = target;
                Ok(Self::NetworkSegmentIsolated {
                    segment: Identifier::parse("faults[].network.segment", segment.to_string())?,
                })
            }
            TypedFaultKind::FirewallPolicyDrift { target, policy } => {
                let _ = target;
                Ok(Self::FirewallPolicyDrift {
                    policy: Identifier::parse("faults[].firewall.policy", policy.to_string())?,
                })
            }
            TypedFaultKind::MqttLocalUnreachable => Ok(Self::MqttLocalUnreachable),
            TypedFaultKind::ControlPanelUpsBatteryDegraded => {
                Ok(Self::ControlPanelUpsBatteryDegraded)
            }
            TypedFaultKind::ControlPanelCircuitProtectorTripped { target } => {
                let protector = target
                    .strip_prefix("control_panel.circuit_protector.")
                    .ok_or_else(|| ScenarioError::InvalidFaultKind {
                        target: target.to_string(),
                        fault_type: "tripped".to_string(),
                    })?;
                Ok(Self::ControlPanelCircuitProtectorTripped {
                    protector: Identifier::parse(
                        "faults[].control_panel.circuit_protector",
                        protector.to_string(),
                    )?,
                })
            }
            TypedFaultKind::ControlPanelPsuDegraded { target } => {
                let psu = target.strip_prefix("control_panel.psu.").ok_or_else(|| {
                    ScenarioError::InvalidFaultKind {
                        target: target.to_string(),
                        fault_type: "degraded".to_string(),
                    }
                })?;
                Ok(Self::ControlPanelPsuDegraded {
                    psu: Identifier::parse("faults[].control_panel.psu", psu.to_string())?,
                })
            }
            TypedFaultKind::EdgeSecondaryTakeoverFailed => Ok(Self::EdgeSecondaryTakeoverFailed),
            TypedFaultKind::DaliFixtureCommandDrop { target, fixture } => {
                let _ = target;
                Ok(Self::DaliFixtureCommandDrop {
                    fixture: FixtureId::parse("faults[].target.fixture", fixture.to_string())?,
                })
            }
        }
    }

    pub fn target(&self) -> String {
        match self {
            Self::BrokerOffline { target } => target.to_string(),
            Self::NetworkSegmentIsolated { segment } => format!("network.segment.{segment}"),
            Self::FirewallPolicyDrift { policy } => format!("firewall.policy.{policy}"),
            Self::ControlPanelCircuitProtectorTripped { protector } => {
                format!("control_panel.circuit_protector.{protector}")
            }
            Self::ControlPanelPsuDegraded { psu } => format!("control_panel.psu.{psu}"),
            Self::DaliFixtureCommandDrop { fixture } => format!("dali.fixture.{fixture}"),
            Self::EdgePrimaryPowerLost => "edge.primary".to_string(),
            Self::WanPrimaryDown => "wan.primary".to_string(),
            Self::MqttDuplicateDelivery { .. } | Self::MqttLocalUnreachable => {
                "mqtt.local".to_string()
            }
            Self::ControlPanelUpsBatteryDegraded => "control_panel.ups".to_string(),
            Self::EdgeSecondaryTakeoverFailed => "edge.secondary".to_string(),
        }
    }

    pub fn fault_type(&self) -> &'static str {
        match self {
            Self::BrokerOffline { .. } => "offline",
            Self::EdgePrimaryPowerLost => "power_lost",
            Self::WanPrimaryDown => "down",
            Self::MqttDuplicateDelivery { .. } => "duplicate_delivery",
            Self::NetworkSegmentIsolated { .. } => "isolated",
            Self::FirewallPolicyDrift { .. } => "drift",
            Self::MqttLocalUnreachable => "unreachable",
            Self::ControlPanelUpsBatteryDegraded => "battery_degraded",
            Self::ControlPanelCircuitProtectorTripped { .. } => "tripped",
            Self::ControlPanelPsuDegraded { .. } => "degraded",
            Self::EdgeSecondaryTakeoverFailed => "takeover_failed",
            Self::DaliFixtureCommandDrop { .. } => "command_drop",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidatedOpsStep {
    Acknowledge {
        alert_id: Option<AlertId>,
        assignee: Option<Identifier>,
    },
    /// Extension boundary for non-promoted ops actions. Promoted behavior must
    /// be added as an enum variant before runtime dispatch depends on it.
    Extension(BTreeMap<String, serde_yaml::Value>),
}

impl ValidatedOpsStep {
    fn try_from_map(map: &BTreeMap<String, serde_yaml::Value>) -> Result<Self, ScenarioError> {
        match map.get("action").and_then(|value| value.as_str()) {
            Some("acknowledge") => Ok(Self::Acknowledge {
                alert_id: map
                    .get("alert_id")
                    .and_then(|value| value.as_str())
                    .map(|value| AlertId::parse("steps[].ops.alert_id", value.to_string()))
                    .transpose()?,
                assignee: map
                    .get("assignee")
                    .and_then(|value| value.as_str())
                    .map(|value| Identifier::parse("steps[].ops.assignee", value.to_string()))
                    .transpose()?,
            }),
            _ => Ok(Self::Extension(map.clone())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidatedAutomationStep {
    HvacAutoMode,
    /// Extension boundary for non-promoted automation actions. Promoted
    /// automation must be modeled as a typed variant before execution logic.
    Extension(BTreeMap<String, serde_yaml::Value>),
}

impl ValidatedAutomationStep {
    fn try_from_map(map: &BTreeMap<String, serde_yaml::Value>) -> Result<Self, ScenarioError> {
        match map.get("type").and_then(|value| value.as_str()) {
            Some("hvac_auto_mode") => Ok(Self::HvacAutoMode),
            _ => Ok(Self::Extension(map.clone())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidatedAssertionKind {
    MqttRetained(ValidatedMqttAssertion),
    ModbusRegister(ValidatedModbusAssertion),
    GuestExperienceField(Condition),
    /// Compatibility boundary for the existing ops assertion evaluator.
    /// Runtime can evaluate the map, but promoted ops steps are typed above.
    Ops(BTreeMap<String, serde_yaml::Value>),
    TargetCondition(ValidatedTargetConditionAssertion),
    Inline(ValidatedInlineAssertionKind),
}

impl ValidatedAssertionKind {
    fn try_from_assertion(
        assertion: &AssertionDefinition,
        modbus: &ModbusModel,
        lighting: &LightingModel,
    ) -> Result<Self, ScenarioError> {
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

        match typed_assertion_kind(assertion)? {
            TypedAssertionKind::MqttRetained(mqtt) => {
                Ok(Self::MqttRetained(ValidatedMqttAssertion::try_from(mqtt)?))
            }
            TypedAssertionKind::ModbusRegister(modbus_assertion) => {
                let assertion = ValidatedModbusAssertion::try_from(modbus_assertion)?;
                if !modbus.has_register(assertion.device.as_str(), assertion.register) {
                    return Err(
                        roomci_device_model::DeviceModelError::UnknownModbusRegister {
                            device: assertion.device.to_string(),
                            register: assertion.register,
                        }
                        .into(),
                    );
                }
                Ok(Self::ModbusRegister(assertion))
            }
            TypedAssertionKind::GuestExperienceField => {
                Ok(Self::GuestExperienceField(Condition::Unaffected))
            }
            TypedAssertionKind::Ops(ops) => Ok(Self::Ops(ops.clone())),
            TypedAssertionKind::TargetCondition(kind) => {
                let target = assertion.target.as_deref().unwrap_or_default();
                let condition = assertion
                    .condition
                    .as_ref()
                    .ok_or(ScenarioError::InvalidAssertionKind)?;
                Ok(Self::TargetCondition(
                    ValidatedTargetConditionAssertion::try_from_parts(target, condition, kind)?,
                ))
            }
            TypedAssertionKind::Inline(kind) => {
                let inline = ValidatedInlineAssertionKind::try_from_kind(kind)?;
                if let ValidatedInlineAssertionKind::SceneConsistencyComplete(scene) = &inline {
                    lighting.assert_scene_exists(scene.as_str())?;
                }
                Ok(Self::Inline(inline))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedMqttAssertion {
    pub topic: MqttTopic,
    /// Adapter assertion boundary: retained payload shape is topic-specific and
    /// intentionally remains YAML until adapter contract rules specialize it.
    pub retained: BTreeMap<String, serde_yaml::Value>,
}

impl TryFrom<&MqttAssertion> for ValidatedMqttAssertion {
    type Error = ScenarioError;

    fn try_from(assertion: &MqttAssertion) -> Result<Self, Self::Error> {
        Ok(Self {
            topic: MqttTopic::parse("assertions[].mqtt.topic", assertion.topic.clone())?,
            retained: assertion.retained.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedModbusAssertion {
    pub device: DeviceId,
    pub register: u32,
    /// Adapter assertion boundary for exact register comparisons.
    pub value: Option<serde_yaml::Value>,
    pub readable_value: Option<f64>,
}

impl TryFrom<&ModbusAssertion> for ValidatedModbusAssertion {
    type Error = ScenarioError;

    fn try_from(assertion: &ModbusAssertion) -> Result<Self, Self::Error> {
        Ok(Self {
            device: DeviceId::parse("assertions[].modbus.device", assertion.device.clone())?,
            register: assertion.register,
            value: assertion.value.clone(),
            readable_value: assertion.readable_value,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Condition {
    Active,
    Available,
    Unaffected,
    False,
    Between { min: f64, max: f64 },
}

impl Condition {
    fn parse_between(condition_text: &str) -> Result<Self, ScenarioError> {
        let rest = condition_text
            .strip_prefix("between ")
            .ok_or(ScenarioError::InvalidAssertionKind)?;
        let (min, max) = rest
            .split_once(" and ")
            .ok_or(ScenarioError::InvalidAssertionKind)?;
        let min = min
            .parse::<f64>()
            .map_err(|_| ScenarioError::InvalidAssertionKind)?;
        let max = max
            .parse::<f64>()
            .map_err(|_| ScenarioError::InvalidAssertionKind)?;
        if !min.is_finite() || !max.is_finite() || min > max {
            return Err(ScenarioError::InvalidAssertionKind);
        }
        Ok(Self::Between { min, max })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidatedTargetConditionAssertion {
    EdgeSecondary { condition: Condition },
    MqttLocal { condition: Condition },
    WanBackup { condition: Condition },
    LivingAreaDiscomfortIndex { condition: Condition },
    UserOverride { condition: Condition },
    GuestExperience { condition: Condition },
}

impl ValidatedTargetConditionAssertion {
    fn try_from_parts(
        target: &str,
        condition: &serde_yaml::Value,
        kind: TargetConditionAssertion,
    ) -> Result<Self, ScenarioError> {
        let condition_text = condition.as_str().unwrap_or_default();
        match kind {
            TargetConditionAssertion::EdgeSecondaryActive => Ok(Self::EdgeSecondary {
                condition: Condition::Active,
            }),
            TargetConditionAssertion::MqttLocalAvailable => Ok(Self::MqttLocal {
                condition: Condition::Available,
            }),
            TargetConditionAssertion::WanBackupActive => Ok(Self::WanBackup {
                condition: Condition::Active,
            }),
            TargetConditionAssertion::LivingAreaDiscomfortIndex => {
                let _ = Identifier::parse("assertions[].target", target.to_string())?;
                Ok(Self::LivingAreaDiscomfortIndex {
                    condition: Condition::parse_between(condition_text)?,
                })
            }
            TargetConditionAssertion::UserOverrideFalse => Ok(Self::UserOverride {
                condition: Condition::False,
            }),
            TargetConditionAssertion::GuestExperienceUnaffected => Ok(Self::GuestExperience {
                condition: Condition::Unaffected,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatedInlineAssertionKind {
    SceneConsistencyComplete(SceneId),
    AccessControlDriftDetected,
    CommissioningChecklistGenerated,
    IntercomRelaySafeEvidence,
    NetworkControlPanelFaultsObserved,
    ComfortTimeseriesObserved,
}

impl ValidatedInlineAssertionKind {
    fn try_from_kind(kind: InlineAssertionKind<'_>) -> Result<Self, ScenarioError> {
        Ok(match kind {
            InlineAssertionKind::SceneConsistencyComplete(scene) => Self::SceneConsistencyComplete(
                SceneId::parse("assertions[].assert.scene", scene.to_string())?,
            ),
            InlineAssertionKind::AccessControlDriftDetected => Self::AccessControlDriftDetected,
            InlineAssertionKind::CommissioningChecklistGenerated => {
                Self::CommissioningChecklistGenerated
            }
            InlineAssertionKind::IntercomRelaySafeEvidence => Self::IntercomRelaySafeEvidence,
            InlineAssertionKind::NetworkControlPanelFaultsObserved => {
                Self::NetworkControlPanelFaultsObserved
            }
            InlineAssertionKind::ComfortTimeseriesObserved => Self::ComfortTimeseriesObserved,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DomainRuntimeConfig {
    pub edge: EdgeRuntimeConfig,
    pub wan: WanRuntimeConfig,
    pub comfort: ComfortRuntimeConfig,
    pub access: AccessRuntimeConfig,
    pub commissioning: CommissioningRuntimeConfig,
}

impl DomainRuntimeConfig {
    fn try_from_scenario(scenario: &ScenarioFile) -> Result<Self, ScenarioError> {
        Ok(Self {
            edge: EdgeRuntimeConfig {
                expected_within: expected_within(&scenario.edge)?,
            },
            wan: WanRuntimeConfig {
                expected_within: expected_within(&scenario.wan)?,
                backup_status: status_at(&scenario.wan, "backup"),
            },
            comfort: ComfortRuntimeConfig {
                target: yaml_mapping_number(&scenario.comfort, "target_discomfort_index"),
                range: comfort_range(&scenario.comfort),
                discomfort_by_target: discomfort_by_target(&scenario.sensors),
            },
            access: AccessRuntimeConfig {
                unexpected_users: unexpected_access_users(&scenario.inputs),
            },
            commissioning: CommissioningRuntimeConfig {
                check_count: commissioning_check_count(&scenario.commissioning),
                site: string_value(&scenario.commissioning, "site"),
            },
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EdgeRuntimeConfig {
    pub expected_within: Option<Duration>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WanRuntimeConfig {
    pub expected_within: Option<Duration>,
    pub backup_status: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComfortRuntimeConfig {
    pub target: Option<f64>,
    pub range: Option<(f64, f64)>,
    pub discomfort_by_target: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessRuntimeConfig {
    pub unexpected_users: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommissioningRuntimeConfig {
    pub check_count: usize,
    pub site: Option<String>,
}

fn validate_mqtt_contract_topics(scenario: &ScenarioFile) -> Result<(), ScenarioError> {
    super::validate_mqtt_contracts(&scenario.mqtt.contracts)?;
    for contract in &scenario.mqtt.contracts {
        Identifier::parse("mqtt.contracts[].name", contract.name.clone())?;
        MqttTopicTemplate::parse(
            "mqtt.contracts[].command_topic",
            contract.command_topic.clone(),
        )?;
        MqttTopicTemplate::parse("mqtt.contracts[].state_topic", contract.state_topic.clone())?;
    }
    Ok(())
}

fn validate_scene_and_alert_ids(scenario: &ScenarioFile) -> Result<(), ScenarioError> {
    for (scene_id, scene) in &scenario.scenes {
        SceneId::parse("scenes.<id>", scene_id.clone())?;
        for fixture_id in scene.fixtures.keys() {
            FixtureId::parse("scenes[].fixtures.<id>", fixture_id.clone())?;
        }
    }
    for alert in &scenario.alerts {
        if let Some(alert_id) = alert.get("id").and_then(|value| value.as_str()) {
            AlertId::parse("alerts[].id", alert_id.to_string())?;
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

fn expected_within(
    config: &BTreeMap<String, serde_yaml::Value>,
) -> Result<Option<Duration>, ScenarioError> {
    let Some(failover) = config.get("failover").and_then(|value| value.as_mapping()) else {
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

fn status_at(config: &BTreeMap<String, serde_yaml::Value>, key: &str) -> Option<String> {
    config
        .get(key)
        .and_then(|value| value.as_mapping())
        .and_then(|mapping| {
            mapping
                .get(serde_yaml::Value::String("status".to_string()))
                .and_then(|value| value.as_str())
        })
        .map(str::to_string)
}

fn yaml_mapping_number(map: &BTreeMap<String, serde_yaml::Value>, key: &str) -> Option<f64> {
    map.get(key).and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_i64().map(|value| value as f64))
    })
}

fn string_value(map: &BTreeMap<String, serde_yaml::Value>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn unexpected_access_users(inputs: &BTreeMap<String, serde_yaml::Value>) -> Vec<String> {
    let identity_users = string_sequence(inputs.get("identity_group"));
    let access_users = string_sequence(inputs.get("access_system_group"));
    access_users
        .into_iter()
        .filter(|user| !identity_users.contains(user))
        .collect()
}

fn commissioning_check_count(commissioning: &BTreeMap<String, serde_yaml::Value>) -> usize {
    let Some(rooms) = commissioning
        .get("rooms")
        .and_then(|value| value.as_sequence())
    else {
        return 0;
    };
    rooms
        .iter()
        .filter_map(|room| room.as_mapping())
        .map(|room| {
            room.get(serde_yaml::Value::String("devices".to_string()))
                .and_then(|value| value.as_sequence())
                .map_or(0, |devices| devices.len())
        })
        .sum()
}

fn string_sequence(value: Option<&serde_yaml::Value>) -> Vec<String> {
    value
        .and_then(|value| value.as_sequence())
        .map(|sequence| {
            sequence
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn comfort_range(comfort: &BTreeMap<String, serde_yaml::Value>) -> Option<(f64, f64)> {
    let range = comfort
        .get("acceptable_range")
        .and_then(|value| value.as_mapping())?;
    let min = range
        .get(serde_yaml::Value::String("min".to_string()))
        .and_then(|value| {
            value
                .as_f64()
                .or_else(|| value.as_i64().map(|value| value as f64))
        })?;
    let max = range
        .get(serde_yaml::Value::String("max".to_string()))
        .and_then(|value| {
            value
                .as_f64()
                .or_else(|| value.as_i64().map(|value| value as f64))
        })?;
    Some((min, max))
}

fn discomfort_by_target(sensors: &BTreeMap<String, serde_yaml::Value>) -> BTreeMap<String, f64> {
    sensors
        .iter()
        .filter_map(|(sensor, value)| {
            let mapping = value.as_mapping()?;
            let temperature = mapping
                .get(serde_yaml::Value::String("temperature".to_string()))
                .and_then(|value| {
                    value
                        .as_f64()
                        .or_else(|| value.as_i64().map(|value| value as f64))
                })?;
            let humidity = mapping
                .get(serde_yaml::Value::String("humidity".to_string()))
                .and_then(|value| {
                    value
                        .as_f64()
                        .or_else(|| value.as_i64().map(|value| value as f64))
                })?;
            Some((
                format!("{sensor}.discomfort_index"),
                discomfort_index(temperature, humidity),
            ))
        })
        .collect()
}

fn discomfort_index(temperature: f64, humidity: f64) -> f64 {
    0.81 * temperature + 0.01 * humidity * (0.99 * temperature - 14.3) + 46.3
}
