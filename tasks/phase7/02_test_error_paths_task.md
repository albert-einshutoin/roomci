# Task 02 — Test Coverage and Error Paths

## Objective

Close coverage gaps for `roomci-fault` and add error-path tests across the workspace so failure modes are covered, not just happy paths.

## Acceptance Criteria

- `roomci-fault` has at least three unit tests covering active, inactive, and bounded fault windows.
- `roomci-scenario`, `roomci-ops`, `roomci-mqtt`, `roomci-edge`, and `roomci-device-model` each gain at least one error-path test.
- `crates/roomci-cli/tests/cli.rs` covers multi-scenario execution, missing-file errors, and invalid flag combinations.
- `cargo test --workspace` runs at least 55 tests, all passing.
- `cargo tarpaulin --workspace` reports coverage >= 80%.
