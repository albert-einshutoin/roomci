//! Local and cloud MQTT broker model used by roomci scenarios.
//!
//! The model is deliberately simplified: it tracks online state per broker,
//! retained payloads per state topic, and a per-client inbox so reconnect
//! semantics can be exercised. It is not an MQTT protocol implementation;
//! it captures the QA-visible behavior that local-first scenarios depend on.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// JSON-shaped payload used for both device commands and retained state.
pub type Payload = BTreeMap<String, serde_json::Value>;

/// Errors produced when publishing through the broker model.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MqttError {
    /// The target broker is offline at publish time.
    #[error("broker {0} is offline")]
    BrokerOffline(String),
    /// The topic does not match the expected `house/.../device/<id>/command` shape.
    #[error("topic is not a device command topic: {0}")]
    InvalidCommandTopic(String),
}

/// Result of a successful publish: which broker accepted it, the derived state
/// topic, the device id (when the topic matched the device-command shape), and
/// the number of QoS1-style deliveries that were applied.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PublishOutcome {
    pub broker: String,
    pub command_topic: String,
    pub state_topic: Option<String>,
    pub device_id: Option<String>,
    pub retained_payload: Option<Payload>,
    pub deliveries: u32,
}

/// Fully matched device-command publish produced by a scenario MQTT contract.
pub struct DeviceCommandPublish<'a> {
    pub broker: &'a str,
    pub client: &'a str,
    pub command_topic: &'a str,
    pub device_id: &'a str,
    pub state_topic: &'a str,
    pub payload: Payload,
    pub deliveries: u32,
}

/// In-memory broker model for the local and cloud MQTT brokers.
///
/// Use [`BrokerModel::new`] to construct one with explicit online states, then
/// drive it with [`BrokerModel::publish_device_command`] from scenario steps.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct BrokerModel {
    online: BTreeMap<String, bool>,
    retained: BTreeMap<String, Payload>,
    client_inboxes: BTreeMap<String, BTreeMap<String, Payload>>,
}

impl BrokerModel {
    /// Build a broker model with explicit online states for the local and
    /// cloud brokers.
    pub fn new(local_online: bool, cloud_online: bool) -> Self {
        let mut online = BTreeMap::new();
        online.insert("mqtt.local".to_string(), local_online);
        online.insert("mqtt.cloud".to_string(), cloud_online);
        Self {
            online,
            retained: BTreeMap::new(),
            client_inboxes: BTreeMap::new(),
        }
    }

    /// Returns whether the named broker is currently online.
    pub fn is_online(&self, broker: &str) -> bool {
        self.online.get(broker).copied().unwrap_or(false)
    }

    /// Toggle the online state for a named broker (used when a fault activates
    /// or recovers).
    pub fn set_online(&mut self, broker: impl Into<String>, online: bool) {
        self.online.insert(broker.into(), online);
    }

    /// Borrow the retained-state map, keyed by state topic.
    pub fn retained(&self) -> &BTreeMap<String, Payload> {
        &self.retained
    }

    /// Publish a device command. The command topic must match
    /// `.../device/<id>/command`; the matching state topic is derived from it.
    ///
    /// QoS1 may deliver the same payload multiple times — for retained state
    /// this is idempotent, so `deliveries` only affects the recorded
    /// [`PublishOutcome::deliveries`] count.
    pub fn publish_device_command(
        &mut self,
        broker: &str,
        client: &str,
        topic: &str,
        payload: Payload,
        deliveries: u32,
    ) -> Result<PublishOutcome, MqttError> {
        if !self.is_online(broker) {
            return Err(MqttError::BrokerOffline(broker.to_string()));
        }
        let device_id = device_id_from_command_topic(topic)
            .ok_or_else(|| MqttError::InvalidCommandTopic(topic.to_string()))?;
        let state_topic = state_topic_for_command_topic(topic)
            .ok_or_else(|| MqttError::InvalidCommandTopic(topic.to_string()))?;
        self.publish_device_command_with_topics(DeviceCommandPublish {
            broker,
            client,
            command_topic: topic,
            device_id: &device_id,
            state_topic: &state_topic,
            payload,
            deliveries,
        })
    }

    /// Publish a device command after a scenario MQTT contract has already
    /// matched the concrete command and state topics.
    pub fn publish_device_command_with_topics(
        &mut self,
        publish: DeviceCommandPublish<'_>,
    ) -> Result<PublishOutcome, MqttError> {
        if !self.is_online(publish.broker) {
            return Err(MqttError::BrokerOffline(publish.broker.to_string()));
        }
        if publish.device_id.is_empty()
            || publish.device_id.contains('/')
            || publish.state_topic.is_empty()
        {
            return Err(MqttError::InvalidCommandTopic(
                publish.command_topic.to_string(),
            ));
        }
        // QoS1 can deliver the same command multiple times. For retained state,
        // the semantic result is the final payload, so repeated deliveries are idempotent.
        let delivery_count = publish.deliveries.max(1);
        for _ in 0..delivery_count {
            self.retained
                .insert(publish.state_topic.to_string(), publish.payload.clone());
        }
        self.client_inboxes
            .entry(publish.client.to_string())
            .or_default()
            .insert(publish.state_topic.to_string(), publish.payload.clone());

        Ok(PublishOutcome {
            broker: publish.broker.to_string(),
            command_topic: publish.command_topic.to_string(),
            state_topic: Some(publish.state_topic.to_string()),
            device_id: Some(publish.device_id.to_string()),
            retained_payload: Some(publish.payload),
            deliveries: delivery_count,
        })
    }

    /// Simulate a client reconnect: the client receives every retained state
    /// topic in one go and the inbox is replaced.
    pub fn reconnect_client(&mut self, client: &str) -> BTreeMap<String, Payload> {
        let retained = self.retained.clone();
        self.client_inboxes
            .insert(client.to_string(), retained.clone());
        retained
    }
}

/// Extract the device id from a `.../device/<id>/command` topic, or return
/// `None` if the topic does not match.
pub fn device_id_from_command_topic(topic: &str) -> Option<String> {
    let parts = topic.split('/').collect::<Vec<_>>();
    if parts.last().copied() != Some("command") {
        return None;
    }
    let device_index = parts.iter().position(|part| *part == "device")?;
    if device_index + 2 != parts.len() - 1 {
        return None;
    }
    let value = parts.get(device_index + 1)?;
    if value.is_empty() {
        return None;
    }
    Some((*value).to_string())
}

/// Derive the state topic that mirrors a `.../command` topic, or return
/// `None` when the suffix does not match.
pub fn state_topic_for_command_topic(topic: &str) -> Option<String> {
    topic
        .strip_suffix("/command")
        .map(|prefix| format!("{prefix}/state"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload() -> Payload {
        BTreeMap::from([
            ("power".to_string(), serde_json::json!(true)),
            ("brightness".to_string(), serde_json::json!(60)),
        ])
    }

    #[test]
    fn retained_state_survives_cloud_outage() {
        let mut broker = BrokerModel::new(true, true);
        broker.set_online("mqtt.cloud", false);

        let outcome = broker
            .publish_device_command(
                "mqtt.local",
                "ipad_controller",
                "house/minakami/room/living/device/living_light/command",
                payload(),
                1,
            )
            .unwrap();

        assert_eq!(outcome.device_id.as_deref(), Some("living_light"));
        assert!(broker
            .retained()
            .contains_key("house/minakami/room/living/device/living_light/state"));
    }

    #[test]
    fn duplicate_delivery_is_idempotent_for_retained_state() {
        let mut broker = BrokerModel::new(true, false);
        let topic = "house/minakami/room/living/device/living_light/command";

        let outcome = broker
            .publish_device_command("mqtt.local", "ipad_controller", topic, payload(), 3)
            .unwrap();

        assert_eq!(outcome.deliveries, 3);
        assert_eq!(broker.retained().len(), 1);
        assert_eq!(
            broker
                .retained()
                .get("house/minakami/room/living/device/living_light/state"),
            Some(&payload())
        );
    }

    #[test]
    fn publishing_to_offline_broker_returns_error() {
        let mut broker = BrokerModel::new(false, false);

        let error = broker
            .publish_device_command(
                "mqtt.local",
                "ipad_controller",
                "house/minakami/room/living/device/living_light/command",
                payload(),
                1,
            )
            .unwrap_err();

        assert_eq!(error, MqttError::BrokerOffline("mqtt.local".to_string()));
    }

    #[test]
    fn non_device_command_topic_is_rejected() {
        let mut broker = BrokerModel::new(true, false);

        let error = broker
            .publish_device_command(
                "mqtt.local",
                "ipad_controller",
                "house/minakami/room/living/status",
                payload(),
                1,
            )
            .unwrap_err();

        assert!(matches!(error, MqttError::InvalidCommandTopic(_)));
    }

    #[test]
    fn malformed_device_command_topic_is_rejected() {
        assert_eq!(device_id_from_command_topic("house/device//command"), None);
        assert_eq!(
            device_id_from_command_topic("house/device/living_light/extra/command"),
            None
        );
    }

    #[test]
    fn reconnect_receives_retained_state() {
        let mut broker = BrokerModel::new(true, false);
        broker
            .publish_device_command(
                "mqtt.local",
                "ipad_controller",
                "house/minakami/room/living/device/living_light/command",
                payload(),
                1,
            )
            .unwrap();

        let inbox = broker.reconnect_client("ipad_controller");

        assert_eq!(inbox.len(), 1);
    }
}
