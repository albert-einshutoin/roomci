# Task 06: Raw Boundary Inventory

## Status

`done`

## Problem

Some raw `serde_yaml::Value`, `serde_json::Value`, and
`BTreeMap<String, serde_yaml::Value>` usage will remain by design. Without an
inventory, future work can accidentally expand raw runtime behavior again.

## Scope

- Inventory remaining raw value usage after Tasks 01-05.
- Classify each remaining use by boundary type.
- Add comments or docs only where the boundary is not obvious.
- Create follow-up tasks only for real debt.

## Implementation Checklist

- [x] Run the raw boundary search commands from `phase_test.md`.
- [x] Classify each remaining hit as YAML compatibility, extension data,
  subsystem adapter, report output, compatibility API, or follow-up debt.
- [x] Add short comments for non-obvious intentional boundaries.
- [x] Update `phase_status.md` with the inventory summary.
- [x] Add follow-up task only if a remaining raw use is not acceptable for Phase
  26.

## Acceptance Criteria

- Remaining raw values are intentional.
- Runtime promoted paths do not gain new raw map/string dispatch.
- Phase 26 has a clear boundary for adapter contract expansion.
