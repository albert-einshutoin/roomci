# 15. Scenario Specification

## Overview

A scenario describes a reproducible smart-home field situation.

A scenario should define:

- devices
- WAN and edge model settings
- MQTT brokers
- external inputs
- commissioning metadata
- operations/BMS rules
- timeline steps
- faults
- assertions
- descriptive metadata (`scenario.description`, `scenario.tags`)

## Top-level structure

The top-level `version` is validated and currently limited to the `0.1` series.

```yaml
version: "0.1"
scenario:
  name: local_first_cloud_outage
  description: Verify local controls survive cloud outage.
  tags: [mqtt, local-first, outage]

mqtt: {}
devices: []
inputs: {}
commissioning: {}
alerts: []
faults: []
steps: []
assertions: []
```

Unknown top-level, `scenario`, and fault keys fail during parsing. The former
`environment`, `network`, `future_milestone`, and `report` sections,
`scenario.clock`, and fault `severity` never controlled execution and are no
longer accepted. Remove them from runnable scenarios. Use `scenario.description`
and `scenario.tags` for descriptive metadata, and CLI output flags for reports.
Keys inside free-form MQTT payloads and device state maps remain data, even if
they have the same spelling as a removed setting. Omitting optional sections
continues to use the existing defaults.

## Time model

Use symbolic relative time:

```txt
T
T+1s
T+5m
T-30m
```

The scenario runner converts this into virtual time.

## Fault declaration

Timed recovery through global or step fault `duration` is unsupported in the
internal model. Integer values with `ms`, `s`, `m`, or `h` units produce
`UnsupportedFaultDuration` with their indexed location before running; malformed
strings produce `InvalidDuration`, and explicit `null` or other non-string values
are parse errors. Omit `duration` to keep the existing persistent fault behavior.
Serialization and `GET /scenario` also omit that key, so their output can be
validated again. `mqtt.local.enabled: false` and
`mqtt.*.retained: false` are also rejected; unknown structured `mqtt` fields
are parse errors. The real-broker recovery command has a separate strict
contract: [External MQTT recovery](EXTERNAL_MQTT_RECOVERY.md).

Faults can be declared globally:

```yaml
faults:
  - at: T+10s
    target: mqtt.cloud
    type: offline
```

Or inside steps:

```yaml
steps:
  - at: T+10s
    fault:
      target: mqtt.cloud
      type: offline
```

## Assertions

Assertions should support:

- device state
- MQTT retained message
- operations notification
- network reachability
- comfort metric
- access-control drift
- commissioning checklist generation
- ticket state
- guest impact

Example:

```yaml
assertions:
  - at: T+20s
    target: guest_experience
    condition: unaffected
```

Access-control drift assertions compare `inputs.identity_group` with
`inputs.access_system_group` and pass when the scenario intentionally detects
stale access users:

```yaml
inputs:
  identity_group:
    - alice@example.com
  access_system_group:
    - alice@example.com
    - retired@example.com

assertions:
  - at: T
    assert:
      access_control_drift: detected
```

Commissioning checklist assertions count declared room devices and pass when
field checks can be generated:

```yaml
commissioning:
  site: minakami
  rooms:
    - id: living
      devices:
        - D411S10
        - floor_heating_01

assertions:
  - at: T
    assert:
      commissioning_checklist: generated
```

## Example: local-first scenario

```yaml
version: "0.1"
scenario:
  name: local_first_cloud_outage
  tags: [mqtt, local-first]

mqtt:
  local:
    retained: true
  cloud:
    enabled: true

devices:
  - id: living_light
    type: light
    protocol: dali
    state:
      power: false
      brightness: 0

faults:
  - at: T+10s
    target: mqtt.cloud
    type: offline

steps:
  - at: T+15s
    mqtt_publish:
      client: ipad_controller
      topic: house/minakami/room/living/device/living_light/command
      payload:
        power: true
        brightness: 60

assertions:
  - at: T+16s
    mqtt:
      topic: house/minakami/room/living/device/living_light/state
      retained:
        power: true
        brightness: 60
  - at: T+20s
    guest_experience: unaffected
```

## Scenario tags

Recommended tags:

```txt
mqtt
local-first
modbus
dali
bms
ops
network
comfort
commissioning
control-panel
intercom
access-control
```

## Report output

Select report formats with the `roomci run` CLI flags (`--markdown`, `--json`,
`--junit`, or `--report-dir`). Report configuration in scenario YAML is not
supported.
