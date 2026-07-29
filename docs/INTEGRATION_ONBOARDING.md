# Integration Onboarding

This path is for a platform, IoT, smart-home, or building-automation engineer evaluating `roomci` in their own repository.

Before replacing public example values, complete the
[Evaluator Intake Kit](EVALUATOR_INTAKE_KIT.md). Missing customer-owned facts
remain blocked; they are never inferred from a public fixture.

## 30-second Path (no clone required)

After installing a released `roomci` binary, scaffold a runnable retained-state
smoke scenario in the current repository:

```bash
roomci init --ci github
roomci validate roomci/smoke.yaml
roomci run roomci/smoke.yaml --verbose
```

The optional workflow uses `albert-einshutoin/roomci@v0.1.1` and writes reports
to `roomci-reports`. `init` performs a full preflight: if any generated target
already exists, it writes nothing and exits with guidance. Use `--force` only
when replacing all generated files is intentional. The generated VS Code YAML
settings file is never merged, avoiding a silent edit to repository settings.

Use [`RELEASING.md`](RELEASING.md) for binary installation and upgrade checks.

## 15-minute Path

1. Pick a PoC pack:
   - Generic MQTT: `make poc-generic-mqtt`
   - Hospitality local-first: `make poc-hospitality`
   - Building automation / BMS: `make poc-building-automation`
   - BMS / operations: `make poc-bms-ops`
2. Inspect the adapter contract used by that pack under `adapter-contracts/examples/`.
3. Copy `adapter-contracts/templates/company_adapter_contract.yaml`.
4. Replace topic templates, device ids, register maps, alert routes, auth assumptions, and acceptance criteria with your real non-production spec.
5. Validate the contract:

```bash
cargo run -p roomci-cli -- adapter validate path/to/your_adapter_contract.yaml
```

6. Start the HTTP/MQTT PoC surface:

```bash
cargo run -p roomci-cli -- serve --config examples/generic_mqtt_retained_state.yaml --port 8080 --mqtt-port 1883
```

7. Drive it from your controller, test client, or script.
8. Collect:
   - `GET /state`
   - `GET /timeline`
   - `GET /reports/latest.json`
   - `GET /reports/latest.md`
   - `GET /reports/latest.junit.xml`

## HTTP Serve API

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/health` | Read `idle`, `running`, `passed`, or `failed` status |
| `GET` | `/scenario` | Inspect loaded scenario metadata |
| `GET` | `/state` | Inspect current emulator state |
| `GET` | `/timeline` | Inspect emitted events |
| `POST` | `/fault` | Inject a fault into the running state |
| `POST` | `/run` | Execute the loaded scenario |
| `POST` | `/finish` | Finalize and create the latest report |
| `GET` | `/reports/latest.json` | JSON report |
| `GET` | `/reports/latest.md` | Markdown report |
| `GET` | `/reports/latest.junit.xml` | JUnit report |

See [`HTTP_SERVE_BEHAVIOR.md`](HTTP_SERVE_BEHAVIOR.md) for timeout, overload, health, and run-lock semantics.

## Troubleshooting

| Symptom | Likely Cause | Fix |
|---|---|---|
| Port already in use | HTTP or MQTT port conflict | Use `--port 0`, `--mqtt-port 0`, or choose another port |
| MQTT publish is ignored | Topic does not match `mqtt.contracts[].command_topic` | Update the adapter/scenario topic template and `{device_id}` placeholder |
| MQTT publish is rejected | JSON payload is malformed or missing required fields | Match `payload.required_fields` in the adapter contract |
| `CONNACK` returns `0x01` | MQTT client is not protocol name `MQTT`, level `4` | Use MQTT 3.1.1 for current serve subset |
| Modbus request returns exception `0x02` | Unit id or register address is not in the scenario map | Update `modbus.devices[].unit_id` and register addresses |
| Modbus write returns exception `0x03` | Register is read-only or value/quantity is outside the subset | Write only configured holding registers with one value |
| `/health` returns HTTP 503 | Latest report failed | Read `/reports/latest.md` and `/timeline` for failure evidence |
| Docker/Compose fails | Docker daemon, image build, or port issue | Run `docker build -t roomci:verify .`, then `make compose-poc` |
| Report seems too narrow | Scenario has too few assertions | Add acceptance criteria to adapter contract and map them to scenario assertions |

## Client Snippets

HTTP health:

```bash
curl -fsS http://127.0.0.1:8080/health
```

Inject a fault:

```bash
curl -fsS -X POST http://127.0.0.1:8080/fault \
  -H 'content-type: application/json' \
  -d '{"target":"mqtt.cloud","type":"offline"}'
```

Collect Markdown evidence:

```bash
curl -fsS http://127.0.0.1:8080/reports/latest.md > reports/latest.md
```
