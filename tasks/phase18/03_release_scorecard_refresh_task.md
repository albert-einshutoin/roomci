# Task 03: Release Scorecard Refresh

## Goal

Refresh public evaluator and readiness docs so they reflect Phase 17 coverage
and still avoid overstating production readiness.

## Scope

- Update readiness/category/evidence docs with Phase 17 coverage.
- State remaining gaps bluntly:
  - samples are reproducibly CI-verified by Task 01
  - adapter contracts still need customer-specific topics, payloads, registers, auth, and acceptance criteria
  - BACnet, KNX, Matter, OPC UA, Zigbee, Thread remain future profiles or non-goals
- Re-score release readiness if the docs currently include a numeric estimate.

## Acceptance Criteria

- Public docs describe the current product as a Smart Home / Building Automation QA contract emulator.
- NOT A HOTEL-like relevance and broader market relevance are both represented.
- Claims are backed by commands, examples, tests, or explicit non-goals.

## Out Of Scope

- Marketing copy that implies full-stack NOT A HOTEL compatibility.
- New runtime features.
