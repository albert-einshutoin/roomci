use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Payload = BTreeMap<String, serde_json::Value>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MqttError {
    #[error("broker {0} is offline")]
    BrokerOffline(String),
    #[error("topic is not a device command topic: {0}")]
    InvalidCommandTopic(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PublishOutcome {
    pub broker: String,
    pub command_topic: String,
    pub state_topic: Option<String>,
    pub device_id: Option<String>,
    pub retained_payload: Option<Payload>,
    pub deliveries: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct BrokerModel {
    online: BTreeMap<String, bool>,
    retained: BTreeMap<String, Payload>,
    client_inboxes: BTreeMap<String, BTreeMap<String, Payload>>,
}

impl BrokerModel {
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

    pub fn is_online(&self, broker: &str) -> bool {
        self.online.get(broker).copied().unwrap_or(false)
    }

    pub fn set_online(&mut self, broker: impl Into<String>, online: bool) {
        self.online.insert(broker.into(), online);
    }

    pub fn retained(&self) -> &BTreeMap<String, Payload> {
        &self.retained
    }

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

        // QoS1 can deliver the same command multiple times. For retained state,
        // the semantic result is the final payload, so repeated deliveries are idempotent.
        let delivery_count = deliveries.max(1);
        for _ in 0..delivery_count {
            self.retained.insert(state_topic.clone(), payload.clone());
        }
        self.client_inboxes
            .entry(client.to_string())
            .or_default()
            .insert(state_topic.clone(), payload.clone());

        Ok(PublishOutcome {
            broker: broker.to_string(),
            command_topic: topic.to_string(),
            state_topic: Some(state_topic),
            device_id: Some(device_id),
            retained_payload: Some(payload),
            deliveries: delivery_count,
        })
    }

    pub fn reconnect_client(&mut self, client: &str) -> BTreeMap<String, Payload> {
        let retained = self.retained.clone();
        self.client_inboxes
            .insert(client.to_string(), retained.clone());
        retained
    }
}

pub fn device_id_from_command_topic(topic: &str) -> Option<String> {
    let parts = topic.split('/').collect::<Vec<_>>();
    if parts.last().copied() != Some("command") {
        return None;
    }
    let device_index = parts.iter().position(|part| *part == "device")?;
    parts
        .get(device_index + 1)
        .map(|value| (*value).to_string())
}

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
