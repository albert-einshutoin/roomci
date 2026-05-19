# Task 03 — External MQTT PoC

## Objective

Allow at least one externally driven MQTT retained-state scenario to run through a real network endpoint instead of only through internal YAML steps.

## Acceptance Criteria

- Provide either an embedded MQTT-compatible endpoint or a broker-adapter integration.
- An external client can publish a device command to a configured command topic.
- `roomci` observes the publish, routes or maps it through the existing model, and records timeline events.
- Retained state behavior is visible to an external subscriber or through the HTTP state/report API.
- Cloud-offline or broker-fault behavior is represented without calling real cloud endpoints.
- Tests cover at least one successful retained-state scenario and one invalid-topic or unsupported-payload path.

## Notes

- Full MQTT broker conformance is not required.
- Prefer adapting an existing broker/library if that reduces protocol risk.
- Document the supported MQTT subset explicitly.
