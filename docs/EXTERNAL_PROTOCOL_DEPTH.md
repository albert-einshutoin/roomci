# External Protocol Depth

`roomci serve` now has two externally drivable surfaces:

- MQTT 3.1.1 subset for command/state contract PoCs.
- HTTP BMS/contact event endpoint for operations and contact-I/O PoCs.

## MQTT Subset Boundary

Current supported MQTT serve subset:

- protocol name `MQTT`
- protocol level `4` (MQTT 3.1.1)
- QoS0 `PUBLISH`
- JSON object payloads
- topic matching through `mqtt.contracts`
- retained-state observation through HTTP `/state` and reports

Current unsupported MQTT behavior:

- QoS1/QoS2 wire acknowledgements
- retained replay to MQTT subscribers
- subscription handling
- MQTT 5 properties
- TLS, ACLs, auth, clustering, persistence

See [`MQTT_SERVE_SUBSET.md`](MQTT_SERVE_SUBSET.md).

## BMS / Contact External Endpoint

Endpoint:

```txt
POST /external/bms/contact
```

Payload:

```json
{
  "source": "contact.sauna_emergency_button",
  "state": "on",
  "severity": "critical"
}
```

Effects:

- appends timeline event `external_bms_contact_observed`
- writes `external.bms.<source>` into `/state`
- keeps the event visible in JSON and Markdown reports after `/finish`

Example client:

```bash
ROOMCI_URL=http://127.0.0.1:8080 examples/controllers/bms_webhook_poc_controller.sh
```

## Boundary

The BMS/contact endpoint is not a production BMS webhook implementation. It is a PoC input surface for external test clients that need to prove contact and operations events can update the same timeline/report model as scenario-mode steps.
