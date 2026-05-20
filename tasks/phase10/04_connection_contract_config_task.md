# Task 04 — Connection Contract Config

## Objective

Make external integration contracts configurable so vendor-specific topic structures, payloads, and device mappings can be adapted without code changes.

## Acceptance Criteria

- Scenario/config schema supports MQTT command/state topic mappings.
- Config supports extracting a device id from a topic using a documented mapping strategy.
- Config supports basic payload expectations for command-to-state updates.
- Docs include examples for generic MQTT and hospitality smart-home mappings.
- Validation catches ambiguous mappings, missing state topics, and unsupported adapter declarations.
- Existing examples remain valid or are migrated with clear compatibility notes.

## Notes

- This is the layer that makes the product useful before private hospitality protocol details are known.
- Keep the first version small; avoid building a general-purpose transformation language unless required.
