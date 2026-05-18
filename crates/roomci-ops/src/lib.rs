use std::collections::BTreeMap;

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OpsError {
    #[error("unknown alert source {0}")]
    UnknownAlertSource(String),
    #[error("unsupported ops assertion {0}")]
    UnsupportedAssertion(String),
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpsAssertionOutcome {
    pub passed: bool,
    pub failures: Vec<String>,
}

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
        let alerts = alerts
            .iter()
            .filter_map(OpsAlert::from_map)
            .collect::<Vec<_>>();
        Self {
            alerts,
            tickets: BTreeMap::new(),
            slack_notifications: BTreeMap::new(),
            phone_calls: BTreeMap::new(),
            runbook_urls: BTreeMap::new(),
        }
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

    pub fn acknowledge(&mut self, assignee: Option<String>) -> Vec<OpsEvent> {
        let mut events = Vec::new();
        for (alert_id, ticket) in &mut self.tickets {
            ticket.status = "acknowledged".to_string();
            ticket.assignee = assignee.clone();
            events.push(OpsEvent::TicketAcknowledged {
                alert_id: alert_id.clone(),
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
        for (key, expected) in assertion {
            match key.as_str() {
                "slack_notification_sent" => {
                    if expected.as_bool() == Some(true) && self.slack_notifications.is_empty() {
                        failures.push("Slack notification was not sent".to_string());
                    }
                }
                "phone_call_triggered" => {
                    if expected.as_bool() == Some(true)
                        && !self.phone_calls.values().any(|sent| *sent)
                    {
                        failures.push("Phone escalation was not triggered".to_string());
                    }
                }
                "runbook_url_included" => {
                    if expected.as_bool() == Some(true) && self.runbook_urls.is_empty() {
                        failures.push("Runbook URL was not included".to_string());
                    }
                }
                "ticket_status" => {
                    let expected_status = expected.as_str().unwrap_or_default();
                    if !self
                        .tickets
                        .values()
                        .any(|ticket| ticket.status == expected_status)
                    {
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
    fn from_map(map: &BTreeMap<String, serde_yaml::Value>) -> Option<Self> {
        let id = map.get("id")?.as_str()?.to_string();
        let source = map.get("source")?.as_str()?.to_string();
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
        Some(Self {
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
        let mut ops = OpsModel::from_config(&sauna_alerts());

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
        let mut ops = OpsModel::from_config(&sauna_alerts());
        ops.apply_contact_change("sauna_emergency_button", "on");

        let events = ops.acknowledge(Some("ops_member_01".to_string()));

        assert!(events.iter().any(|event| matches!(
            event,
            OpsEvent::TicketAcknowledged { assignee, .. }
                if assignee.as_deref() == Some("ops_member_01")
        )));
        let assertion = BTreeMap::from([(
            "ticket_status".to_string(),
            serde_yaml::Value::String("acknowledged".to_string()),
        )]);
        assert!(ops.evaluate_assertion(&assertion).unwrap().passed);
    }

    #[test]
    fn validates_alert_contact_sources() {
        let ops = OpsModel::from_config(&sauna_alerts());

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
}
