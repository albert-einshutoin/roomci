# MQTT Serve Subset

`roomci serve --mqtt-port <port>` starts a minimal MQTT 3.1.1 ingress for pre-adoption PoC tests.

This is not a production broker and not a conformance suite. It exists so an external controller can publish command payloads over a real MQTT-shaped TCP endpoint while `roomci` records retained-state behavior through the existing HTTP state/report API.

The HTTP observation API used with this MQTT ingress is documented in [`HTTP_SERVE_BEHAVIOR.md`](HTTP_SERVE_BEHAVIOR.md).

## Supported

- MQTT 3.1.1 `CONNECT`
- `CONNACK` success response
- QoS 0 `PUBLISH`
- UTF-8 topic names
- JSON object payloads
- one `{device_id}` placeholder in configured command/state topic mappings
- retained-state observation through:
  - `GET /state`
  - `GET /timeline`
  - `GET /reports/latest.json`
  - `GET /reports/latest.md`
  - `GET /reports/latest.junit.xml`

## Not Supported

- QoS 1 / QoS 2 wire-level acknowledgements
- `SUBSCRIBE` / `UNSUBSCRIBE`
- retained message replay to MQTT subscribers
- sessions, will messages, keepalive enforcement
- TLS, ACLs, authentication, clustering, persistence
- MQTT 5 properties

## CONNECT Validation

The MQTT ingress accepts only:

- protocol name: `MQTT`
- protocol level: `4` (MQTT 3.1.1)

Unsupported protocol names or levels receive `CONNACK` return code `0x01` (`unacceptable protocol version`) and the TCP connection is closed. For example, MQTT 3.1 `MQIsdp` and MQTT 5 protocol level `5` are rejected.

## Contract Config

Scenario files can declare MQTT connection contracts:

```yaml
mqtt:
  local:
    retained: true
  contracts:
    - name: generic_device_retained_state
      adapter: mqtt_v3_qos0_subset
      command_topic: fleet/demo/site/lab/device/{device_id}/command
      state_topic: fleet/demo/site/lab/device/{device_id}/state
      device_id_from_topic: placeholder:{device_id}
      payload:
        required_fields: [online, sample_interval_seconds]
```

Validation rejects unsupported adapters, missing topic mappings, duplicate command mappings, and unsupported device-id extraction strategies.

## Real Integration Inputs

To adapt this PoC to a real customer or vendor, the integrator must provide:

- command topics
- state topics
- payload schemas and required fields
- QoS, retained, reconnect, and session expectations
- auth/TLS/network assumptions
- pass/fail acceptance criteria
- Modbus register maps, if Modbus is involved
- BMS webhook/API contracts, if operations flows are involved

Any organization's compatibility is not claimed without their actual integration contract.
