# Phase 25.2 Status: Validated Graph Completion and Audit Gate

## Status

`done`

## Task Board

| Task | Status | Owner | Notes |
|---|---|---|---|
| 01 Dependency Audit Gate | `done` | Codex | Removed the `rumqttc` dev-dependency and kept MQTT integration coverage with a local std/TCP MQTT smoke client. |
| 02 Validated Subsystem Graph | `done` | Codex | `ValidatedScenario` now owns `ValidatedRuntimeInputs`; runtime no longer rebuilds promoted subsystem models from `scenario.raw()`. |
| 03 Raw Classifier API Containment | `done` | Codex | Borrowed raw classifier helpers are crate-private and only feed owned validated variants. |
| 04 Typed Ops and Automation Steps | `done` | Codex | Promoted `ops.action: acknowledge` and `automation.type: hvac_auto_mode` are typed variants; unknown maps are explicit extension data. |
| 05 Domain Target Enums | `done` | Codex | Promoted command, fault, and sensor target paths use typed target wrappers/enums and render strings at output boundaries. |
| 06 Raw Boundary Inventory | `done` | Codex | Remaining raw values are classified as YAML compatibility, extension data, subsystem adapter payload, report/output, or compatibility API boundaries. |
| 07 Phase 26 Readiness Gate | `done` | Codex | Full quality matrix and compatibility checks passed; Phase 26 is unblocked. |

## Quality Gates

- `cargo fmt --all --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo audit`
- `cargo run -p roomci-cli -- validate examples/local_first_cloud_outage.yaml`
- `cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --json /tmp/roomci-phase25-2-local-first.json`
- `cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml`

## Blockers

None.

## Evidence

- `cargo fmt --all --check` passed.
- `cargo check --workspace --all-targets` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace` passed.
- `cargo audit` passed.
- `cargo test -p roomci-cli standard_mqtt_client_publishes_retained_state_through_serve` passed with the in-repo MQTT smoke client.
- `cargo tree -i rumqttc`, `cargo tree -i rustls-webpki@0.102.8`, and
  `cargo tree -i rustls-pemfile@2.2.0` reported no matching packages.
- `cargo run -p roomci-cli -- validate examples/local_first_cloud_outage.yaml` passed.
- `cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --json /tmp/roomci-phase25-2-local-first.json` passed with 1 scenario passed and 0 failed.
- `cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml` passed.

## Review Search Results

- `rg -n "scenario\\.raw\\(\\)|typed_step_kind\\(|typed_fault_kind\\(|typed_assertion_kind\\(" crates/roomci-core/src crates/roomci-scenario/src`
  now reports only crate-private classifier definitions in `roomci-scenario` and
  validation-time construction calls in `validated.rs`; there are no
  `scenario.raw()` hits in `roomci-core`.
- `rg -n "BTreeMap<String, serde_yaml::Value>|serde_yaml::Value|serde_json::Value" crates/roomci-core/src crates/roomci-scenario/src`
  reports intentional boundaries:
  - YAML compatibility boundary: raw schema structs in `schema.rs`;
  - extension-data boundary: non-promoted ops/automation maps;
  - subsystem adapter boundary: Modbus values, MQTT payload/retained payloads,
    and future topic-specific contract payloads;
  - report/output boundary: `StateMap` and JSON timeline/report values;
  - compatibility API boundary: `ValidatedScenario::raw()` remains available for
    callers that need the original scenario file, but core runtime construction
    no longer depends on it.

## Phase 26 Recommendation

Go. Phase 26 can start because the RustSec audit gate is green, promoted runtime
inputs are held in the validated graph, raw classifier APIs are not public, and
remaining raw value use is limited to documented compatibility, extension,
adapter, and output boundaries.
