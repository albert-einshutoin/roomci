# Phase 10 Goal — Pre-adoption PoC Productization

## Goal

Turn `roomci` from a scenario-only CLI emulator into a pre-adoption PoC product that external smart-home, edge, BMS, and device-controller services can connect to during local development or CI.

The target product claim after this phase is:

```txt
Bring your MQTT topics, Modbus register map, and BMS webhook contract; roomci provides the local emulator, fault scheduler, observability timeline, and CI-ready reports.
```

This phase must not claim hospitality compatibility. Instead, it should make `roomci` ready to adapt once a real integration contract is provided.

## In Scope

- Implement a real `roomci serve` mode that starts localhost-bound test endpoints.
- Provide an HTTP control and report API for health, scenario metadata, state, timeline, fault injection, run execution, and report retrieval.
- Add an MQTT-compatible PoC endpoint or broker adapter sufficient for external publish/subscribe tests against retained state behavior.
- Add a pluggable connection-contract model for MQTT topic mappings, payload schemas, and future protocol adapters.
- Add Docker Compose E2E with a sample external controller process that connects to `roomci` as a black-box dependency.
- Produce JSON, Markdown, and JUnit reports from externally driven interactions.
- Document exactly what is generic and configurable versus what still requires a customer/vendor-specific contract.

## Out of Scope

- Full MQTT broker conformance.
- Full Modbus, DALI, KNX, BACnet, Matter, SIP, or PBX conformance.
- hospitality private protocol compatibility.
- Real Slack, phone, cloud, SIP, device, or BMS calls.
- Production-grade access control, tenant isolation, or public network exposure.

## Product Boundary

Phase 10 should make the product useful before a vendor-specific integration by supporting this workflow:

```txt
roomci serve --config scenario.yaml
  -> external controller connects to localhost endpoints
  -> controller publishes commands / sends webhooks / reads mock state
  -> roomci applies scheduled faults and records observed events
  -> test calls finish/report endpoint
  -> CI consumes exit code and JUnit/JSON/Markdown reports
```

## Exit Criteria

- `roomci serve --config <scenario>` starts a localhost service instead of only printing a config-check message.
- `roomci serve --config <scenario> --check` remains supported and validates without starting endpoints.
- HTTP API exposes at least:
  - `GET /health`
  - `GET /scenario`
  - `GET /state`
  - `GET /timeline`
  - `POST /fault`
  - `POST /finish`
  - `GET /reports/latest.json`
  - `GET /reports/latest.md`
  - `GET /reports/latest.junit.xml`
- At least one external sample controller drives `roomci` through a real network endpoint in Docker Compose.
- At least one MQTT retained-state scenario can be driven by an external client rather than only by internal YAML steps.
- The service binds to localhost by default and refuses broad network exposure unless explicitly configured.
- README and docs clearly state that Phase 10 provides a PoC integration surface, not full protocol conformance.
- `make verify` includes the serve-mode E2E gate or an equivalent CI target.
