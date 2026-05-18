# Task 01 — Matter-like Profile Mapping

## Objective

Map Matter-like clusters to canonical roomci capabilities.

## Implementation Scope

- Add profile mapping table.
- Implement export from roomci device to profile metadata.
- Implement validation for supported clusters.
- Keep mappings independent from protocol transport.

## Acceptance Criteria

- Every supported cluster has a unit test.
- Export output is stable and snapshot-tested.
- Unsupported clusters produce actionable validation errors.

## References

- `docs/04_protocol_adapters.md`
