# Phase 15 Goal: Evaluator Friction Removal

## Goal

Turn the current protocol-backed PoC product into a cleaner release-candidate evaluation surface by removing the highest-friction gaps found during the final self-review.

The phase should not broaden roomci into a full smart-home stack emulator. It should deepen the currently claimed MQTT / Modbus / hospitality-core contract surface enough that an external evaluator can run it repeatedly, understand the limits, and map their own specs without hidden manual interpretation.

## Product Outcome

- MQTT external clients can exercise a more realistic retained-state loop.
- Modbus TCP support covers the next practical subset beyond single-register reads/writes.
- Docker protocol smoke does not depend on ad hoc runtime package installation.
- Serve runtime code is split enough that protocol handlers can evolve independently.
- Public protocol claims have machine-checkable evidence links.

## Non-Goals

- Full MQTT broker implementation.
- Full Modbus TCP/RTU implementation.
- BACnet, KNX, Matter, DALI, SIP, ONVIF, or OPC UA runtime implementation.
- NOT A HOTEL private compatibility claims without private adapter specs and traces.
