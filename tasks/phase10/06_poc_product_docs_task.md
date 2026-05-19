# Task 06 — PoC Product Docs

## Objective

Update product docs so `roomci` reads as a pre-adoption PoC tool rather than only a scenario-demo project.

## Acceptance Criteria

- README explains both modes:
  - scenario mode: internal YAML execution
  - serve mode: external clients connect to localhost endpoints
- Docs include a pre-adoption PoC integration checklist.
- Docs explain what information a real customer/vendor must provide:
  - MQTT topics and payload schemas
  - QoS/retained/session expectations
  - Modbus register maps, if used
  - BMS webhook/API contract, if used
  - auth/TLS/network assumptions
  - pass/fail acceptance criteria
- Docs explicitly state current protocol subset and non-goals.
- NOT A HOTEL-facing wording says compatibility requires their actual integration contract.
- Quick Start includes a serve-mode E2E command.

## Notes

- The strongest public claim should be: configurable PoC integration surface, not private-system compatibility.
