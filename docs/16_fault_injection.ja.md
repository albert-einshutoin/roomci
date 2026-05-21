# 16. 障害注入

## 目的

Fault injection is the core value of `roomci`. It allows teams to test whether smart-home systems continue operating or fail safely under realistic field conditions.

## Fault categories

### MQTT faults

```txt
broker_offline
duplicate_delivery
delayed_delivery
retained_state_missing
reconnect_storm
bridge_queue_backlog
```

### Edge faults

```txt
edge_primary_failure
edge_secondary_unavailable
command_router_crash
state_sync_stale
local_only_mode
```

### Device faults

```txt
offline
command_drop
delayed_response
stuck_state
wrong_mapping
error_code
flapping
```

### Modbus faults

```txt
timeout
wrong_unit_id
illegal_function
illegal_data_address
exception_response
register_stuck
endianness_mismatch
```

### DALI faults

```txt
fixture_missing
wrong_address
wrong_group
command_drop
delayed_scene
throughput_limit
```

### Contact I/O faults

```txt
contact_stuck_on
contact_stuck_off
flapping
relay_failed_to_pulse
false_alarm
```

### Network faults

```txt
wan_down
starlink_degraded
packet_loss
latency
dns_failure
vlan_leak
router_reboot
l3_switch_failure
```

### Control panel faults

```txt
psu_failure
ups_degraded
circuit_trip
short_circuit
edge_power_lost
```

### Operations faults

```txt
slack_delivery_failed
phone_escalation_failed
runbook_missing
ticket_not_created
recovery_notification_missing
```

## Fault schema

```yaml
faults:
  - at: T+10s
    target: mqtt.cloud
    type: offline
    duration: 10m
```

## Target naming convention

```txt
mqtt.local
mqtt.cloud
edge.primary
edge.secondary
modbus.floor_heating_01
dali.fixture.D411S10
contact.sauna_emergency_button
network.wan.primary
ops.slack
control_panel.CP-01.power.dc24v.psu_01
```

## Fault severity

Faults may include expected severity:

```yaml
severity: high
```

But actual guest impact should be computed by assertions whenever possible.

## Guest impact model

```yaml
assertions:
  - at: T+20s
    guest_experience:
      expected_impact: none
```

Impact levels:

```txt
none
low
medium
high
critical
```

## Example: Slack failure with phone fallback

```yaml
faults:
  - at: T+1s
    target: ops.slack
    type: delivery_failed

steps:
  - at: T
    contact:
      id: sauna_emergency_button
      state: on

assertions:
  - at: T+5s
    ops:
      phone_call_triggered: true
      alert_ticket_created: true
```

## Example: Modbus timeout

```yaml
faults:
  - at: T
    target: modbus.floor_heating_01
    type: timeout

assertions:
  - at: T+5s
    ops:
      slack_notification_sent: true
  - at: T+5s
    device:
      id: floor_heating_01
      state:
        degraded: true
```
