# 05. Edge Server Emulator

## Purpose

The edge server / home control server emulator is the local brain of the simulated smart home.

It receives commands from the local MQTT broker, maps them to device-specific protocol actions, updates retained state, and handles local-first fallback behavior.

## Responsibilities

- subscribe to local MQTT command topics
- route commands to device adapters
- publish retained state updates
- run automation logic
- handle local-only mode when cloud is unavailable
- simulate primary/secondary failover
- expose health endpoint
- expose optional HTTP control API for tests

## Edge server model

```yaml
edge:
  id: edge_main
  mode: active
  redundancy:
    enabled: true
    secondary: edge_backup
  mqtt:
    local_broker: mqtt://local-broker:1883
    cloud_broker: mqtt://cloud-broker:1883
```

## Command routing example

Input MQTT command:

```json
{
  "target": "living_scene",
  "action": "activate",
  "value": "welcome"
}
```

Edge server expands this into protocol commands:

```txt
DALI fixture D411S10 -> set_level 60
DALI fixture D411S11 -> set_level 40
KNX group 1/0/1 -> write true
HVAC living -> set target_temperature 24
```

## Edge failover

Scenario:

```yaml
scenario:
  name: edge_server_failover

faults:
  - at: T+10s
    target: edge.primary
    type: power_lost

assertions:
  - at: T+12s
    target: edge.secondary
    condition: active
  - at: T+15s
    target: mqtt.local
    condition: available
  - at: T+20s
    target: guest_experience
    condition: unaffected
```

## Local-only mode

When cloud is offline:

- local MQTT remains available
- edge server continues local device control
- cloud bridge buffers or marks state as pending
- operations report shows degraded cloud sync but guest controls remain functional

## Health endpoints

Recommended HTTP endpoints:

```txt
GET  /health
GET  /edge/status
POST /edge/faults
POST /edge/recover
GET  /edge/events
```

## Lua hook future

Because NOT A HOTEL's smart-home stack includes Lua, a future version can support Lua hooks for control logic.

Example:

```yaml
hooks:
  on_scene_activate: scripts/welcome_scene.lua
```

Example Lua:

```lua
function on_scene_activate(ctx)
  ctx:dali_set_level("D411S10", 60)
  ctx:dali_set_level("D411S11", 40)
  ctx:modbus_write("floor_heating_01", 40001, 245)
  ctx:contact_output("relay_welcome", true)
end
```

## Go backend sample future

A Go sample should demonstrate how a backend service can interact with `roomci` over HTTP/MQTT without knowing that real devices are absent.
