use std::{collections::BTreeSet, fs, path::PathBuf};

use super::*;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn read_scenario_schema(path: &str) -> serde_json::Value {
    let schema_path = fixture(path);
    let schema_text = fs::read_to_string(schema_path).unwrap();
    serde_json::from_str(&schema_text).unwrap()
}

fn schema_required_fields(schema: &serde_json::Value) -> BTreeSet<String> {
    schema["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap().to_string())
        .collect()
}

fn schema_properties(schema: &serde_json::Value) -> BTreeSet<String> {
    schema["properties"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect()
}

#[test]
fn tracks_scenario_schema_drift_with_runtime_contract() {
    let roomci_schema = read_scenario_schema("schemas/scenario.schema.json");
    let vscode_schema = read_scenario_schema("tools/vscode-roomci/schemas/scenario.schema.json");

    assert_eq!(roomci_schema, vscode_schema);

    let expected_required_fields = BTreeSet::from([
        "version".to_string(),
        "scenario".to_string(),
        "assertions".to_string(),
    ]);
    assert_eq!(
        schema_required_fields(&roomci_schema),
        expected_required_fields
    );

    let expected_properties = BTreeSet::from([
        "version".to_string(),
        "scenario".to_string(),
        "environment".to_string(),
        "network".to_string(),
        "wan".to_string(),
        "sensors".to_string(),
        "comfort".to_string(),
        "inputs".to_string(),
        "commissioning".to_string(),
        "future_milestone".to_string(),
        "edge".to_string(),
        "mqtt".to_string(),
        "devices".to_string(),
        "modbus".to_string(),
        "lighting".to_string(),
        "scenes".to_string(),
        "contacts".to_string(),
        "alerts".to_string(),
        "faults".to_string(),
        "steps".to_string(),
        "assertions".to_string(),
        "report".to_string(),
    ])
    .into_iter()
    .collect();
    assert_eq!(schema_properties(&roomci_schema), expected_properties);

    assert_eq!(
        roomci_schema["properties"]["assertions"]["minItems"],
        serde_json::json!(1)
    );
    assert_eq!(
        roomci_schema["properties"]["version"]["pattern"],
        "^v?0\\.1(?:\\.(?:0|[1-9]\\d*))?$"
    );
    assert_eq!(
        roomci_schema["properties"]["scenario"]["required"][0],
        "name"
    );
    assert_eq!(
        roomci_schema["properties"]["scenario"]["properties"]["name"]["pattern"],
        "^[a-z0-9][a-z0-9_]*$"
    );
}

#[test]
fn rejects_invalid_scenario_version_string() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "banana"
scenario:
  name: invalid_scenario_version
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::InvalidScenarioVersion {
            version,
            ..
        } if version == "banana"
    ));
}

#[test]
fn rejects_unsupported_scenario_version_value() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.2"
scenario:
  name: unsupported_scenario_version
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::InvalidScenarioVersion {
            version,
            ..
        } if version == "0.2"
    ));
}

#[test]
fn accepts_supported_scenario_version_prefix() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "v0.1"
scenario:
  name: supported_scenario_version_prefix
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
fn rejects_empty_assertions_array() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: no_assertions
assertions: []
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::ScenarioContract {
            field,
            ..
        } if field == "assertions"
    ));
}

#[test]
fn rejects_invalid_scenario_name_pattern() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: UpperCamel
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::ScenarioContract {
            field,
            ..
        } if field == "scenario.name"
    ));
}

#[test]
fn validates_latest_local_first_scenario() {
    let scenario = load_scenario(fixture("examples/local_first_cloud_outage.yaml")).unwrap();

    validate_scenario(&scenario).unwrap();
    let validated = ValidatedScenario::try_from(&scenario).unwrap();

    assert_eq!(scenario.scenario.name, "local_first_cloud_outage");
    assert!(scenario.scenario.tags.contains(&"local-first".to_string()));
    assert!(!validated.scheduled_events().is_empty());
    assert!(validated
        .devices()
        .iter()
        .any(|device| device.id.as_str() == "living_light"));
}

#[test]
fn resolves_symbolic_time_without_clock() {
    assert_eq!(resolve_time_offset("T+15s").unwrap(), Duration::seconds(15));
    assert_eq!(resolve_time_offset("T").unwrap(), Duration::zero());
}

#[test]
fn rejects_invalid_scenario_version_in_validation() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: banana
scenario:
  name: invalid_scenario_version
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let error = ValidatedScenario::try_from(&scenario).unwrap_err();
    assert!(matches!(
        error,
        ScenarioError::InvalidScenarioVersion { .. }
    ));
}

#[test]
fn rejects_unsupported_scenario_version_in_validation() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "1.0"
scenario:
  name: unsupported_scenario_version
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let error = ValidatedScenario::try_from(&scenario).unwrap_err();
    assert!(matches!(
        error,
        ScenarioError::InvalidScenarioVersion { .. }
    ));
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
fn rejects_unknown_target_condition_assertion() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: invalid_target_condition_assertion
steps:
  - at: T
    event: no-op
assertions:
  - at: T+1s
    target: imaginary.subsystem
    condition: active
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(error, ScenarioError::InvalidAssertionKind));
}

#[test]
fn rejects_empty_promoted_identifier_at_validated_boundary() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: invalid_empty_device_id
devices:
  - id: ""
    type: light
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

    assert!(matches!(
        error,
        ScenarioError::InvalidIdentifier { field, .. } if field == "devices[].id"
    ));
}

#[test]
fn rejects_invalid_mqtt_publish_topic_at_validated_boundary() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: invalid_mqtt_topic
devices:
  - id: living_light
    type: light
    protocol: dali
steps:
  - at: T
    mqtt_publish:
      client: ipad_controller
      topic: house/+/device/living_light/command
      payload:
        power: true
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::InvalidMqttTopic { field, .. } if field == "steps[].mqtt_publish.topic"
    ));
}

#[test]
fn rejects_malformed_between_condition_at_validated_boundary() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: invalid_between_condition
steps:
  - at: T
    event: no-op
assertions:
  - at: T+1s
    target: living_area.discomfort_index
    condition: between warm and 75
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(error, ScenarioError::InvalidAssertionKind));
}

#[test]
fn rejects_non_finite_between_condition_at_validated_boundary() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: invalid_non_finite_between_condition
steps:
  - at: T
    event: no-op
assertions:
  - at: T+1s
    target: living_area.discomfort_index
    condition: between NaN and 75
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(error, ScenarioError::InvalidAssertionKind));
}

#[test]
fn projects_domain_config_into_validated_runtime_config() {
    let scenario = load_scenario(fixture("examples/starlink_failover.yaml")).unwrap();

    let validated = ValidatedScenario::try_from(&scenario).unwrap();

    assert!(validated.domain_config().wan.expected_within.is_some());
    assert_eq!(
        validated.domain_config().wan.backup_status.as_deref(),
        Some("standby")
    );
}

#[test]
fn validates_promoted_ops_acknowledge_as_typed_step() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: typed_ops_acknowledge
steps:
  - at: T
    ops:
      action: acknowledge
      alert_id: sauna_emergency_button
      assignee: ops_member_01
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let validated = ValidatedScenario::try_from(&scenario).unwrap();

    assert!(validated.scheduled_events().iter().any(|event| {
        matches!(
            event.kind(),
            ValidatedEventKind::Step(ValidatedStepKind::Ops(ValidatedOpsStep::Acknowledge {
                alert_id: Some(alert_id),
                assignee: Some(assignee),
            })) if alert_id.as_str() == "sauna_emergency_button"
                && assignee.as_str() == "ops_member_01"
        )
    }));
}

#[test]
fn rejects_malformed_promoted_ops_acknowledge_at_validated_boundary() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: invalid_ops_acknowledge
steps:
  - at: T
    ops:
      action: acknowledge
      alert_id: ""
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::InvalidIdentifier { field, .. } if field == "steps[].ops.alert_id"
    ));
}

#[test]
fn preserves_unknown_ops_as_extension_step() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: ops_extension
steps:
  - at: T
    ops:
      action: custom_escalation
      queue: frontdesk
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let validated = ValidatedScenario::try_from(&scenario).unwrap();

    assert!(validated.scheduled_events().iter().any(|event| {
        matches!(
            event.kind(),
            ValidatedEventKind::Step(ValidatedStepKind::Ops(ValidatedOpsStep::Extension(map)))
                if map.get("action").and_then(|value| value.as_str())
                    == Some("custom_escalation")
                    && map.get("queue").and_then(|value| value.as_str()) == Some("frontdesk")
        )
    }));
}

#[test]
fn validates_promoted_hvac_automation_as_typed_step() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: typed_hvac_automation
steps:
  - at: T
    automation:
      type: hvac_auto_mode
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let validated = ValidatedScenario::try_from(&scenario).unwrap();

    assert!(validated.scheduled_events().iter().any(|event| {
        matches!(
            event.kind(),
            ValidatedEventKind::Step(ValidatedStepKind::Automation(
                ValidatedAutomationStep::HvacAutoMode
            ))
        )
    }));
}

#[test]
fn preserves_unknown_automation_as_extension_step() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: automation_extension
steps:
  - at: T
    automation:
      type: vendor_specific_sequence
      profile: evening
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let validated = ValidatedScenario::try_from(&scenario).unwrap();

    assert!(validated.scheduled_events().iter().any(|event| {
        matches!(
            event.kind(),
            ValidatedEventKind::Step(ValidatedStepKind::Automation(
                ValidatedAutomationStep::Extension(map)
            )) if map.get("type").and_then(|value| value.as_str())
                    == Some("vendor_specific_sequence")
                    && map.get("profile").and_then(|value| value.as_str()) == Some("evening")
        )
    }));
}

#[test]
fn rejects_invalid_promoted_command_target_at_validated_boundary() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: invalid_command_target
steps:
  - at: T
    command:
      target: "scene."
      action: activate
assertions:
  - at: T+1s
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::InvalidIdentifier { field, .. } if field == "steps[].command.target"
    ));
}

#[test]
fn validates_adapter_contract_examples() {
    for path in [
        "adapter-contracts/templates/company_adapter_contract.yaml",
        "adapter-contracts/examples/generic_mqtt_edge_device.yaml",
        "adapter-contracts/examples/hospitality_local_first_room.yaml",
        "adapter-contracts/examples/building_automation_bms.yaml",
        "adapter-contracts/examples/matter_gateway_profile.yaml",
        "adapter-contracts/examples/bacnet_contract_profile.yaml",
        "adapter-contracts/examples/knx_group_address_profile.yaml",
        "adapter-contracts/examples/opcua_contract_profile.yaml",
    ] {
        let contract = load_adapter_contract(fixture(path)).unwrap();
        validate_adapter_contract(&contract).unwrap();
    }
}

#[test]
fn rejects_adapter_contract_without_surface() {
    let contract: AdapterContract = serde_yaml::from_str(
        r#"
version: adapter.v1
adapter:
  name: empty-contract
acceptance:
  criteria: [Run a thing.]
  report_formats: [json]
"#,
    )
    .unwrap();

    assert!(matches!(
        validate_adapter_contract(&contract),
        Err(ScenarioError::InvalidAdapterContract(message))
            if message.contains("at least one")
    ));
}

#[test]
fn rejects_adapter_contract_with_invalid_modbus_access() {
    let contract: AdapterContract = serde_yaml::from_str(
        r#"
version: adapter.v1
adapter:
  name: invalid-modbus-access
modbus:
  devices:
    - id: meter
      registers:
        - address: 40001
          name: energy
          type: holding
          access: admin
acceptance:
  criteria: [Register map is valid.]
  report_formats: [json]
"#,
    )
    .unwrap();

    assert!(matches!(
        validate_adapter_contract(&contract),
        Err(ScenarioError::InvalidAdapterContract(message))
            if message.contains("unsupported access")
    ));
}

#[test]
fn rejects_adapter_contract_with_invalid_bms_hardening_fields() {
    let invalid_content_type: AdapterContract = serde_yaml::from_str(
        r#"
version: adapter.v1
adapter:
  name: invalid-bms-content-type
bms:
  alerts:
    - id: emergency
      source: contact.emergency
      severity: critical
      content_type: text/plain
      channels: [slack]
acceptance:
  criteria: [BMS alert contract is valid.]
  report_formats: [json]
"#,
    )
    .unwrap();
    assert!(matches!(
        validate_adapter_contract(&invalid_content_type),
        Err(ScenarioError::InvalidAdapterContract(message))
            if message.contains("unsupported content_type")
    ));

    let invalid_hmac: AdapterContract = serde_yaml::from_str(
        r#"
version: adapter.v1
adapter:
  name: invalid-bms-hmac
bms:
  alerts:
    - id: emergency
      source: contact.emergency
      severity: critical
      severity_enum: [info, warning]
      hmac:
        header: X-Signature
        algorithm: sha1
        secret_ref: env:SECRET
      replay_window_seconds: 0
      channels: [slack]
acceptance:
  criteria: [BMS alert contract is valid.]
  report_formats: [json]
"#,
    )
    .unwrap();
    assert!(matches!(
        validate_adapter_contract(&invalid_hmac),
        Err(ScenarioError::InvalidAdapterContract(message))
            if message.contains("severity critical is not declared")
    ));
}

#[test]
fn rejects_adapter_contract_with_invalid_protocol_profile() {
    let invalid_knx_direction: AdapterContract = serde_yaml::from_str(
        r#"
version: adapter.v1
adapter:
  name: invalid-knx-profile
protocol_profiles:
  knx:
    - name: scene
      gateway: knx-gateway
      group_address: 1/2/3
      datapoint_type: DPT-1.001
      direction: tunnel
      expected_value: true
      function: scene activation
acceptance:
  criteria: [KNX profile validates.]
  report_formats: [json]
"#,
    )
    .unwrap();
    assert!(matches!(
        validate_adapter_contract(&invalid_knx_direction),
        Err(ScenarioError::InvalidAdapterContract(message))
            if message.contains("unsupported direction")
    ));

    let missing_matter_expected_state: AdapterContract = serde_yaml::from_str(
        r#"
version: adapter.v1
adapter:
  name: invalid-matter-profile
protocol_profiles:
  matter:
    - name: light
      gateway: matter-gateway
      device_id: light-01
      endpoint_id: 1
      cluster: OnOff
      attribute: OnOff
      command: On
      expected_state: {}
acceptance:
  criteria: [Matter profile validates.]
  report_formats: [json]
"#,
    )
    .unwrap();
    assert!(matches!(
        validate_adapter_contract(&missing_matter_expected_state),
        Err(ScenarioError::InvalidAdapterContract(message))
            if message.contains("expected_state")
    ));
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
fn rejects_step_with_multiple_actions() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: multi_action_step
steps:
  - at: T
    event: no-op
    command:
      target: scene.welcome
      action: activate
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
fn rejects_invalid_edge_status() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: invalid_edge_status
edge:
  primary:
    status: warm
steps:
  - at: T
    event: no-op
assertions:
  - at: T+1s
    guest_experience: unaffected
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::Edge(roomci_edge::EdgeError::InvalidConfig(_))
    ));
}

#[test]
fn rejects_malformed_modbus_config() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: malformed_modbus
modbus:
  devices:
    - id: floor_heating
      holding_registers:
        bad-address:
          value: 1
steps:
  - at: T
    event: no-op
assertions:
  - at: T+1s
    guest_experience: unaffected
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::DeviceModel(roomci_device_model::DeviceModelError::InvalidModbusConfig(
            _
        ))
    ));
}

#[test]
fn rejects_unknown_fault_kind() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: unknown_fault_kind
faults:
  - at: T
    target: edge.primary
    type: imaginary_failure
steps:
  - at: T
    event: no-op
assertions:
  - at: T+1s
    guest_experience: unaffected
"#,
    )
    .unwrap();

    let error = validate_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        ScenarioError::InvalidFaultKind {
            target,
            fault_type
        } if target == "edge.primary" && fault_type == "imaginary_failure"
    ));
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

#[test]
fn rejects_out_of_range_duration() {
    let error = parse_duration("9999999999999h").unwrap_err();

    assert!(matches!(error, ScenarioError::InvalidDuration(_)));
}
