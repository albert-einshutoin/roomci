# Task 04: Typed Ops and Automation Steps

## Status

`done`

## Problem

Known `ops` and `automation` behavior still dispatches by reading raw map keys
inside runtime. `acknowledge` and `hvac_auto_mode` are promoted behaviors and
should be typed before Phase 26 adds more contract semantics.

## Scope

- Add typed variants for known ops and automation steps.
- Keep unknown ops/automation extension data explicitly separated.
- Move validation of required fields to the validated boundary.
- Preserve current scenario YAML.

## Implementation Checklist

- [x] Add tests for valid `ops.action: acknowledge`.
- [x] Add tests for missing or malformed `ops.alert_id` where required.
- [x] Add tests for valid `automation.type: hvac_auto_mode`.
- [x] Add tests for unknown extension data being preserved or rejected according
  to the chosen boundary.
- [x] Add `ValidatedOpsStep` and `ValidatedAutomationStep` enums.
- [x] Update runtime to match typed variants instead of map strings.
- [x] Document any remaining map-driven ops/automation as extension boundary.

## Acceptance Criteria

- Promoted ops/automation behavior uses exhaustive enum matching.
- Invalid promoted ops/automation input fails before execution.
- Existing examples remain compatible.
