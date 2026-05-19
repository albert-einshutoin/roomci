# 03. Architecture

## High-level architecture

```txt
                +-------------------------+
                |        CI Runner        |
                | GitHub Actions / Local  |
                +-----------+-------------+
                            |
                            v
+-------------------------------------------------------------+
|                         roomci                              |
|                                                             |
|  +-------------------+      +----------------------------+   |
|  | Scenario Runner   |----->| Assertion / Report Engine  |   |
|  +-------------------+      +----------------------------+   |
|             |                                               |
|             v                                               |
|  +-------------------+      +----------------------------+   |
|  | Virtual Time      |----->| Event Timeline             |   |
|  +-------------------+      +----------------------------+   |
|             |                                               |
|             v                                               |
|  +-------------------------------------------------------+   |
|  | Emulation Modules                                     |   |
|  |                                                       |   |
|  | MQTT | Edge | Devices | BMS | Network | Comfort       |   |
|  +-------------------------------------------------------+   |
+-------------------------------------------------------------+
```

## Runtime modes

### 1. Scenario mode

Runs a scenario file and exits with pass/fail.

```bash
roomci run examples/local_first_cloud_outage.yaml
```

Use this in CI.

### 2. Future service mode

Would start HTTP/MQTT/adapter endpoints for integration tests. This is not part of the current CLI.

```bash
# planned, not implemented in v0.1
roomci serve --config examples/house.yaml
```

Use this future mode for local development or backend/mobile E2E tests once implemented.

### 3. Compose stack mode

Starts multiple service containers to reproduce a full smart-home stack.

```bash
docker compose -f compose/docker-compose.yml up
```

Use this when demonstrating Floci-like service emulation.

## Core modules

### roomci-core

- scenario parser
- virtual time engine
- event timeline
- assertions
- report generation

### roomci-mqtt

- local broker model
- cloud broker mock
- retained messages
- QoS behavior simulation
- reconnect/recovery simulation

### roomci-edge

- home control server emulator
- local command routing
- device command adapters
- local-first fallback behavior

### roomci-devices

- Modbus mock
- DALI-like lighting mock
- KNX-like legacy bus
- contact I/O
- HVAC
- curtain/blind
- sauna/bath

### roomci-control-panel

- 24V power supply fault
- UPS degradation
- circuit protector trip
- redundant edge failover

### roomci-network

- local network partition
- WAN outage
- ISP degradation
- Starlink-style failover
- VLAN segmentation checks

### roomci-ops

- BMS alert
- Slack notification mock
- phone escalation mock
- ticket state
- runbook links
- recovery notifications

### roomci-comfort

- temperature/humidity model
- discomfort index calculation
- HVAC auto mode
- user override events

## Data flow: local-first operation

```txt
iPad Controller
   |
   | MQTT command
   v
Local MQTT Broker
   |
   | retained command/state
   v
Edge Server / Home Control Server
   |
   | protocol-specific command
   v
Device Adapter
   |
   | resulting state
   v
Local MQTT Broker retained state
   |
   v
iPad Controller receives updated state
```

## Data flow: cloud bridge

```txt
Local MQTT Broker
   |
   v
Edge MQTT Client / Bridge
   |
   v
Cloud MQTT Broker Mock / AWS IoT Core-like endpoint
   |
   v
Rule Router
   |
   v
SQS-like Queue
   |
   v
Cloud Run Callback Mock
```

## Design choices

### Why not full protocol compatibility?

Full KNX, DALI, BACnet, and SIP implementations would be too large for an interview-oriented OSS MVP. `roomci` should provide behavioral simulation sufficient for QA scenarios.

### Why Docker-first?

The target workflow is local and CI testing. Docker makes it easy to start deterministic service dependencies without requiring devices, gateways, or physical wiring.

### Why YAML scenarios?

Commissioning knowledge is often procedural. YAML can turn checklists into executable scenarios.

### Why Rust?

Rust fits because `roomci` needs:

- deterministic state machines
- concurrent network servers
- low-overhead Docker images
- single-binary distribution
- safe handling of event order, timeouts, retries, and faults

Rust is not positioned as a replacement for Go/Lua/Swift/TypeScript production stacks. It is used as an external QA engine.
