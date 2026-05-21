# Task 02: BACnet Contract Profile

## Goal

Add a BACnet contract profile for object/property/event-style building
automation QA without implementing a BACnet/IP endpoint.

## Scope

- Add adapter contract example:
  - `adapter-contracts/examples/bacnet_contract_profile.yaml`
- Add scenario example or dry-run fixture:
  - `examples/bacnet_contract_profile.yaml`
- Model:
  - device id
  - object type
  - object instance
  - property id/name
  - expected value
  - event/alarm class
- Document required customer inputs and explicit non-goals.

## Acceptance Criteria

- BACnet profile validates.
- Support matrix says `contract_profile` or `future_profile`, not
  `conformance_subset`.
- Docs reject BACnet/IP endpoint, COV, BBMD, routing, and certification claims.

## Test Commands

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/bacnet_contract_profile.yaml
cargo run -p roomci-cli -- validate examples/bacnet_contract_profile.yaml
make protocol-evidence
```

## Out Of Scope

- BACnet/IP wire endpoint.
- COV subscriptions.
- BBMD/routing.
- Certification testing.
