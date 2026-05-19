# Task 03 — Domain Packs

## Objective

Separate the reusable core from domain-specific scenario packs.

## Acceptance Criteria

- Add `docs/DOMAIN_PACKS.md`.
- Define the core modules:
  - MQTT/device contracts
  - edge routing/failover
  - failure injection
  - report generation
  - CI execution
- Define domain packs:
  - hospitality smart home
  - building automation
  - BMS / operations
  - commissioning
  - access control
  - generic MQTT edge devices
- Map current examples to those domain packs.
- Do not physically move examples unless all README/CI/Compose paths are updated.
