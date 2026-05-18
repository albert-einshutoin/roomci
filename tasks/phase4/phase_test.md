# Phase 4 Test Plan

## Quality Gates

- Shadow documents are schema-tested.
- Desired/reported/delta behavior has unit and integration coverage.
- Rejected responses are deterministic and explain validation errors.
- Shadow behavior never bypasses core command validation.

## Required Test Cases

1. Get empty or initial shadow for configured device.
2. Desired lock update produces canonical command.
3. Reported lock state updates after successful command.
4. Delta exists when desired differs from reported.
5. Invalid desired property returns rejected response.
6. Offline fault prevents reported state from pretending success.

## CI Expectations

- REST-like shadow tests run with HTTP adapter tests.
- MQTT-like topic tests can be integration-gated with Phase 2 broker tests.

## Done Means

Phase 4 is done when cloud-IoT backend tests can exercise shadow semantics locally without AWS.
