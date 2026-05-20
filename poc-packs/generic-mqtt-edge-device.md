# Generic MQTT Edge-device PoC Pack

Run:

```bash
make poc-generic-mqtt
```

## Includes

- Adapter contract: `adapter-contracts/examples/generic_mqtt_edge_device.yaml`
- Scenarios:
  - `examples/generic_mqtt_retained_state.yaml`
  - `examples/generic_mqtt_duplicate_delivery.yaml`
- Reports:
  - `reports/poc_generic_mqtt_retained_state.{json,md,xml}`
  - `reports/poc_generic_mqtt_duplicate_delivery.{json,md,xml}`

## Acceptance Checklist

- Adapter contract validates.
- MQTT command topic maps to retained state topic.
- Duplicate delivery remains idempotent.
- JSON, Markdown, and JUnit reports are generated.

## Replace With Real Spec

- MQTT topic templates.
- Required JSON payload fields.
- Device-id extraction rule.
- QoS, retained-state, reconnect, and session expectations.
