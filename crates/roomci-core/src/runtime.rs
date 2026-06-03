use std::collections::BTreeMap;

use chrono::Duration;
use roomci_device_model::{ContactModel, LightingEvent, LightingModel, ModbusModel};
use roomci_edge::EdgeModel;
use roomci_mqtt::{device_id_from_command_topic, BrokerModel};
use roomci_ops::{OpsEvent, OpsModel};
use roomci_scenario::{
    yaml_map_to_json, CommandTarget, Condition, ValidatedAutomationStep, ValidatedEventKind,
    ValidatedFaultKind, ValidatedIntercomStep, ValidatedMqttPublishStep, ValidatedOpsStep,
    ValidatedScenario, ValidatedScheduledEvent, ValidatedSensorReadingStep, ValidatedStepKind,
};

use crate::{AssertionResult, CoreError, StateMap, TimelineEvent};

#[derive(Debug)]
pub(crate) struct RuntimeState {
    pub(crate) states: BTreeMap<String, StateMap>,
    pub(crate) protocols: BTreeMap<String, Option<String>>,
    pub(crate) broker: BrokerModel,
    pub(crate) edge: EdgeModel,
    pub(crate) edge_primary_failed_at: Option<Duration>,
    pub(crate) edge_failover_at: Option<Duration>,
    pub(crate) edge_expected_within: Option<Duration>,
    pub(crate) wan_primary_failed_at: Option<Duration>,
    pub(crate) wan_failover_at: Option<Duration>,
    pub(crate) wan_expected_within: Option<Duration>,
    pub(crate) wan_backup_status: Option<String>,
    pub(crate) duplicate_delivery_by_topic: BTreeMap<String, u32>,
    pub(crate) modbus: ModbusModel,
    pub(crate) lighting: LightingModel,
    pub(crate) contacts: ContactModel,
    pub(crate) ops: OpsModel,
    pub(crate) comfort: ComfortRuntimeState,
    pub(crate) access: AccessRuntimeState,
    pub(crate) commissioning: CommissioningRuntimeState,
    pub(crate) timeline: Vec<TimelineEvent>,
}

#[derive(Debug)]
pub(crate) struct ComfortRuntimeState {
    pub(crate) target: Option<f64>,
    pub(crate) range: Option<(f64, f64)>,
    pub(crate) discomfort_by_target: BTreeMap<String, f64>,
    pub(crate) reading_history: BTreeMap<String, Vec<f64>>,
    pub(crate) user_override_count: u32,
}

#[derive(Debug)]
pub(crate) struct AccessRuntimeState {
    pub(crate) unexpected_users: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct CommissioningRuntimeState {
    pub(crate) check_count: usize,
    pub(crate) site: Option<String>,
}

impl RuntimeState {
    pub(crate) fn new(scenario: &ValidatedScenario) -> Self {
        let domain_config = scenario.domain_config();
        let runtime_inputs = scenario.runtime_inputs();
        let states = scenario
            .devices()
            .iter()
            .map(|device| (device.id.to_string(), device.state.clone()))
            .collect::<BTreeMap<_, _>>();
        let protocols = scenario
            .devices()
            .iter()
            .map(|device| {
                (
                    device.id.to_string(),
                    device.protocol.as_ref().map(ToString::to_string),
                )
            })
            .collect::<BTreeMap<_, _>>();

        Self {
            states,
            protocols,
            broker: BrokerModel::new(true, runtime_inputs.broker_cloud_enabled),
            edge: runtime_inputs.edge.clone(),
            edge_primary_failed_at: None,
            edge_failover_at: None,
            edge_expected_within: domain_config.edge.expected_within,
            wan_primary_failed_at: None,
            wan_failover_at: None,
            wan_expected_within: domain_config.wan.expected_within,
            wan_backup_status: domain_config.wan.backup_status.clone(),
            duplicate_delivery_by_topic: BTreeMap::new(),
            modbus: runtime_inputs.modbus.clone(),
            lighting: runtime_inputs.lighting.clone(),
            contacts: runtime_inputs.contacts.clone(),
            ops: runtime_inputs.ops.clone(),
            comfort: ComfortRuntimeState {
                target: domain_config.comfort.target,
                range: domain_config.comfort.range,
                discomfort_by_target: domain_config.comfort.discomfort_by_target.clone(),
                reading_history: BTreeMap::new(),
                user_override_count: 0,
            },
            access: AccessRuntimeState {
                unexpected_users: domain_config.access.unexpected_users.clone(),
            },
            commissioning: CommissioningRuntimeState {
                check_count: domain_config.commissioning.check_count,
                site: domain_config.commissioning.site.clone(),
            },
            timeline: Vec::new(),
        }
    }

    pub(crate) fn apply_event(
        &mut self,
        event: &ValidatedScheduledEvent,
    ) -> Result<Option<AssertionResult>, CoreError> {
        match event.kind() {
            ValidatedEventKind::GlobalFault(fault) => {
                self.apply_fault(event.at(), fault);
                Ok(None)
            }
            ValidatedEventKind::Step(step) => {
                match step {
                    ValidatedStepKind::Event(event_name) => {
                        self.push(event.at(), "event", None, event_name);
                    }
                    ValidatedStepKind::Fault(fault) => {
                        self.apply_fault(event.at(), fault);
                    }
                    ValidatedStepKind::Command(command) => {
                        self.apply_command(event.at(), &command.target, command.action.as_str());
                    }
                    ValidatedStepKind::ModbusWrite(modbus_write) => {
                        self.apply_modbus_write(
                            event.at(),
                            modbus_write.device.as_str(),
                            modbus_write.register,
                            modbus_write.value.clone(),
                        );
                    }
                    ValidatedStepKind::MqttPublish(mqtt_publish) => {
                        self.apply_mqtt_publish(event.at(), mqtt_publish);
                    }
                    ValidatedStepKind::Contact(contact) => {
                        let contact_id = contact.id.to_string();
                        let contact_state = contact.state.to_string();
                        self.contacts.set_state(&contact_id, &contact_state)?;
                        self.push(
                            event.at(),
                            "contact_changed",
                            Some(contact_id.clone()),
                            format!("contact state changed to {contact_state}"),
                        );
                        for ops_event in self.ops.apply_contact_change(&contact_id, &contact_state)
                        {
                            self.push_ops_event(event.at(), ops_event);
                        }
                    }
                    ValidatedStepKind::Intercom(intercom) => {
                        self.apply_intercom_step(event.at(), intercom);
                    }
                    ValidatedStepKind::SensorReading(sensor_reading) => {
                        self.apply_sensor_reading(event.at(), sensor_reading);
                    }
                    ValidatedStepKind::Ops(ops) => {
                        self.apply_ops_step(event.at(), ops);
                    }
                    ValidatedStepKind::Automation(automation) => {
                        self.apply_automation(event.at(), automation);
                    }
                }
                Ok(None)
            }
            ValidatedEventKind::Assertion(assertion) => {
                Ok(Some(self.evaluate_assertion(assertion)))
            }
        }
    }

    fn apply_sensor_reading(&mut self, at: Duration, reading: &ValidatedSensorReadingStep) {
        let discomfort = discomfort_index(reading.temperature, reading.humidity);
        let discomfort_key = format!("{}.discomfort_index", reading.target);
        self.comfort
            .discomfort_by_target
            .insert(discomfort_key, discomfort);

        let mut state = StateMap::new();
        state.insert(
            "temperature".to_string(),
            serde_json::Value::from(reading.temperature),
        );
        state.insert(
            "humidity".to_string(),
            serde_json::Value::from(reading.humidity),
        );
        state.insert(
            "discomfort_index".to_string(),
            serde_json::Value::from(discomfort),
        );
        if let Some(occupancy) = reading.occupancy {
            state.insert("occupancy".to_string(), serde_json::Value::Bool(occupancy));
        }
        if let Some(zone) = &reading.zone {
            state.insert("zone".to_string(), serde_json::Value::String(zone.clone()));
        }

        let history = self
            .comfort
            .reading_history
            .entry(reading.target.to_string())
            .or_default();
        history.push(discomfort);
        let oscillation_detected = has_recent_oscillation(history);
        state.insert(
            "oscillation_detected".to_string(),
            serde_json::Value::Bool(oscillation_detected),
        );
        self.states
            .insert(format!("comfort.{}", reading.target), state);
        self.push(
            at,
            "comfort_sensor_reading_recorded",
            Some(reading.target.to_string()),
            format!(
                "temperature {} humidity {} discomfort {}",
                reading.temperature, reading.humidity, discomfort
            ),
        );
        if oscillation_detected {
            self.push(
                at,
                "comfort_oscillation_detected",
                Some(reading.target.to_string()),
                "recent comfort readings changed direction repeatedly".to_string(),
            );
        }
    }

    fn apply_intercom_step(&mut self, at: Duration, intercom: &ValidatedIntercomStep) {
        let state_key = format!("intercom.{}", intercom.id);
        let mut state = StateMap::new();
        state.insert(
            "event".to_string(),
            serde_json::Value::String(intercom.event.to_string()),
        );
        state.insert(
            "outcome".to_string(),
            serde_json::Value::String(intercom.outcome.to_string()),
        );
        state.insert(
            "real_unlock_controlled".to_string(),
            serde_json::Value::Bool(false),
        );
        if let Some(fallback) = &intercom.fallback {
            state.insert(
                "fallback".to_string(),
                serde_json::Value::String(fallback.clone()),
            );
        }
        self.states.insert(state_key.clone(), state);

        let event_type = match (intercom.event.as_str(), intercom.outcome.as_str()) {
            ("pin_check", "accepted") => "intercom_pin_accepted",
            ("pin_check", "rejected") => "intercom_pin_rejected",
            ("relay_pulse", "requested") => "relay_pulse_requested",
            ("staff_call", "attempted") => "staff_call_attempted",
            (_, "failed") => "intercom_failure_observed",
            _ => "intercom_event_observed",
        };
        self.push(
            at,
            event_type,
            Some(state_key.clone()),
            format!(
                "intercom safe mock observed {} with outcome {}",
                intercom.event, intercom.outcome
            ),
        );
        if let Some(fallback) = &intercom.fallback {
            self.push(
                at,
                "intercom_fallback_used",
                Some(state_key),
                format!("safe fallback selected: {fallback}"),
            );
        }
    }

    fn apply_fault(&mut self, at: Duration, fault: &ValidatedFaultKind) {
        match fault {
            ValidatedFaultKind::BrokerOffline { target } => {
                self.broker.set_online(target.as_str(), false);
            }
            ValidatedFaultKind::EdgePrimaryPowerLost => {
                self.edge_primary_failed_at = Some(at);
                if let Some(outcome) = self.edge.apply_power_lost_to_primary() {
                    self.edge_failover_at = Some(at);
                    self.push(
                        at,
                        "edge_failover",
                        Some(outcome.to),
                        format!("edge failover completed from {}", outcome.from),
                    );
                }
            }
            ValidatedFaultKind::WanPrimaryDown => {
                self.wan_primary_failed_at = Some(at);
                self.wan_backup_status = Some("active".to_string());
                self.wan_failover_at = Some(at);
                self.push(
                    at,
                    "wan_failover",
                    Some("wan.backup".to_string()),
                    "backup WAN activated after primary failure",
                );
                let event = self.ops.record_slack_notification("wan_failover", None);
                self.push_ops_event(at, event);
            }
            ValidatedFaultKind::MqttDuplicateDelivery {
                topic: Some(topic),
                count,
            } => {
                self.duplicate_delivery_by_topic
                    .insert(topic.to_string(), count.unwrap_or(2).max(2));
            }
            ValidatedFaultKind::MqttDuplicateDelivery { topic: None, .. } => {}
            ValidatedFaultKind::NetworkSegmentIsolated { segment } => {
                self.record_fault_profile(
                    at,
                    &fault.target(),
                    "network_segment_isolated",
                    format!("network segment {segment} isolated"),
                );
            }
            ValidatedFaultKind::FirewallPolicyDrift { policy } => {
                self.record_fault_profile(
                    at,
                    &fault.target(),
                    "firewall_policy_drift_detected",
                    format!("firewall policy {policy} drift detected"),
                );
            }
            ValidatedFaultKind::MqttLocalUnreachable => {
                self.record_fault_profile(
                    at,
                    "mqtt.local",
                    "local_broker_unreachable",
                    "local MQTT broker unreachable".to_string(),
                );
            }
            ValidatedFaultKind::ControlPanelUpsBatteryDegraded => {
                self.record_fault_profile(
                    at,
                    "control_panel.ups",
                    "control_panel_ups_degraded",
                    "control-panel UPS battery degraded".to_string(),
                );
            }
            ValidatedFaultKind::ControlPanelCircuitProtectorTripped { .. } => {
                self.record_fault_profile(
                    at,
                    &fault.target(),
                    "control_panel_circuit_protector_tripped",
                    format!("{} tripped", fault.target()),
                );
            }
            ValidatedFaultKind::ControlPanelPsuDegraded { .. } => {
                self.record_fault_profile(
                    at,
                    &fault.target(),
                    "control_panel_redundant_psu_degraded",
                    format!("{} degraded", fault.target()),
                );
            }
            ValidatedFaultKind::EdgeSecondaryTakeoverFailed => {
                self.record_fault_profile(
                    at,
                    "edge.secondary",
                    "edge_redundancy_takeover_failed",
                    "secondary edge takeover failed".to_string(),
                );
            }
            ValidatedFaultKind::DaliFixtureCommandDrop { fixture } => {
                if let Err(error) = self.lighting.drop_command_for_fixture(fixture.as_str()) {
                    self.push(
                        at,
                        "fault_rejected",
                        Some(fault.target()),
                        error.to_string(),
                    );
                    return;
                }
            }
        }
        let target = fault.target();
        self.push(
            at,
            "fault_activated",
            Some(target),
            format!("{} fault activated", fault.fault_type()),
        );
    }

    fn record_fault_profile(
        &mut self,
        at: Duration,
        target: &str,
        event_type: &str,
        message: String,
    ) {
        let mut state = StateMap::new();
        state.insert(
            "status".to_string(),
            serde_json::Value::String("faulted".to_string()),
        );
        state.insert(
            "fault_profile".to_string(),
            serde_json::Value::String(event_type.to_string()),
        );
        state.insert(
            "bms_evidence".to_string(),
            serde_json::Value::String("recorded".to_string()),
        );
        self.states.insert(target.to_string(), state);
        self.push(at, event_type, Some(target.to_string()), message);
        let event = self.ops.record_slack_notification(event_type, None);
        self.push_ops_event(at, event);
    }

    fn apply_command(&mut self, at: Duration, target: &CommandTarget, action: &str) {
        let display_target = target.display_target();
        self.push(
            at,
            "command_received",
            Some(display_target),
            format!("{} command received", action),
        );
        if action == "activate" {
            if let CommandTarget::Scene(scene) = target {
                self.activate_scene(at, scene.as_str());
            }
        }
    }

    fn apply_modbus_write(
        &mut self,
        at: Duration,
        device: &str,
        register: u32,
        value: serde_yaml::Value,
    ) {
        match self.modbus.write(device, register, value) {
            Ok(json_value) => self.push(
                at,
                "modbus_write",
                Some(device.to_string()),
                format!("register {} set to {}", register, json_value),
            ),
            Err(error) => self.push(
                at,
                "modbus_write_rejected",
                Some(device.to_string()),
                error.to_string(),
            ),
        }
    }

    fn activate_scene(&mut self, at: Duration, scene: &str) {
        match self.lighting.activate_scene(scene) {
            Ok(events) => {
                for event in events {
                    match event {
                        LightingEvent::LevelChanged { fixture, level } => self.push(
                            at,
                            "dali_level_changed",
                            Some(fixture),
                            format!("fixture level changed to {level}"),
                        ),
                        LightingEvent::CommandDropped { fixture } => self.push(
                            at,
                            "dali_command_dropped",
                            Some(fixture),
                            "fixture command dropped".to_string(),
                        ),
                    }
                }
            }
            Err(error) => self.push(
                at,
                "scene_activation_failed",
                Some(scene.to_string()),
                error.to_string(),
            ),
        }
        self.push(
            at,
            "scene_activation_requested",
            Some(scene.to_string()),
            "scene activation requested".to_string(),
        );
    }

    fn apply_mqtt_publish(&mut self, at: Duration, publish: &ValidatedMqttPublishStep) {
        self.push(
            at,
            "mqtt_publish",
            Some(publish.client.to_string()),
            format!("published {}", publish.topic),
        );

        let deliveries = self
            .duplicate_delivery_by_topic
            .get(publish.topic.as_str())
            .copied()
            .unwrap_or(1);
        let Some(device_id) = device_id_from_command_topic(publish.topic.as_str()) else {
            self.push(
                at,
                "mqtt_publish_failed",
                Some("mqtt.local".to_string()),
                format!("topic is not a device command topic: {}", publish.topic),
            );
            return;
        };
        match self.edge.route_mqtt_command(
            publish.client.as_str(),
            &device_id,
            self.protocols.get(&device_id).cloned().flatten(),
        ) {
            Ok(routed) => self.push(
                at,
                "edge_command_routed",
                Some(routed.edge_id),
                format!(
                    "{} routed command from {} to {}",
                    routed.action, routed.source_client, routed.target_device
                ),
            ),
            Err(error) => {
                self.push(
                    at,
                    "edge_command_failed",
                    Some(device_id),
                    error.to_string(),
                );
                return;
            }
        }

        let payload = yaml_map_to_json(&publish.payload);
        match self.broker.publish_device_command(
            "mqtt.local",
            publish.client.as_str(),
            publish.topic.as_str(),
            payload,
            deliveries,
        ) {
            Ok(outcome) => {
                if let (Some(device_id), Some(retained_payload)) =
                    (outcome.device_id.clone(), outcome.retained_payload)
                {
                    self.states.insert(device_id.clone(), retained_payload);
                    let state_topic = outcome.state_topic.unwrap_or_default();
                    let delivery_word = if outcome.deliveries == 1 {
                        "delivery"
                    } else {
                        "deliveries"
                    };
                    self.push(
                        at,
                        "mqtt_retained_state_updated",
                        Some(device_id),
                        format!(
                            "retained state updated at {} after {} {}",
                            state_topic, outcome.deliveries, delivery_word
                        ),
                    );
                }
            }
            Err(error) => {
                self.push(
                    at,
                    "mqtt_publish_failed",
                    Some("mqtt.local".to_string()),
                    error.to_string(),
                );
            }
        }
    }

    fn apply_ops_step(&mut self, at: Duration, ops: &ValidatedOpsStep) {
        if let ValidatedOpsStep::Acknowledge { alert_id, assignee } = ops {
            for event in self.ops.acknowledge(
                alert_id.as_ref().map(|id| id.as_str()),
                assignee.as_ref().map(ToString::to_string),
            ) {
                self.push_ops_event(at, event);
            }
        }
        self.push(at, "ops_action", None, "ops action applied");
    }

    fn apply_automation(&mut self, at: Duration, automation: &ValidatedAutomationStep) {
        if matches!(automation, ValidatedAutomationStep::HvacAutoMode) {
            if let Some(target) = self.comfort.target {
                self.comfort
                    .discomfort_by_target
                    .insert("living_area.discomfort_index".to_string(), target);
                self.push(
                    at,
                    "comfort_auto_mode_applied",
                    Some("living_area".to_string()),
                    format!("target discomfort index set to {target}"),
                );
            }
        }
        self.push(at, "automation_started", None, "automation started");
    }

    fn push_ops_event(&mut self, at: Duration, event: OpsEvent) {
        match event {
            OpsEvent::SlackNotificationSent {
                alert_id,
                runbook_url,
            } => self.push(
                at,
                "ops_slack_notification_sent",
                Some(alert_id),
                match runbook_url {
                    Some(url) => format!("Slack notification sent with runbook {url}"),
                    None => "Slack notification sent".to_string(),
                },
            ),
            OpsEvent::PhoneCallTriggered { alert_id } => self.push(
                at,
                "ops_phone_call_triggered",
                Some(alert_id),
                "Phone escalation triggered".to_string(),
            ),
            OpsEvent::TicketOpened { alert_id, status } => self.push(
                at,
                "ops_ticket_opened",
                Some(alert_id),
                format!("Ops ticket opened with status {status}"),
            ),
            OpsEvent::TicketAcknowledged { alert_id, assignee } => self.push(
                at,
                "ops_ticket_acknowledged",
                Some(alert_id),
                match assignee {
                    Some(assignee) => format!("Ops ticket acknowledged by {assignee}"),
                    None => "Ops ticket acknowledged".to_string(),
                },
            ),
            OpsEvent::RunbookUrlIncluded {
                alert_id,
                runbook_url,
            } => self.push(
                at,
                "ops_runbook_url_included",
                Some(alert_id),
                format!("Runbook URL included: {runbook_url}"),
            ),
        }
    }

    fn evaluate_assertion(
        &self,
        assertion: &roomci_scenario::ValidatedAssertionKind,
    ) -> AssertionResult {
        crate::assertions::evaluate_assertion(self, assertion)
    }

    fn push(
        &mut self,
        at: Duration,
        event_type: impl Into<String>,
        target: Option<String>,
        message: impl Into<String>,
    ) {
        self.timeline.push(TimelineEvent {
            at: format_duration(at),
            event_type: event_type.into(),
            target,
            message: message.into(),
        });
    }

    pub(crate) fn evaluate_between_condition(&self, target: &str, condition: Condition) -> bool {
        let Some(actual) = self.comfort.discomfort_by_target.get(target).copied() else {
            return false;
        };
        if let Condition::Between { min, max } = condition {
            return actual >= min && actual <= max;
        }
        if let Some((min, max)) = self.comfort.range {
            return actual >= min && actual <= max;
        }
        false
    }
}

fn discomfort_index(temperature: f64, humidity: f64) -> f64 {
    0.81 * temperature + 0.01 * humidity * (0.99 * temperature - 14.3) + 46.3
}

fn has_recent_oscillation(history: &[f64]) -> bool {
    if history.len() < 4 {
        return false;
    }
    let recent = &history[history.len() - 4..];
    let mut directions = Vec::new();
    for pair in recent.windows(2) {
        let delta = pair[1] - pair[0];
        if delta.abs() < f64::EPSILON {
            continue;
        }
        directions.push(delta.signum());
    }
    directions.len() >= 3 && directions.windows(2).all(|pair| pair[0] != pair[1])
}

fn format_duration(duration: Duration) -> String {
    if duration == Duration::zero() {
        return "T".to_string();
    }
    let milliseconds = duration.num_milliseconds();
    let sign = if milliseconds >= 0 { "+" } else { "-" };
    let seconds = milliseconds.abs() / 1000;
    if seconds % 60 == 0 && seconds >= 60 {
        format!("T{}{}m", sign, seconds / 60)
    } else {
        format!("T{}{}s", sign, seconds)
    }
}
