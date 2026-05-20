# Task 10 — Evaluation Evidence Pack

## Goal

Create an evidence pack that lets evaluators judge `roomci` quickly and fairly, instead of relying on pitch language.

## Why This Matters

Evaluation is maximized when the product shows reproducible proof:

- what runs
- what fails
- what reports are produced
- what protocol subset is supported
- what company-specific inputs are needed
- what would be implemented next for a serious adoption

## Implementation Scope

- Add a generated or manually maintained evidence index with:
  - commands to run
  - expected pass/fail behavior
  - generated report paths
  - protocol features covered
  - unsupported features disclosed
- Add evaluator scorecards for:
  - hospitality-focused hospitality smart home
  - generic IoT/MQTT edge-device fleet
  - building automation / Modbus / BMS
  - CI/platform engineering
- Each scorecard should include:
  - current fit score
  - required customer inputs
  - integration effort estimate
  - missing protocol depth
  - go/no-go criteria
- Link evidence back to tests, examples, docs, and `make verify`.

## Acceptance Criteria

- An evaluator can inspect one evidence pack and understand current product maturity.
- hospitality team-specific evaluation is measurable without claiming private compatibility.
- Generic company evaluation is measurable without hospitality context.
- Evidence pack remains tied to actual commands and artifacts, not static marketing claims.
