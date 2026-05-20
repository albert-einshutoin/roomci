# Task 05 — Customer PoC Packs

## Goal

Provide ready-to-run PoC packs that make `roomci` feel useful to different companies even when their exact internal specs are unknown.

## Implementation Scope

- Add PoC packs for:
  - generic MQTT edge-device fleet
  - hospitality/hospitality-focused local-first room
  - building automation with Modbus and contact I/O
  - BMS/operations escalation
- Each pack should include:
  - scenario
  - adapter contract template
  - external controller/client example
  - expected report artifacts
  - acceptance checklist
  - customization notes
- Add a "what to replace with your real spec" section for every pack.

## Acceptance Criteria

- An evaluator can run each pack with one command.
- Each pack identifies required company-specific inputs.
- Each pack produces JSON, Markdown, and JUnit evidence.
- hospitality-focused pack avoids claiming private compatibility while showing exactly where their contracts would plug in.
