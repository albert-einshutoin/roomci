# 07. Fault Injection Model

## Goal

Simulate realistic smart-room failures that affect guest experience.

## Fault Types

| Fault | Description | Applies to |
|---|---|---|
| `offline` | Device unavailable | all devices |
| `degraded` | Device responds slowly or inconsistently | gateway, controller, devices |
| `latency` | Add response delay | all commands/events |
| `timeout` | Command never returns success | all actuators |
| `command_drop` | Command is silently dropped | lock, light, climate, cover |
| `command_reject` | Command returns explicit failure | lock, climate, controller |
| `duplicate_event` | Same event emitted multiple times | sensors, gateways |
| `out_of_order_event` | Event ordering inversion | MQTT/event streams |
| `stale_sensor_value` | Sensor value stops updating | sensors |
| `partial_scene_failure` | Some scene members fail | room_controller |
| `battery_low` | Battery warning | lock, sensors |
| `jammed` | Physical lock/cover failure | lock, cover |
| `gateway_reconnect` | Disconnect/reconnect sequence | gateway |

## Fault Scope

```yaml
fault:
  target: lock_entrance
  type: offline
  from: "T-5m"
  to: "T+3m"
```

## Deterministic Faults

Default in CI.

```yaml
fault:
  target: gateway_main
  type: latency
  latency_ms: 3000
```

## Probabilistic Faults

Must specify seed.

```yaml
fault:
  target: light_living
  type: command_drop
  probability: 0.3
  seed: 42
```

## Fault Interaction Rules

1. `offline` overrides all normal commands.
2. `jammed` affects physical actuation even if connectivity is online.
3. `stale_sensor_value` affects telemetry but not command execution.
4. `latency` can cause scenario-level timeouts.
5. `gateway` faults can affect all devices behind the gateway.

## Guest Impact Mapping

| Fault | Typical guest impact |
|---|---|
| Lock offline | high |
| AC preheat failed | medium/high |
| Welcome scene partial failure | low/medium |
| Occupancy stale value | medium/security |
| Gateway offline | high if no local fallback |
| Curtain failure | low/medium |

## Recovery Expectations

Roomci should allow recovery assertions.

Examples:

```yaml
recovery:
  expect_event: fallback_access_issued
  within: "60s"
```

```yaml
recovery:
  expect_event: staff_notification_sent
  within: "120s"
```
