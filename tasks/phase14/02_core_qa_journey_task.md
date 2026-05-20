# Task 02 — Core QA Journey

## Goal

Define and provide a single evaluator journey that demonstrates the core product promise end to end.

## Target Journey

```txt
local controller / external client
  -> MQTT command/state contract
  -> edge server routing or failover
  -> device protocol behavior
  -> network or control-panel fault
  -> BMS/ops escalation
  -> CI-ready report evidence
```

## Implementation Scope

- Decide whether to create one new scenario or a scripted PoC pack that chains existing scenarios.
- Include at least:
  - local MQTT retained state
  - edge server behavior
  - Modbus, DALI-like, contact I/O, or HVAC behavior
  - network/WAN or control-panel fault
  - BMS/ops alert evidence
  - JSON/Markdown/JUnit report artifacts
- Add a Make target if the journey is a PoC pack.
- Document what private inputs a real hospitality evaluator would replace.

## Acceptance Criteria

- The journey can be run from a clean checkout.
- The generated report shows a timeline that would make sense to software, field, and operations stakeholders.
- The journey does not claim private compatibility with any specific production system.
