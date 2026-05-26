# Phase 26 Status: Adapter Contract Generalization

## Status

`todo`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_payload_shape_validation_task.md` | `todo` | Unassigned | Pending | Add typed payload schema constraints beyond required fields |
| `02_topic_identity_diagnostics_task.md` | `todo` | Unassigned | Pending | Improve placeholder and identity mapping errors |
| `03_acceptance_assertion_mapping_task.md` | `todo` | Unassigned | Pending | Link acceptance criteria to scenario assertions/evidence |
| `04_evaluator_intake_kit_task.md` | `todo` | Unassigned | Pending | Document minimum customer specs needed for a real PoC |

## Quality Gates

- `cargo fmt --all --check`
- `cargo test --workspace --all-targets`
- `cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml`

## Notes

Phase 26 can proceed without a customer because it improves the generic adapter
contract and evaluator intake surface. It must not claim compatibility with a
specific deployment until a user supplies real topics, payload examples,
register maps, auth assumptions, and acceptance criteria.
