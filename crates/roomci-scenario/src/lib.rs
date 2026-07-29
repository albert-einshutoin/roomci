//! Scenario file format and validator for roomci.
//!
//! This crate owns the wire format declared in `examples/*.yaml`: it parses a
//! [`ScenarioFile`], validates that every step, fault, and assertion refers
//! to a known device/scene/contact, and provides time helpers used by
//! `roomci-core` to evaluate scenarios on a virtual clock.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
};

use chrono::Duration;
use roomci_device_model::DeviceModelError;
use roomci_edge::EdgeError;
use roomci_ops::OpsError;
use thiserror::Error;

const MAX_EXACT_F64_INTEGER: f64 = 9_007_199_254_740_991.0;
pub(crate) const MAX_MQTT_TOPIC_BYTES: usize = u16::MAX as usize;
const MAX_MQTT_PAYLOAD_ENUM_VALUES: usize = 128;
const MAX_MQTT_PAYLOAD_ENUM_VALUE_BYTES: usize = 16 * 1024;
const MAX_MQTT_CONTRACT_NAME_BYTES: usize = 128;
const MAX_DIAGNOSTIC_VALUE_CHARS: usize = 256;
const MAX_EVIDENCE_ID_BYTES: usize = 128;
const MAX_YAML_DOCUMENT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_ACCEPTANCE_CRITERIA: usize = 256;
const MAX_ACCEPTANCE_CRITERION_BYTES: usize = 4096;
const MAX_ACCEPTANCE_MAPPINGS: usize = 256;
const MAX_ASSERTION_REFERENCES_PER_MAPPING: usize = 256;
const MAX_ARTIFACT_REFERENCES_PER_MAPPING: usize = 16;
const MAX_REPORT_FORMATS: usize = 16;
const MAX_REPORT_FORMAT_BYTES: usize = 64;
const MAX_MAPPING_SCENARIOS: usize = 256;
const MAX_ASSERTIONS_PER_SCENARIO: usize = 4096;
const SUPPORTED_EVIDENCE_ARTIFACTS: &[&str] = &[
    "json",
    "markdown",
    "junit",
    "timeline-json",
    "timeline-ndjson",
    "observability-json",
    "github-summary",
];

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
    #[error("failed to parse scenario {path}: {reason}")]
    Parse { path: String, reason: String },
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
    #[error(
        "MQTT contract {0} field mqtt.contracts[{0}].device_id_from_topic uses unsupported strategy {1}; only placeholder:{{device_id}} is supported"
    )]
    UnsupportedMqttDeviceIdStrategy(String, String),
    #[error("invalid adapter contract: {0}")]
    InvalidAdapterContract(String),
    #[error("invalid scenario version {version}: {reason}")]
    InvalidScenarioVersion { version: String, reason: String },
    #[error("scenario contract violation: {field} {reason}")]
    ScenarioContract { field: String, reason: String },
    #[error(transparent)]
    DeviceModel(#[from] DeviceModelError),
    #[error(transparent)]
    Edge(#[from] EdgeError),
    #[error(transparent)]
    Ops(#[from] OpsError),
}

/// A publish topic matched to one configured MQTT command/state contract.
#[derive(Debug, Clone, PartialEq)]
pub struct MqttContractPublishMatch<'a> {
    pub contract: &'a MqttConnectionContract,
    pub device_id: String,
    pub state_topic: String,
}

/// Errors produced while checking a concrete MQTT publish against contracts.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MqttContractPublishError {
    #[error("topic did not match any configured MQTT contract: {topic}")]
    TopicMismatch { topic: String },
    #[error("payload missing required fields for {contract}: {fields}")]
    MissingRequiredFields { contract: String, fields: String },
    #[error("payload field {field} is invalid for {contract}: {reason}")]
    InvalidField {
        contract: String,
        field: String,
        reason: String,
    },
}

/// Match and validate a concrete MQTT publish against configured contracts.
pub fn validate_mqtt_contract_publish<'a>(
    contracts: &'a [MqttConnectionContract],
    topic: &str,
    payload: &BTreeMap<String, serde_json::Value>,
) -> Result<MqttContractPublishMatch<'a>, MqttContractPublishError> {
    let matched = match_mqtt_contracts(contracts, topic).ok_or_else(|| {
        MqttContractPublishError::TopicMismatch {
            topic: topic.to_string(),
        }
    })?;

    let missing_fields = matched
        .contract
        .payload
        .required_fields
        .iter()
        .filter(|field| !payload.contains_key(*field))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_fields.is_empty() {
        return Err(MqttContractPublishError::MissingRequiredFields {
            contract: matched.contract.name.clone(),
            fields: missing_fields.join(", "),
        });
    }

    for (field, constraint) in &matched.contract.payload.fields {
        let Some(value) = payload.get(field) else {
            continue;
        };
        validate_mqtt_payload_value(value, constraint).map_err(|reason| {
            MqttContractPublishError::InvalidField {
                contract: matched.contract.name.clone(),
                field: field.clone(),
                reason,
            }
        })?;
    }

    Ok(matched)
}

fn validate_mqtt_payload_value(
    value: &serde_json::Value,
    constraint: &MqttPayloadFieldConstraint,
) -> Result<(), String> {
    if !mqtt_payload_value_matches_type(value, constraint.field_type) {
        return Err(format!("expected {}", constraint.field_type.as_str()));
    }
    if !constraint.enum_values.is_empty()
        && !mqtt_payload_enum_contains(&constraint.enum_values, value)?
    {
        return Err(format!(
            "expected one of {} allowed enum values",
            constraint.enum_values.len()
        ));
    }
    if let Some(number) = value.as_number() {
        if let Some(minimum) = &constraint.minimum {
            let ordering = compare_json_numbers(number, minimum)
                .ok_or_else(ambiguous_numeric_comparison_reason)?;
            if ordering == Ordering::Less {
                return Err(format!("value {number} is below minimum {minimum}"));
            }
        }
        if let Some(maximum) = &constraint.maximum {
            let ordering = compare_json_numbers(number, maximum)
                .ok_or_else(ambiguous_numeric_comparison_reason)?;
            if ordering == Ordering::Greater {
                return Err(format!("value {number} is above maximum {maximum}"));
            }
        }
    }
    Ok(())
}

fn mqtt_payload_enum_contains(
    enum_values: &[serde_json::Value],
    value: &serde_json::Value,
) -> Result<bool, String> {
    let Some(number) = value.as_number() else {
        return Ok(enum_values.contains(value));
    };
    let mut has_ambiguous_numeric_candidate = false;

    for candidate in enum_values {
        if mqtt_payload_enum_values_equal(value, candidate) {
            return Ok(true);
        }
        let Some(candidate_number) = candidate.as_number() else {
            continue;
        };
        if compare_json_numbers(number, candidate_number).is_none() {
            // Reject rather than round mixed representations of large values,
            // because accepting a nearby integer would weaken the payload contract.
            has_ambiguous_numeric_candidate = true;
        }
    }

    if has_ambiguous_numeric_candidate {
        Err(ambiguous_numeric_comparison_reason())
    } else {
        Ok(false)
    }
}

fn mqtt_payload_enum_values_equal(left: &serde_json::Value, right: &serde_json::Value) -> bool {
    match (left.as_number(), right.as_number()) {
        (Some(left), Some(right)) => compare_json_numbers(left, right) == Some(Ordering::Equal),
        _ => left == right,
    }
}

fn compare_json_numbers(left: &serde_json::Number, right: &serde_json::Number) -> Option<Ordering> {
    match (json_integer_value(left), json_integer_value(right)) {
        (Some(left), Some(right)) => Some(compare_json_integers(left, right)),
        (None, None) => left.as_f64()?.partial_cmp(&right.as_f64()?),
        (Some(_), None) | (None, Some(_)) => {
            let left = left.as_f64()?;
            let right = right.as_f64()?;
            if left.abs() > MAX_EXACT_F64_INTEGER || right.abs() > MAX_EXACT_F64_INTEGER {
                None
            } else {
                left.partial_cmp(&right)
            }
        }
    }
}

fn ambiguous_numeric_comparison_reason() -> String {
    "cannot compare integer and floating-point values outside ±9007199254740991; use consistent integer notation for exact large values".to_string()
}

#[derive(Clone, Copy)]
enum JsonInteger {
    Signed(i64),
    Unsigned(u64),
}

fn json_integer_value(number: &serde_json::Number) -> Option<JsonInteger> {
    number
        .as_i64()
        .map(JsonInteger::Signed)
        .or_else(|| number.as_u64().map(JsonInteger::Unsigned))
}

fn compare_json_integers(left: JsonInteger, right: JsonInteger) -> Ordering {
    match (left, right) {
        (JsonInteger::Signed(left), JsonInteger::Signed(right)) => left.cmp(&right),
        (JsonInteger::Unsigned(left), JsonInteger::Unsigned(right)) => left.cmp(&right),
        (JsonInteger::Signed(left), JsonInteger::Unsigned(right)) => {
            if left.is_negative() {
                Ordering::Less
            } else {
                (left as u64).cmp(&right)
            }
        }
        (JsonInteger::Unsigned(left), JsonInteger::Signed(right)) => {
            if right.is_negative() {
                Ordering::Greater
            } else {
                left.cmp(&(right as u64))
            }
        }
    }
}

fn mqtt_payload_value_matches_type(
    value: &serde_json::Value,
    field_type: MqttPayloadFieldType,
) -> bool {
    match field_type {
        MqttPayloadFieldType::String => value.is_string(),
        MqttPayloadFieldType::Integer => value.is_i64() || value.is_u64(),
        MqttPayloadFieldType::Number => value.is_number(),
        MqttPayloadFieldType::Boolean => value.is_boolean(),
        MqttPayloadFieldType::Object => value.is_object(),
        MqttPayloadFieldType::Array => value.is_array(),
    }
}

/// Return the first MQTT contract whose command topic matches the publish topic.
pub fn match_mqtt_contracts<'a>(
    contracts: &'a [MqttConnectionContract],
    topic: &str,
) -> Option<MqttContractPublishMatch<'a>> {
    contracts
        .iter()
        .find_map(|contract| match_mqtt_contract(contract, topic))
}

/// Match one MQTT contract against a concrete command topic.
pub fn match_mqtt_contract<'a>(
    contract: &'a MqttConnectionContract,
    topic: &str,
) -> Option<MqttContractPublishMatch<'a>> {
    if !mqtt_runtime_topic_is_valid(topic) {
        return None;
    }
    let device_id = extract_mqtt_placeholder_value(&contract.command_topic, topic, "{device_id}")?;
    let state_topic = contract.state_topic.replace("{device_id}", &device_id);
    if !mqtt_runtime_topic_is_valid(&state_topic) {
        return None;
    }
    Some(MqttContractPublishMatch {
        contract,
        device_id,
        state_topic,
    })
}

/// Extract a single path-segment placeholder value from a concrete topic.
pub fn extract_mqtt_placeholder_value(
    template: &str,
    value: &str,
    placeholder: &str,
) -> Option<String> {
    let (prefix, suffix) = template.split_once(placeholder)?;
    let rest = value.strip_prefix(prefix)?;
    let extracted = rest.strip_suffix(suffix)?;
    if extracted.is_empty()
        || extracted.contains('/')
        || extracted.chars().any(char::is_control)
        || extracted.chars().any(char::is_whitespace)
    {
        return None;
    }
    Some(extracted.to_string())
}

fn mqtt_runtime_topic_is_valid(topic: &str) -> bool {
    !topic.is_empty()
        && topic.len() <= MAX_MQTT_TOPIC_BYTES
        && !topic.chars().any(char::is_control)
        && !topic.chars().any(char::is_whitespace)
        && !topic.contains('#')
        && !topic.contains('+')
}

/// Read a scenario YAML file from disk and deserialize it.
///
/// Wraps I/O and YAML parsing errors with the file path so the caller gets
/// actionable error messages.
pub fn load_scenario(path: impl AsRef<Path>) -> Result<ScenarioFile, ScenarioError> {
    let path_ref = path.as_ref();
    let path_display = sanitize_diagnostic_value(&path_ref.display().to_string());
    let contents = read_bounded_yaml(path_ref, &path_display)?;
    serde_yaml::from_str(&contents).map_err(|source| ScenarioError::Parse {
        path: path_display,
        reason: sanitize_diagnostic_value(&source.to_string()),
    })
}

/// Read a company adapter contract YAML file from disk and deserialize it.
pub fn load_adapter_contract(path: impl AsRef<Path>) -> Result<AdapterContract, ScenarioError> {
    let path_ref = path.as_ref();
    let path_display = sanitize_diagnostic_value(&path_ref.display().to_string());
    let contents = read_bounded_yaml(path_ref, &path_display)?;
    serde_yaml::from_str(&contents).map_err(|source| ScenarioError::Parse {
        path: path_display,
        reason: sanitize_diagnostic_value(&source.to_string()),
    })
}

fn read_bounded_yaml(path: &Path, path_display: &str) -> Result<String, ScenarioError> {
    let file = File::open(path).map_err(|source| ScenarioError::Read {
        path: path_display.to_string(),
        source,
    })?;
    let mut contents = String::new();
    file.take(MAX_YAML_DOCUMENT_BYTES + 1)
        .read_to_string(&mut contents)
        .map_err(|source| ScenarioError::Read {
            path: path_display.to_string(),
            source,
        })?;
    if contents.len() as u64 > MAX_YAML_DOCUMENT_BYTES {
        return Err(ScenarioError::Read {
            path: path_display.to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("YAML document exceeds {MAX_YAML_DOCUMENT_BYTES} bytes"),
            ),
        });
    }
    Ok(contents)
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
    validate_acceptance_mappings(&contract.acceptance)?;

    Ok(())
}

fn validate_acceptance_mappings(acceptance: &AdapterAcceptance) -> Result<(), ScenarioError> {
    if acceptance.criteria.len() > MAX_ACCEPTANCE_CRITERIA {
        return Err(ScenarioError::InvalidAdapterContract(format!(
            "acceptance.criteria must contain at most {MAX_ACCEPTANCE_CRITERIA} entries"
        )));
    }
    if acceptance.mappings.len() > MAX_ACCEPTANCE_MAPPINGS {
        return Err(ScenarioError::InvalidAdapterContract(format!(
            "acceptance.mappings must contain at most {MAX_ACCEPTANCE_MAPPINGS} entries"
        )));
    }
    if acceptance.report_formats.len() > MAX_REPORT_FORMATS {
        return Err(ScenarioError::InvalidAdapterContract(format!(
            "acceptance.report_formats must contain at most {MAX_REPORT_FORMATS} entries"
        )));
    }
    let mut report_formats = BTreeSet::new();
    for (index, report_format) in acceptance.report_formats.iter().enumerate() {
        if report_format.is_empty()
            || report_format.len() > MAX_REPORT_FORMAT_BYTES
            || report_format.chars().any(char::is_control)
        {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "acceptance.report_formats[{index}] must be non-empty, control-free, and at most {MAX_REPORT_FORMAT_BYTES} bytes"
            )));
        }
        if !report_formats.insert(report_format.as_str()) {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "acceptance.report_formats[{index}] duplicates report format"
            )));
        }
    }
    if !acceptance.mappings.is_empty() {
        let mut criteria = BTreeSet::new();
        for (index, criterion) in acceptance.criteria.iter().enumerate() {
            if criterion.trim().is_empty()
                || criterion.len() > MAX_ACCEPTANCE_CRITERION_BYTES
                || criterion.chars().any(char::is_control)
            {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "acceptance.criteria[{index}] must be non-empty, control-free, and at most {MAX_ACCEPTANCE_CRITERION_BYTES} bytes when mappings are declared"
                )));
            }
            if !criteria.insert(criterion.as_str()) {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "acceptance.criteria[{index}] duplicates acceptance criterion"
                )));
            }
        }
    }
    let mut mapping_ids = BTreeSet::new();
    let mut mapped_criteria = BTreeSet::new();
    for (index, mapping) in acceptance.mappings.iter().enumerate() {
        let path = format!("acceptance.mappings[{index}]");
        validate_evidence_id(&format!("{path}.id"), &mapping.id)?;
        if !mapping_ids.insert(mapping.id.as_str()) {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "{path}.id duplicates acceptance mapping id {}",
                mapping.id
            )));
        }
        if !acceptance.criteria.contains(&mapping.criterion) {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "{path}.criterion does not match any acceptance.criteria entry"
            )));
        }
        if !mapped_criteria.insert(mapping.criterion.as_str()) {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "{path}.criterion duplicates a mapped acceptance criterion; combine its evidence references under one stable id"
            )));
        }
        if mapping.assertions.is_empty() && mapping.artifacts.is_empty() {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "{path} must reference at least one assertion or artifact"
            )));
        }
        if mapping.assertions.len() > MAX_ASSERTION_REFERENCES_PER_MAPPING {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "{path}.assertions must contain at most {MAX_ASSERTION_REFERENCES_PER_MAPPING} entries"
            )));
        }
        if mapping.artifacts.len() > MAX_ARTIFACT_REFERENCES_PER_MAPPING {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "{path}.artifacts must contain at most {MAX_ARTIFACT_REFERENCES_PER_MAPPING} entries"
            )));
        }

        let mut assertion_refs = BTreeSet::new();
        for (reference_index, reference) in mapping.assertions.iter().enumerate() {
            let reference_path = format!("{path}.assertions[{reference_index}]");
            if reference.scenario.len() > MAX_EVIDENCE_ID_BYTES
                || validate_scenario_name(&reference.scenario).is_err()
            {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "{reference_path}.scenario must be at most {MAX_EVIDENCE_ID_BYTES} bytes and match ^[a-z0-9][a-z0-9_]*$"
                )));
            }
            validate_evidence_id(&format!("{reference_path}.assertion"), &reference.assertion)?;
            if !assertion_refs.insert(reference) {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "{reference_path} duplicates assertion reference {}:{}",
                    reference.scenario, reference.assertion
                )));
            }
        }

        let mut artifacts = BTreeSet::new();
        for (artifact_index, artifact) in mapping.artifacts.iter().enumerate() {
            let artifact_path = format!("{path}.artifacts[{artifact_index}]");
            if !SUPPORTED_EVIDENCE_ARTIFACTS.contains(&artifact.as_str()) {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "{artifact_path} uses unsupported evidence artifact {}",
                    sanitize_diagnostic_value(artifact)
                )));
            }
            if !report_formats.contains(artifact.as_str()) {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "{artifact_path} references {}, which is not declared in acceptance.report_formats",
                    sanitize_diagnostic_value(artifact)
                )));
            }
            if !artifacts.insert(artifact.as_str()) {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "{artifact_path} duplicates evidence artifact {artifact}"
                )));
            }
        }
    }
    Ok(())
}

fn validate_evidence_id(field: &str, value: &str) -> Result<(), ScenarioError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_EVIDENCE_ID_BYTES
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || (index > 0 && matches!(byte, b'_' | b'-'))
        });
    if valid {
        Ok(())
    } else {
        Err(ScenarioError::InvalidAdapterContract(format!(
            "{field} must be at most {MAX_EVIDENCE_ID_BYTES} bytes and match ^[a-z0-9][a-z0-9_-]*$"
        )))
    }
}

/// Cross-check adapter evidence references against explicitly supplied
/// scenarios. This does not claim artifacts already exist; it only proves the
/// referenced assertion IDs and declared artifact kinds are resolvable.
pub fn validate_adapter_scenario_mapping(
    contract: &AdapterContract,
    scenarios: &[ScenarioFile],
) -> Result<(), ScenarioError> {
    validate_adapter_contract(contract)?;
    if scenarios.len() > MAX_MAPPING_SCENARIOS {
        return Err(ScenarioError::InvalidAdapterContract(format!(
            "scenario inputs must contain at most {MAX_MAPPING_SCENARIOS} entries"
        )));
    }

    let mut assertion_index = BTreeMap::<&str, BTreeSet<&str>>::new();
    for scenario in scenarios {
        validate_assertion_count(&scenario.assertions)?;
        validate_scenario(scenario)?;
        let names = scenario
            .assertions
            .iter()
            .filter_map(|assertion| assertion.name.as_deref())
            .collect::<BTreeSet<_>>();
        if assertion_index
            .insert(scenario.scenario.name.as_str(), names)
            .is_some()
        {
            return Err(ScenarioError::InvalidAdapterContract(format!(
                "scenario input duplicates scenario.name {}",
                sanitize_diagnostic_value(&scenario.scenario.name)
            )));
        }
    }

    for (mapping_index, mapping) in contract.acceptance.mappings.iter().enumerate() {
        for (reference_index, reference) in mapping.assertions.iter().enumerate() {
            let path =
                format!("acceptance.mappings[{mapping_index}].assertions[{reference_index}]");
            let assertions = assertion_index
                .get(reference.scenario.as_str())
                .ok_or_else(|| {
                    ScenarioError::InvalidAdapterContract(format!(
                        "{path}.scenario references missing supplied scenario {}",
                        reference.scenario
                    ))
                })?;
            if !assertions.contains(reference.assertion.as_str()) {
                return Err(ScenarioError::InvalidAdapterContract(format!(
                    "{path}.assertion references missing named assertion {} in scenario {}",
                    reference.assertion, reference.scenario
                )));
            }
        }
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

pub(crate) fn validate_assertion_names(
    assertions: &[AssertionDefinition],
) -> Result<(), ScenarioError> {
    let mut assertion_names = BTreeSet::new();
    for (index, assertion) in assertions.iter().enumerate() {
        let Some(name) = assertion.name.as_deref() else {
            continue;
        };
        if validate_evidence_id(&format!("assertions[{index}].name"), name).is_err() {
            return Err(ScenarioError::ScenarioContract {
                field: format!("assertions[{index}].name"),
                reason: format!(
                    "must be at most {MAX_EVIDENCE_ID_BYTES} bytes and match ^[a-z0-9][a-z0-9_-]*$"
                ),
            });
        }
        if !assertion_names.insert(name) {
            return Err(ScenarioError::ScenarioContract {
                field: format!("assertions[{index}].name"),
                reason: format!("duplicates named assertion {name}"),
            });
        }
    }
    Ok(())
}

pub(crate) fn validate_assertion_count(
    assertions: &[AssertionDefinition],
) -> Result<(), ScenarioError> {
    if assertions.len() > MAX_ASSERTIONS_PER_SCENARIO {
        return Err(ScenarioError::ScenarioContract {
            field: "assertions".to_string(),
            reason: format!("must contain at most {MAX_ASSERTIONS_PER_SCENARIO} items"),
        });
    }
    Ok(())
}

fn validate_scenario_name(name: &str) -> Result<(), ScenarioError> {
    if name.is_empty() {
        return Err(ScenarioError::ScenarioContract {
            field: "scenario.name".to_string(),
            reason: "must not be empty".to_string(),
        });
    }

    let mut chars = name.chars();
    let first = chars
        .next()
        .ok_or_else(|| ScenarioError::ScenarioContract {
            field: "scenario.name".to_string(),
            reason: "must match ^[a-z0-9][a-z0-9_]*$".to_string(),
        })?;

    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err(ScenarioError::ScenarioContract {
            field: "scenario.name".to_string(),
            reason: "must match ^[a-z0-9][a-z0-9_]*$".to_string(),
        });
    }

    for c in chars {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(ScenarioError::ScenarioContract {
                field: "scenario.name".to_string(),
                reason: "must match ^[a-z0-9][a-z0-9_]*$".to_string(),
            });
        }
    }

    Ok(())
}

fn validate_scenario_version(version: &str) -> Result<(), ScenarioError> {
    if version.trim() != version {
        return Err(ScenarioError::InvalidScenarioVersion {
            version: version.to_string(),
            reason: "must not contain leading or trailing whitespace".to_string(),
        });
    }

    let normalized = if let Some(rest) = version.strip_prefix('v') {
        rest
    } else if let Some(rest) = version.strip_prefix('V') {
        rest
    } else {
        version
    };

    if normalized.is_empty() {
        return Err(ScenarioError::InvalidScenarioVersion {
            version: version.to_string(),
            reason: "must contain a major version".to_string(),
        });
    }

    let parts = normalized.split('.').collect::<Vec<_>>();
    if !(1..=3).contains(&parts.len()) {
        return Err(ScenarioError::InvalidScenarioVersion {
            version: version.to_string(),
            reason: "must use <major>.<minor>[.<patch>] format".to_string(),
        });
    }

    for part in &parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return Err(ScenarioError::InvalidScenarioVersion {
                version: version.to_string(),
                reason: "must contain only numeric semver components".to_string(),
            });
        }
        if part.len() > 1 && part.starts_with('0') {
            return Err(ScenarioError::InvalidScenarioVersion {
                version: version.to_string(),
                reason: "must not use leading zero in numeric components".to_string(),
            });
        }
    }

    let major = parts[0]
        .parse::<u64>()
        .map_err(|_| ScenarioError::InvalidScenarioVersion {
            version: version.to_string(),
            reason: "major version must fit u64".to_string(),
        })?;
    let minor = if parts.len() >= 2 {
        parts[1]
            .parse::<u64>()
            .map_err(|_| ScenarioError::InvalidScenarioVersion {
                version: version.to_string(),
                reason: "minor version must fit u64".to_string(),
            })?
    } else {
        0
    };

    let supported = matches!((major, minor), (0, 1));
    if !supported {
        return Err(ScenarioError::InvalidScenarioVersion {
            version: version.to_string(),
            reason: "unsupported scenario version. Supported versions: 0.1".to_string(),
        });
    }

    Ok(())
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
        validate_mqtt_contract_name(&contract.name)?;
        let command_topic_path = format!("mqtt.contracts[{}].command_topic", contract.name);
        let state_topic_path = format!("mqtt.contracts[{}].state_topic", contract.name);
        if contract.adapter != "mqtt_v3_qos0_subset" {
            return Err(ScenarioError::UnsupportedMqttAdapter(
                sanitize_diagnostic_value(&contract.adapter),
            ));
        }
        MqttTopic::parse(&command_topic_path, contract.command_topic.clone())?;
        MqttTopic::parse(&state_topic_path, contract.state_topic.clone())?;
        if contract.device_id_from_topic != "placeholder:{device_id}" {
            return Err(ScenarioError::UnsupportedMqttDeviceIdStrategy(
                contract.name.clone(),
                sanitize_diagnostic_value(&contract.device_id_from_topic),
            ));
        }
        let command_placeholders =
            mqtt_topic_placeholders(&command_topic_path, &contract.command_topic)?;
        let state_placeholders = mqtt_topic_placeholders(&state_topic_path, &contract.state_topic)?;
        for (path, value, placeholders) in [
            (
                command_topic_path.as_str(),
                contract.command_topic.as_str(),
                &command_placeholders,
            ),
            (
                state_topic_path.as_str(),
                contract.state_topic.as_str(),
                &state_placeholders,
            ),
        ] {
            if placeholders
                .iter()
                .filter(|placeholder| placeholder.as_str() == "{device_id}")
                .count()
                != 1
            {
                return invalid_mqtt_contract_topic(
                    path,
                    value,
                    "must include exactly one {device_id} placeholder".to_string(),
                );
            }
        }
        let command_placeholder_set = command_placeholders.iter().collect::<BTreeSet<_>>();
        let state_placeholder_set = state_placeholders.iter().collect::<BTreeSet<_>>();
        if command_placeholder_set != state_placeholder_set {
            return invalid_mqtt_contract_topic(
                &state_topic_path,
                &contract.state_topic,
                format!(
                    "placeholders must match command_topic; command_topic has [{}], state_topic has [{}]",
                    command_placeholders.join(", "),
                    state_placeholders.join(", ")
                ),
            );
        }
        for (path, value, placeholders) in [
            (
                command_topic_path.as_str(),
                contract.command_topic.as_str(),
                &command_placeholders,
            ),
            (
                state_topic_path.as_str(),
                contract.state_topic.as_str(),
                &state_placeholders,
            ),
        ] {
            if let Some(unsupported) = placeholders
                .iter()
                .find(|placeholder| placeholder.as_str() != "{device_id}")
            {
                return invalid_mqtt_contract_topic(
                    path,
                    value,
                    format!("only the {{device_id}} placeholder is supported; found {unsupported}"),
                );
            }
        }
        if let Some(existing) =
            command_topics.insert(contract.command_topic.clone(), contract.name.clone())
        {
            return Err(ScenarioError::AmbiguousMqttMapping(format!(
                "{} used by {} and {}",
                sanitize_diagnostic_value(&contract.command_topic),
                existing,
                contract.name
            )));
        }
        validate_mqtt_payload_expectation(contract)?;
    }
    Ok(())
}

fn validate_mqtt_contract_name(name: &str) -> Result<(), ScenarioError> {
    if name.trim().is_empty()
        || name.len() > MAX_MQTT_CONTRACT_NAME_BYTES
        || name.chars().any(char::is_whitespace)
        || name.chars().any(char::is_control)
        || name.contains('[')
        || name.contains(']')
    {
        return Err(ScenarioError::InvalidIdentifier {
            field: "mqtt.contracts[].name".to_string(),
            value: sanitize_diagnostic_value(name),
        });
    }
    Ok(())
}

fn mqtt_topic_placeholders(field: &str, value: &str) -> Result<Vec<String>, ScenarioError> {
    let mut placeholders = Vec::new();
    let mut opening = None;

    for (index, character) in value.char_indices() {
        match (character, opening) {
            ('{', None) => opening = Some(index),
            ('{', Some(_)) => {
                return invalid_mqtt_contract_topic(
                    field,
                    value,
                    "contains a nested or unclosed placeholder".to_string(),
                );
            }
            ('}', Some(start)) => {
                if index == start + 1 {
                    return invalid_mqtt_contract_topic(
                        field,
                        value,
                        "contains an empty placeholder".to_string(),
                    );
                }
                placeholders.push(value[start..=index].to_string());
                opening = None;
            }
            ('}', None) => {
                return invalid_mqtt_contract_topic(
                    field,
                    value,
                    "contains a closing brace without an opening brace".to_string(),
                );
            }
            _ => {}
        }
    }
    if opening.is_some() {
        return invalid_mqtt_contract_topic(
            field,
            value,
            "contains an unclosed placeholder".to_string(),
        );
    }
    Ok(placeholders)
}

fn invalid_mqtt_contract_topic<T>(
    field: &str,
    value: &str,
    reason: String,
) -> Result<T, ScenarioError> {
    Err(ScenarioError::InvalidMqttTopic {
        field: field.to_string(),
        value: sanitize_diagnostic_value(value),
        reason: sanitize_diagnostic_value(&reason),
    })
}

/// Escape and bound untrusted text before embedding it in CLI diagnostics.
pub fn sanitize_diagnostic_value(value: &str) -> String {
    let mut sanitized = String::new();
    let mut characters = value.chars();
    for character in characters.by_ref().take(MAX_DIAGNOSTIC_VALUE_CHARS) {
        if diagnostic_character_requires_escape(character) {
            sanitized.extend(character.escape_default());
        } else {
            sanitized.push(character);
        }
    }
    if characters.next().is_some() {
        sanitized.push('…');
    }
    sanitized
}

fn diagnostic_character_requires_escape(character: char) -> bool {
    character.is_control()
        || (character.is_whitespace() && character != ' ')
        || matches!(
            character,
            '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

fn validate_mqtt_payload_expectation(
    contract: &MqttConnectionContract,
) -> Result<(), ScenarioError> {
    let path = format!("mqtt.contracts[{}].payload", contract.name);
    let required =
        validate_payload_field_names(&path, "required_fields", &contract.payload.required_fields)?;
    let optional =
        validate_payload_field_names(&path, "optional_fields", &contract.payload.optional_fields)?;

    if let Some(field) = required.intersection(&optional).next() {
        return invalid_adapter_contract(format!(
            "{path}.fields.{field} cannot be both required and optional"
        ));
    }

    for optional_field in &optional {
        if !contract.payload.fields.contains_key(optional_field) {
            return invalid_adapter_contract(format!(
                "{path}.optional_fields contains {optional_field}, but {path}.fields.{optional_field} is not declared"
            ));
        }
    }

    for (field, constraint) in &contract.payload.fields {
        if field.trim().is_empty() {
            return invalid_adapter_contract(format!("{path}.fields contains an empty field name"));
        }
        if field.chars().any(char::is_control) {
            return invalid_adapter_contract(format!(
                "{path}.fields field names must not contain control characters"
            ));
        }
        if !required.contains(field) && !optional.contains(field) {
            return invalid_adapter_contract(format!(
                "{path}.fields.{field} must be listed in required_fields or optional_fields"
            ));
        }
        validate_mqtt_payload_field_constraint(&path, field, constraint)?;
    }
    Ok(())
}

fn validate_payload_field_names(
    path: &str,
    list_name: &str,
    fields: &[String],
) -> Result<BTreeSet<String>, ScenarioError> {
    let mut unique = BTreeSet::new();
    for field in fields {
        if field.trim().is_empty() {
            return invalid_adapter_contract(format!(
                "{path}.{list_name} contains an empty field name"
            ));
        }
        if field.chars().any(char::is_control) {
            return invalid_adapter_contract(format!(
                "{path}.{list_name} field names must not contain control characters"
            ));
        }
        if !unique.insert(field.clone()) {
            return invalid_adapter_contract(format!(
                "{path}.{list_name} contains duplicate field {field}"
            ));
        }
    }
    Ok(unique)
}

fn validate_mqtt_payload_field_constraint(
    path: &str,
    field: &str,
    constraint: &MqttPayloadFieldConstraint,
) -> Result<(), ScenarioError> {
    let field_path = format!("{path}.fields.{field}");
    for (bound_name, bound) in [
        ("minimum", constraint.minimum.as_ref()),
        ("maximum", constraint.maximum.as_ref()),
    ] {
        if let Some(bound) = bound {
            if !constraint.field_type.is_numeric() {
                return invalid_adapter_contract(format!(
                    "{field_path}.{bound_name} is only supported for integer or number fields"
                ));
            }
            if constraint.field_type == MqttPayloadFieldType::Integer
                && json_integer_value(bound).is_none()
            {
                return invalid_adapter_contract(format!(
                    "{field_path}.{bound_name} must be an integer"
                ));
            }
            // Integer JSON numbers remain exact across the full i64/u64 range.
            // Floating-point bounds cannot distinguish adjacent integers above
            // 2^53, so reject only that ambiguous notation and tell contract
            // authors to use an integer literal for larger exact boundaries.
            if json_integer_value(bound).is_none()
                && bound
                    .as_f64()
                    .is_some_and(|value| value.abs() > MAX_EXACT_F64_INTEGER)
            {
                return invalid_adapter_contract(format!(
                    "{field_path}.{bound_name} floating-point bounds must be within ±9007199254740991; use an integer literal for larger exact bounds"
                ));
            }
        }
    }
    if let (Some(minimum), Some(maximum)) = (&constraint.minimum, &constraint.maximum) {
        match compare_json_numbers(minimum, maximum) {
            Some(Ordering::Greater) => {
                return invalid_adapter_contract(format!(
                    "{field_path}.minimum must be less than or equal to maximum"
                ));
            }
            None => {
                return invalid_adapter_contract(format!(
                    "{field_path} {}",
                    ambiguous_numeric_comparison_reason()
                ));
            }
            _ => {}
        }
    }

    if constraint.enum_values.len() > MAX_MQTT_PAYLOAD_ENUM_VALUES {
        return invalid_adapter_contract(format!(
            "{field_path}.enum must contain at most {MAX_MQTT_PAYLOAD_ENUM_VALUES} values"
        ));
    }
    let mut enum_value_bytes = 0_usize;
    for value in &constraint.enum_values {
        if !mqtt_payload_value_matches_type(value, constraint.field_type) {
            return invalid_adapter_contract(format!(
                "{field_path}.enum value {value} does not match declared type {}",
                constraint.field_type.as_str()
            ));
        }
        let Some(value_bytes) = mqtt_payload_enum_scalar_size(value) else {
            return invalid_adapter_contract(format!(
                "{field_path}.enum is only supported for scalar string, number, or boolean fields"
            ));
        };
        enum_value_bytes = enum_value_bytes
            .checked_add(value_bytes)
            .filter(|size| *size <= MAX_MQTT_PAYLOAD_ENUM_VALUE_BYTES)
            .ok_or_else(|| {
                ScenarioError::InvalidAdapterContract(format!(
                    "{field_path}.enum values must use at most {MAX_MQTT_PAYLOAD_ENUM_VALUE_BYTES} bytes"
                ))
            })?;
    }
    for (index, value) in constraint.enum_values.iter().enumerate() {
        // Declaration-time duplicate detection must use the same equality as
        // runtime matching so the documented enum set has one stable meaning.
        if constraint.enum_values[..index]
            .iter()
            .any(|candidate| mqtt_payload_enum_values_equal(candidate, value))
        {
            return invalid_adapter_contract(format!(
                "{field_path}.enum contains duplicate value {value}"
            ));
        }
        if let Some(number) = value.as_number() {
            for bound in [constraint.minimum.as_ref(), constraint.maximum.as_ref()]
                .into_iter()
                .flatten()
            {
                if compare_json_numbers(number, bound).is_none() {
                    return invalid_adapter_contract(format!(
                        "{field_path}.enum value {value} {}",
                        ambiguous_numeric_comparison_reason()
                    ));
                }
            }
            if constraint.minimum.as_ref().is_some_and(|minimum| {
                compare_json_numbers(number, minimum) == Some(Ordering::Less)
            }) || constraint.maximum.as_ref().is_some_and(|maximum| {
                compare_json_numbers(number, maximum) == Some(Ordering::Greater)
            }) {
                return invalid_adapter_contract(format!(
                    "{field_path}.enum value {value} is outside the declared range"
                ));
            }
        }
    }
    Ok(())
}

fn mqtt_payload_enum_scalar_size(value: &serde_json::Value) -> Option<usize> {
    match value {
        serde_json::Value::String(value) => Some(value.len()),
        serde_json::Value::Number(value) => Some(value.to_string().len()),
        serde_json::Value::Bool(_) => Some(1),
        serde_json::Value::Null | serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            None
        }
    }
}

fn invalid_adapter_contract<T>(message: String) -> Result<T, ScenarioError> {
    Err(ScenarioError::InvalidAdapterContract(message))
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
        "s" => Duration::try_seconds(amount)
            .ok_or_else(|| ScenarioError::InvalidDuration(value.to_string())),
        "m" => Duration::try_minutes(amount)
            .ok_or_else(|| ScenarioError::InvalidDuration(value.to_string())),
        "h" => Duration::try_hours(amount)
            .ok_or_else(|| ScenarioError::InvalidDuration(value.to_string())),
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
