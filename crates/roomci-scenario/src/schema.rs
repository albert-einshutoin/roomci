use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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

/// Top-level company adapter contract document.
///
/// Adapter contracts capture customer-specific protocol details separately
/// from the core emulator. They are intentionally stricter than arbitrary YAML
/// so an evaluator gets actionable feedback before wiring a private spec into
/// scenarios.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AdapterContract {
    pub version: String,
    pub adapter: AdapterMetadata,
    #[serde(default)]
    pub site: AdapterSite,
    #[serde(default)]
    pub devices: Vec<AdapterDevice>,
    #[serde(default)]
    pub mqtt: AdapterMqtt,
    #[serde(default)]
    pub modbus: AdapterModbus,
    #[serde(default)]
    pub bms: AdapterBms,
    #[serde(default)]
    pub edge: AdapterEdge,
    #[serde(default)]
    pub auth: BTreeMap<String, serde_yaml::Value>,
    pub acceptance: AdapterAcceptance,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AdapterMetadata {
    pub name: String,
    #[serde(default)]
    pub domain_pack: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct AdapterSite {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub rooms: Vec<BTreeMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AdapterDevice {
    pub id: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct AdapterMqtt {
    #[serde(default)]
    pub contracts: Vec<MqttConnectionContract>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct AdapterModbus {
    #[serde(default)]
    pub devices: Vec<AdapterModbusDevice>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AdapterModbusDevice {
    pub id: String,
    #[serde(default)]
    pub unit_id: Option<u8>,
    #[serde(default)]
    pub registers: Vec<AdapterModbusRegister>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AdapterModbusRegister {
    pub address: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub register_type: String,
    pub access: String,
    #[serde(default)]
    pub scale: Option<f64>,
    #[serde(default)]
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct AdapterBms {
    #[serde(default)]
    pub alerts: Vec<AdapterBmsAlert>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AdapterBmsAlert {
    pub id: String,
    pub source: String,
    pub severity: String,
    #[serde(default)]
    pub channels: Vec<String>,
    #[serde(default)]
    pub ticket_lifecycle: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct AdapterEdge {
    #[serde(default)]
    pub commands: Vec<AdapterEdgeCommand>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AdapterEdgeCommand {
    pub name: String,
    pub source: String,
    pub target: String,
    pub expected_state: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct AdapterAcceptance {
    #[serde(default)]
    pub criteria: Vec<String>,
    #[serde(default)]
    pub report_formats: Vec<String>,
}
