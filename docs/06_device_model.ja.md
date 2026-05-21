# 06. デバイスモデル

## 設計 goal

The device model should be expressive enough for QA scenarios but not attempt full physical fidelity.

A device must have:

- stable ID
- type
- protocol adapter
- current state
- command interface
- telemetry/state topics
- failure modes
- field metadata

## Device schema

```yaml
devices:
  - id: living_light_01
    type: light
    protocol: dali
    area: living
    metadata:
      fixture_address: D411S10
      floor: 1
      drawing_ref: E-201
    state:
      power: false
      brightness: 0
```

## Core device types

| Type | 目的 | MVP |
|---|---|---|
| light | lighting and scenes | Yes |
| hvac | air conditioner / heating | Yes |
| floor_heating | temperature-controlled heating | Yes |
| contact_input | alarms, sensors, dry contacts | Yes |
| contact_output | relays, unlock, speaker trigger | Yes |
| curtain | curtain / blind control | Should |
| sauna | sauna temperature / emergency alert | Should |
| bath | bath / cold bath temp control | Should |
| power_meter | electricity metering | Should |
| humidity_sensor | comfort automation | Should |
| temperature_sensor | comfort automation | Should |
| intercom | DoorBird-like future | Later |

## Device state principles

- State should be serializable to JSON.
- State should be publishable as retained MQTT messages.
- State updates should be timestamped.
- Device errors should include both guest impact and field-debug hints.

## Example: HVAC

```yaml
- id: hvac_living
  type: hvac
  protocol: modbus
  state:
    power: true
    mode: cool
    target_temperature: 24.0
    internal_temperature: 26.0
    error_code: null
```

## Example: Contact input

```yaml
- id: sauna_emergency_button
  type: contact_input
  protocol: dry_contact
  state:
    value: off
    criticality: emergency
    location: sauna_room
```

## Example: Contact output

```yaml
- id: gate_relay
  type: contact_output
  protocol: relay
  state:
    value: off
    pulse_duration_ms: 1000
```

## Field metadata

Field metadata is important because failure reports should help people debug in the real world.

Recommended metadata:

```yaml
metadata:
  house: setouchi_villa_01
  area: living
  panel: CP-01
  terminal_block: TB-12
  drawing_ref: E-201
  runbook_url: https://example.com/runbooks/living-light
  owner_team: smart-home
```

## Device fault model

Common device faults:

- offline
- command_drop
- delayed_response
- stale_state
- wrong_mapping
- error_code
- flapping
- power_lost
- degraded
- partial_failure

## Guest impact levels

Every failed assertion should estimate impact:

| Level | Meaning |
|---|---|
| none | no expected guest impact |
| low | minor delay or degradation |
| medium | noticeable but recoverable failure |
| high | guest experience significantly affected |
| critical | safety / access / emergency issue |
