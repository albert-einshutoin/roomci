# Phase 24 Status: Core Architecture Hardening

## Status

`done`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_typed_assertion_model_task.md` | `done` | Codex | `cargo test -p roomci-scenario rejects_unknown_target_condition_assertion`; `cargo test -p roomci-core` | Typed assertion classification rejects unknown target/condition assertions and core evaluation now matches promoted kinds |
| `02_runtime_domain_state_split_task.md` | `done` | Codex | `cargo test -p roomci-core runtime_groups_customer_independent_domain_state`; `cargo test -p roomci-core` | Comfort, access, and commissioning state now live in cohesive sub-states |
| `03_parser_property_tests_task.md` | `done` | Codex | `cargo test -p roomci-serve bounded` | Bounded MQTT parser and Modbus handler malformed-input regressions added |
| `04_concurrent_serve_overlay_tests_task.md` | `done` | Codex | `cargo test -p roomci-serve overlapped_run_state_bms_and_report_requests_leave_consistent_reports` | Overlapped run/state/BMS/report requests leave coherent reports |
| `05_protocol_server_library_adr_task.md` | `done` | Codex | `docs/adr/0001-serve-protocol-server-strategy.md` | ADR keeps current hand-written subsets and defines library adoption thresholds |

## Quality Gates

- `cargo fmt --all --check`
- `cargo test --workspace --all-targets`
- report/timeline compatibility checks for affected scenarios

## Notes

Phase 24 changed internal organization and test coverage only. Public scenario
YAML and report formats remain compatible.
