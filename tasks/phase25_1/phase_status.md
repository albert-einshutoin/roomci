# Phase 25.1 Status: Rust Type Boundary Maximization

## Status

`done`

## Task Board

| Task | Status | Owner | Notes |
|---|---|---|---|
| 01 Validated Scenario Boundary | `done` | Codex | `ValidatedScenario` now owns the raw-to-typed boundary before runtime execution. |
| 02 Identifier and Topic Newtypes | `done` | Codex | Promoted IDs and MQTT topics/templates parse through newtypes. |
| 03 Validated Step, Fault, and Assertion Model | `done` | Codex | Runtime consumes owned validated step, fault, and assertion variants. |
| 04 Typed Condition Parser | `done` | Codex | Promoted target/condition assertions parse into `Condition` variants once. |
| 05 Runtime Typed State | `done` | Codex | Promoted runtime state remains typed internally and reports convert at the boundary. |
| 06 Domain Config Projection | `done` | Codex | Runtime-read edge/WAN/comfort/access/commissioning config projects into typed config. |
| 07 API Deprecation and Cleanup | `done` | Codex | Core execution no longer calls borrowed `typed_*` classifiers directly. |
| 08 Phase 26 Readiness Gate | `done` | Codex | Quality gates passed; Phase 26 may start on top of the typed boundary. |

## Quality Gates

- `cargo fmt --all --check` - passed
- `cargo check --workspace --all-targets` - passed
- `cargo clippy --workspace --all-targets -- -D warnings` - passed
- `cargo test --workspace` - passed
- `cargo run -p roomci-cli -- validate examples/local_first_cloud_outage.yaml` - passed
- `cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --json /tmp/roomci-phase25-1-local-first.json` - passed
- `cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml` - passed

## Blockers

None.

## Next Action

Phase 26 can start. Remaining raw `serde_yaml::Value`/`serde_json::Value`
usage is intentionally retained at YAML compatibility, extension-data,
subsystem-config, or report boundaries.
