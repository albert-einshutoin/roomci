# 15. Scenario Specification

## Overview

A scenario describes a reproducible smart-home field situation.

A scenario should define:

- environment
- devices
- network
- MQTT brokers
- operations/BMS rules
- timeline steps
- faults
- assertions
- reporting metadata

## Top-level structure

```yaml
version: "0.1"
scenario:
  name: local_first_cloud_outage
  description: Verify local controls survive cloud outage.
  tags: [mqtt, local-first, outage]

environment: {}
network: {}
mqtt: {}
devices: []
alerts: []
faults: []
steps: []
assertions: []
report: {}
```

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
- ticket state
- guest impact

Example:

```yaml
assertions:
  - at: T+20s
    target: guest_experience
    condition: unaffected
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

## Reporting fields

```yaml
report:
  title: Local-first cloud outage test
  owner: smart-home
  severity_on_failure: high
  output:
    markdown: true
    json: true
    junit: true
```
