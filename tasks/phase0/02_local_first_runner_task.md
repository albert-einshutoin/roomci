# Task 02 — Local-first Runner

## Objective

Make `local_first_cloud_outage.yaml` execute as the first product-defining scenario.

## Implementation Scope

- Model local and cloud MQTT availability.
- Treat `mqtt.cloud` outage as isolated from `mqtt.local`.
- Handle `mqtt_publish` steps.
- Apply device state updates from local command topics.
- Evaluate MQTT retained assertions.
- Evaluate `guest_experience: unaffected`.

## Acceptance Criteria

- Cloud outage scenario passes.
- Timeline shows cloud outage, local publish, retained state update, and assertions.
- Report explains why local guest experience remained unaffected.
