# 09. DALIライティング戦略

## なぜ DALI matters

Lighting is a major part of architectural experience. DALI-like simulation is useful because lighting failures are often experienced as quality degradation rather than obvious system crashes.

The goal is to model:

- fixture addresses
- groups
- scenes
- dimming levels
- partial scene failure
- command throughput
- commissioning mistakes

## スコープ

v0.1 should implement a DALI-like logical model, not a full DALI wire protocol.

## Fixture model

```yaml
lighting:
  fixtures:
    - id: D411S10
      protocol: dali
      area: living
      type: line_light
      group: welcome_scene
      level: 0
    - id: D411S11
      protocol: dali
      area: living
      type: downlight
      group: welcome_scene
      level: 0
```

## Scene model

```yaml
scenes:
  welcome:
    fixtures:
      D411S10: 60
      D411S11: 40
      D512B01: 20
```

## Common failure modes

| Fault | Meaning |
|---|---|
| command_drop | fixture never receives command |
| delayed_response | fixture changes level too slowly |
| wrong_address | command targets wrong fixture |
| wrong_group | fixture not included in intended scene |
| stuck_level | fixture remains at old dimming level |
| gateway_timeout | bridge/gateway fails to execute command |
| throughput_limit | too many commands sent too quickly |

## Scene consistency assertion

```yaml
assert:
  scene: welcome
  consistency: complete
```

This checks whether every fixture reached its expected state.

## Demo: partial scene failure

```yaml
scenario:
  name: welcome_scene_partial_failure

faults:
  - target: dali.fixture.D411S10
    type: command_drop

steps:
  - at: T
    command:
      target: scene.welcome
      action: activate
  - at: T+3s
    assert:
      scene: welcome
      consistency: complete
```

## Report example

```txt
FAIL welcome_scene_partial_failure

Scene consistency violation:
- D411S10 expected level 60, actual 0
- D411S11 expected level 40, actual 40

Guest impact:
Medium

Possible field causes:
- wrong DALI address
- fixture not assigned to scene group
- gateway command timeout
- wiring issue
- scene mapping config mismatch
```

## KNX/DALI throughput scenario

Some deployments may have lighting commands passing through a bus with lower throughput. A scenario can model slow scene transitions.

```yaml
faults:
  - target: lighting.bus.main
    type: throughput_limit
    commands_per_second: 20
```

Assertion:

```yaml
assert:
  scene: all_lights_off
  completed_within: 5s
```

## Future: floorplan mapping

A future module can map fixtures onto a floorplan/CAD coordinate system.

```yaml
fixture_map:
  D411S10:
    x: 1200
    y: 3400
    drawing_ref: L-101
```

This supports commissioning reports that tell field engineers exactly where to check.
