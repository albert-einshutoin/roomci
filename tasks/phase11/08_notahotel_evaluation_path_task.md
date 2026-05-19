# Task 08 — NOT A HOTEL Evaluation Path

## Goal

Preserve and strengthen the NOT A HOTEL evaluation path while keeping `roomci` positioned as an industry-wide IoT, SmartHome, and building-automation emulator.

## Why This Matters

The strongest evaluation from NOT A HOTEL will not come from pretending to know their private implementation. It will come from showing that `roomci` understands the kind of local-first, edge, MQTT, Modbus, BMS, commissioning, and operations problems they likely face, and that their actual contracts can be plugged in cleanly.

## Implementation Scope

- Add a NOT A HOTEL-style evaluator guide that explains:
  - what `roomci` can demonstrate today
  - what real NOT A HOTEL specs are required for a serious PoC
  - which adapter contracts those specs would fill
  - which scenarios map to hospitality smart-home quality risks
  - which claims are intentionally not made
- Maintain a hospitality/local-first PoC path with:
  - local controller to local MQTT
  - edge failover
  - Modbus-style equipment control
  - contact I/O emergency
  - BMS/ops escalation
  - WAN failover
  - commissioning report
- Add a "handoff request" checklist for NOT A HOTEL:
  - MQTT topics and payload schemas
  - retained/QoS expectations
  - edge command/state behavior
  - Modbus maps and scaling rules
  - BMS event/webhook contracts
  - auth/network assumptions
  - acceptance criteria and failure examples
- Add a scoring rubric that lets NOT A HOTEL evaluate the PoC without relying on private compatibility claims.

## Acceptance Criteria

- NOT A HOTEL appears as a high-signal hospitality evaluation path, not as a claimed compatibility target.
- The docs make it obvious what they would need to provide next.
- Every NOT A HOTEL-specific unknown maps to a generic adapter/contract input.
- The guide explains why `roomci` remains useful even before private specs are provided.
