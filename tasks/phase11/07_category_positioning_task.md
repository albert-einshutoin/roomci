# Task 07 — Category Positioning and Competitive Readiness

## Goal

Make the product narrative strong enough that IoT and SmartHome emulator evaluations naturally consider `roomci` as a first-choice candidate.

## Implementation Scope

- Add a positioning document that compares `roomci` against:
  - real-device staging environments
  - generic MQTT brokers
  - ad hoc mock scripts
  - Home Assistant-based test setups
  - cloud-only IoT emulators
  - hardware-in-the-loop setups
- Explain where `roomci` wins:
  - contract-first local/CI emulation
  - field-failure reproducibility
  - reports for software and field teams
  - protocol adapter templates
  - company-specific specs as configuration
- Explain where `roomci` does not win yet:
  - full protocol conformance
  - hardware timing/electrical behavior
  - production control-plane replacement
  - unknown private vendor features
- Add a short evaluator checklist for deciding whether `roomci` fits a company.

## Acceptance Criteria

- The narrative is ambitious without claiming private compatibility.
- Floci-like or SmartHome emulator comparisons are framed by use case, not vague superiority.
- The docs make clear why `roomci` should be evaluated before building another one-off mock harness.
