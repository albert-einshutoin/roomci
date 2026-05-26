# Task 02: Validated Subsystem Graph

## Status

`done`

## Problem

`roomci-core` accepts `ValidatedScenario`, but still uses `scenario.raw()` to
rebuild promoted subsystem models and config. That keeps raw YAML maps in the
runtime path.

## Scope

- Move promoted subsystem inputs into the validated graph.
- Keep raw `ScenarioFile` accessible only for compatibility/report boundaries.
- Avoid changing the public YAML schema.
- Preserve existing scenario behavior.

## Implementation Checklist

- [x] Add a test or review assertion showing runtime construction no longer
  calls `scenario.raw()` for promoted subsystem inputs.
- [x] Add validated subsystem fields or a `RuntimeInputs` struct to
  `ValidatedScenario`.
- [x] Populate edge, Modbus, lighting, contacts, ops, broker config, and domain
  config during validation.
- [x] Update `RuntimeState::new` to consume validated inputs.
- [x] Keep report metadata access explicit and minimal.
- [x] Run the compatibility checks from `phase_test.md`.

## Acceptance Criteria

- `roomci-core` no longer parses promoted subsystem config from raw scenario
  maps.
- Raw scenario access in core is limited to metadata or documented compatibility
  needs.
- Existing scenarios produce the same results.
