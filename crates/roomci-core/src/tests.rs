use std::path::PathBuf;

use roomci_scenario::load_scenario;

use super::*;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
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
fn commissioning_checklist_generation_passes() {
    let scenario = load_scenario(fixture("examples/commissioning_checklist.yaml")).unwrap();

    let report = run_scenario(&scenario).unwrap();

    assert_eq!(report.result, RunResult::Passed);
    assert!(report.assertions.iter().any(|assertion| {
        assertion.assertion_type == "commissioning_checklist"
            && assertion.message.contains("2 commissioning checks")
    }));
}
