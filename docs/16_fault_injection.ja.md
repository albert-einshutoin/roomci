# 16. 障害注入

## 目的

障害注入は `roomci` の中核的な価値です。現実的な現場条件の下で、スマートホームシステムが継続稼働するか、安全に失敗するかをチームがテストできるようにします。

## 障害カテゴリ

### MQTT 障害

```txt
broker_offline
duplicate_delivery
delayed_delivery
retained_state_missing
reconnect_storm
bridge_queue_backlog
```

### エッジ障害

```txt
edge_primary_failure
edge_secondary_unavailable
command_router_crash
state_sync_stale
local_only_mode
```

### デバイス障害

```txt
offline
command_drop
delayed_response
stuck_state
wrong_mapping
error_code
flapping
```

### Modbus 障害

```txt
timeout
wrong_unit_id
illegal_function
illegal_data_address
exception_response
register_stuck
endianness_mismatch
```

### DALI 障害

```txt
fixture_missing
wrong_address
wrong_group
command_drop
delayed_scene
throughput_limit
```

### 接点 I/O 障害

```txt
contact_stuck_on
contact_stuck_off
flapping
relay_failed_to_pulse
false_alarm
```

### ネットワーク障害

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

### 制御パネル障害

```txt
psu_failure
ups_degraded
circuit_trip
short_circuit
edge_power_lost
```

### 運用障害

```txt
slack_delivery_failed
phone_escalation_failed
runbook_missing
ticket_not_created
recovery_notification_missing
```

## 障害スキーマ

```yaml
faults:
  - at: T+10s
    target: mqtt.cloud
    type: offline
    duration: 10m
```

## ターゲット命名規則

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

## 障害の重大度

障害には期待される重大度を含めることができます。

```yaml
severity: high
```

ただし、実際のゲストへの影響は可能な限りアサーションで算出する必要があります。

## ゲスト影響モデル

```yaml
assertions:
  - at: T+20s
    guest_experience:
      expected_impact: none
```

影響レベル:

```txt
none
low
medium
high
critical
```

## 例: Slack 障害と電話フォールバック

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

## 例: Modbus タイムアウト

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
