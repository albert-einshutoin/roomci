# Task 04 — Generic MQTT Contract Examples

## Objective

Add generic MQTT/device examples that are not tied to hospitality smart-home naming.

## Acceptance Criteria

- Add at least two generic MQTT examples.
- Examples should exercise reusable behavior such as:
  - retained state contract
  - duplicate delivery idempotency
  - reconnect recovery
  - offline local behavior
  - command/state topic mapping
- Examples pass `roomci run`.
- Examples are included in `roomci validate`.
- README and `docs/GENERIC_MQTT_CONTRACTS.md` explain them.

## Suggested Examples

- `examples/generic_mqtt_retained_state.yaml`
- `examples/generic_mqtt_duplicate_delivery.yaml`
