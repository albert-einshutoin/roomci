//! Scenario runner core for roomci.
//!
//! This crate ties together the MQTT, edge, device, and ops models, takes a
//! parsed [`ScenarioFile`], and produces a [`RunReport`] containing the
//! assertion outcomes, timeline of emitted events, and final device/retained
//! MQTT state.
//!
//! The runner uses **virtual time** — every scenario step, fault, and
//! assertion is offset from a logical anchor (`T`, `T+1s`, `T+10m`, ...) and
//! evaluated in order, with assertions placed after same-instant steps so
//! state changes are visible before they are observed.

use std::collections::BTreeMap;

mod assertions;
mod runtime;

use runtime::RuntimeState;

use roomci_device_model::DeviceModelError;
use roomci_edge::EdgeError;
use roomci_mqtt::MqttError;
use roomci_ops::OpsError;
use roomci_scenario::{ScenarioError, ScenarioFile, ValidatedScenario};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Per-device state map: an ordered map from JSON-style field name to value.
pub type StateMap = BTreeMap<String, serde_json::Value>;

/// Errors produced by [`run_scenario`].
///
/// Each variant wraps an error from one of the sibling crates so the caller
/// can distinguish scenario-loading failures from runtime errors in a specific
/// subsystem (MQTT, edge, device model, ops).
#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
    #[error(transparent)]
    Mqtt(#[from] MqttError),
    #[error(transparent)]
    Edge(#[from] EdgeError),
    #[error(transparent)]
    DeviceModel(#[from] DeviceModelError),
    #[error(transparent)]
    Ops(#[from] OpsError),
}

/// Outcome of executing a scenario end-to-end.
///
/// `final_state` is the per-device state map after every step has been
/// applied, and `retained_messages` is the contents of the local MQTT broker's
/// retained store, both useful for downstream report renderers in
/// `roomci-report`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RunReport {
    pub schema_version: String,
    pub run_id: String,
    pub generated_by: String,
    pub scenario_name: String,
    pub result: RunResult,
    pub timeline: Vec<TimelineEvent>,
    pub assertions: Vec<AssertionResult>,
    pub final_state: BTreeMap<String, StateMap>,
    pub retained_messages: BTreeMap<String, StateMap>,
}

/// Top-level pass/fail result for a scenario run.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunResult {
    Passed,
    Failed,
}

/// One event recorded on the scenario timeline: an absolute virtual-time
/// stamp, the event type (`fault_activated`, `mqtt_retained_state_updated`,
/// `edge_failover`, ...), an optional target, and a human-readable message.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TimelineEvent {
    pub at: String,
    pub event_type: String,
    pub target: Option<String>,
    pub message: String,
}

/// Outcome of evaluating one assertion.
///
/// On failure, `impact_level` and `impact_message` describe the guest-visible
/// impact so report renderers can surface meaningful recovery actions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AssertionResult {
    pub name: String,
    pub assertion_type: String,
    pub passed: bool,
    pub message: String,
    pub impact_level: Option<String>,
    pub impact_message: Option<String>,
}

/// Execute a parsed scenario and produce a [`RunReport`].
///
/// Steps and assertions are sorted by their virtual-time offset (with
/// assertions placed after same-instant steps), then evaluated against an
/// internal runtime that owns the broker, edge, device, and ops models.
///
/// Returns [`CoreError`] if the scenario fails validation or if any subsystem
/// rejects a request.
pub fn run_scenario(scenario: &ScenarioFile) -> Result<RunReport, CoreError> {
    let scenario = ValidatedScenario::try_from(scenario)?;

    let mut runtime = RuntimeState::new(&scenario);

    let mut assertions = Vec::new();
    for event in scenario.scheduled_events() {
        if let Some(assertion) = runtime.apply_event(event)? {
            assertions.push(assertion);
        }
    }
    assertions.append(&mut runtime.runtime_failures);

    let result = if assertions.iter().all(|assertion| assertion.passed) {
        RunResult::Passed
    } else {
        RunResult::Failed
    };

    Ok(RunReport {
        schema_version: "roomci.report.v1".to_string(),
        run_id: scenario.run_id().to_string(),
        generated_by: "roomci".to_string(),
        scenario_name: scenario.scenario_name().to_string(),
        result,
        timeline: runtime.timeline,
        assertions,
        final_state: runtime.states,
        retained_messages: runtime.broker.retained().clone(),
    })
}

#[cfg(test)]
mod tests;
