# 02. 製品要件

## Product name

`roomci`

## Product category

Docker-based smart-home QA and operations emulator for local development and CI.

## Target users

- Smart-home software engineers
- IoT engineers
- Field engineers
- Network engineers
- Commissioning engineers
- BMS / LCM / operations tooling engineers
- QA engineers
- Product engineers working across app, edge, devices, and operations

## 問題ステートメント

Smart-home systems are difficult to test because real behavior spans:

- mobile or tablet controllers
- local edge servers
- cloud services
- local and cloud MQTT brokers
- industrial or building-automation protocols
- physical devices
- electrical contacts
- network infrastructure
- BMS alerts and human operations
- field commissioning and maintenance

Many failures appear only on-site. Reproducing them requires devices, wiring, network topology, and operational context.

## 製品目的

Make smart-home field failures reproducible in Docker and CI.

## 成功基準

### 技術的成功

- A developer can run `docker compose up` and get a simulated one-house smart-home stack.
- Scenarios can be expressed in YAML.
- Local MQTT and edge server behavior can be tested without real devices.
- Cloud outage and WAN outage can be simulated.
- Modbus register maps can be validated.
- DALI-like scene failures can be tested.
- Contact I/O alerts can trigger BMS-like notifications.
- Reports can be consumed by GitHub Actions and CI systems.

### 製品成功

- The tool demonstrates deep understanding of local-first smart-home architecture.
- It makes commissioning knowledge executable.
- It models guest-impacting failures, not just protocol calls.
- It gives useful failure reports with possible field causes and suggested checks.

## MVP要件

### 機能要件

| ID | Requirement | Priority |
|---|---|---|
| FR-001 | Run scenario YAML files | Must |
| FR-002 | Provide Docker image | Must |
| FR-003 | Provide Docker Compose example | Must |
| FR-004 | Emulate local MQTT retained state | Must |
| FR-005 | Emulate edge server command routing | Must |
| FR-006 | Simulate cloud outage while local operation continues | Must |
| FR-007 | Emulate Modbus TCP register map | Must |
| FR-008 | Emulate DALI-like lighting scene | Must |
| FR-009 | Emulate contact I/O alert | Must |
| FR-010 | Generate Markdown report | Must |
| FR-011 | Generate JSON report | Must |
| FR-012 | Generate JUnit report | Should |
| FR-013 | Simulate QoS1 duplicate delivery | Should |
| FR-014 | Simulate edge server failover | Should |
| FR-015 | BMS Slack/phone/ticket mock | Should |
| FR-016 | Comfort automation module | Could |
| FR-017 | Intercom / DoorBird / SIP mock | Later |

### 非機能要件

| ID | Requirement | Priority |
|---|---|---|
| NFR-001 | CI-friendly startup time | Must |
| NFR-002 | Deterministic scenario execution | Must |
| NFR-003 | Single binary or lightweight container | Should |
| NFR-004 | Human-readable failure reports | Must |
| NFR-005 | No real device access required | Must |
| NFR-006 | Protocol mocks should be explicit approximations | Must |
| NFR-007 | Extensible module architecture | Should |

## 非目的

- Production device control
- Full KNX / DALI / BACnet protocol certification
- Real SIP trunking
- Real Twilio / Slack / PagerDuty calls in default mode
- Replacing a production BMS
- Replacing real commissioning

## MVPリリース定義

A v0.1 release is acceptable when the following commands work:

```bash
roomci validate examples/local_first_cloud_outage.yaml
roomci run examples/local_first_cloud_outage.yaml --report-md report.md --report-json report.json --junit report.xml

docker compose -f compose/docker-compose.yml up --abort-on-container-exit
```

## UX原則

Reports should be written for both software engineers and field engineers.

A bad report says:

```txt
assertion failed: device.state != expected
```

A good report says:

```txt
DALI fixture D411S10 did not reach the expected dimming level.
Possible field causes:
- wrong DALI address
- fixture not assigned to scene group
- wiring issue
- gateway command timeout
```
