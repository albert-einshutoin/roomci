# 12. 制御パネル障害モデル

## なぜ model the control panel?

The control panel is the physical heart of a smart home. It connects lighting, HVAC, locks, sauna, bath, sensors, relays, edge servers, and power systems.

A useful emulator should model not only devices but also the failure modes around the control panel.

## Control panel components

```yaml
control_panel:
  id: CP-01
  power:
    dc24v:
      supplies:
        - id: psu_01
          status: ok
        - id: psu_02
          status: ok
      redundancy_unit: enabled
      ups:
        id: ups_24v_01
        status: ok
        battery_health: 100
      circuit_protectors:
        - id: cp_light_line_01
          status: closed
        - id: cp_sensor_line_01
          status: closed
  edge_servers:
    primary: edge_primary
    secondary: edge_secondary
```

## Faults

| Fault | Meaning |
|---|---|
| psu_failure | one 24V power supply fails |
| ups_degraded | UPS battery health below threshold |
| circuit_trip | a protected circuit trips |
| short_circuit | simulated short on one branch |
| edge_primary_failure | primary edge server stops |
| edge_failover_failed | secondary does not take over |
| module_offline | one control module becomes unreachable |

Executable Phase 17 profiles:

| Target | Type | Timeline Evidence |
|---|---|---|
| `control_panel.ups` | `battery_degraded` | `control_panel_ups_degraded` |
| `control_panel.circuit_protector.<id>` | `tripped` | `control_panel_circuit_protector_tripped` |
| `control_panel.psu.<id>` | `degraded` | `control_panel_redundant_psu_degraded` |
| `edge.secondary` | `takeover_failed` | `edge_redundancy_takeover_failed` |

These profiles are QA evidence only. They do not validate wiring safety,
breaker behavior, or physical redundancy.

## Scenario: PSU failure

```yaml
scenario:
  name: psu_redundancy

faults:
  - at: T+10s
    target: control_panel.CP-01.power.dc24v.psu_01
    type: psu_failure

assertions:
  - at: T+11s
    target: control_panel.CP-01.power.dc24v
    condition: powered
  - at: T+12s
    target: ops
    event: psu_failure_alert_sent
```

## Scenario: circuit protector trip

```yaml
scenario:
  name: circuit_protector_isolates_short

faults:
  - at: T
    target: circuit.light_line_01
    type: short_circuit

assertions:
  - at: T+1s
    target: circuit.light_line_01
    condition: tripped
  - at: T+1s
    target: circuit.sensor_line_01
    condition: alive
  - at: T+2s
    target: ops
    event: circuit_trip_alert_sent
```

## Scenario: edge failover

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

## Reporting requirements

A control-panel fault report should include:

- failed component
- affected circuits/devices
- guest impact
- operations impact
- likely field checks
- whether redundancy worked
- whether BMSアラート fired

Example:

```txt
PASS edge_server_failover

Fault:
- edge.primary power_lost at T+10s

Redundancy:
- edge.secondary active at T+12s

Guest impact:
- none

Field checks:
- inspect primary edge server power
- verify 24V line
- check UPS/breaker status
```
