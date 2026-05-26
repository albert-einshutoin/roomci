use roomci_edge::EdgeStatus;
use roomci_scenario::{
    yaml_map_to_json, Condition, ValidatedAssertionKind, ValidatedInlineAssertionKind,
    ValidatedModbusAssertion, ValidatedMqttAssertion, ValidatedTargetConditionAssertion,
};

use crate::{runtime::RuntimeState, AssertionResult};

pub(crate) fn evaluate_assertion(
    runtime: &RuntimeState,

    assertion: &ValidatedAssertionKind,
) -> AssertionResult {
    match assertion {
        ValidatedAssertionKind::MqttRetained(mqtt) => evaluate_mqtt_retained(runtime, mqtt),
        ValidatedAssertionKind::ModbusRegister(modbus) => evaluate_modbus_register(runtime, modbus),
        ValidatedAssertionKind::GuestExperienceField(condition) => {
            evaluate_guest_experience_field(runtime, *condition)
        }
        ValidatedAssertionKind::Ops(ops) => evaluate_ops(runtime, ops),
        ValidatedAssertionKind::TargetCondition(kind) => evaluate_target_condition(runtime, kind),
        ValidatedAssertionKind::Inline(kind) => evaluate_inline_assertion(runtime, kind),
    }
}

fn evaluate_mqtt_retained(
    runtime: &RuntimeState,
    mqtt: &ValidatedMqttAssertion,
) -> AssertionResult {
    let expected = yaml_map_to_json(&mqtt.retained);
    let actual = runtime.broker.retained().get(mqtt.topic.as_str());
    let passed = actual == Some(&expected);
    AssertionResult {
        name: format!("mqtt_retained:{}", mqtt.topic),
        assertion_type: "mqtt_retained".to_string(),
        passed,
        message: if passed {
            "retained MQTT state matched".to_string()
        } else {
            format!("retained MQTT state mismatch: expected {expected:?}, got {actual:?}")
        },
        impact_level: if passed {
            None
        } else {
            Some("high".to_string())
        },
        impact_message: if passed {
            None
        } else {
            Some(
                "Local controller state did not synchronize through retained MQTT state."
                    .to_string(),
            )
        },
    }
}

fn evaluate_modbus_register(
    runtime: &RuntimeState,
    modbus: &ValidatedModbusAssertion,
) -> AssertionResult {
    let actual = runtime
        .modbus
        .value(modbus.device.as_str(), modbus.register);
    let passed = if let Some(expected_readable) = modbus.readable_value {
        runtime
            .modbus
            .readable_value(modbus.device.as_str(), modbus.register)
            .map(|value| (value - expected_readable).abs() < f64::EPSILON)
            .unwrap_or(false)
    } else if let Some(expected) = &modbus.value {
        let expected_json = serde_json::to_value(expected).unwrap_or(serde_json::Value::Null);
        actual == Some(&expected_json)
    } else {
        false
    };
    AssertionResult {
        name: format!("modbus:{}:{}", modbus.device, modbus.register),
        assertion_type: "modbus_register".to_string(),
        passed,
        message: if passed {
            "Modbus register matched expected value".to_string()
        } else {
            format!("Modbus register mismatch: got {actual:?}")
        },
        impact_level: if passed {
            None
        } else {
            Some("medium".to_string())
        },
        impact_message: if passed {
            None
        } else {
            Some("Register-map behavior did not match commissioning expectation.".to_string())
        },
    }
}

fn evaluate_guest_experience_field(
    runtime: &RuntimeState,
    condition: Condition,
) -> AssertionResult {
    let local_ok = runtime.broker.is_online("mqtt.local");
    let passed = condition == Condition::Unaffected && local_ok;
    AssertionResult {
        name: "guest_experience".to_string(),
        assertion_type: "guest_experience".to_string(),
        passed,
        message: if passed {
            "guest experience remained unaffected by upstream outage".to_string()
        } else {
            "guest experience was affected".to_string()
        },
        impact_level: if passed {
            None
        } else {
            Some("high".to_string())
        },
        impact_message: if passed {
            None
        } else {
            Some("Local-first control did not preserve guest experience.".to_string())
        },
    }
}

fn evaluate_ops(
    runtime: &RuntimeState,
    ops: &std::collections::BTreeMap<String, serde_yaml::Value>,
) -> AssertionResult {
    match runtime.ops.evaluate_assertion(ops) {
        Ok(outcome) => AssertionResult {
            name: "ops".to_string(),
            assertion_type: "ops".to_string(),
            passed: outcome.passed,
            message: if outcome.passed {
                "ops escalation matched expected state".to_string()
            } else {
                format!("ops escalation mismatch: {}", outcome.failures.join("; "))
            },
            impact_level: if outcome.passed {
                None
            } else {
                Some("high".to_string())
            },
            impact_message: if outcome.passed {
                None
            } else {
                Some("Operations response did not meet the emergency alert contract.".to_string())
            },
        },
        Err(error) => AssertionResult {
            name: "ops".to_string(),
            assertion_type: "ops".to_string(),
            passed: false,
            message: error.to_string(),
            impact_level: Some("unknown".to_string()),
            impact_message: Some("The ops assertion is not supported by the runner.".to_string()),
        },
    }
}

fn evaluate_target_condition(
    runtime: &RuntimeState,
    kind: &ValidatedTargetConditionAssertion,
) -> AssertionResult {
    match kind {
        ValidatedTargetConditionAssertion::EdgeSecondary { condition } => {
            let status_passed = *condition == Condition::Active
                && runtime.edge.secondary_status() == Some(EdgeStatus::Active);
            let timing_passed = runtime
                .edge_expected_within
                .map(|expected_within| {
                    match (runtime.edge_primary_failed_at, runtime.edge_failover_at) {
                        (Some(failed_at), Some(failover_at)) => {
                            failover_at - failed_at <= expected_within
                        }
                        _ => false,
                    }
                })
                .unwrap_or(true);
            let passed = status_passed && timing_passed;
            AssertionResult {
                name: "edge.secondary".to_string(),
                assertion_type: "edge_state".to_string(),
                passed,
                message: if passed {
                    if timing_passed {
                        "secondary edge server is active".to_string()
                    } else {
                        "secondary edge server did not activate within expected failover window"
                            .to_string()
                    }
                } else {
                    "secondary edge server is not active".to_string()
                },
                impact_level: if passed {
                    None
                } else {
                    Some("high".to_string())
                },
                impact_message: if passed {
                    None
                } else {
                    Some("Edge failover did not preserve local control.".to_string())
                },
            }
        }
        ValidatedTargetConditionAssertion::MqttLocal { condition } => {
            let passed =
                *condition == Condition::Available && runtime.broker.is_online("mqtt.local");
            AssertionResult {
                name: "mqtt.local".to_string(),
                assertion_type: "broker_state".to_string(),
                passed,
                message: if passed {
                    "local MQTT broker is available".to_string()
                } else {
                    "local MQTT broker is unavailable".to_string()
                },
                impact_level: if passed {
                    None
                } else {
                    Some("high".to_string())
                },
                impact_message: if passed {
                    None
                } else {
                    Some("Local MQTT broker is unavailable during failover.".to_string())
                },
            }
        }
        ValidatedTargetConditionAssertion::WanBackup { condition } => {
            let status_passed = *condition == Condition::Active
                && runtime.wan_backup_status.as_deref() == Some("active");
            let timing_passed = runtime
                .wan_expected_within
                .map(|expected_within| {
                    match (runtime.wan_primary_failed_at, runtime.wan_failover_at) {
                        (Some(failed_at), Some(failover_at)) => {
                            failover_at - failed_at <= expected_within
                        }
                        _ => false,
                    }
                })
                .unwrap_or(true);
            let passed = status_passed && timing_passed;
            AssertionResult {
                name: "wan.backup".to_string(),
                assertion_type: "wan_failover".to_string(),
                passed,
                message: if passed {
                    "backup WAN is active".to_string()
                } else {
                    "backup WAN is not active within expected failover window".to_string()
                },
                impact_level: if passed {
                    None
                } else {
                    Some("high".to_string())
                },
                impact_message: if passed {
                    None
                } else {
                    Some("WAN failover did not preserve internet resilience.".to_string())
                },
            }
        }
        ValidatedTargetConditionAssertion::LivingAreaDiscomfortIndex { condition } => {
            let target = "living_area.discomfort_index";
            let passed = runtime.evaluate_between_condition(target, *condition);
            AssertionResult {
                name: target.to_string(),
                assertion_type: "comfort_metric".to_string(),
                passed,
                message: if passed {
                    "comfort metric stayed within expected range".to_string()
                } else {
                    "comfort metric did not reach expected range".to_string()
                },
                impact_level: if passed {
                    None
                } else {
                    Some("medium".to_string())
                },
                impact_message: if passed {
                    None
                } else {
                    Some("HVAC auto mode did not meet comfort expectation.".to_string())
                },
            }
        }
        ValidatedTargetConditionAssertion::UserOverride { condition } => {
            let target = "user_override";
            let passed = *condition == Condition::False && runtime.comfort.user_override_count == 0;
            AssertionResult {
                name: target.to_string(),
                assertion_type: "comfort_user_override".to_string(),
                passed,
                message: if passed {
                    "no user override occurred".to_string()
                } else {
                    "user override occurred".to_string()
                },
                impact_level: if passed {
                    None
                } else {
                    Some("medium".to_string())
                },
                impact_message: if passed {
                    None
                } else {
                    Some("Guest manually overrode comfort automation.".to_string())
                },
            }
        }
        ValidatedTargetConditionAssertion::GuestExperience { condition } => {
            let passed = *condition == Condition::Unaffected
                && runtime.broker.is_online("mqtt.local")
                && runtime.edge.active_id().is_some();
            AssertionResult {
                name: "guest_experience".to_string(),
                assertion_type: "guest_experience".to_string(),
                passed,
                message: if passed {
                    "guest experience remained unaffected by local edge availability".to_string()
                } else {
                    "guest experience was affected".to_string()
                },
                impact_level: if passed {
                    None
                } else {
                    Some("high".to_string())
                },
                impact_message: if passed {
                    None
                } else {
                    Some(
                        "Local edge or MQTT availability did not preserve guest experience."
                            .to_string(),
                    )
                },
            }
        }
    }
}

fn evaluate_inline_assertion(
    runtime: &RuntimeState,
    kind: &ValidatedInlineAssertionKind,
) -> AssertionResult {
    match kind {
        ValidatedInlineAssertionKind::SceneConsistencyComplete(scene) => {
            evaluate_scene_consistency(runtime, scene.as_str())
        }
        ValidatedInlineAssertionKind::AccessControlDriftDetected => {
            evaluate_access_control_drift(runtime)
        }
        ValidatedInlineAssertionKind::CommissioningChecklistGenerated => {
            evaluate_commissioning_checklist(runtime)
        }
        ValidatedInlineAssertionKind::IntercomRelaySafeEvidence => {
            evaluate_intercom_relay_evidence(runtime)
        }
        ValidatedInlineAssertionKind::NetworkControlPanelFaultsObserved => {
            evaluate_network_control_panel_faults(runtime)
        }
        ValidatedInlineAssertionKind::ComfortTimeseriesObserved => {
            evaluate_comfort_timeseries(runtime)
        }
    }
}

fn evaluate_scene_consistency(runtime: &RuntimeState, scene: &str) -> AssertionResult {
    let failures = match runtime.lighting.scene_consistency_failures(scene) {
        Ok(failures) => failures,
        Err(_) => {
            return AssertionResult {
                name: format!("scene_consistency:{scene}"),
                assertion_type: "scene_consistency".to_string(),
                passed: false,
                message: format!("scene {scene} is not defined"),
                impact_level: Some("medium".to_string()),
                impact_message: Some("Scene mapping is missing from the scenario.".to_string()),
            };
        }
    };
    let passed = failures.is_empty();
    AssertionResult {
        name: format!("scene_consistency:{scene}"),
        assertion_type: "scene_consistency".to_string(),
        passed,
        message: if passed {
            "DALI-like scene reached expected levels".to_string()
        } else {
            format!(
                "DALI-like scene consistency violation: {}",
                failures.join("; ")
            )
        },
        impact_level: if passed {
            None
        } else {
            Some("medium".to_string())
        },
        impact_message: if passed {
            None
        } else {
            Some("Lighting scene did not match intended guest ambience.".to_string())
        },
    }
}

fn evaluate_access_control_drift(runtime: &RuntimeState) -> AssertionResult {
    let passed = !runtime.access.unexpected_users.is_empty();
    AssertionResult {
        name: "access_control_drift".to_string(),
        assertion_type: "access_control_drift".to_string(),
        passed,
        message: if passed {
            format!(
                "detected unexpected access users: {}",
                runtime.access.unexpected_users.join(", ")
            )
        } else {
            "no unexpected access users were detected".to_string()
        },
        impact_level: if passed {
            None
        } else {
            Some("high".to_string())
        },
        impact_message: if passed {
            None
        } else {
            Some("Access-control drift detection did not flag stale unlock permission.".to_string())
        },
    }
}

fn evaluate_network_control_panel_faults(runtime: &RuntimeState) -> AssertionResult {
    let required = [
        "network_segment_isolated",
        "firewall_policy_drift_detected",
        "control_panel_ups_degraded",
        "control_panel_circuit_protector_tripped",
        "control_panel_redundant_psu_degraded",
    ];
    let missing = required
        .iter()
        .filter(|event_type| {
            !runtime
                .timeline
                .iter()
                .any(|event| event.event_type == **event_type)
        })
        .copied()
        .collect::<Vec<_>>();
    let passed = missing.is_empty()
        && runtime.states.values().any(|state| {
            state.get("bms_evidence").and_then(|value| value.as_str()) == Some("recorded")
        });
    AssertionResult {
        name: "network_control_panel_faults".to_string(),
        assertion_type: "network_control_panel_faults".to_string(),
        passed,
        message: if passed {
            "network and control-panel fault profiles produced BMS evidence".to_string()
        } else {
            format!(
                "network/control-panel fault evidence incomplete; missing: {}",
                missing.join(", ")
            )
        },
        impact_level: if passed {
            None
        } else {
            Some("medium".to_string())
        },
        impact_message: if passed {
            None
        } else {
            Some("Infrastructure fault QA evidence is incomplete.".to_string())
        },
    }
}

fn evaluate_comfort_timeseries(runtime: &RuntimeState) -> AssertionResult {
    let reading_count = runtime
        .timeline
        .iter()
        .filter(|event| event.event_type == "comfort_sensor_reading_recorded")
        .count();
    let has_comfort_state = runtime.states.keys().any(|key| key.starts_with("comfort."));
    let passed = reading_count >= 2 && has_comfort_state;
    AssertionResult {
        name: "comfort_timeseries".to_string(),
        assertion_type: "comfort_timeseries".to_string(),
        passed,
        message: if passed {
            format!("{reading_count} comfort sensor readings were replayed")
        } else {
            "comfort time-series replay did not produce enough sensor evidence".to_string()
        },
        impact_level: if passed {
            None
        } else {
            Some("medium".to_string())
        },
        impact_message: if passed {
            None
        } else {
            Some("Comfort automation QA lacks deterministic sensor-zone evidence.".to_string())
        },
    }
}

fn evaluate_intercom_relay_evidence(runtime: &RuntimeState) -> AssertionResult {
    let has_intercom_event = runtime
        .timeline
        .iter()
        .any(|event| event.event_type.starts_with("intercom_"));
    let has_relay_request = runtime
        .timeline
        .iter()
        .any(|event| event.event_type == "relay_pulse_requested");
    let real_unlock_controlled = runtime
        .states
        .iter()
        .filter(|(key, _)| key.starts_with("intercom."))
        .any(|(_, state)| {
            state
                .get("real_unlock_controlled")
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
        });
    let passed = has_intercom_event && has_relay_request && !real_unlock_controlled;
    AssertionResult {
        name: "intercom_relay_safe_evidence".to_string(),
        assertion_type: "intercom_relay_safe_evidence".to_string(),
        passed,
        message: if passed {
            "intercom and relay evidence was captured without controlling a real unlock".to_string()
        } else {
            "intercom/relay safe evidence is incomplete or attempted real unlock control"
                .to_string()
        },
        impact_level: if passed {
            None
        } else {
            Some("high".to_string())
        },
        impact_message: if passed {
            None
        } else {
            Some("Access-control QA evidence is not safe enough for pre-adoption PoC.".to_string())
        },
    }
}

fn evaluate_commissioning_checklist(runtime: &RuntimeState) -> AssertionResult {
    let passed = runtime.commissioning.check_count > 0;
    AssertionResult {
        name: "commissioning_checklist".to_string(),
        assertion_type: "commissioning_checklist".to_string(),
        passed,
        message: if passed {
            match &runtime.commissioning.site {
                Some(site) => format!(
                    "generated {count} commissioning checks for {site}",
                    count = runtime.commissioning.check_count
                ),
                None => format!(
                    "generated {count} commissioning checks",
                    count = runtime.commissioning.check_count
                ),
            }
        } else {
            "no commissioning checks were generated".to_string()
        },
        impact_level: if passed {
            None
        } else {
            Some("medium".to_string())
        },
        impact_message: if passed {
            None
        } else {
            Some("Commissioning checklist generation did not produce field checks.".to_string())
        },
    }
}
