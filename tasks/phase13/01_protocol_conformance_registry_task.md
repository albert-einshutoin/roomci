# Task 01 — Protocol Conformance Registry

## Goal

Create a single source of truth for protocol claims so `roomci` can say exactly which official specifications it references, which subset it implements, and how that subset is verified.

## Implementation Scope

- Add a protocol conformance registry document, for example `docs/PROTOCOL_CONFORMANCE_REGISTRY.md`.
- Link it from `docs/PROTOCOL_SUPPORT_MATRIX.md`, `docs/README.md`, and README where protocol claims are made.
- For each protocol, record:
  - official spec URL
  - supported version or target version
  - current support level
  - implemented subset
  - unsupported behavior
  - explicit rejection behavior
  - verification command
  - Docker/Compose smoke coverage
- Include MQTT, MQTT 5, Modbus, BACnet, OPC UA, Zigbee, Thread, KNX, and Matter.

## Acceptance Criteria

- Registry exists and is linked from the support matrix.
- Registry explicitly distinguishes behavior models, wire endpoints, conformance subsets, and unsupported future profiles.
- No protocol row claims full conformance unless backed by implementation and tests.
- `docs/PROTOCOL_SUPPORT_MATRIX.md` remains consistent with the registry.
