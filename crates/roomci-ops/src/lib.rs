//! Operations (BMS/alert) mock used by roomci scenarios.
//!
//! Captures the QA-visible shape of an external monitoring/alert system:
//! tickets, Slack notifications, phone escalations, and runbook URLs. The
//! scenario runner drives this mock when contact inputs flip to `on` and
//! evaluates it with [`OpsModel::evaluate_assertion`] against the scenario
//! `assertions.ops.*` fields.

use std::collections::BTreeMap;

use thiserror::Error;

/// Errors emitted while configuring or evaluating the ops mock.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OpsError {
    #[error("unknown alert source {0}")]
    UnknownAlertSource(String),
    #[error("invalid alert config: {0}")]
    InvalidAlertConfig(String),
    #[error("unsupported ops assertion {0}")]
    UnsupportedAssertion(String),
}

/// Discrete ops-side side effects emitted to the scenario timeline when an
/// alert fires or is acknowledged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpsEvent {
    SlackNotificationSent {
        alert_id: String,
        runbook_url: Option<String>,
    },
    PhoneCallTriggered {
        alert_id: String,
    },
    TicketOpened {
        alert_id: String,
        status: String,
    },
    TicketAcknowledged {
        alert_id: String,
        assignee: Option<String>,
    },
    RunbookUrlIncluded {
        alert_id: String,
        runbook_url: String,
    },
}

/// Aggregate outcome of evaluating an `assertions.ops.*` block, with the
/// list of individual check failures when `passed` is false.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpsAssertionOutcome {
    pub passed: bool,
    pub failures: Vec<String>,
}

/// In-memory ops mock: declared alerts plus the ticket/notification state
/// each alert has produced so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpsModel {
    alerts: Vec<OpsAlert>,
    tickets: BTreeMap<String, TicketState>,
    slack_notifications: BTreeMap<String, Option<String>>,
    phone_calls: BTreeMap<String, bool>,
    runbook_urls: BTreeMap<String, String>,
}

impl OpsModel {
    pub fn from_config(alerts: &[BTreeMap<String, serde_yaml::Value>]) -> Self {
        Self::try_from_config(alerts).unwrap_or_default()
    }

    pub fn try_from_config(
        alerts: &[BTreeMap<String, serde_yaml::Value>],
    ) -> Result<Self, OpsError> {
        let alerts = alerts
            .iter()
            .map(OpsAlert::from_map)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            alerts,
            tickets: BTreeMap::new(),
            slack_notifications: BTreeMap::new(),
            phone_calls: BTreeMap::new(),
            runbook_urls: BTreeMap::new(),
        })
    }

    pub fn validate_sources<F>(&self, mut has_contact: F) -> Result<(), OpsError>
    where
        F: FnMut(&str) -> bool,
    {
        for alert in &self.alerts {
            let Some(contact_id) = alert.source.strip_prefix("contact.") else {
                return Err(OpsError::UnknownAlertSource(alert.source.clone()));
            };
            if !has_contact(contact_id) {
                return Err(OpsError::UnknownAlertSource(alert.source.clone()));
            }
        }
        Ok(())
    }

    pub fn apply_contact_change(&mut self, contact_id: &str, state: &str) -> Vec<OpsEvent> {
        if state != "on" {
            return Vec::new();
        }
        let mut events = Vec::new();
        for alert in self
            .alerts
            .iter()
            .filter(|alert| alert.source == format!("contact.{contact_id}"))
        {
            self.tickets.insert(
                alert.id.clone(),
                TicketState {
                    status: "open".to_string(),
                    assignee: None,
                },
            );
            events.push(OpsEvent::TicketOpened {
                alert_id: alert.id.clone(),
                status: "open".to_string(),
            });
            if alert.notify_slack {
                self.slack_notifications
                    .insert(alert.id.clone(), alert.runbook_url.clone());
                events.push(OpsEvent::SlackNotificationSent {
                    alert_id: alert.id.clone(),
                    runbook_url: alert.runbook_url.clone(),
                });
            }
            if alert.notify_phone {
                self.phone_calls.insert(alert.id.clone(), true);
                events.push(OpsEvent::PhoneCallTriggered {
                    alert_id: alert.id.clone(),
                });
            }
            if let Some(runbook_url) = &alert.runbook_url {
                self.runbook_urls
                    .insert(alert.id.clone(), runbook_url.clone());
                events.push(OpsEvent::RunbookUrlIncluded {
                    alert_id: alert.id.clone(),
                    runbook_url: runbook_url.clone(),
                });
            }
        }
        events
    }

    pub fn record_slack_notification(
        &mut self,
        alert_id: impl Into<String>,
        runbook_url: Option<String>,
    ) -> OpsEvent {
        let alert_id = alert_id.into();
        self.slack_notifications
            .insert(alert_id.clone(), runbook_url.clone());
        OpsEvent::SlackNotificationSent {
            alert_id,
            runbook_url,
        }
    }

    pub fn acknowledge(
        &mut self,
        alert_id: Option<&str>,
        assignee: Option<String>,
    ) -> Vec<OpsEvent> {
        let mut events = Vec::new();
        for (ticket_alert_id, ticket) in &mut self.tickets {
            if let Some(expected_alert_id) = alert_id {
                if ticket_alert_id != expected_alert_id {
                    continue;
                }
            }
            ticket.status = "acknowledged".to_string();
            ticket.assignee = assignee.clone();
            events.push(OpsEvent::TicketAcknowledged {
                alert_id: ticket_alert_id.clone(),
                assignee: assignee.clone(),
            });
        }
        events
    }

    pub fn evaluate_assertion(
        &self,
        assertion: &BTreeMap<String, serde_yaml::Value>,
    ) -> Result<OpsAssertionOutcome, OpsError> {
        let mut failures = Vec::new();
        let alert_id = assertion
            .get("alert_id")
            .and_then(|value| value.as_str())
            .or_else(|| assertion.get("source").and_then(|value| value.as_str()));
        for (key, expected) in assertion {
            match key.as_str() {
                "alert_id" | "source" => {}
                "slack_notification_sent" => {
                    if expected.as_bool() == Some(true)
                        && !contains_key_or_any(&self.slack_notifications, alert_id)
                    {
                        failures.push("Slack notification was not sent".to_string());
                    }
                }
                "phone_call_triggered" => {
                    if expected.as_bool() == Some(true)
                        && !bool_key_or_any(&self.phone_calls, alert_id)
                    {
                        failures.push("Phone escalation was not triggered".to_string());
                    }
                }
                "runbook_url_included" => {
                    if expected.as_bool() == Some(true)
                        && !contains_key_or_any(&self.runbook_urls, alert_id)
                    {
                        failures.push("Runbook URL was not included".to_string());
                    }
                }
                "ticket_status" => {
                    let expected_status = expected.as_str().unwrap_or_default();
                    if !ticket_status_matches(&self.tickets, alert_id, expected_status) {
                        failures.push(format!(
                            "No ticket reached expected status {expected_status}"
                        ));
                    }
                }
                _ => return Err(OpsError::UnsupportedAssertion(key.clone())),
            }
        }
        Ok(OpsAssertionOutcome {
            passed: failures.is_empty(),
            failures,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpsAlert {
    id: String,
    source: String,
    notify_slack: bool,
    notify_phone: bool,
    runbook_url: Option<String>,
}

impl OpsAlert {
    fn from_map(map: &BTreeMap<String, serde_yaml::Value>) -> Result<Self, OpsError> {
        let id = required_string(map, "id")?;
        let source = required_string(map, "source")?;
        let notify = map.get("notify").and_then(|value| value.as_mapping());
        let notify_slack = notify
            .and_then(|mapping| yaml_mapping_get(mapping, "slack"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let notify_phone = notify
            .and_then(|mapping| yaml_mapping_get(mapping, "phone"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let runbook_url = map
            .get("runbook_url")
            .and_then(|value| value.as_str())
            .map(str::to_string);
        Ok(Self {
            id,
            source,
            notify_slack,
            notify_phone,
            runbook_url,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TicketState {
    status: String,
    assignee: Option<String>,
}

fn yaml_mapping_get<'a>(
    mapping: &'a serde_yaml::Mapping,
    key: &str,
) -> Option<&'a serde_yaml::Value> {
    mapping.get(serde_yaml::Value::String(key.to_string()))
}

fn required_string(
    map: &BTreeMap<String, serde_yaml::Value>,
    key: &str,
) -> Result<String, OpsError> {
    map.get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| OpsError::InvalidAlertConfig(format!("missing or invalid {key}")))
}

fn contains_key_or_any<T>(map: &BTreeMap<String, T>, alert_id: Option<&str>) -> bool {
    match alert_id {
        Some(alert_id) => map.contains_key(alert_id),
        None => !map.is_empty(),
    }
}

fn bool_key_or_any(map: &BTreeMap<String, bool>, alert_id: Option<&str>) -> bool {
    match alert_id {
        Some(alert_id) => map.get(alert_id).copied().unwrap_or(false),
        None => map.values().any(|sent| *sent),
    }
}

fn ticket_status_matches(
    tickets: &BTreeMap<String, TicketState>,
    alert_id: Option<&str>,
    expected_status: &str,
) -> bool {
    match alert_id {
        Some(alert_id) => tickets
            .get(alert_id)
            .map(|ticket| ticket.status == expected_status)
            .unwrap_or(false),
        None => tickets
            .values()
            .any(|ticket| ticket.status == expected_status),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sauna_alerts() -> Vec<BTreeMap<String, serde_yaml::Value>> {
        serde_yaml::from_str(
            r#"
- id: sauna_emergency_button
  source: contact.sauna_emergency_button
  notify:
    slack: true
    phone: true
  runbook_url: https://example.com/runbooks/sauna-emergency
"#,
        )
        .unwrap()
    }

    #[test]
    fn critical_contact_triggers_notifications_and_runbook() {
        let mut ops = OpsModel::try_from_config(&sauna_alerts()).unwrap();

        let events = ops.apply_contact_change("sauna_emergency_button", "on");

        assert!(events
            .iter()
            .any(|event| matches!(event, OpsEvent::SlackNotificationSent { .. })));
        assert!(events
            .iter()
            .any(|event| matches!(event, OpsEvent::PhoneCallTriggered { .. })));
        assert!(events.iter().any(|event| matches!(
            event,
            OpsEvent::RunbookUrlIncluded { runbook_url, .. }
                if runbook_url == "https://example.com/runbooks/sauna-emergency"
        )));
    }

    #[test]
    fn acknowledge_updates_ticket_status() {
        let mut ops = OpsModel::try_from_config(&sauna_alerts()).unwrap();
        ops.apply_contact_change("sauna_emergency_button", "on");

        let events = ops.acknowledge(
            Some("sauna_emergency_button"),
            Some("ops_member_01".to_string()),
        );

        assert!(events.iter().any(|event| matches!(
            event,
            OpsEvent::TicketAcknowledged { assignee, .. }
                if assignee.as_deref() == Some("ops_member_01")
        )));
        let assertion = BTreeMap::from([
            (
                "alert_id".to_string(),
                serde_yaml::Value::String("sauna_emergency_button".to_string()),
            ),
            (
                "ticket_status".to_string(),
                serde_yaml::Value::String("acknowledged".to_string()),
            ),
        ]);
        assert!(ops.evaluate_assertion(&assertion).unwrap().passed);
    }

    #[test]
    fn scoped_assertion_does_not_pass_with_other_alert_state() {
        let alerts: Vec<BTreeMap<String, serde_yaml::Value>> = serde_yaml::from_str(
            r#"
- id: sauna_emergency_button
  source: contact.sauna_emergency_button
  notify:
    slack: true
- id: kitchen_alarm
  source: contact.kitchen_alarm
  notify:
    slack: true
"#,
        )
        .unwrap();
        let mut ops = OpsModel::try_from_config(&alerts).unwrap();
        ops.apply_contact_change("kitchen_alarm", "on");

        let assertion = BTreeMap::from([
            (
                "alert_id".to_string(),
                serde_yaml::Value::String("sauna_emergency_button".to_string()),
            ),
            (
                "slack_notification_sent".to_string(),
                serde_yaml::Value::Bool(true),
            ),
        ]);

        assert!(!ops.evaluate_assertion(&assertion).unwrap().passed);
    }

    #[test]
    fn validates_alert_contact_sources() {
        let ops = OpsModel::try_from_config(&sauna_alerts()).unwrap();

        assert!(ops
            .validate_sources(|contact| contact == "sauna_emergency_button")
            .is_ok());
        assert_eq!(
            ops.validate_sources(|_| false),
            Err(OpsError::UnknownAlertSource(
                "contact.sauna_emergency_button".to_string()
            ))
        );
    }

    #[test]
    fn records_synthetic_slack_notification_for_network_events() {
        let mut ops = OpsModel::default();

        ops.record_slack_notification("wan_failover", None);

        let assertion = BTreeMap::from([(
            "slack_notification_sent".to_string(),
            serde_yaml::Value::Bool(true),
        )]);
        assert!(ops.evaluate_assertion(&assertion).unwrap().passed);
    }

    #[test]
    fn rejects_malformed_alert_config() {
        let alerts: Vec<BTreeMap<String, serde_yaml::Value>> = serde_yaml::from_str(
            r#"
- source: contact.sauna_emergency_button
"#,
        )
        .unwrap();

        assert_eq!(
            OpsModel::try_from_config(&alerts),
            Err(OpsError::InvalidAlertConfig(
                "missing or invalid id".to_string()
            ))
        );
    }
}
