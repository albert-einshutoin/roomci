# Phase 26 Status: Adapter Contract Generalization

## Status

`in_progress`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_payload_shape_validation_task.md` | `done` | Codex | `MqttPayloadFieldConstraint`; `typed_mqtt_payload_constraints_validate_runtime_values`; `integer_payload_ranges_are_exact_beyond_f64_precision`; `make verify` | Backward-compatible required/optional type, enum, and range validation with fail-closed numeric precision and typo handling |
| `02_topic_identity_diagnostics_task.md` | `done` | Codex | Contract-specific topic paths; exact placeholder and strategy tests; `make verify` | Runtime matching unchanged; invalid private topic assumptions now produce actionable diagnostics |
| `03_acceptance_assertion_mapping_task.md` | `todo` | Unassigned | Pending | Link acceptance criteria to scenario assertions/evidence |
| `04_evaluator_intake_kit_task.md` | `todo` | Unassigned | Pending | Document minimum customer specs needed for a real PoC |

## Quality Gates

- `cargo fmt --all --check`
- `cargo test --workspace --all-targets`
- `cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml`

Latest Task 01 verification:

- `make verify` passed with 87.61% workspace coverage.
- All shipped adapter contracts and scenarios validated.
- Docker, Compose, MQTT/Modbus protocol smoke, SDK smoke, report evidence, and
  VS Code asset checks passed.
- Rust and security reviews reported no remaining merge blockers.

Latest Task 02 verification:

- Topic diagnostics identify the contract and exact command/state/identity
  field.
- Missing, repeated, malformed, unsupported, and mismatched placeholders fail
  before runtime matching.
- Untrusted diagnostic values are escaped and bounded; template and expanded
  topic lengths fail closed at the MQTT wire limit.
- All shipped adapter contracts and scenarios remain valid.

## Notes

Phase 26 can proceed without a customer because it improves the generic adapter
contract and evaluator intake surface. It must not claim compatibility with a
specific deployment until a user supplies real topics, payload examples,
register maps, auth assumptions, and acceptance criteria.
