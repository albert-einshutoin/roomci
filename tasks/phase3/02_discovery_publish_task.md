# Task 02 — Discovery Publish Flow

## Objective

Publish discovery payloads during MQTT startup and refresh.

## Implementation Scope

- Publish config topics under `homeassistant/{component}/{node_id}/{object_id}/config`.
- Decide retained message behavior.
- Add configuration flag to enable or disable discovery publishing.
- Add Compose example or docs for local validation.

## Acceptance Criteria

- Discovery publishing does not block normal MQTT command/state flow.
- Re-publishing is deterministic and does not create duplicate unique IDs.
- Docs clearly describe this as `Discovery-like`, not certified compatibility.

## References

- `docs/09_security_and_license_notes.md`
