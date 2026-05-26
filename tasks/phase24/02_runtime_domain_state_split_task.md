# Task 02: Runtime Domain State Split

## Status

`done`

## Problem

`RuntimeState` has accumulated unrelated domain state for comfort, access,
commissioning, network, operations, and protocol overlays.

## Scope

- Extract cohesive sub-state structs such as `ComfortRuntimeState`,
  `AccessRuntimeState`, and `CommissioningRuntimeState`.
- Keep public reports and timeline events stable.
- Avoid broad behavior refactors in the same change.

## Acceptance Criteria

- Runtime code is easier to navigate by domain.
- Existing report snapshots and CLI tests continue to pass.
- No domain state is silently dropped from final reports.

## Evidence

- `cargo test -p roomci-core runtime_groups_customer_independent_domain_state`
- `cargo test -p roomci-core`
