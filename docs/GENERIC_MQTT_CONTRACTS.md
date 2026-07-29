# Generic MQTT Contracts

## Purpose

Generic MQTT scenarios show the reusable `roomci` core without hospitality-specific room, guest, or property naming.

The current MQTT model is a behavioral emulator. It validates a simple command/state contract:

```txt
.../device/<device_id>/command -> .../device/<device_id>/state
```

The payload published to the command topic becomes retained state on the derived state topic after edge routing succeeds.

## Current Supported Contract

Supported today:

- local broker availability state
- retained command/state update behavior
- duplicate delivery idempotency through a scheduled fault
- edge routing into a declared device id
- configurable `mqtt.contracts` topic mappings with `{device_id}` extraction
- serve-mode MQTT 3.1.1 `CONNECT` + QoS0 `PUBLISH` PoC ingress
- JSON, Markdown, and JUnit reporting from the same run

Not supported today:

- full MQTT broker conformance
- arbitrary transformation language for topic/payload mapping
- QoS2, session persistence, authorization, TLS, ACLs, or clustering
- production broker replacement behavior

See [`MQTT_SERVE_SUBSET.md`](MQTT_SERVE_SUBSET.md) for the serve-mode protocol boundary.

## Examples

### Retained State

`examples/generic_mqtt_retained_state.yaml` publishes a generic edge-device command:

```txt
fleet/demo/site/lab/device/env_sensor_01/command
```

The assertion checks the retained state topic:

```txt
fleet/demo/site/lab/device/env_sensor_01/state
```

Run it with:

```bash
cargo run -p roomci-cli -- run examples/generic_mqtt_retained_state.yaml --verbose
```

### Duplicate Delivery

`examples/generic_mqtt_duplicate_delivery.yaml` injects a `duplicate_delivery` fault for a command topic and verifies the retained state still has one semantic final value.

Run it with:

```bash
cargo run -p roomci-cli -- run examples/generic_mqtt_duplicate_delivery.yaml --verbose
```

Or run both generic MQTT examples:

```bash
make demo-generic-mqtt
```

## Configurable Topic Mapping

For pre-adoption PoC use, scenarios can declare topic mappings:

```yaml
mqtt:
  contracts:
    - name: generic_device_retained_state
      adapter: mqtt_v3_qos0_subset
      command_topic: fleet/demo/site/lab/device/{device_id}/command
      state_topic: fleet/demo/site/lab/device/{device_id}/state
      device_id_from_topic: placeholder:{device_id}
      payload:
        required_fields: [online, sample_interval_seconds]
        optional_fields: [mode]
        fields:
          online:
            type: boolean
          sample_interval_seconds:
            type: integer
            minimum: 1
          mode:
            type: string
            enum: [normal, low_power]
```

That keeps vendor-specific MQTT naming and payload assumptions out of code and
lets a real integration contract be supplied later. `fields` is intentionally
limited to JSON value types, enums, and numeric ranges; it is not a nested JSON
Schema engine.
