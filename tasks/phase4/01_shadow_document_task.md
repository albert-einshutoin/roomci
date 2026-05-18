# Task 01 — Shadow Document Model

## Objective

Implement local shadow document semantics on top of the canonical device model.

## Implementation Scope

- Add `roomci-shadow` crate or module.
- Define desired, reported, delta, metadata, and version fields.
- Map thing names to roomci devices.
- Implement validation for supported desired properties.

## Acceptance Criteria

- Shadow documents serialize to stable JSON.
- Delta is derived, not hand-mutated.
- Unsupported desired fields produce rejected results.
- Version increments are deterministic.

## References

- `docs/04_protocol_adapters.md`
