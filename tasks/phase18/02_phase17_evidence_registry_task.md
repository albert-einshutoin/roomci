# Task 02: Phase 17 Evidence Registry

## Goal

Make Phase 17 product claims machine-checkable in the same spirit as the
protocol evidence checker.

## Scope

- Add or extend an evidence registry for:
  - intercom/relay safe mock
  - network/control-panel fault profiles
  - BMS contract hardening
  - comfort time-series replay
  - adapter sample wiring
- Link each claim to example files, tests, docs, and commands.
- Add a checker script or extend the existing evidence checker.

## Acceptance Criteria

- A missing example, test name, or doc reference causes the evidence check to fail.
- The registry distinguishes executable evidence from docs-only guidance.
- Non-goals remain explicit for real unlocks, physical safety, vendor cloud, and full protocol conformance.

## Out Of Scope

- Claiming certification or private customer compatibility.
