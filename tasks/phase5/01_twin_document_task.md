# Task 01 — Twin Document Model

## Objective

Implement Device Twin-like desired and reported property documents.

## Implementation Scope

- Define twin document structure.
- Map configured devices to twin paths.
- Validate desired property patches.
- Validate reported property patches.
- Preserve canonical state as the source of truth.

## Acceptance Criteria

- Desired and reported sections serialize predictably.
- Patches reject unknown devices and unsupported properties.
- Canonical state remains authoritative.

## References

- `docs/04_protocol_adapters.md`
