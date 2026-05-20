# Task 04: Serve Protocol Module Extraction

## Why

`roomci-serve` now owns HTTP, MQTT, Modbus, state overlays, and report rendering in one source file. It still passes verification, but future protocol work will become riskier if all handlers stay coupled.

## Acceptance Criteria

- Split protocol handlers into focused modules without changing public behavior.
- Keep shared state mutation explicit and testable.
- Preserve existing black-box tests.
- Add focused module-level tests where extraction creates new seams.
- Keep coverage above 80%.

## Out of Scope

- Rewriting the serve runtime to async.
- Changing public CLI flags.
- Adding new protocol behavior during extraction.
