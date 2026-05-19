# Task 02 — Protocol Support Matrix

## Goal

Create a precise support matrix that shows what `roomci` can emulate today, what is externally drivable, and what remains a documented non-goal.

## Why This Matters

IoT and SmartHome companies often have company-specific MQTT topics, Modbus maps, BMS contracts, cloud events, and edge-controller behavior. The product should make those unknowns explicit rather than pretending to know them.

## Implementation Scope

- Document support level for each protocol/domain:
  - MQTT
  - Modbus
  - DALI-like lighting
  - contact I/O
  - BMS/webhooks
  - edge controller
  - WAN/network failover
  - access/intercom
  - comfort/HVAC automation
- Use support levels:
  - `scenario_model`
  - `serve_endpoint`
  - `external_client_tested`
  - `conformance_subset`
  - `unsupported`
- For each row, include:
  - current implementation evidence
  - missing endpoint/protocol work
  - required customer inputs
  - production non-goals
- Link each row to examples, docs, and tests.

## Acceptance Criteria

- A reader can tell the difference between behavior modeling and wire-protocol compatibility.
- NOT A HOTEL compatibility is framed as requiring their real contracts.
- Generic company evaluation is framed as filling in adapters/specs, not waiting for private knowledge.
- The matrix becomes the source of truth for future protocol roadmap decisions.
