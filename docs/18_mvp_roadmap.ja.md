# 18. MVPロードマップ

## v0.1: Local-first QA MVP

### 目的

Demonstrate that one smart-home stack can be reproduced in Docker and tested in CI.

### 機能

- CLI: `roomci run`, `roomci validate`, `roomci serve --check`
- Scenario YAML parser
- Virtual time and event timeline
- Local MQTT retained-state model
- Edge server emulator
- iPad controller simulator
- Modbus TCP device mock
- DALI-like lighting mock
- Contact I/O mock
- BMSアラート mock
- Cloud outage and local-only scenario
- Markdown / JSON / JUnit reports
- Docker image
- Docker Compose example
- GitHub Actions example

Long-running `roomci serve` adapters are post-v0.1. The current MVP includes `roomci serve --check` for service-mode configuration validation.

### Required demos

1. Cloud outage but local lighting control works
2. Modbus floor heating 0.1°C control
3. DALI scene partial failure
4. Sauna emergency contact triggers Slack/phone/ticket
5. Edge server failover

## v0.1.5: Cloud bridge and PMS automation

### 機能

- cloud MQTT broker mock
- AWS IoT Core-like publish endpoint
- rule-router mock
- SQS-like queue
- Cloud Run callback mock
- PMS event emulator
- checkout / cleaning-completed automation scenarios

### Demos

- checkout resets lights/HVAC/sauna/cold bath
- cloud feedback queued and delivered after recovery
- cloud command reaches local broker after bridge reconnect

## v0.2: Field commissioning and operations depth

### 機能

- KNX-like group address model
- KNX throughput bottleneck simulation
- DALI layout/group validation
- VLAN isolation checker
- Starlink failover profile
- control-panel power model
- UPS/circuit protector simulation
- comfort automation module
- Lua hook support
- Go backend sample

### Demos

- KNX lighting throughput delay
- VLAN leak exposes device-control network
- Starlink backup activates
- HVAC auto mode reaches DI target
- circuit short isolated by circuit protector

## v0.3: Intercom, access control, and BMS maturity

### 機能

- DoorBird-like API mock
- PIN check-in flow
- SIP/DTMF event simulation
- Twilio outage / local PBX fallback
- UniFi access-control drift checker
- Okta LDAP group sync mock
- ONVIF-like camera event
- PoE speaker webhook mock
- floorplan-based BMS view concept

### Demos

- DTMF opens gate relay
- staff call falls back to local PBX
- retired user remains in access group and is detected
- alert location shown on floorplan mock

## v0.4+: Intelligence and scale

### 機能

- real time-series replay
- AI-assisted incident summary
- predictive anomaly detection sample
- CAD/Rebro export concept
- device inventory and spare parts model
- commissioning checklist generator
- dogfooding feedback replay

## スコープ control rule

If a feature does not improve one of the following, defer it:

- local-first guest experience reliability
- commissioning repeatability
- field debugging clarity
- BMS/ops response quality
- protocol/device integration confidence
- interview/demo storytelling strength

## Public repository milestones

### Milestone 1

- README
- architecture docs
- one working CLI stub
- example scenarios

### Milestone 2

- scenario runner
- reports
- local MQTT model
- edge emulator

### Milestone 3

- Modbus mock
- DALI mock
- contact alert
- Docker image

### Milestone 4

- GitHub Actions
- 5 polished demos
- demo video
- article explaining design decisions
