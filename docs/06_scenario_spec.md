# 06. Scenario Specification

## File Type

YAML.

## Top-level Structure

```yaml
version: "0.1"
scenario:
  name: checkin_lock_offline
  description: Smart lock goes offline before guest arrival.
  clock:
    start: "2026-08-10T14:00:00+09:00"
    mode: simulated

room:
  id: room_nasu_001
  devices: []

steps: []
assertions: []
```

## Device Definition

```yaml
room:
  id: room_nasu_001
  devices:
    - id: lock_entrance
      type: smart_lock
      initial_state:
        lock_state: locked
        connectivity: online

    - id: ac_living
      type: climate
      initial_state:
        mode: off
        setpoint_celsius: 28
        temperature_celsius: 28
```

## Step Types

### Event step

```yaml
- at: "T-30m"
  event: pre_checkin_started
```

### Command step

```yaml
- at: "T-10m"
  command:
    target: ac_living
    action: set_temperature
    value: 24
```

### Fault step

```yaml
- at: "T-5m"
  fault:
    target: lock_entrance
    type: offline
    duration: "8m"
```

### Sensor update step

```yaml
- at: "T-3m"
  state:
    target: temperature_living
    patch:
      temperature_celsius: 28.2
```

### Assertion step

```yaml
- at: "T+60s"
  assert:
    event: fallback_access_issued
    within: "60s"
```

## Time Expressions

- Absolute ISO 8601: `2026-08-10T15:00:00+09:00`
- Relative to guest arrival: `T-30m`, `T`, `T+60s`
- Durations: `30s`, `5m`, `1h`

## Assertions

### Device state assertion

```yaml
assertions:
  - at: "T+1m"
    type: device_state
    target: lock_entrance
    expect:
      lock_state: unlocked
```

### Event assertion

```yaml
assertions:
  - at: "T+1m"
    type: event_emitted
    event: fallback_access_issued
    within: "60s"
```

### Sensor threshold assertion

```yaml
assertions:
  - at: "T"
    type: sensor_threshold
    target: temperature_living
    condition: "<= 25.0"
```

### Scene consistency assertion

```yaml
assertions:
  - at: "T"
    type: scene_consistency
    scene: welcome
    expect: complete
```

## Guest Impact

Each assertion can define impact.

```yaml
impact:
  level: high
  message: Guest cannot enter room without fallback access.
```
