# Task 03 — Adapter Contract Kit

## Goal

Add templates and validation for company-specific integration contracts so evaluators can adapt `roomci` to their own systems without editing runtime code.

## Why This Matters

The target product position is not "roomci already knows every company's private protocol." It is "roomci gives you the contract format and emulator harness so your private protocol details can be mapped quickly and safely."

## Implementation Scope

- Add contract templates for:
  - MQTT topics and payload schemas
  - Modbus devices, registers, scaling, access mode, and units
  - BMS/webhook events, severity, routing, and acknowledgement
  - edge-controller commands and expected state transitions
  - device identity, room/site hierarchy, and retained state
  - auth assumptions and test credentials
  - acceptance criteria and report expectations
- Add example filled templates:
  - generic MQTT edge device
  - hospitality local-first room
  - building automation/BMS
- Add schema validation for adapter contracts.
- Add docs explaining how to convert a real system spec into a `roomci` contract.

## Acceptance Criteria

- A new company can copy a template and fill in real protocol details.
- Invalid or incomplete contracts fail validation with actionable errors.
- Existing examples can be represented through the adapter contract kit.
- The docs state which details must come from the customer/vendor.
