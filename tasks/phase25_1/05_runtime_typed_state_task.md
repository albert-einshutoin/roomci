# Task 05: Runtime Typed State

## Status

`done`

## Problem

The runtime can still represent promoted domain state with generic JSON values
in places where Rust structs and enums would make invalid transitions harder to
express.

## Scope

- Identify promoted state paths that runtime logic reads or writes today.
- Introduce typed structs/enums for those paths first.
- Keep JSON conversion at report/output boundaries.
- Avoid a broad rewrite of every incidental report value.

## Implementation Checklist

- [x] Inventory current runtime state reads/writes and mark which are promoted
  domain state versus report-only metadata.
- [x] Add tests for at least one promoted typed state transition per affected
  domain.
- [x] Introduce typed state structs for promoted comfort, intercom, contact,
  edge, broker, or fault paths that runtime behavior depends on.
- [x] Convert typed state to JSON only when producing reports or external
  responses.
- [x] Remove direct JSON mutation from promoted execution paths.
- [x] Document any remaining JSON state as report-boundary or intentionally
  dynamic extension data.

## Acceptance Criteria

- Promoted runtime state transitions are represented by Rust types.
- Report JSON shape remains compatible.
- Remaining generic JSON usage has a documented boundary reason.
