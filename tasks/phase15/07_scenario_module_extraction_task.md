# Task 07: Scenario Module Extraction

## Why

`crates/roomci-scenario/src/lib.rs` is above the 800-line maintainability target. Scenario parsing, schema structs, defaults, validation, adapter contracts, and fixture tests are all concentrated in one file. That makes future adapter-contract and protocol-profile work harder to review.

## Acceptance Criteria

- Split `roomci-scenario` into focused modules such as schema, validation, adapter_contract, defaults, and loading.
- Preserve YAML compatibility for all shipped examples and adapter contracts.
- Keep validation errors at least as actionable as they are today.
- Add or keep tests for shipped scenario examples, adapter contract examples, invalid payloads, and unsupported protocol boundaries.
- Keep the largest edited source file at or below the 800-line maintainability target, or document a justified exception in the task status.
- Keep coverage above 80%.

## Out of Scope

- Replacing `serde_yaml`.
- Redesigning the scenario DSL.
- Adding new domain-pack fields unless required to preserve existing behavior.
