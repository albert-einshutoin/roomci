# Task 03: KNX Contract Profile

## Goal

Add a KNX contract profile for group-address based QA without implementing KNX
bus behavior or importing ETS projects.

## Scope

- Add adapter contract example:
  - `adapter-contracts/examples/knx_group_address_profile.yaml`
- Add scenario example or dry-run fixture:
  - `examples/knx_group_address_profile.yaml`
- Model:
  - group address
  - datapoint type
  - direction
  - expected value
  - room/device identity mapping
  - scene or function label
- Document required customer inputs and non-goals.

## Acceptance Criteria

- KNX profile validates.
- Docs state that real ETS exports and group-address maps must come from the
  evaluator.
- No KNX bus, telegram timing, gateway, or certification claim is made.

## Test Commands

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/knx_group_address_profile.yaml
cargo run -p roomci-cli -- validate examples/knx_group_address_profile.yaml
make protocol-evidence
```

## Out Of Scope

- ETS import.
- KNX/IP tunneling or routing.
- Bus timing.
- Device certification.
