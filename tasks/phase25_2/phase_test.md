# Phase 25.2 Test Plan

## Goal

Prove that the validated graph is the real runtime boundary, not just a
pre-validation layer, and that the Rust dependency/security gate is green before
Phase 26 starts.

## Required Commands

Run these before marking Phase 25.2 done:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
```

## Compatibility Checks

```bash
cargo run -p roomci-cli -- validate examples/local_first_cloud_outage.yaml
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --json /tmp/roomci-phase25-2-local-first.json
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml
```

## Review Checks

Run targeted searches and record the results in `phase_status.md`:

```bash
rg -n "scenario\\.raw\\(\\)|typed_step_kind\\(|typed_fault_kind\\(|typed_assertion_kind\\(" crates/roomci-core/src crates/roomci-scenario/src
rg -n "BTreeMap<String, serde_yaml::Value>|serde_yaml::Value|serde_json::Value" crates/roomci-core/src crates/roomci-scenario/src
```

Remaining hits must be classified as one of:

- YAML compatibility boundary;
- extension-data boundary;
- subsystem adapter boundary;
- report/output boundary;
- intentional public compatibility API;
- follow-up debt with an explicit issue or phase task.

## New Regression Coverage

Add tests that cover:

- audit dependency risk no longer appears in the dependency graph;
- runtime can execute a scenario through validated subsystem inputs;
- raw classifier helpers are not needed by `roomci-core`;
- known ops `acknowledge` action maps to a typed step;
- known automation `hvac_auto_mode` maps to a typed step;
- invalid promoted target strings fail during validation.

## Done Criteria

- All required commands pass.
- Compatibility checks pass.
- Remaining raw boundaries are intentional and documented.
- Phase 26 is unblocked only after audit and validated graph checks are green.
