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

use chrono::Duration;
use roomci_device_model::DeviceModelError;
use roomci_edge::EdgeError;
use roomci_mqtt::MqttError;
use roomci_ops::OpsError;
use roomci_scenario::{resolve_time_offset, validate_scenario, ScenarioError, ScenarioFile};
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
    validate_scenario(scenario)?;

    let mut runtime = RuntimeState::new(scenario)?;
    let mut events = Vec::new();
    for fault in &scenario.faults {
        let at = fault
            .at
            .as_deref()
            .map(resolve_time_offset)
            .transpose()?
            .unwrap_or_else(Duration::zero);
        events.push(ScheduledEvent::GlobalFault(at, fault.clone()));
    }
    for step in &scenario.steps {
        events.push(ScheduledEvent::Step(
            resolve_time_offset(&step.at)?,
            step.clone(),
        ));
    }
    for assertion in &scenario.assertions {
        events.push(ScheduledEvent::Assertion(
            resolve_time_offset(&assertion.at)?,
            assertion.clone(),
        ));
    }
    events.sort_by_key(|event| (event.at().num_milliseconds(), event.order()));

    let mut assertions = Vec::new();
    for event in events {
        if let Some(assertion) = runtime.apply_event(event)? {
            assertions.push(assertion);
        }
    }

    let result = if assertions.iter().all(|assertion| assertion.passed) {
        RunResult::Passed
    } else {
        RunResult::Failed
    };

    Ok(RunReport {
        scenario_name: scenario.scenario.name.clone(),
        result,
        timeline: runtime.timeline,
        assertions,
        final_state: runtime.states,
        retained_messages: runtime.broker.retained().clone(),
    })
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum ScheduledEvent {
    GlobalFault(Duration, roomci_scenario::FaultStep),
    Step(Duration, roomci_scenario::ScenarioStep),
    Assertion(Duration, roomci_scenario::AssertionDefinition),
}

impl ScheduledEvent {
    fn at(&self) -> Duration {
        match self {
            ScheduledEvent::GlobalFault(at, _)
            | ScheduledEvent::Step(at, _)
            | ScheduledEvent::Assertion(at, _) => *at,
        }
    }

    fn order(&self) -> u8 {
        match self {
            ScheduledEvent::GlobalFault(_, _) | ScheduledEvent::Step(_, _) => 0,
            ScheduledEvent::Assertion(_, _) => 1,
        }
    }
}

#[cfg(test)]
mod tests;
