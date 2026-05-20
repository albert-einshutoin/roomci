# Task 01 — Stack Coverage Map

## Goal

Create a coverage map for the reported hospitality smart-home stack so `roomci` has a clear product boundary.

## Implementation Scope

- Add a document such as `docs/HOSPITALITY_STACK_COVERAGE.md`.
- Classify each reported technology group:
  - languages/application stack
  - MQTT / TCP/IP / protocol transport
  - Modbus / DALI / KNX / contact I/O
  - cloud / edge / MQTT brokers
  - control panel / electrical / UPS / redundancy
  - network / VLAN / WAN / Starlink-style failover
  - BMS / monitoring / operations notification
  - access control / intercom
  - comfort sensors / HVAC automation
  - design / construction / CAD tools
- Use coverage tiers:
  - cover now
  - cover next
  - mock/contract only
  - future profile
  - out of scope
- Link the map from README, docs index, and relevant Phase 11/13 docs.

## Acceptance Criteria

- The map clearly says `roomci` is not a full hospitality-stack emulator.
- Local MQTT, edge, Modbus, DALI-like, contact I/O, BMS, network failover, control-panel fault, and comfort automation are marked as core or next-core coverage.
- Cloud services, access systems, intercom vendors, CAD tools, and physical electrical safety are not marked as full emulation targets.
