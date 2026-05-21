# 13. Comfort Automation

## Why comfort automation belongs in roomci

Smart-home quality is not only whether commands execute. It is whether the space becomes comfortable without the guest needing to operate devices.

`roomci-comfort` models:

- temperature
- humidity
- discomfort index
- ceiling vs living-area sensors
- HVAC auto mode
- user override events
- room-specific tuning

## Discomfort index

Default formula:

```txt
DI = 0.81 * temperature + 0.01 * humidity * (0.99 * temperature - 14.3) + 46.3
```

## Sensor model

```yaml
sensors:
  ceiling:
    temperature: 30.5
    humidity: 62.1
  living_area:
    temperature: 28.5
    humidity: 49.4
```

## Deterministic time-series replay

Phase 17 adds executable sensor-reading steps for deterministic CI replay.
This is not a hardware or thermal simulation; it is a contract-level replay of
sensor-zone observations that downstream comfort automation can use as
evidence.

```yaml
steps:
  - at: T
    sensor_reading:
      target: living_area
      zone: living
      temperature: 25.0
      humidity: 55.0
      occupancy: true
  - at: T+30s
    sensor_reading:
      target: ceiling_area
      zone: ceiling
      temperature: 27.0
      humidity: 50.0
      occupancy: false
assertions:
  - at: T+31s
    assert:
      comfort_timeseries: observed
```

Each reading updates `comfort.<target>` in final state with temperature,
humidity, occupancy, zone, discomfort index, and a simple oscillation flag.

## Comfort target

Room-specific target:

```yaml
comfort:
  target_discomfort_index: 74
  acceptable_range:
    min: 72
    max: 76
```

## HVAC auto control scenario

```yaml
scenario:
  name: comfort_auto_mode

sensors:
  ceiling:
    temperature: 30.5
    humidity: 62.1
  living_area:
    temperature: 28.5
    humidity: 49.4

comfort:
  target_discomfort_index: 74
  acceptable_range:
    min: 72
    max: 76

steps:
  - at: T
    automation:
      type: hvac_auto_mode
      room: living
  - at: T+30m
    assert:
      target: living_area.discomfort_index
      condition: between 72 and 76
  - at: T+2h
    assert:
      target: user_override
      condition: false
```

## User override model

A user override is a signal that auto mode may not match comfort expectation.

```yaml
user_events:
  - at: T+25m
    type: hvac_manual_override
    new_temperature: 25
```

Assertions:

```yaml
assert:
  user_override_count: 0
```

## Humidity spike scenario

```yaml
faults:
  - at: T+20m
    target: living_area.humidity
    type: spike
    value: 75
    duration: 10m
```

The system should test whether the control algorithm reacts smoothly instead of oscillating.

## Report example

```txt
FAIL comfort_auto_mode

Comfort target:
- target DI: 74
- acceptable: 72..76

Actual:
- DI at T+30m: 78.2

User behavior:
- manual override occurred at T+35m

Possible causes:
- target DI too low/high for room
- sensor placement mismatch
- HVAC internal sensor differs from living-area sensor
- humidity spike not handled
```

## Future: external data replay

A future version can ingest real time-series data and replay conditions in CI.

```bash
roomci replay-comfort data/fukuoka_doma_2025-09.csv
```
