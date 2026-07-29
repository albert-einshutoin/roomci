use std::path::PathBuf;

use roomci_scenario::{load_scenario, ScenarioError, ScenarioFile, ValidatedScenario};
use serde_json::json;

use super::*;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn run_scenario_rejects_invalid_version() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "banana"
scenario:
  name: invalid_scenario_version

assertions:
  - at: T+1s
    target: user_override
    condition: false
"#,
    )
    .unwrap();

    let error = run_scenario(&scenario).unwrap_err();

    assert!(matches!(
        error,
        CoreError::Scenario(ScenarioError::InvalidScenarioVersion {
            version,
            ..
        }) if version == "banana"
    ));
}

#[test]
fn local_first_cloud_outage_passes() {
    let scenario = load_scenario(fixture("examples/local_first_cloud_outage.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "fault_activated"
            && event.target.as_deref() == Some("mqtt.cloud")));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "mqtt_retained_state_updated"));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "edge_command_routed"));
}

#[test]
fn duplicate_delivery_keeps_single_retained_state() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: duplicate_delivery
  tags: [mqtt]
mqtt:
  local:
    retained: true
devices:
  - id: living_light
    type: light
    protocol: dali
    state:
      power: false
faults:
  - at: T
    target: mqtt.local
    type: duplicate_delivery
    topic: house/minakami/room/living/device/living_light/command
    count: 3
steps:
  - at: T+1s
    mqtt_publish:
      client: ipad_controller
      topic: house/minakami/room/living/device/living_light/command
      payload:
        power: true
assertions:
  - at: T+2s
    mqtt:
      topic: house/minakami/room/living/device/living_light/state
      retained:
        power: true
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert_eq!(report.retained_messages.len(), 1);
    assert!(report
        .timeline
        .iter()
        .any(|event| event.message.contains("3 deliveries")));
}

#[test]
fn edge_server_failover_passes() {
    let scenario = load_scenario(fixture("examples/edge_server_failover.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "edge_failover"));
}

#[test]
fn modbus_floor_heating_passes() {
    let scenario = load_scenario(fixture("examples/modbus_floor_heating.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "modbus_write"));
}

#[test]
fn protocol_conformance_smoke_passes() {
    let scenario = load_scenario(fixture("examples/protocol_conformance_smoke.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "mqtt_retained_state_updated"));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "modbus_write"));
}

#[test]
fn hardware_ci_mqtt_room_fleet_usecase_passes() {
    let scenario = load_scenario(fixture("examples/hardware_ci_mqtt_room_fleet.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .retained_messages
        .contains_key("fleet/hardware-ci/site/lab/room/room101/device/room101_light_panel/state"));
    assert!(report
        .retained_messages
        .contains_key("fleet/hardware-ci/site/lab/room/room101/device/room101_thermostat/state"));
    assert!(
        report
            .timeline
            .iter()
            .filter(|event| event.event_type == "mqtt_retained_state_updated")
            .count()
            >= 2
    );
}

#[test]
fn hardware_ci_modbus_bms_commissioning_usecase_passes() {
    let scenario = load_scenario(fixture(
        "examples/hardware_ci_modbus_bms_commissioning.yaml",
    ))
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "modbus_write"
            && event.target.as_deref() == Some("ahu_supply_fan_01")));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "contact_changed"
            && event.target.as_deref() == Some("plantroom_leak_sensor_01")));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "ops_ticket_acknowledged"));
}

#[test]
fn hardware_ci_mixed_protocol_regression_usecase_passes() {
    let scenario = load_scenario(fixture(
        "examples/hardware_ci_mixed_protocol_regression.yaml",
    ))
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .retained_messages
        .contains_key("fleet/hardware-ci/site/lab/zone/zone_a/device/edge_gateway_01/state"));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "fault_activated"
            && event.target.as_deref() == Some("network.segment.field_lab")));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "modbus_write"
            && event.target.as_deref() == Some("vav_controller_01")));
}

#[test]
fn dali_scene_partial_failure_is_detected() {
    let scenario = load_scenario(fixture("examples/dali_scene_partial_failure.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Failed);
    assert!(report
        .assertions
        .iter()
        .any(|assertion| assertion.assertion_type == "scene_consistency" && !assertion.passed));
}

#[test]
fn contact_input_changes_are_recorded_for_ops_phase() {
    let scenario = load_scenario(fixture("examples/bms_sauna_emergency_alert.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "contact_changed"
            && event.target.as_deref() == Some("sauna_emergency_button")));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "ops_slack_notification_sent"));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "ops_phone_call_triggered"));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "ops_runbook_url_included"
            && event.message.contains("sauna-emergency")));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "ops_ticket_acknowledged"
            && event.message.contains("ops_member_01")));
}

#[test]
fn assertions_are_evaluated_in_timeline_order() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: ops_timeline_assertions
contacts:
  inputs:
    - id: sauna_emergency_button
      state: off
alerts:
  - id: sauna_emergency_button
    source: contact.sauna_emergency_button
    notify:
      slack: true
steps:
  - at: T
    contact:
      id: sauna_emergency_button
      state: on
  - at: T+20s
    ops:
      action: acknowledge
      alert_id: sauna_emergency_button
assertions:
  - at: T+1s
    ops:
      alert_id: sauna_emergency_button
      ticket_status: open
  - at: T+30s
    ops:
      alert_id: sauna_emergency_button
      ticket_status: acknowledged
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert_eq!(report.assertions.len(), 2);
}

#[test]
fn mqtt_command_does_not_update_state_when_edge_is_unavailable() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: no_active_edge
edge:
  primary:
    id: edge_primary
    status: failed
mqtt:
  local:
    retained: true
devices:
  - id: living_light
    type: light
    protocol: dali
    state:
      power: false
steps:
  - at: T
    mqtt_publish:
      client: ipad_controller
      topic: house/minakami/room/living/device/living_light/command
      payload:
        power: true
assertions:
  - at: T+1s
    mqtt:
      topic: house/minakami/room/living/device/living_light/state
      retained:
        power: true
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Failed);
    assert!(report.retained_messages.is_empty());
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "edge_command_failed"));
}

#[test]
fn mqtt_contract_missing_required_fields_fails_run() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: mqtt_contract_missing_required_fields
mqtt:
  local:
    retained: true
  contracts:
    - name: device_state
      command_topic: fleet/demo/device/{device_id}/command
      state_topic: fleet/demo/device/{device_id}/state
      payload:
        required_fields: [online, sample_interval_seconds]
devices:
  - id: env_sensor_01
    type: sensor
    protocol: mqtt
    state:
      online: false
      sample_interval_seconds: 60
steps:
  - at: T
    mqtt_publish:
      client: edge_contract_test
      topic: fleet/demo/device/env_sensor_01/command
      payload:
        online: true

assertions:
  - at: T+1s
    target: user_override
    condition: false
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Failed);
    assert!(report.retained_messages.is_empty());
    assert!(report.assertions.iter().any(|assertion| {
        assertion.assertion_type == "mqtt_contract"
            && assertion
                .message
                .contains("payload missing required fields")
    }));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "mqtt_publish_failed"
            && event.message.contains("sample_interval_seconds")));
}

#[test]
fn mqtt_contract_unmatched_command_topic_fails_run() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: mqtt_contract_unmatched_command_topic
mqtt:
  local:
    retained: true
  contracts:
    - name: device_state
      command_topic: fleet/demo/device/{device_id}/command
      state_topic: fleet/demo/device/{device_id}/state
      payload:
        required_fields: [online]
devices:
  - id: env_sensor_01
    type: sensor
    protocol: mqtt
    state:
      online: false
steps:
  - at: T
    mqtt_publish:
      client: edge_contract_test
      topic: fleet/demo/device/env_sensor_01/set
      payload:
        online: true

assertions:
  - at: T+1s
    target: user_override
    condition: false
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Failed);
    assert!(report.retained_messages.is_empty());
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "mqtt_publish_failed"
            && event.message.contains("topic did not match")));
}

#[test]
fn mqtt_publish_unknown_device_id_fails_run() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: mqtt_publish_unknown_device
mqtt:
  local:
    retained: true
  contracts:
    - name: device_state
      command_topic: fleet/demo/device/{device_id}/command
      state_topic: fleet/demo/device/{device_id}/state
      payload:
        required_fields: [online]
devices:
  - id: env_sensor_01
    type: sensor
    protocol: mqtt
    state:
      online: false
steps:
  - at: T
    mqtt_publish:
      client: edge_contract_test
      topic: fleet/demo/device/unknown_sensor/command
      payload:
        online: true

assertions:
  - at: T+1s
    target: user_override
    condition: false
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Failed);
    assert!(report.retained_messages.is_empty());
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "mqtt_publish_failed"
            && event.message.contains("unknown device id")));
}

#[test]
fn starlink_failover_passes() {
    let scenario = load_scenario(fixture("examples/starlink_failover.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "wan_failover"));
    assert!(report
        .assertions
        .iter()
        .any(|assertion| assertion.assertion_type == "wan_failover" && assertion.passed));
}

#[test]
fn comfort_auto_mode_passes() {
    let scenario = load_scenario(fixture("examples/comfort_auto_mode.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .assertions
        .iter()
        .any(|assertion| assertion.assertion_type == "comfort_metric" && assertion.passed));
    assert!(report.assertions.iter().any(|assertion| {
        assertion.assertion_type == "comfort_user_override" && assertion.passed
    }));
}

#[test]
fn runtime_groups_customer_independent_domain_state() {
    let scenario = load_scenario(fixture("examples/comfort_auto_mode.yaml")).unwrap();

    let scenario = ValidatedScenario::try_from(&scenario).unwrap();
    let runtime = runtime::RuntimeState::new(&scenario);

    assert!(runtime.comfort.target.is_some());
    assert!(runtime.access.unexpected_users.is_empty());
    assert_eq!(runtime.commissioning.check_count, 0);
}

#[test]
fn command_step_updates_known_device_state() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: command_updates_device_state
devices:
  - id: living_light
    type: light
    protocol: dali
    state:
      power: off
steps:
  - at: T
    command:
      target: living_light
      action: turn_on

assertions:
  - at: T+1s
    target: user_override
    condition: false
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert_eq!(
        report
            .final_state
            .get("living_light")
            .and_then(|state| state.get("power"))
            .unwrap(),
        &json!("on")
    );
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "command_state_updated"));
}

#[test]
fn set_brightness_with_value_updates_state() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: set_brightness_applies_value
devices:
  - id: living_light
    type: light
    protocol: dali
    state:
      power: on
      brightness: 0
steps:
  - at: T
    command:
      target: living_light
      action: set_brightness
      value: 60

assertions:
  - at: T+1s
    target: user_override
    condition: false
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert_eq!(
        report
            .final_state
            .get("living_light")
            .and_then(|state| state.get("brightness"))
            .unwrap(),
        &json!(60)
    );
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "command_state_updated"));
}

#[test]
fn set_brightness_without_value_is_rejected() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario:
  name: set_brightness_requires_value
devices:
  - id: living_light
    type: light
    protocol: dali
    state:
      power: on
      brightness: 0
steps:
  - at: T
    command:
      target: living_light
      action: set_brightness

assertions:
  - at: T+1s
    target: user_override
    condition: false
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "command_rejected"));
    assert!(!report
        .timeline
        .iter()
        .any(|event| event.event_type == "command_state_updated"));
    assert_eq!(
        report
            .final_state
            .get("living_light")
            .and_then(|state| state.get("brightness"))
            .unwrap(),
        &json!(0)
    );
}

#[test]
fn device_command_value_example_applies_brightness() {
    let scenario = load_scenario(fixture("examples/device_command_value.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    let light = report.final_state.get("living_light").unwrap();
    assert_eq!(light.get("power").unwrap(), &json!("on"));
    assert_eq!(light.get("brightness").unwrap(), &json!(60));
    assert!(!report
        .timeline
        .iter()
        .any(|event| event.event_type == "command_rejected"));
}

#[test]
fn access_permission_drift_passes_when_stale_user_is_detected() {
    let scenario = load_scenario(fixture("examples/access_permission_drift.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report.assertions.iter().any(|assertion| {
        assertion.assertion_type == "access_control_drift"
            && assertion.message.contains("retired@example.com")
    }));
}

#[test]
fn intercom_relay_safe_mock_passes_without_real_unlock_control() {
    let scenario = load_scenario(fixture("examples/intercom_relay_safe_mock.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "intercom_pin_accepted"));
    assert!(report
        .timeline
        .iter()
        .any(|event| event.event_type == "relay_pulse_requested"));
    assert!(report.assertions.iter().any(|assertion| {
        assertion.assertion_type == "intercom_relay_safe_evidence" && assertion.passed
    }));
    assert_eq!(
        report
            .final_state
            .get("intercom.front_gate")
            .and_then(|state| state.get("real_unlock_controlled"))
            .and_then(|value| value.as_bool()),
        Some(false)
    );
}

#[test]
fn network_control_panel_fault_profiles_emit_bms_evidence() {
    let scenario = load_scenario(fixture(
        "examples/network_control_panel_fault_profiles.yaml",
    ))
    .unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    for event_type in [
        "network_segment_isolated",
        "firewall_policy_drift_detected",
        "control_panel_ups_degraded",
        "control_panel_circuit_protector_tripped",
        "control_panel_redundant_psu_degraded",
    ] {
        assert!(
            report
                .timeline
                .iter()
                .any(|event| event.event_type == event_type),
            "missing {event_type}"
        );
    }
    assert!(report.assertions.iter().any(|assertion| {
        assertion.assertion_type == "network_control_panel_faults" && assertion.passed
    }));
    assert_eq!(
        report
            .final_state
            .get("control_panel.ups")
            .and_then(|state| state.get("bms_evidence"))
            .and_then(|value| value.as_str()),
        Some("recorded")
    );
}

#[test]
fn comfort_timeseries_replay_updates_zone_evidence() {
    let scenario = load_scenario(fixture("examples/comfort_timeseries_replay.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(
        report
            .timeline
            .iter()
            .filter(|event| event.event_type == "comfort_sensor_reading_recorded")
            .count()
            >= 3
    );
    assert!(report
        .assertions
        .iter()
        .any(|assertion| assertion.assertion_type == "comfort_timeseries" && assertion.passed));
    assert_eq!(
        report
            .final_state
            .get("comfort.living_area")
            .and_then(|state| state.get("zone"))
            .and_then(|value| value.as_str()),
        Some("living")
    );
}

#[test]
fn commissioning_checklist_generation_passes() {
    let scenario = load_scenario(fixture("examples/commissioning_checklist.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report.assertions.iter().any(|assertion| {
        assertion.assertion_type == "commissioning_checklist"
            && assertion.message.contains("2 commissioning checks")
    }));
}

#[test]
fn named_assertion_keeps_diagnostic_name_and_evidence_reference() {
    let scenario: ScenarioFile = serde_yaml::from_str(
        r#"
version: "0.1"
scenario: { name: named_assertion_evidence }
mqtt:
  local: { enabled: true }
assertions:
  - at: T
    name: local_broker_available
    target: mqtt.local
    condition: available
"#,
    )
    .unwrap();

    let report = run_scenario(&scenario).unwrap();
    let assertion = &report.assertions[0];
    assert_eq!(
        assertion.reference_id.as_deref(),
        Some("local_broker_available")
    );
    assert_eq!(assertion.name, "mqtt.local");
    assert!(serde_json::to_value(&report).unwrap()["assertions"][0]
        .get("reference_id")
        .is_some());
}
