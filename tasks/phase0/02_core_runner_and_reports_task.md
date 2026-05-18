# Task 02 — Core Runner and Reports

## Objective

Implement deterministic scenario execution and report generation.

## Implementation Scope

- Parse scenario clock and relative time expressions.
- Build room/device registry from YAML.
- Apply command, state, event, and fault steps in timeline order.
- Evaluate assertions.
- Emit JSON, Markdown, and JUnit reports.
- Add golden fixtures for sample scenarios.

## Acceptance Criteria

- `checkin_lock_offline.yaml` produces a failed or passed result according to assertion events emitted by the runner.
- `ac_preheat_failed.yaml` evaluates the temperature threshold assertion.
- JSON report includes event timeline and assertion results.
- Markdown report includes scenario name, result, guest impact, timeline, failed assertions, and suggested recovery field when available.
- JUnit report maps each assertion to a test case.

## Risks

- Treating guest impact text as decoration instead of part of the report contract.
- Allowing wall-clock time into CI execution.

## References

- `docs/06_scenario_spec.md`
- `docs/07_fault_injection.md`
