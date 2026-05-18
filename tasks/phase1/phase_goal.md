# Phase 1 Goal — HTTP Adapter

## Goal

Expose roomci's canonical engine through a local HTTP API so backend and app tests can control devices, inject faults, inspect state, and read timelines.

## In Scope

- `serve` mode with HTTP listener.
- Healthcheck endpoint.
- Room and device read endpoints.
- Device command endpoint.
- Fault injection endpoint.
- Timeline endpoint.
- Scenario run endpoint for local integration tests.

## Non-goals

- MQTT support.
- Authentication beyond local defaults.
- Cloud-IoT shaped APIs.
- Public internet deployment.

## Deliverables

- `roomci serve --config room.yaml --http 127.0.0.1:8080`.
- REST API routes listed in `docs/08_adapter_implementation_plan.md`.
- API errors with stable JSON shape.
- HTTP integration tests.

## Exit Criteria

- A backend test can unlock an emulated smart lock through HTTP.
- State changes go through the core engine, not direct adapter mutation.
- Faults injected through HTTP affect later commands.
- `/healthz` reports service status and version.
