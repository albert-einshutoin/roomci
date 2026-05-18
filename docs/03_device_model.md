# 03. Device Model

## Design Principle

Do not model vendors first. Model capabilities first.

A Philips Hue light, Matter light, Home Assistant MQTT light, and hotel room light should map to the same canonical capabilities.

## Core Entities

```text
Room
  Device[]
    Capability[]
    State
    Metadata
```

## Room

```yaml
room:
  id: room_nasu_001
  name: NASU Villa 001
  timezone: Asia/Tokyo
```

## Device

```yaml
devices:
  - id: lock_entrance
    name: Entrance Smart Lock
    type: smart_lock
    capabilities:
      - lock
      - battery
      - connectivity
```

## Canonical Device Types

| Type | Required capabilities | Optional capabilities |
|---|---|---|
| `smart_lock` | `lock`, `connectivity` | `battery`, `tamper`, `door_contact` |
| `light` | `on_off` | `brightness`, `color_temperature`, `rgb`, `scene_member` |
| `climate` | `thermostat`, `temperature_measurement` | `humidity`, `mode`, `fan_mode` |
| `cover` | `cover_position` | `tilt`, `obstruction` |
| `temperature_sensor` | `temperature_measurement` | `battery`, `connectivity` |
| `humidity_sensor` | `humidity_measurement` | `battery`, `connectivity` |
| `occupancy_sensor` | `occupancy` | `motion`, `presence_confidence` |
| `network_gateway` | `connectivity` | `latency`, `packet_loss`, `uplink` |
| `room_controller` | `scene_controller` | `automation`, `local_fallback` |

## Capability Definitions

### `lock`

State:

```yaml
lock_state: locked | unlocked | jammed | unknown
```

Commands:

```yaml
- lock
- unlock
```

Events:

```yaml
- locked
- unlocked
- jammed
- lock_failed
```

### `on_off`

State:

```yaml
power: on | off
```

Commands:

```yaml
- turn_on
- turn_off
```

### `brightness`

State:

```yaml
brightness: 0..100
```

Commands:

```yaml
- set_brightness
```

### `thermostat`

State:

```yaml
mode: off | heat | cool | auto | dry | fan
setpoint_celsius: number
```

Commands:

```yaml
- set_mode
- set_temperature
```

### `temperature_measurement`

State:

```yaml
temperature_celsius: number
last_updated_at: timestamp
```

Events:

```yaml
- temperature_changed
```

### `occupancy`

State:

```yaml
occupied: true | false
confidence: 0..1
```

### `connectivity`

State:

```yaml
status: online | offline | degraded
latency_ms: number
packet_loss: 0..1
last_seen_at: timestamp
```

## Scenes

Scenes are grouped commands with consistency assertions.

```yaml
scenes:
  welcome:
    commands:
      - target: light_living
        action: turn_on
      - target: light_living
        action: set_brightness
        value: 70
      - target: curtain_living
        action: open
      - target: ac_living
        action: set_temperature
        value: 24
```

## Hospitality State Layer

A room also has hospitality-level states.

```yaml
room_state:
  lifecycle: vacant | pre_checkin | occupied | checkout_requested | cleaning | maintenance
  guest_present: true | false
  access_mode: normal | fallback | staff_only
```

This layer makes `roomci` different from a generic device emulator.
