# Phase 6 Goal — Hue-like Lighting and Scene Adapter

## Goal

Support room, zone, group, and scene-oriented lighting tests, including partial welcome-scene failures.

## In Scope

- Hue-like lighting and group vocabulary.
- Room/zone/group model for lights and covers where useful.
- Scene activation.
- Scene consistency assertions.
- Partial scene failure fault.

## Non-goals

- Full Philips Hue API clone.
- Vendor app compatibility.
- Bridge discovery or real device pairing.

## Deliverables

- Scene model integrated with canonical commands.
- Scene activation API or adapter behavior.
- `welcome_scene_partial_failure` example.
- Golden reports for complete and partial scene results.

## Exit Criteria

- A scene activates multiple devices through canonical commands.
- Partial failure is visible in the report timeline and assertion output.
- Scene consistency assertion can fail with guest impact context.
