# Task 04: Serve Protocol Module Extraction

## Why

`roomci-serve` now owns HTTP, MQTT, Modbus, state overlays, and report rendering in one source file. It still passes verification, but future protocol work will become riskier if all handlers stay coupled.

## Acceptance Criteria

- Split protocol handlers into focused modules without changing public behavior.
- Target modules should make the runtime shape obvious, for example `http`, `mqtt`, `modbus`, and shared `state`/routing helpers.
- Keep shared state mutation explicit and testable.
- Preserve existing black-box tests.
- Add focused module-level tests where extraction creates new seams.
- If this is paired with Task 01, the only intended behavior addition is the MQTT SUBSCRIBE/SUBACK + retained replay subset covered by Task 01.
- Keep coverage above 80%.

## Out of Scope

- Rewriting the serve runtime to async.
- Changing public CLI flags.
- Adding protocol behavior beyond Task 01's narrow retained subscribe loop during extraction.
