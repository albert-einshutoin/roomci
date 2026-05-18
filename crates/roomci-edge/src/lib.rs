use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EdgeError {
    #[error("no active edge server is available")]
    NoActiveEdge,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RoutedCommand {
    pub edge_id: String,
    pub source_client: String,
    pub target_device: String,
    pub protocol: Option<String>,
    pub action: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct FailoverOutcome {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EdgeServer {
    pub id: String,
    pub status: EdgeStatus,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeStatus {
    Active,
    Standby,
    Failed,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EdgeModel {
    primary: EdgeServer,
    secondary: Option<EdgeServer>,
    failover_enabled: bool,
}

impl Default for EdgeModel {
    fn default() -> Self {
        Self {
            primary: EdgeServer {
                id: "edge_primary".to_string(),
                status: EdgeStatus::Active,
            },
            secondary: Some(EdgeServer {
                id: "edge_secondary".to_string(),
                status: EdgeStatus::Standby,
            }),
            failover_enabled: true,
        }
    }
}

impl EdgeModel {
    pub fn from_config(config: &BTreeMap<String, serde_yaml::Value>) -> Self {
        let primary =
            edge_server_from_config(config.get("primary"), "edge_primary", EdgeStatus::Active);
        let secondary = config.get("secondary").map(|value| {
            edge_server_from_config(Some(value), "edge_secondary", EdgeStatus::Standby)
        });
        let failover_enabled = config
            .get("failover")
            .and_then(|value| value.as_mapping())
            .and_then(|mapping| mapping.get(serde_yaml::Value::String("enabled".to_string())))
            .and_then(|value| value.as_bool())
            .unwrap_or(true);

        Self {
            primary,
            secondary,
            failover_enabled,
        }
    }

    pub fn active_id(&self) -> Option<&str> {
        if self.primary.status == EdgeStatus::Active {
            return Some(&self.primary.id);
        }
        self.secondary
            .as_ref()
            .filter(|edge| edge.status == EdgeStatus::Active)
            .map(|edge| edge.id.as_str())
    }

    pub fn secondary_status(&self) -> Option<EdgeStatus> {
        self.secondary.as_ref().map(|edge| edge.status)
    }

    pub fn route_mqtt_command(
        &self,
        source_client: &str,
        target_device: &str,
        protocol: Option<String>,
    ) -> Result<RoutedCommand, EdgeError> {
        let edge_id = self.active_id().ok_or(EdgeError::NoActiveEdge)?.to_string();
        Ok(RoutedCommand {
            edge_id,
            source_client: source_client.to_string(),
            target_device: target_device.to_string(),
            protocol,
            action: "mqtt_command_route".to_string(),
        })
    }

    pub fn apply_power_lost_to_primary(&mut self) -> Option<FailoverOutcome> {
        if self.primary.status == EdgeStatus::Failed {
            return None;
        }
        self.primary.status = EdgeStatus::Failed;
        if !self.failover_enabled {
            return None;
        }
        let secondary = self.secondary.as_mut()?;
        secondary.status = EdgeStatus::Active;
        Some(FailoverOutcome {
            from: self.primary.id.clone(),
            to: secondary.id.clone(),
        })
    }
}

fn edge_server_from_config(
    value: Option<&serde_yaml::Value>,
    fallback_id: &str,
    fallback_status: EdgeStatus,
) -> EdgeServer {
    let Some(mapping) = value.and_then(|value| value.as_mapping()) else {
        return EdgeServer {
            id: fallback_id.to_string(),
            status: fallback_status,
        };
    };
    let id = mapping
        .get(serde_yaml::Value::String("id".to_string()))
        .and_then(|value| value.as_str())
        .unwrap_or(fallback_id)
        .to_string();
    let status = mapping
        .get(serde_yaml::Value::String("status".to_string()))
        .and_then(|value| value.as_str())
        .map(parse_status)
        .unwrap_or(fallback_status);

    EdgeServer { id, status }
}

fn parse_status(value: &str) -> EdgeStatus {
    match value {
        "active" => EdgeStatus::Active,
        "standby" => EdgeStatus::Standby,
        "failed" => EdgeStatus::Failed,
        _ => EdgeStatus::Standby,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_command_through_active_edge() {
        let edge = EdgeModel::default();

        let routed = edge
            .route_mqtt_command("ipad_controller", "living_light", Some("dali".to_string()))
            .unwrap();

        assert_eq!(routed.edge_id, "edge_primary");
        assert_eq!(routed.target_device, "living_light");
    }

    #[test]
    fn primary_power_loss_activates_secondary() {
        let mut edge = EdgeModel::default();

        let outcome = edge.apply_power_lost_to_primary().unwrap();

        assert_eq!(outcome.from, "edge_primary");
        assert_eq!(outcome.to, "edge_secondary");
        assert_eq!(edge.secondary_status(), Some(EdgeStatus::Active));
    }
}
