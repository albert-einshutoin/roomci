# Phase 25.1 Test Plan

## Goal

Prove that Rust type-boundary hardening preserves current product behavior while
rejecting malformed scenario/runtime inputs earlier and more clearly.

## Required Commands

Run these before marking Phase 25.1 done:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Scenario Compatibility Checks

Run the existing representative scenarios after the validated model is wired
into runtime:

```bash
cargo run -p roomci-cli -- validate examples/local_first_cloud_outage.yaml
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --json /tmp/roomci-phase25-1-local-first.json
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml
```

If an example name changes, replace it with the current equivalent and record
the exact command in `phase_status.md`.

## New Regression Coverage

Add tests that cover:

- empty or malformed device, scene, fixture, contact, alert, broker, and edge
  identifiers;
- invalid MQTT topics and topic templates;
- unknown step, fault, assertion, and condition variants;
- malformed domain config values for promoted runtime paths;
- successful conversion of existing examples from raw scenario data into the
  validated scenario model.

## Done Criteria

- All required commands pass.
- Compatibility checks pass.
- New malformed-input tests fail with typed, actionable validation errors.
- The final self-review confirms Phase 26 can start without adding more raw
  string/value handling to the runtime core.
