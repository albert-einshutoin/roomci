# roomci

[![smart-home-ci](https://github.com/albert-einshutoin/roomci/actions/workflows/smart-home-ci.yml/badge.svg)](https://github.com/albert-einshutoin/roomci/actions/workflows/smart-home-ci.yml)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Coverage](https://img.shields.io/badge/coverage-86%25-brightgreen.svg)](#quality-gates)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Local-first Smart Home QA & Operations Emulator for CI**

`roomci` is a Docker-friendly emulator for reproducing smart-home commissioning, local-first control, field QA, BMS alerting, and operations scenarios without real devices or an on-site environment.

It is designed as a **LocalStack-like emulator**, but for smart-home and building-automation systems:

- LocalStack: emulate cloud services for local and CI testing.
- roomci: emulate a smart-home control stack for local and CI testing.

## Tagline

> Reproduce smart-home field failures before guests experience them.

## Quick start

```bash
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

# Or use the Docker image
docker build -t roomci:latest .
docker run --rm -v "$PWD/examples:/scenarios:ro" roomci:latest \
  run /scenarios/starlink_failover.yaml
```

## Passing demo scenarios

| Scenario | What it shows |
|---|---|
| `examples/local_first_cloud_outage.yaml` | iPad → local MQTT → edge → device works while the cloud broker is offline; retained state survives the outage. |
| `examples/edge_server_failover.yaml` | Primary edge loses power; standby is promoted and routes the next command. |
| `examples/modbus_floor_heating.yaml` | Floor-heating setpoint reaches the Modbus register with 0.1 °C precision. |
| `examples/bms_sauna_emergency_alert.yaml` | Sauna over-temperature contact opens; BMS escalates to Slack, phone, and ticket runbook. |
| `examples/starlink_failover.yaml` | WAN failover to Starlink within the configured budget. |
| `examples/comfort_auto_mode.yaml` | Discomfort index drives HVAC auto-mode; user override is respected. |
| `examples/access_permission_drift.yaml` | Detect stale access-system users who no longer exist in the identity group. |
| `examples/commissioning_checklist.yaml` | Generate field commissioning checks from room and device declarations. |

These scenarios are expected to pass with `roomci run`.

## Failure-report demo scenario

| Scenario | What it shows |
|---|---|
| `examples/dali_scene_partial_failure.yaml` | DALI scene activation with one fixture missing; `roomci run` exits non-zero and emits a guest-impact failure report. |

## Reports

`roomci run` emits three report formats from the same execution:

- `--report-json <path>` — full machine-readable run report (timeline, assertions, final state, retained MQTT messages).
- `--report-md <path>` — human-readable Markdown summary with guest-impact framing.
- `--junit <path>` — JUnit XML for CI dashboards (GitHub Actions, GitLab CI, Jenkins).

CI artifacts from `.github/workflows/smart-home-ci.yml` upload all three formats for every passing demo scenario and the intentional failure-report demo.

## CLI reference

```text
roomci run <scenarios...>
  --report-json <path>   write JSON report for the last scenario
  --report-md   <path>   write Markdown report for the last scenario
  --junit       <path>   write JUnit XML report for the last scenario
  --verbose              print every timeline event
  --quiet                suppress per-scenario detail
  --dry-run              validate only; do not execute

roomci validate <scenarios...>
  load and validate one or more scenario files without executing them

roomci serve --config <scenario> --check
  validate service-mode configuration without starting a long-running process
```

Exit codes: `0` (all scenarios passed), `1` (one or more assertions failed), `2` (load, validate, or runtime error).

## Quality gates

`.github/workflows/smart-home-ci.yml` enforces:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`
- `cargo doc --workspace --no-deps` (`RUSTDOCFLAGS=-D warnings`)
- `cargo tarpaulin --workspace --fail-under 80`

Current measurements: **69 tests** pass, **86.57%** line coverage.

## Core concept

Modern hospitality smart homes are not just IoT devices. They combine:

- iPad / mobile controllers
- local MQTT brokers
- edge servers / home control servers
- cloud MQTT brokers and cloud event pipelines
- DALI / KNX / Modbus / contact I/O / HVAC / lighting devices
- BMS alerts, Slack notifications, phone escalation, and runbooks
- network segmentation, WAN failover, Starlink-style backup paths
- commissioning, field QA, maintenance, and continuous operation

`roomci` makes these dependencies reproducible in CI so teams can test failure scenarios before guests experience them.

## Primary use cases

1. **Local-first control QA** — verify that local iPad → local MQTT → edge → device control still works when the cloud is unavailable.
2. **Commissioning QA** — turn field commissioning checks into reusable YAML scenarios.
3. **Building-automation protocol simulation** — DALI-like lighting, Modbus registers, contact I/O alerts, HVAC, KNX-like legacy bus behavior.
4. **BMS / operations alert simulation** — verify Slack / phone-call / ticket / runbook flows for emergency alerts and recoveries.
5. **Network failure simulation** — ISP outage, Starlink-style failover, VLAN isolation issues, local-only operation.
6. **Comfort automation simulation** — discomfort-index targets, HVAC auto mode, user override, room-specific tuning.

## What this is not

`roomci` is not intended to be:

- a full KNX, DALI, BACnet, or Matter implementation
- a replacement for production smart-home controllers
- a real BMS product
- a production-grade SIP / PBX implementation

The goal is to model enough behavior to make smart-home QA, field failures, commissioning, and operations flows testable in local and CI environments.

## Directory structure

```txt
roomci/
  crates/                    Rust workspace
    roomci-cli/              CLI entry point (binary)
    roomci-core/             scenario runner (virtual time, assertions)
    roomci-mqtt/             local + cloud MQTT broker model
    roomci-edge/             redundant edge-server emulator
    roomci-device-model/     Modbus, DALI lighting, contact I/O
    roomci-fault/            fault scheduling primitives
    roomci-ops/              BMS / Slack / phone / runbook escalation
    roomci-report/           JSON / Markdown / JUnit renderers
    roomci-scenario/         YAML scenario loader + validator
  examples/                  8 passing demos and 1 failure-report demo
  schemas/scenario.schema.json   JSON Schema for scenario files
  docs/                      architecture, protocols, scenario spec
  tasks/                     phase-by-phase build log
  .github/workflows/         CI pipeline
  Dockerfile                 multi-stage image (binary only)
```

## Positioning

> I analyzed public NOT A HOTEL smart-home hiring pages, videos, and engineering articles. The public materials suggest their smart-home work is not just IoT device control; it is a local-first, MQTT-driven, edge-server-based, building-automation and operations platform. I designed `roomci` as a Docker-friendly external QA emulator to turn field QA, commissioning knowledge, and operational failure modes into repeatable CI scenarios.
