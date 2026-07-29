# Pre-adoption PoC Checklist

Use this checklist when evaluating `roomci` before adopting it for a real smart-home, edge-device, BMS, or building-automation workflow.

Collect and map the customer-owned inputs first with the
[Evaluator Intake Kit](EVALUATOR_INTAKE_KIT.md). A public PoC pack is a
baseline, not evidence of compatibility with a private deployment.

## 1. Pick the Integration Mode

- Scenario mode: run YAML scenarios fully inside `roomci`.
- Serve mode over HTTP: external tests call localhost endpoints and collect reports.
- Serve mode over MQTT subset: external clients publish command payloads to `--mqtt-port`, then observe state and reports through HTTP.

## 2. Provide Protocol Contracts

For MQTT:

- command topic templates
- state topic templates
- device-id extraction strategy
- JSON payload fields and required fields
- QoS and retained-state expectations
- reconnect/session expectations

For Modbus:

- register map
- register type and scale
- writable vs read-only addresses
- expected commissioning values

For BMS / operations:

- alert source mapping
- webhook/API contract
- notification targets
- ticket lifecycle
- runbook links

## 3. Define Pass/Fail Criteria

- expected retained state
- expected device state
- expected failover timing
- expected guest or operator impact
- expected report artifacts
- non-zero behavior for intentional failures

## 4. Run the PoC Gates

```bash
cargo run --release -- serve --config examples/generic_mqtt_retained_state.yaml --mqtt-port 1883
make compose-poc
make verify
```

`GET /health` should report `idle` before the controller finishes the PoC and `passed` after `/finish` for a successful run. A `failed` health response returns HTTP 503 so CI gates fail closed.

## 5. Confirm Boundaries

`roomci` is a QA contract emulator. It does not claim private-system compatibility, full MQTT broker behavior, production BMS behavior, or replacement of real device commissioning.

For any organization, real integration requires their actual MQTT topics, payload schemas, device/register maps, BMS contracts, auth/TLS assumptions, and acceptance criteria.
