//! Edge (home-control) server emulator.
//!
//! Models a redundant edge pair (primary + optional secondary) and routes MQTT
//! commands through whichever edge is currently active. When the primary
//! loses power the secondary is promoted, provided failover is enabled.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors produced by the edge model.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EdgeError {
    /// Edge config contains a malformed or unsupported value.
    #[error("invalid edge config: {0}")]
    InvalidConfig(String),
    /// Neither the primary nor the secondary edge is currently active.
    #[error("no active edge server is available")]
    NoActiveEdge,
}

/// Outcome of a successful route: which edge handled the command and which
/// device/client/protocol were involved.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RoutedCommand {
    pub edge_id: String,
    pub source_client: String,
    pub target_device: String,
    pub protocol: Option<String>,
    pub action: String,
}

/// Result of a primary → secondary failover.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct FailoverOutcome {
    pub from: String,
    pub to: String,
}

/// One edge server in the redundant pair.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EdgeServer {
    pub id: String,
    pub status: EdgeStatus,
}

/// Operational state for an [`EdgeServer`].
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeStatus {
    Active,
    Standby,
    Failed,
}

/// Two-edge model with an optional secondary and a flag controlling automatic
/// failover.
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
    /// Build an [`EdgeModel`] from the scenario's `edge:` configuration map.
    ///
    /// Missing values fall back to a sensible default: a primary named
    /// `edge_primary` in [`EdgeStatus::Active`] and a secondary named
    /// `edge_secondary` in [`EdgeStatus::Standby`], with failover enabled.
    #[deprecated(note = "use try_from_config to surface malformed scenario config")]
    pub fn from_config(config: &BTreeMap<String, serde_yaml::Value>) -> Self {
        Self::try_from_config(config).unwrap_or_default()
    }

    /// Build an [`EdgeModel`] from config, returning a typed error instead of
    /// silently coercing unsupported operational states.
    pub fn try_from_config(
        config: &BTreeMap<String, serde_yaml::Value>,
    ) -> Result<Self, EdgeError> {
        let primary =
            edge_server_from_config(config.get("primary"), "edge_primary", EdgeStatus::Active)?;
        let secondary = config
            .get("secondary")
            .map(|value| {
                edge_server_from_config(Some(value), "edge_secondary", EdgeStatus::Standby)
            })
            .transpose()?;
        let failover_enabled = config
            .get("failover")
            .and_then(|value| value.as_mapping())
            .and_then(|mapping| mapping.get(serde_yaml::Value::String("enabled".to_string())))
            .and_then(|value| value.as_bool())
            .unwrap_or(true);

        Ok(Self {
            primary,
            secondary,
            failover_enabled,
        })
    }

    /// Returns the id of the currently active edge, or `None` if both are
    /// failed or standing by.
    pub fn active_id(&self) -> Option<&str> {
        if self.primary.status == EdgeStatus::Active {
            return Some(&self.primary.id);
        }
        self.secondary
            .as_ref()
            .filter(|edge| edge.status == EdgeStatus::Active)
            .map(|edge| edge.id.as_str())
    }

    /// Borrow the secondary's status, if a secondary is configured.
    pub fn secondary_status(&self) -> Option<EdgeStatus> {
        self.secondary.as_ref().map(|edge| edge.status)
    }

    /// Route an MQTT command through the active edge to a target device.
    ///
    /// Returns [`EdgeError::NoActiveEdge`] when neither edge is active.
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

    /// Simulate power loss to the primary edge.
    ///
    /// When failover is enabled and a standby secondary exists, the secondary
    /// is promoted to [`EdgeStatus::Active`] and a [`FailoverOutcome`] is
    /// returned. Returns `None` if the primary was already failed, failover is
    /// disabled, or no secondary is configured.
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
) -> Result<EdgeServer, EdgeError> {
    let Some(mapping) = value.and_then(|value| value.as_mapping()) else {
        return Ok(EdgeServer {
            id: fallback_id.to_string(),
            status: fallback_status,
        });
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
        .transpose()?
        .unwrap_or(fallback_status);

    Ok(EdgeServer { id, status })
}

fn parse_status(value: &str) -> Result<EdgeStatus, EdgeError> {
    match value {
        "active" => Ok(EdgeStatus::Active),
        "standby" => Ok(EdgeStatus::Standby),
        "failed" => Ok(EdgeStatus::Failed),
        _ => Err(EdgeError::InvalidConfig(format!(
            "unsupported edge status {value}; expected active, standby, or failed"
        ))),
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
    fn routing_fails_when_no_edge_is_active() {
        let edge = EdgeModel {
            primary: EdgeServer {
                id: "edge_primary".to_string(),
                status: EdgeStatus::Failed,
            },
            secondary: None,
            failover_enabled: true,
        };

        let error = edge
            .route_mqtt_command("ipad_controller", "living_light", None)
            .unwrap_err();

        assert_eq!(error, EdgeError::NoActiveEdge);
    }

    #[test]
    fn power_loss_with_failover_disabled_does_not_promote_secondary() {
        let mut edge = EdgeModel {
            primary: EdgeServer {
                id: "edge_primary".to_string(),
                status: EdgeStatus::Active,
            },
            secondary: Some(EdgeServer {
                id: "edge_secondary".to_string(),
                status: EdgeStatus::Standby,
            }),
            failover_enabled: false,
        };

        let outcome = edge.apply_power_lost_to_primary();

        assert!(outcome.is_none());
        assert_eq!(edge.secondary_status(), Some(EdgeStatus::Standby));
        assert!(edge.active_id().is_none());
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
