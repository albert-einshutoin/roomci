# HTTP Serve MVP Plan

## Goal

Turn `roomci serve` into a localhost-bound virtual system that external tests can drive before full MQTT-compatible serve mode exists.

This is a plan, not the current implementation. Today, `roomci serve --config <scenario> --check` validates configuration and exits; long-running adapters are not enabled yet.

## Why HTTP First

HTTP should come before full MQTT broker compatibility because it gives CI harnesses a deterministic control and observation surface with less protocol risk.

The HTTP API can expose scenario metadata, current state, timeline, fault injection, and reports while the existing internal runner remains the source of truth. That makes the next implementation step useful without claiming broker compatibility.

## Proposed Endpoints

| Method | Path | Purpose |
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

## What This Enables

An external test can:

1. Start `roomci serve --config scenario.yaml`.
2. Poll `GET /health`.
3. Drive controller logic or call `POST /run`.
4. Inject a fault with `POST /fault`.
5. Read `GET /timeline` and `GET /state`.
6. Collect `GET /reports/latest` for CI.

## What MQTT-compatible Serve Mode Requires Later

MQTT-compatible serve mode should be added after the HTTP surface is stable. It requires:

- choosing embedded broker versus broker adapter
- a supported MQTT subset document
- configurable command/state topic mappings
- retained message behavior exposed through the same report model
- local/cloud broker fault behavior
- external client tests
- clear non-goals around TLS, ACLs, clustering, and full MQTT conformance

The product claim should remain "QA contract emulator" until a real MQTT compatibility surface is implemented and tested.
