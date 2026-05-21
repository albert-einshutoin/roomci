# HTTP Serve MVPプラン

## 目的

Turn `roomci serve` into a localhost-bound virtual system that external tests can drive before full MQTT-compatible serve mode exists.

This document records the Phase 10 serve-mode direction. The first implementation step now starts a localhost-bound HTTP control/report API from `roomci serve --config <scenario>`, while `--check` still validates configuration and exits.

## なぜ HTTP First

HTTP should come before full MQTT broker compatibility because it gives CI harnesses a deterministic control and observation surface with less protocol risk.

The HTTP API can expose scenario metadata, current state, timeline, fault injection, and reports while the existing internal runner remains the source of truth. That makes the next implementation step useful without claiming broker compatibility.

## Proposed Endpoints

| Method | Path | 目的 |
|---|---|---|
| `GET` | `/health` | Confirm the emulator is alive and report version/scenario id |
| `GET` | `/scenario` | Return loaded scenario metadata and configured contracts |
| `GET` | `/state` | Return current device, MQTT retained, edge, ops, and fault state |
| `GET` | `/timeline` | Return recorded timeline events |
| `POST` | `/run` | Execute or advance the configured scenario in service mode |
| `POST` | `/fault` | Inject a fault using the existing `target` and `type` shape |
| `GET` | `/reports/latest` | Return the latest report, with content negotiation or format query |

Useful report variants:

```txt
GET /reports/latest?format=json
GET /reports/latest?format=markdown
GET /reports/latest?format=junit
```

The current implementation also exposes explicit report paths:

```txt
GET /reports/latest.json
GET /reports/latest.md
GET /reports/latest.junit.xml
```

## What This Enables

An external test can:

1. Start `roomci serve --config scenario.yaml`.
2. Poll `GET /health`.
3. Drive controller logic or call `POST /run`.
4. Inject a fault with `POST /fault`.
5. Read `GET /timeline` and `GET /state`.
6. Collect `GET /reports/latest` for CI.

The current black-box PoC uses `make compose-poc`. Docker Compose starts `roomci serve` and then runs `examples/controllers/http_poc_controller.sh` as a separate controller process that only talks to the HTTP API.

## MQTT-compatible Serve Mode Boundary

The current `--mqtt-port` surface supports a narrow PoC subset: MQTT 3.1.1 `CONNECT` and QoS0 `PUBLISH` with JSON object payloads. It updates retained state through configured `mqtt.contracts`, and the result is observed through the HTTP state/report API.

Full broker compatibility still requires:

- a supported MQTT subset document
- clear non-goals around TLS, ACLs, clustering, and full MQTT conformance
- MQTT subscriber replay if external subscription is required
- QoS1/QoS2 wire-level behavior if required by the customer contract

The product claim should remain "QA contract emulator" rather than "production MQTT broker."
