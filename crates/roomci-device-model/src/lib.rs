use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub type StateMap = BTreeMap<String, serde_json::Value>;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RoomDefinition {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub devices: Vec<DeviceDefinition>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DeviceDefinition {
    pub id: String,
    #[serde(rename = "type")]
    pub device_type: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub initial_state: BTreeMap<String, serde_yaml::Value>,
}

pub fn yaml_state_to_json(state: &BTreeMap<String, serde_yaml::Value>) -> StateMap {
    state
        .iter()
        .map(|(key, value)| {
            let json_value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
            (key.clone(), json_value)
        })
        .collect()
}

pub fn command_is_supported(device_type: &str, action: &str) -> bool {
    match device_type {
        "smart_lock" => matches!(action, "lock" | "unlock"),
        "light" => matches!(action, "turn_on" | "turn_off" | "set_brightness"),
        "climate" => matches!(action, "set_mode" | "set_temperature"),
        "cover" => matches!(action, "open" | "close" | "set_position"),
        "room_controller" => matches!(action, "activate_scene"),
        _ => false,
    }
}

pub fn apply_command_state(
    device_type: &str,
    action: &str,
    value: Option<&serde_json::Value>,
    state: &mut StateMap,
) {
    match (device_type, action) {
        ("smart_lock", "lock") => {
            state.insert("lock_state".to_string(), serde_json::json!("locked"));
        }
        ("smart_lock", "unlock") => {
            state.insert("lock_state".to_string(), serde_json::json!("unlocked"));
        }
        ("light", "turn_on") => {
            state.insert("power".to_string(), serde_json::json!("on"));
        }
        ("light", "turn_off") => {
            state.insert("power".to_string(), serde_json::json!("off"));
        }
        ("light", "set_brightness") => {
            if let Some(value) = value {
                state.insert("brightness".to_string(), value.clone());
            }
        }
        ("climate", "set_mode") => {
            if let Some(value) = value {
                state.insert("mode".to_string(), value.clone());
            }
        }
        ("climate", "set_temperature") => {
            if let Some(value) = value {
                state.insert("setpoint_celsius".to_string(), value.clone());
            }
        }
        ("cover", "open") => {
            state.insert("position".to_string(), serde_json::json!(100));
        }
        ("cover", "close") => {
            state.insert("position".to_string(), serde_json::json!(0));
        }
        ("cover", "set_position") => {
            if let Some(value) = value {
                state.insert("position".to_string(), value.clone());
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smart_lock_unlock_updates_state() {
        let mut state = StateMap::new();

        apply_command_state("smart_lock", "unlock", None, &mut state);

        assert_eq!(
            state.get("lock_state"),
            Some(&serde_json::json!("unlocked"))
        );
    }

    #[test]
    fn climate_temperature_updates_setpoint() {
        let mut state = StateMap::new();

        apply_command_state(
            "climate",
            "set_temperature",
            Some(&serde_json::json!(24)),
            &mut state,
        );

        assert_eq!(state.get("setpoint_celsius"), Some(&serde_json::json!(24)));
    }

    #[test]
    fn unsupported_command_is_rejected_by_capability_check() {
        assert!(!command_is_supported("smart_lock", "set_temperature"));
    }
}
