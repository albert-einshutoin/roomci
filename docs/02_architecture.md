# 02. Architecture

## High-level Architecture

```text
              +-------------------+
              |  Backend / App    |
              |  under test       |
              +---------+---------+
                        |
          HTTP / MQTT / WebSocket
                        |
+-----------------------v-----------------------+
|                    roomci                     |
|                                               |
|  +----------------+   +--------------------+  |
|  | Protocol       |   | Scenario Runner     |  |
|  | Adapters       |   |                    |  |
|  | - HTTP         |   | - timeline         |  |
|  | - MQTT         |   | - assertions       |  |
|  | - shadow-like  |   | - reports          |  |
|  +--------+-------+   +----------+---------+  |
|           |                      |            |
|  +--------v----------------------v---------+  |
|  |              Core Engine                |  |
|  | - device registry                       |  |
|  | - capability model                      |  |
|  | - state machine                         |  |
|  | - event bus                             |  |
|  | - fault injection                       |  |
|  +------------------+----------------------+  |
|                     |                         |
|  +------------------v----------------------+  |
|  |             Persistence                  |  |
|  | - in-memory state                        |  |
|  | - optional JSON snapshot                 |  |
|  +-----------------------------------------+  |
+-----------------------------------------------+
```

## Crate Layout

```text
roomci/
  crates/
    roomci-core/
    roomci-device-model/
    roomci-scenario/
    roomci-fault/
    roomci-http/
    roomci-mqtt/
    roomci-shadow/
    roomci-report/
    roomci-cli/
  docker/
  examples/
  schemas/
```

## Core Modules

### Device Registry

Stores rooms, devices, capabilities, state, and metadata.

### Capability Model

A device is composed from capabilities.

Examples:
- `lock` capability
- `on_off` capability
- `brightness` capability
- `temperature_measurement` capability
- `thermostat` capability
- `occupancy` capability
- `cover_position` capability

### Event Bus

Internal event stream.

Events:
- `CommandReceived`
- `CommandAccepted`
- `CommandRejected`
- `StateChanged`
- `TelemetryPublished`
- `FaultActivated`
- `FaultCleared`
- `AssertionPassed`
- `AssertionFailed`

### Scenario Runner

Runs YAML-defined timeline steps against the engine.

### Fault Engine

Applies deterministic or probabilistic failure behavior.

### Protocol Adapters

Expose external APIs over HTTP, MQTT, and cloud-IoT-like compatibility layers.

## Runtime Modes

### `serve`

Long-running emulator server.

```bash
roomci serve --config room.yaml --http 0.0.0.0:8080 --mqtt 0.0.0.0:1883
```

### `run`

Run a scenario to completion and exit.

```bash
roomci run scenarios/checkin_lock_offline.yaml --junit report.xml --markdown report.md
```

### `validate`

Validate scenario/config files.

```bash
roomci validate scenarios/*.yaml
```

## State Consistency

All device state changes go through the core engine.

Adapters must not mutate state directly.

```text
Adapter -> Command -> Core Engine -> State Transition -> Events -> Adapter Notifications
```

## Determinism

CI scenarios must be deterministic by default.

- Random faults require a fixed seed.
- Timeline clock is simulated.
- Real time is optional only in `serve` mode.

## Report Outputs

- JSON: machine-readable details.
- JUnit XML: CI integration.
- Markdown: PR summaries.
- Timeline: human-readable event sequence.
