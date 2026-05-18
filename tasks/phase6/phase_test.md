# Phase 6 Test Plan

## Quality Gates

- Scene activation tests cover all member commands.
- Partial failure tests verify both failed and successful members.
- Golden Markdown report shows scene impact clearly.
- Scene assertions do not depend on command ordering unless explicitly required.

## Required Test Cases

1. Welcome scene turns on light, sets brightness, opens curtain, and sets AC target.
2. Partial scene failure records which member failed.
3. Scene consistency assertion passes when all members reach expected state.
4. Scene consistency assertion fails when one member is blocked by a fault.
5. Scene timeline remains deterministic.

## CI Expectations

- Scene tests run without real Hue bridge or vendor SDK.
- Golden report changes require explicit review.

## Done Means

Phase 6 is done when ambience-level smart-room failures are testable as a product experience, not isolated device commands.
