# roomci

[![CI](https://img.shields.io/badge/ci-make%20verify-blue.svg)](#quality-gates)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Coverage](https://img.shields.io/badge/coverage-88.07%25-green.svg)](#quality-gates)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

📖 **[日本語はこちら](README.ja.md)** | **[English](README.md)**

**MQTT / Edge / Device QA Contract Emulator for CI**

`roomci` is a Docker-friendly **QA contract emulator** for MQTT-based edge, smart-home, and building-automation systems. It turns command/state contracts, local-first failure modes, device-adapter behavior, commissioning checks, and operations escalation into repeatable local and CI scenarios.

It is a **behavioral emulator**: it validates the behavior a controller, edge service, device adapter, or operations flow should satisfy without requiring real devices or an on-site environment.

It is designed as a **LocalStack-like emulator**, but for MQTT/edge/device QA:

- LocalStack: emulate cloud services for local and CI testing.
- roomci: emulate device, edge, MQTT, and operations contracts for local and CI testing.

## Tagline

> Reproduce edge-device and smart-home field failures before users experience them.

## Why this exists

MQTT and edge-device quality is not only whether one command works. It is whether state synchronization, retained messages, local-first control, edge failover, device-adapter behavior, and operations escalation survive realistic failure modes.

Hospitality smart home is the strongest included domain pack because failures affect guest experience directly: cloud outages, lighting scene drift, access-permission drift, comfort automation, and BMS escalation. `roomci` models this domain-specific depth while framing it as one application of a reusable QA contract emulator.

`roomci` turns those failure modes into repeatable scenarios that can run locally, in Docker, or in CI.

## Try the strongest demo

Run the local-first outage scenario with a verbose timeline:

```bash
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --verbose
```

Then run the intentional failure-report demo:

```bash
cargo run -p roomci-cli -- run examples/dali_scene_partial_failure.yaml \
  --report-md reports/dali_scene_partial_failure.md
```

Expected result: `roomci run` exits with code `1` and writes a guest-impact report.

Report excerpt:

```text
Result: failed
scene_consistency:welcome
DALI-like scene consistency violation: D411S10 expected level 60, actual 0
Guest impact: Lighting scene did not match intended guest ambience.
```

For the full interview walkthrough, see [`docs/INTERVIEW_DEMO.md`](docs/INTERVIEW_DEMO.md). For canonical product positioning, see [`docs/PRODUCT_GUIDE.md`](docs/PRODUCT_GUIDE.md), [`docs/DOMAIN_PACKS.md`](docs/DOMAIN_PACKS.md), and [`docs/GENERIC_MQTT_CONTRACTS.md`](docs/GENERIC_MQTT_CONTRACTS.md).
For protocol subset claims, see [`docs/PROTOCOL_CONFORMANCE_REGISTRY.md`](docs/PROTOCOL_CONFORMANCE_REGISTRY.md).

For evaluator onboarding, use [`docs/INTEGRATION_ONBOARDING.md`](docs/INTEGRATION_ONBOARDING.md), [`docs/EVALUATION_EVIDENCE_PACK.md`](docs/EVALUATION_EVIDENCE_PACK.md), and [`docs/CATEGORY_READINESS.md`](docs/CATEGORY_READINESS.md).
For real hardware capture replay in Docker CI, see [`docs/HARDWARE_TO_DOCKER_CI_USECASES.md`](docs/HARDWARE_TO_DOCKER_CI_USECASES.md).
For dependency updates, RustSec exceptions, and the `serde_yaml` compatibility
hold, see [`docs/DEPENDENCY_POLICY.md`](docs/DEPENDENCY_POLICY.md).

## Quick start

```bash
# Full curated demo
make demo

# Hospitality smart-home domain demo
make demo-hospitality

# Generic MQTT contract demo
make demo-generic-mqtt

# Black-box serve-mode PoC with an external controller container
make compose-poc

# Real-hardware contract replay as Docker CI use cases
make hardware-ci-usecases-smoke

# Full local verification
make verify

# Build the workspace
cargo build --workspace --release

# Validate a scenario
cargo run --release -- validate examples/local_first_cloud_outage.yaml

# Run a single scenario and emit reports
cargo run --release -- run examples/local_first_cloud_outage.yaml \
  --report-json reports/local_first.json \
  --report-md   reports/local_first.md \
  --junit       reports/local_first.xml

# Run multiple scenarios and aggregate exit codes
cargo run --release -- run \
  examples/local_first_cloud_outage.yaml \
  examples/modbus_floor_heating.yaml \
  examples/bms_sauna_emergency_alert.yaml \
  examples/access_permission_drift.yaml \
  examples/commissioning_checklist.yaml

# Dry-run (validate only, do not execute)
cargo run --release -- run --dry-run examples/comfort_auto_mode.yaml

# Verbose timeline output
cargo run --release -- run --verbose examples/edge_server_failover.yaml

# Service-mode config check
cargo run --release -- serve --config examples/local_first_cloud_outage.yaml --check

# Start localhost HTTP control/report API
cargo run --release -- serve --config examples/generic_mqtt_retained_state.yaml --port 8080

# Start HTTP plus minimal MQTT ingress for external PoC clients
cargo run --release -- serve \
  --config examples/generic_mqtt_retained_state.yaml \
  --port 8080 \
  --mqtt-port 1883

# Or use the Docker image
docker build -t roomci:latest .
docker run --rm -v "$PWD/examples:/scenarios:ro" roomci:latest \
  run /scenarios/starlink_failover.yaml
```

## Demo paths

| Demo | Audience | Command |
|---|---|---|
| Curated full demo | General product review | `make demo` |
| Hospitality domain pack | Hospitality and smart-home engineering teams | `make demo-hospitality` |
| Generic MQTT contracts | MQTT / edge-device platform teams | `make demo-generic-mqtt` |
| Serve-mode PoC | Teams validating external-controller integration shape | `make compose-poc` |
| Hardware-to-Docker CI | Teams converting real-device captures into repeatable CI evidence | `make hardware-ci-usecases-smoke` |

## Passing demo scenarios

| Scenario | What it shows |
|---|---|
| `examples/generic_mqtt_retained_state.yaml` | Generic MQTT command/state contract updates retained state without hospitality naming. |
| `examples/generic_mqtt_duplicate_delivery.yaml` | Duplicate MQTT delivery remains idempotent for retained command/state behavior. |
| `examples/local_first_cloud_outage.yaml` | iPad → local MQTT → edge → device works while the cloud broker is offline; retained state survives the outage. |
| `examples/edge_server_failover.yaml` | Primary edge loses power; standby is promoted and routes the next command. |
| `examples/modbus_floor_heating.yaml` | Floor-heating setpoint reaches the Modbus register with 0.1 °C precision. |
| `examples/bms_sauna_emergency_alert.yaml` | Sauna over-temperature contact opens; BMS escalates to Slack, phone, and ticket runbook. |
| `examples/starlink_failover.yaml` | Backup WAN failover within the configured budget. |
| `examples/comfort_auto_mode.yaml` | Discomfort index drives HVAC auto-mode; user override is respected. |
| `examples/comfort_timeseries_replay.yaml` | Deterministic sensor-zone readings update discomfort-index evidence. |
| `examples/access_permission_drift.yaml` | Detect stale access-system users who no longer exist in the identity group. |
| `examples/commissioning_checklist.yaml` | Generate field commissioning checks from room and device declarations. |
| `examples/intercom_relay_safe_mock.yaml` | PIN/call/relay evidence is captured without controlling real unlocks. |
| `examples/network_control_panel_fault_profiles.yaml` | Network, firewall, UPS, circuit, and redundant-PSU fault profiles emit BMS evidence. |

These scenarios are expected to pass with `roomci run`.

## Protocol-profile fixtures

These B Tier fixtures validate customer-supplied protocol maps as
`contract_profile` metadata. They are not wire-protocol endpoints or
certification evidence.

| Scenario | What it shows |
|---|---|
| `examples/matter_gateway_profile.yaml` | Matter gateway endpoint/cluster/attribute/command mapping as contract evidence. |
| `examples/bacnet_contract_profile.yaml` | BACnet object/property/event mapping as contract evidence. |
| `examples/knx_group_address_profile.yaml` | KNX group-address/datapoint/function mapping as contract evidence. |
| `examples/opcua_contract_profile.yaml` | OPC UA node/attribute/event mapping as contract evidence. |

Run them with `make protocol-profile-smoke`.

## Local VSCode Authoring Assets

`tools/vscode-roomci/` contains a local-only VSCode asset bundle for scenario
authoring: YAML schema association, snippets, and task definitions for `roomci
validate`, `roomci run`, `make verify`, and `make protocol-evidence`. It is not
a Marketplace extension. Validate it with `make vscode-assets-check`.

## Failure-report demo scenario

| Scenario | What it shows |
|---|---|
| `examples/dali_scene_partial_failure.yaml` | DALI scene activation with one fixture missing; `roomci run` exits non-zero and emits a guest-impact failure report. |

## Reports

`roomci run` emits CI-friendly report and evidence formats from the same execution:

## Use in GitHub Actions

Add this action after checkout to run scenarios, upload complete evidence, and
append a concise pass/fail table to the job summary:

```yaml
- uses: albert-einshutoin/roomci@v0.1.0
  with:
    scenarios: examples/local_first_cloud_outage.yaml examples/edge_server_failover.yaml
    report-dir: roomci-reports
```

Copy the complete workflow from
[`examples/github-actions/roomci-poc.yml`](examples/github-actions/roomci-poc.yml).
This Docker action builds from source on each run. A faster published-image
variant is tracked in [#10](https://github.com/albert-einshutoin/roomci/issues/10).
Scenario paths containing spaces are unsupported. `extra-args` is restricted
to `--verbose`, `--quiet`, `--dry-run`, and `--run-id VALUE`; output-path flags
are rejected so the root action container cannot write outside its report
bundle contract.

- `--report-json <path>` — full machine-readable run report (timeline, assertions, final state, retained MQTT messages).
- `--report-md <path>` — human-readable Markdown summary with guest-impact framing.
- `--junit <path>` — JUnit XML for CI dashboards (GitHub Actions, GitLab CI, Jenkins).
- `--timeline-json <path>` — stable timeline event export for log/search/trace ingestion.
- `--timeline-ndjson <path>` — newline-delimited timeline event export.
- `--observability-json <path>` — deterministic counters and run summary for external observability ingestion.
- `--run-id <id>` — stable run correlation id used by JSON, timeline, and observability artifacts.
- `--report-dir <dir>` — one command writes JSON, Markdown, JUnit, timeline, and observability evidence for every scenario plus `summary.json`.
- `--github-summary <path>` — append the aggregate Markdown result to a GitHub Step Summary-compatible file. When this flag is absent, `GITHUB_STEP_SUMMARY` is detected automatically; automatic write failures only warn.

For a CI artifact set covering multiple scenarios:

```bash
roomci run \
  examples/local_first_cloud_outage.yaml \
  examples/edge_server_failover.yaml \
  --report-dir reports/poc
```

This creates `reports/poc/summary.json` and one numbered directory per input
file (for example, `01_local_first_cloud_outage/`). Each scenario directory
contains `report.json`, `report.md`, `report.junit.xml`, `timeline.json`,
`timeline.ndjson`, and `observability.json`. The stable
`roomci.summary.v1` summary contains aggregate pass/fail counts and an entry
for every input scenario. `--dry-run --report-dir <dir>` writes only that
summary, marking each entry as `dry_run: true`.
Use a fresh or run-specific report directory so artifacts from an earlier,
larger scenario set are not mistaken for output from the current run.

CI artifacts from `.github/workflows/smart-home-ci.yml` upload report,
timeline, and observability formats for every passing demo scenario and the
intentional failure-report demo. See
[`docs/S_TIER_EVIDENCE_GUIDE.md`](docs/S_TIER_EVIDENCE_GUIDE.md) for the
copy-paste evaluator path.

For Python automation and scenario debugging, see
[`docs/DEVELOPER_WORKFLOW.md`](docs/DEVELOPER_WORKFLOW.md).

For Matter, BACnet, KNX, and OPC UA contract-profile boundaries, see
[`docs/B_TIER_PROTOCOL_PROFILES.md`](docs/B_TIER_PROTOCOL_PROFILES.md).

## CLI reference

```text
roomci run <scenarios...>
  --report-dir <dir>     write per-scenario evidence and summary.json
  --report-json <path>   write JSON report for the last scenario
  --report-md   <path>   write Markdown report for the last scenario
  --junit       <path>   write JUnit XML report for the last scenario
  --timeline-json <path> write stable timeline JSON for the last scenario
  --timeline-ndjson <path>
                         write stable newline-delimited timeline JSON
  --observability-json <path>
                         write observability summary JSON
  --run-id <id>          set the run correlation id for exported artifacts
  --verbose              print every timeline event
  --quiet                suppress per-scenario detail
  --dry-run              validate only; do not execute

roomci validate <scenarios...>
  load and validate one or more scenario files without executing them

roomci debug <scenario>
  --debug-json <path>    write deterministic debug JSON
  --debug-md   <path>    write deterministic debug Markdown

roomci serve --config <scenario> --check
  validate service-mode configuration without starting a long-running process

roomci serve --config <scenario>
  start a localhost-bound HTTP control/report API

roomci serve --config <scenario> --mqtt-port <port>
  also start the minimal MQTT 3.1.1 CONNECT + QoS0 PUBLISH PoC ingress
```

Exit codes: `0` (all scenarios passed), `1` (one or more assertions failed), `2` (load, validate, or runtime error).

## Serve-mode HTTP API

`roomci serve` starts on `127.0.0.1:8080` by default. It refuses non-loopback hosts unless `--allow-non-loopback` is passed.

Current HTTP endpoints:

- `GET /health`
- `GET /scenario`
- `GET /state`
- `GET /timeline`
- `GET /timeline.export.json`
- `GET /timeline.ndjson`
- `POST /fault`
- `POST /run`
- `POST /finish`
- `GET /reports/latest`
- `GET /reports/latest.json`
- `GET /reports/latest.md`
- `GET /reports/latest.junit.xml`
- `GET /observability/latest.json`

This is a local PoC integration surface, not a production API or MQTT broker replacement.

## Serve-mode MQTT PoC

`roomci serve --mqtt-port <port>` enables a minimal MQTT-shaped ingress for external PoC clients. It supports MQTT 3.1.1 `CONNECT` and QoS0 `PUBLISH` with JSON object payloads. Published topics are matched against `mqtt.contracts` in the scenario file, and matching payloads update retained state visible through the HTTP state and report endpoints.

Example contract:

```yaml
mqtt:
  contracts:
    - name: generic_device_retained_state
      adapter: mqtt_v3_qos0_subset
      command_topic: fleet/demo/site/lab/device/{device_id}/command
      state_topic: fleet/demo/site/lab/device/{device_id}/state
      device_id_from_topic: placeholder:{device_id}
      payload:
        required_fields: [online, sample_interval_seconds]
```

The supported subset is documented in [`docs/MQTT_SERVE_SUBSET.md`](docs/MQTT_SERVE_SUBSET.md). The integration checklist is in [`docs/PRE_ADOPTION_POC_CHECKLIST.md`](docs/PRE_ADOPTION_POC_CHECKLIST.md).

`roomci serve --modbus-port <port>` enables a minimal Modbus TCP subset for external PoC clients. It supports MBAP, read holding/input register, write single register, and documented exception responses. The supported subset is documented in [`docs/MODBUS_TCP_SUBSET.md`](docs/MODBUS_TCP_SUBSET.md).

The black-box PoC path is:

```bash
make compose-poc
```

That command starts `roomci serve` in Docker Compose, runs `examples/controllers/http_poc_controller.sh` as a separate controller service, drives the HTTP API, and writes JSON, Markdown, and JUnit reports under `reports/`.

## Quality gates

`.github/workflows/smart-home-ci.yml` enforces:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`
- `cargo doc --workspace --no-deps` (`RUSTDOCFLAGS=-D warnings`)
- `cargo tarpaulin --workspace --fail-under 80`
- `bash ./scripts/check-readme-quality.sh`

Current measurements: **169 tests** pass, **88.07%** line coverage.

Use [`docs/RELEASE_CHECKLIST.md`](docs/RELEASE_CHECKLIST.md) before presenting a release or company evaluation build. Use [`docs/PROTOCOL_SUPPORT_MATRIX.md`](docs/PROTOCOL_SUPPORT_MATRIX.md) as the source of truth for behavior-model support versus wire-protocol support.

## Distribution

Release tarballs and checksums are published for Linux x86_64/ARM64 and macOS
Intel/Apple Silicon. The runtime image is
`ghcr.io/albert-einshutoin/roomci:<version>`; no mutable `latest` tag is
published. See [`docs/RELEASING.md`](docs/RELEASING.md) for tag, attestation,
install/upgrade commands, and deferred-platform policy.

Company-specific adapter contracts can be validated before runtime work:

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml
```

Evaluator PoC packs are available as one-command targets:

```bash
make poc-generic-mqtt
make poc-core-qa
make poc-hospitality
make poc-building-automation
make poc-bms-ops
```

Each PoC target writes one JSON/Markdown/JUnit report set per scenario under
`reports/`. When `roomci run` is called directly with multiple scenarios, report
flags still write the last scenario only; use per-scenario commands for
presentation evidence.

## Core concept

The reusable core is a QA contract emulator:

- scenario definitions
- MQTT command/state and retained-state contracts
- edge routing and failover behavior
- failure injection
- device and operations models
- JSON, Markdown, and JUnit reports
- CI execution through Cargo, Docker, and Compose

Hospitality smart homes are a high-signal domain pack on top of that core. They combine:

- iPad / mobile controllers
- local MQTT brokers
- edge servers / home control servers
- cloud MQTT brokers and cloud event pipelines
- DALI / KNX / Modbus / contact I/O / HVAC / lighting devices
- BMS alerts, Slack notifications, phone escalation, and runbooks
- network segmentation, WAN failover, backup-link paths
- commissioning, field QA, maintenance, and continuous operation

`roomci` makes these dependencies reproducible in CI so teams can test failure scenarios before users or guests experience them.

## Primary use cases

1. **Generic MQTT contract QA** — verify command/state topics, retained state, duplicate delivery, and local broker availability.
2. **Local-first control QA** — verify that local controller → local MQTT → edge → device control still works when the cloud is unavailable.
3. **Commissioning QA** — turn field commissioning checks into reusable YAML scenarios.
4. **Building-automation protocol simulation** — DALI-like lighting, Modbus registers, contact I/O alerts, HVAC, KNX-like legacy bus behavior.
5. **BMS / operations alert simulation** — verify Slack / phone-call / ticket / runbook flows for emergency alerts and recoveries.
6. **Network failure simulation** — ISP outage, backup-WAN failover, VLAN isolation issues, local-only operation.
7. **Comfort automation simulation** — discomfort-index targets, HVAC auto mode, user override, room-specific tuning.

## What this is not

`roomci` is not intended to be:

- a production MQTT broker or MQTT broker replacement
- a protocol conformance suite
- a full KNX, DALI, BACnet, or Matter implementation
- a replacement for production smart-home controllers
- a real BMS product
- a production-grade SIP / PBX implementation

The goal is to model enough behavior to make MQTT/edge/device contracts, smart-home QA, field failures, commissioning, and operations flows testable in local and CI environments.

## Directory structure

```txt
roomci/
  crates/                    Rust workspace
    roomci-cli/              CLI entry point (binary)
    roomci-core/             scenario runner (virtual time, assertions)
    roomci-mqtt/             local + cloud MQTT behavior model
    roomci-edge/             redundant edge-server emulator
    roomci-device-model/     Modbus, DALI lighting, contact I/O
    roomci-ops/              BMS / Slack / phone / runbook escalation
    roomci-report/           JSON / Markdown / JUnit renderers
    roomci-scenario/         YAML scenario loader + validator
  examples/                  generic MQTT, hospitality, building automation, and failure-report demos
  schemas/scenario.schema.json   JSON Schema for scenario files
  docs/                      architecture, protocols, scenario spec
  tasks/                     phase-by-phase build log
  .github/workflows/         CI pipeline
  Dockerfile                 multi-stage image (binary only)
```

## Positioning

`roomci` is positioned as a reusable MQTT/edge/building-automation QA contract emulator. The strongest shipped domain pack is hospitality smart home: a local-first, MQTT-driven, edge-server-based system that combines device control, building-automation protocols, field commissioning, BMS operations, and network reliability into a cohesive stay-experience model.

The product does not depend on any specific company's private implementation. A real integration would require your team's actual MQTT topics, payload schemas, device/register maps, BMS/webhook contracts, authentication model, and acceptance criteria.
