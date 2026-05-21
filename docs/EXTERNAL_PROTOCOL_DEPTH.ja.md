# 外部プロトコル深度

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

Required fields: `source` (string), `state` (string). Optional: `severity`
(string). Unknown fields are ignored.

### Validation responses

| Body shape | HTTP | `error` |
|---|---:|---|
| valid JSON object with `source` and `state` strings | 202 | (n/a, returns `accepted:true`) |
| missing `source` (or non-string `source`) | 400 | `missing_source` |
| missing `state` (or non-string `state`) | 400 | `missing_state` |
| body that does not parse as JSON | 400 | `invalid_json` |

`severity` is **not** enum-validated — any string is accepted and stored
under a sanitized form. The `application/json` Content-Type header is
**not** enforced; the handler parses the body regardless of the request
Content-Type. Stricter validation is deferred to the Adapter Contract
Kit (Phase 11 Task 03) so the runtime stays library-agnostic for now.

### Sanitization

External clients can feed adversarial values into `source`, `state`,
and `severity`. To prevent collisions with MQTT device-id namespaces
and to keep Markdown rendering safe:

- `source`, `state`, and `severity` are sanitized for storage. Any
  character outside `[A-Za-z0-9._:/-]` is replaced with `_`. Empty
  values become `unknown`.
- Timeline messages built from external input have their control
  characters (newline, carriage return, other ASCII control bytes)
  replaced with a space so Markdown reports cannot be hijacked.

The 202 response echoes the sanitized values so the controller can
confirm exactly what the server stored.

### Effects on serve state

- Appends a timeline event with `event_type:
  external_bms_contact_observed` and `target` equal to the sanitized
  source, queued on the external-observation overlay.
- Stores the observation under `external_observations[<sanitized
  source>]` so `GET /state` exposes it as a dedicated, observable
  bucket separate from device retained state.
- Survives `POST /run`: at the run success boundary the overlay
  timeline events are appended to the new `latest_report.timeline`,
  and each observation is merged into `latest_report.final_state`
  under the key `external.bms.<sanitized source>`. The overlay is
  drained at this point.

This is the contract that makes the BMS endpoint useful for
multi-step PoCs: an external controller can post a BMS event,
optionally trigger `POST /run`, and still see that event in the
rendered JSON/Markdown/JUnit reports.

### Example client

```bash
ROOMCI_URL=http://127.0.0.1:8080 examples/controllers/bms_webhook_poc_controller.sh
```

The example controller asserts each step lands on the server: the
202 response, the `external_observations` bucket on `/state`, the
event_type on `/timeline`, `finished:true` on `/finish`, and the
event presence in the rendered `latest.md`.

## Boundary

The BMS/contact endpoint is not a production BMS webhook
implementation. It is a PoC input surface for external test clients
that need to prove contact and operations events can update the
same timeline/report model as scenario-mode steps. Hardened webhook
features (HMAC signatures, replay protection, schema-versioning,
retry semantics, Content-Type enforcement, severity enums) are
deferred to the アダプターコントラクトキット and remain out of scope for
the runtime itself.
