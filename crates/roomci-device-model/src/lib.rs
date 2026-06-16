//! Device-level mocks used by roomci scenarios.
//!
//! This crate provides three protocol-agnostic models that the scenario
//! runner drives:
//!
//! - [`ModbusModel`] for Modbus holding/input register devices (floor
//!   heating, HVAC, etc.).
//! - [`LightingModel`] for DALI-like lighting fixtures with per-fixture
//!   levels, scene targets, and command-drop fault injection.
//! - [`ContactModel`] for binary contact inputs (emergency buttons, door
//!   sensors).
//!
//! Each model validates its config and surfaces structured
//! [`DeviceModelError`] variants so scenario authors get actionable error
//! messages.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

/// Per-device state map. Mirrors `roomci_core::StateMap` and is used by
/// [`apply_command_state`] to mutate state in place.
pub type StateMap = BTreeMap<String, serde_json::Value>;

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

/// Returns whether `action` on `device_type` is meaningless without a `value`
/// payload (e.g. `set_brightness`). The runtime rejects such commands when no
/// value is supplied instead of silently treating them as a no-op.
pub fn command_requires_value(device_type: &str, action: &str) -> bool {
    matches!(
        (device_type, action),
        ("light", "set_brightness")
            | ("climate", "set_mode")
            | ("climate", "set_temperature")
            | ("cover", "set_position")
    )
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

/// Errors emitted by the Modbus, lighting, and contact models when a
/// scenario references an unknown device or writes a read-only register.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DeviceModelError {
    #[error("invalid Modbus config: {0}")]
    InvalidModbusConfig(String),
    #[error("unknown Modbus device {0}")]
    UnknownModbusDevice(String),
    #[error("unknown Modbus register {device}:{register}")]
    UnknownModbusRegister { device: String, register: u32 },
    #[error("Modbus register {device}:{register} is read-only")]
    ReadOnlyModbusRegister { device: String, register: u32 },
    #[error("invalid DALI lighting config: {0}")]
    InvalidLightingConfig(String),
    #[error("unknown DALI scene {0}")]
    UnknownScene(String),
    #[error("unknown DALI fixture {0}")]
    UnknownFixture(String),
    #[error("invalid contact config: {0}")]
    InvalidContactConfig(String),
    #[error("unknown contact input {0}")]
    UnknownContact(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModbusRegisterAccess {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModbusRegister {
    pub value: serde_json::Value,
    pub value_type: Option<String>,
    pub access: ModbusRegisterAccess,
}

/// Modbus device model: stores per-device register maps and enforces
/// read-only access for `input_registers` / `discrete_inputs`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModbusModel {
    devices: BTreeMap<String, BTreeMap<u32, ModbusRegister>>,
}

impl ModbusModel {
    #[deprecated(note = "use try_from_config to surface malformed scenario config")]
    pub fn from_config(modbus: &BTreeMap<String, serde_yaml::Value>) -> Self {
        Self::try_from_config(modbus).unwrap_or_default()
    }

    pub fn try_from_config(
        modbus: &BTreeMap<String, serde_yaml::Value>,
    ) -> Result<Self, DeviceModelError> {
        let mut devices = BTreeMap::new();
        let Some(device_values) = modbus.get("devices").and_then(|value| value.as_sequence())
        else {
            return Ok(Self { devices });
        };
        for device_value in device_values {
            let device_map = device_value.as_mapping().ok_or_else(|| {
                DeviceModelError::InvalidModbusConfig(
                    "devices[] entries must be YAML maps".to_string(),
                )
            })?;
            let Some(device_id) =
                yaml_mapping_get(device_map, "id").and_then(|value| value.as_str())
            else {
                return Err(DeviceModelError::InvalidModbusConfig(
                    "devices[].id must be a string".to_string(),
                ));
            };
            let registers = devices
                .entry(device_id.to_string())
                .or_insert_with(BTreeMap::new);
            for (section, access) in [
                ("holding_registers", ModbusRegisterAccess::ReadWrite),
                ("coils", ModbusRegisterAccess::ReadWrite),
                ("input_registers", ModbusRegisterAccess::ReadOnly),
                ("discrete_inputs", ModbusRegisterAccess::ReadOnly),
            ] {
                let Some(section_map) =
                    yaml_mapping_get(device_map, section).and_then(|value| value.as_mapping())
                else {
                    continue;
                };
                for (address, definition) in section_map {
                    let address = yaml_key_u32(address).ok_or_else(|| {
                        DeviceModelError::InvalidModbusConfig(format!(
                            "register address in {device_id}.{section} must be u32"
                        ))
                    })?;
                    let definition_map = definition.as_mapping().ok_or_else(|| {
                        DeviceModelError::InvalidModbusConfig(format!(
                            "register {device_id}:{address} definition must be a YAML map"
                        ))
                    })?;
                    let value = yaml_mapping_get(definition_map, "value").ok_or_else(|| {
                        DeviceModelError::InvalidModbusConfig(format!(
                            "register {device_id}:{address} must declare value"
                        ))
                    })?;
                    let value_type = yaml_mapping_get(definition_map, "type")
                        .and_then(|value| value.as_str())
                        .map(str::to_string);
                    registers.insert(
                        address,
                        ModbusRegister {
                            value: serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
                            value_type,
                            access: access.clone(),
                        },
                    );
                }
            }
        }
        Ok(Self { devices })
    }

    pub fn assert_writable(&self, device: &str, register: u32) -> Result<(), DeviceModelError> {
        let register_model = self.register(device, register)?;
        if register_model.access == ModbusRegisterAccess::ReadOnly {
            return Err(DeviceModelError::ReadOnlyModbusRegister {
                device: device.to_string(),
                register,
            });
        }
        Ok(())
    }

    pub fn has_register(&self, device: &str, register: u32) -> bool {
        self.devices
            .get(device)
            .and_then(|registers| registers.get(&register))
            .is_some()
    }

    pub fn write(
        &mut self,
        device: &str,
        register: u32,
        value: serde_yaml::Value,
    ) -> Result<serde_json::Value, DeviceModelError> {
        self.assert_writable(device, register)?;
        let json_value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
        let register_model = self.register_mut(device, register)?;
        register_model.value = json_value.clone();
        Ok(json_value)
    }

    pub fn value(&self, device: &str, register: u32) -> Option<&serde_json::Value> {
        self.devices
            .get(device)
            .and_then(|registers| registers.get(&register))
            .map(|register| &register.value)
    }

    pub fn readable_value(&self, device: &str, register: u32) -> Option<f64> {
        let register = self.devices.get(device)?.get(&register)?;
        let raw = register
            .value
            .as_f64()
            .or_else(|| register.value.as_i64().map(|value| value as f64))?;
        match register.value_type.as_deref() {
            Some("decimal_0_1") => Some(raw / 10.0),
            _ => Some(raw),
        }
    }

    fn register(&self, device: &str, register: u32) -> Result<&ModbusRegister, DeviceModelError> {
        let Some(registers) = self.devices.get(device) else {
            return Err(DeviceModelError::UnknownModbusDevice(device.to_string()));
        };
        registers
            .get(&register)
            .ok_or_else(|| DeviceModelError::UnknownModbusRegister {
                device: device.to_string(),
                register,
            })
    }

    fn register_mut(
        &mut self,
        device: &str,
        register: u32,
    ) -> Result<&mut ModbusRegister, DeviceModelError> {
        let Some(registers) = self.devices.get_mut(device) else {
            return Err(DeviceModelError::UnknownModbusDevice(device.to_string()));
        };
        registers
            .get_mut(&register)
            .ok_or_else(|| DeviceModelError::UnknownModbusRegister {
                device: device.to_string(),
                register,
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LightingEvent {
    LevelChanged { fixture: String, level: i64 },
    CommandDropped { fixture: String },
}

/// DALI-like lighting model: per-fixture levels, named scene targets, and a
/// drop-list used to simulate ballast command failures.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LightingModel {
    levels: BTreeMap<String, i64>,
    scene_targets: BTreeMap<String, BTreeMap<String, i64>>,
    command_drops: BTreeSet<String>,
}

impl LightingModel {
    #[deprecated(note = "use try_from_config to surface malformed scenario config")]
    pub fn from_config(
        lighting: &BTreeMap<String, serde_yaml::Value>,
        scenes: &BTreeMap<String, BTreeMap<String, i64>>,
    ) -> Self {
        Self::try_from_config(lighting, scenes).unwrap_or_else(|_| Self {
            levels: BTreeMap::new(),
            scene_targets: scenes.clone(),
            command_drops: BTreeSet::new(),
        })
    }

    pub fn try_from_config(
        lighting: &BTreeMap<String, serde_yaml::Value>,
        scenes: &BTreeMap<String, BTreeMap<String, i64>>,
    ) -> Result<Self, DeviceModelError> {
        let mut levels = BTreeMap::new();
        let Some(fixtures) = lighting
            .get("fixtures")
            .and_then(|value| value.as_sequence())
        else {
            return Ok(Self {
                levels,
                scene_targets: scenes.clone(),
                command_drops: BTreeSet::new(),
            });
        };
        for fixture in fixtures {
            let mapping = fixture.as_mapping().ok_or_else(|| {
                DeviceModelError::InvalidLightingConfig(
                    "fixtures[] entries must be YAML maps".to_string(),
                )
            })?;
            let Some(id) = yaml_mapping_get(mapping, "id").and_then(|value| value.as_str()) else {
                return Err(DeviceModelError::InvalidLightingConfig(
                    "fixtures[].id must be a string".to_string(),
                ));
            };
            let level = yaml_mapping_get(mapping, "level")
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            levels.insert(id.to_string(), level);
        }
        Ok(Self {
            levels,
            scene_targets: scenes.clone(),
            command_drops: BTreeSet::new(),
        })
    }

    pub fn assert_fixture_exists(&self, fixture: &str) -> Result<(), DeviceModelError> {
        if self.levels.contains_key(fixture) {
            Ok(())
        } else {
            Err(DeviceModelError::UnknownFixture(fixture.to_string()))
        }
    }

    pub fn assert_scene_exists(&self, scene: &str) -> Result<(), DeviceModelError> {
        if self.scene_targets.contains_key(scene) {
            Ok(())
        } else {
            Err(DeviceModelError::UnknownScene(scene.to_string()))
        }
    }

    pub fn assert_scene_targets_exist(&self) -> Result<(), DeviceModelError> {
        for targets in self.scene_targets.values() {
            for fixture in targets.keys() {
                self.assert_fixture_exists(fixture)?;
            }
        }
        Ok(())
    }

    pub fn drop_command_for_fixture(&mut self, fixture: &str) -> Result<(), DeviceModelError> {
        self.assert_fixture_exists(fixture)?;
        self.command_drops.insert(fixture.to_string());
        Ok(())
    }

    pub fn activate_scene(&mut self, scene: &str) -> Result<Vec<LightingEvent>, DeviceModelError> {
        let Some(targets) = self.scene_targets.get(scene).cloned() else {
            return Err(DeviceModelError::UnknownScene(scene.to_string()));
        };
        let mut events = Vec::new();
        for (fixture, level) in targets {
            if self.command_drops.contains(&fixture) {
                events.push(LightingEvent::CommandDropped { fixture });
            } else {
                self.levels.insert(fixture.clone(), level);
                events.push(LightingEvent::LevelChanged { fixture, level });
            }
        }
        Ok(events)
    }

    pub fn scene_consistency_failures(&self, scene: &str) -> Result<Vec<String>, DeviceModelError> {
        let Some(targets) = self.scene_targets.get(scene) else {
            return Err(DeviceModelError::UnknownScene(scene.to_string()));
        };
        let mut failures = Vec::new();
        for (fixture, expected) in targets {
            let actual = self.levels.get(fixture).copied().unwrap_or(0);
            if actual != *expected {
                failures.push(format!(
                    "{fixture} expected level {expected}, actual {actual}"
                ));
            }
        }
        Ok(failures)
    }
}

/// Contact-input model: tracks declared contacts and their on/off state.
/// Used together with `roomci_ops` to trigger alerts on state changes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContactModel {
    states: BTreeMap<String, String>,
}

impl ContactModel {
    #[deprecated(note = "use try_from_config to surface malformed scenario config")]
    pub fn from_config(contacts: &BTreeMap<String, serde_yaml::Value>) -> Self {
        Self::try_from_config(contacts).unwrap_or_default()
    }

    pub fn try_from_config(
        contacts: &BTreeMap<String, serde_yaml::Value>,
    ) -> Result<Self, DeviceModelError> {
        let mut states = BTreeMap::new();
        let Some(inputs) = contacts.get("inputs").and_then(|value| value.as_sequence()) else {
            return Ok(Self { states });
        };
        for input in inputs {
            let mapping = input.as_mapping().ok_or_else(|| {
                DeviceModelError::InvalidContactConfig(
                    "inputs[] entries must be YAML maps".to_string(),
                )
            })?;
            let Some(id) = yaml_mapping_get(mapping, "id").and_then(|value| value.as_str()) else {
                return Err(DeviceModelError::InvalidContactConfig(
                    "inputs[].id must be a string".to_string(),
                ));
            };
            let state = yaml_mapping_get(mapping, "state")
                .and_then(|value| value.as_str())
                .unwrap_or("off");
            states.insert(id.to_string(), state.to_string());
        }
        Ok(Self { states })
    }

    pub fn set_state(&mut self, id: &str, state: &str) -> Result<(), DeviceModelError> {
        if !self.states.contains_key(id) {
            return Err(DeviceModelError::UnknownContact(id.to_string()));
        }
        self.states.insert(id.to_string(), state.to_string());
        Ok(())
    }

    pub fn has_contact(&self, id: &str) -> bool {
        self.states.contains_key(id)
    }
}

fn yaml_mapping_get<'a>(
    mapping: &'a serde_yaml::Mapping,
    key: &str,
) -> Option<&'a serde_yaml::Value> {
    mapping.get(serde_yaml::Value::String(key.to_string()))
}

fn yaml_key_u32(value: &serde_yaml::Value) -> Option<u32> {
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| value.as_str().and_then(|value| value.parse::<u32>().ok()))
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

    #[test]
    fn modbus_decimal_readable_value_uses_register_metadata() {
        let model = ModbusModel::try_from_config(
            &serde_yaml::from_str(
                r#"
devices:
  - id: floor_heating
    holding_registers:
      40001:
        type: decimal_0_1
        value: 245
"#,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(model.readable_value("floor_heating", 40001), Some(24.5));
    }

    #[test]
    fn modbus_rejects_writes_to_read_only_registers() {
        let mut model = ModbusModel::try_from_config(
            &serde_yaml::from_str(
                r#"
devices:
  - id: floor_heating
    input_registers:
      30001:
        type: decimal_0_1
        value: 228
"#,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            model.write("floor_heating", 30001, serde_yaml::Value::from(230)),
            Err(DeviceModelError::ReadOnlyModbusRegister {
                device: "floor_heating".to_string(),
                register: 30001
            })
        );
    }

    #[test]
    fn lighting_scene_reports_partial_failure() {
        let scenes = BTreeMap::from([(
            "welcome".to_string(),
            BTreeMap::from([("D411S10".to_string(), 60), ("D411S11".to_string(), 40)]),
        )]);
        let mut model = LightingModel::try_from_config(
            &serde_yaml::from_str(
                r#"
fixtures:
  - id: D411S10
    level: 0
  - id: D411S11
    level: 0
"#,
            )
            .unwrap(),
            &scenes,
        )
        .unwrap();

        model.drop_command_for_fixture("D411S10").unwrap();
        model.activate_scene("welcome").unwrap();

        assert_eq!(
            model.scene_consistency_failures("welcome").unwrap(),
            vec!["D411S10 expected level 60, actual 0".to_string()]
        );
    }

    #[test]
    fn lighting_rejects_scene_with_unknown_fixture() {
        let scenes = BTreeMap::from([(
            "welcome".to_string(),
            BTreeMap::from([("missing_fixture".to_string(), 60)]),
        )]);
        let model = LightingModel::try_from_config(
            &serde_yaml::from_str(
                r#"
fixtures:
  - id: D411S10
    level: 0
"#,
            )
            .unwrap(),
            &scenes,
        )
        .unwrap();

        assert_eq!(
            model.assert_scene_targets_exist(),
            Err(DeviceModelError::UnknownFixture(
                "missing_fixture".to_string()
            ))
        );
    }

    #[test]
    fn modbus_value_returns_none_for_unknown_device() {
        let model = ModbusModel::default();

        assert_eq!(model.value("unknown_device", 40001), None);
        assert_eq!(model.readable_value("unknown_device", 40001), None);
        assert!(!model.has_register("unknown_device", 40001));
    }

    #[test]
    fn modbus_write_fails_for_unknown_device() {
        let mut model = ModbusModel::default();

        let error = model
            .write("unknown_device", 40001, serde_yaml::Value::from(1))
            .unwrap_err();

        assert_eq!(
            error,
            DeviceModelError::UnknownModbusDevice("unknown_device".to_string())
        );
    }

    #[test]
    fn contact_set_state_fails_for_unknown_id() {
        let mut contacts = ContactModel::default();

        let error = contacts.set_state("missing", "on").unwrap_err();

        assert_eq!(
            error,
            DeviceModelError::UnknownContact("missing".to_string())
        );
    }

    #[test]
    fn lighting_activate_scene_fails_for_unknown_scene() {
        let mut model = LightingModel::default();

        let error = model.activate_scene("missing").unwrap_err();

        assert_eq!(error, DeviceModelError::UnknownScene("missing".to_string()));
    }
}
