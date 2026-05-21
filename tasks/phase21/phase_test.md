# Phase 21 Test Plan

Phase 21 is complete when B Tier protocol profiles are useful as scoped
contract prototypes and impossible to mistake for full implementations.

## Required Gates

- `make verify`
- `make protocol-evidence`
- `cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/*.yaml`

## New Phase 21 Gates

- Matter profile example validates.
- BACnet profile example validates.
- KNX profile example validates.
- OPC UA profile example validates.
- Protocol support matrix and conformance registry list each profile as
  `future_profile` or `contract_profile`, never `conformance_subset`.
- Evidence checker rejects accidental full-conformance wording for these
  profiles.

## Acceptance Criteria

- Each B Tier protocol has:
  - adapter contract example
  - docs page or section
  - support matrix row
  - non-goal list
  - evidence registry entry
- No runtime endpoint is required unless deliberately scoped as a prototype.
