# Task 05: B Tier Evidence And Docs Gate

## Goal

Make B Tier protocol-profile claims mechanically checkable and impossible to
confuse with conformance claims.

## Scope

- Update:
  - `docs/PROTOCOL_SUPPORT_MATRIX.md`
  - `docs/PROTOCOL_CONFORMANCE_REGISTRY.md`
  - `docs/protocol-evidence.json`
  - `docs/EVALUATION_EVIDENCE_PACK.md`
  - `docs/GENERIC_SMARTHOME_EVALUATOR_CHECKLIST.md`
- Add checker rules if needed:
  - B Tier profiles must have examples/docs/non-goals.
  - B Tier profiles must not use `conformance_subset`.
  - B Tier profile docs must include "not certification" wording.
- Add Makefile target:
  - `make protocol-profile-smoke`

## Acceptance Criteria

- `make protocol-profile-smoke` validates all B Tier profile examples.
- `make protocol-evidence` checks B Tier profile evidence.
- Docs clearly separate:
  - implemented wire subsets: MQTT, Modbus
  - contract profiles: Matter, BACnet, KNX, OPC UA
  - non-goals: full certification and production gateway behavior

## Test Commands

```bash
make protocol-profile-smoke
make protocol-evidence
rg -n "contract_profile|not certification|BACnet|KNX|OPC UA|Matter" docs
```

## Out Of Scope

- Implementing all B Tier protocols as runtime endpoints.
- Adding Zigbee or Thread runtime behavior.
