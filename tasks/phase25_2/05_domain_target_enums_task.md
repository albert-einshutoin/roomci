# Task 05: Domain Target Enums

## Status

`done`

## Problem

Several promoted target fields remain broad `String` values even though runtime
behavior depends on target shape. Rust cannot prevent target mixups until those
are modeled as enums.

## Scope

- Add target enums for promoted command, fault, assertion, and sensor targets.
- Keep string rendering at report/timeline boundaries.
- Validate target prefixes once during scenario conversion.
- Leave genuinely dynamic extension targets documented as extension data.

## Implementation Checklist

- [x] Add `CommandTarget` for scene activation and any existing promoted command
  target forms.
- [x] Promote fault target components for MQTT broker, edge, WAN, network
  segment, firewall policy, control-panel, and DALI fixture forms into typed
  fault variants.
- [x] Keep promoted target/condition assertions as typed
  `ValidatedTargetConditionAssertion` variants.
- [x] Add `SensorTarget` or a narrower comfort target wrapper where runtime
  records sensor evidence.
- [x] Update runtime to match target enums.
- [x] Add invalid target regression tests.
- [x] Confirm timeline/report string output remains stable.

## Acceptance Criteria

- Promoted target mixups are prevented by enum types.
- Runtime no longer strips target prefixes repeatedly for promoted behavior.
- External output remains string-compatible.
