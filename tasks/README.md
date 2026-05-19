# roomci Task Plan

This task plan now follows `roomci-docs-latest` as the product source of truth.

The product target is:

```txt
Local-first Smart Home QA & Operations Emulator for CI
```

## Phase Map

| Phase | Focus | Primary Outcome |
|---|---|---|
| Phase 0 | Latest scenario contract, CLI, reports | `local_first_cloud_outage.yaml` validates and runs from the new schema |
| Phase 1 | Local MQTT retained-state model | MQTT publish/state/retained assertions work in scenario mode |
| Phase 2 | Edge server emulator | iPad/controller commands route through edge to devices during cloud outage |
| Phase 3 | Device mocks | Modbus TCP, DALI-like lighting, and contact I/O scenarios run |
| Phase 4 | Ops/BMS mock | Contact alerts trigger Slack/phone/ticket/runbook mock outputs |
| Phase 5 | Docker and CI packaging | Docker image, Compose, and GitHub Actions examples execute the demos |
| Phase 6 | Reliability depth | Edge failover, network/WAN failover, comfort automation, and future integrations |
| Phase 7 | Production readiness | Docs, coverage, CLI, and CI quality gates are hardened |
| Phase 8 | Public release and interview polish | Public claims, demos, license, and release gates are honest and reproducible |
| Phase 9 | Generic MQTT contract positioning | Product is reframed as a generic MQTT / edge / device QA contract emulator |
| Phase 10 | Pre-adoption PoC productization | External clients can connect to localhost endpoints and produce CI-ready reports |
| Phase 11 | Integration-ready emulator platform | Company-specific specs can be mapped through contracts, PoC packs, and external protocol endpoints |
| Phase 12 | Serve runtime hardening and release plumbing | Existing serve endpoints survive evaluator-style concurrency, malformed clients, and public release checks |

## Progress Management

Use:

- `tasks/status.md` for the top-level phase board.
- `tasks/phase*/phase_status.md` for task-level status, quality gates, blockers, and next action.

Valid statuses:

- `todo`
- `in_progress`
- `blocked`
- `review`
- `done`

## Quality Policy

- A task is not `done` until its acceptance criteria pass.
- A phase is not `done` until its `phase_test.md` quality gates pass.
- Reports must remain useful to both software engineers and field engineers.
- Protocol support must be documented as `*-like` unless real protocol conformance is implemented.
- Default behavior must not call real Slack, phone, cloud, SIP, or device endpoints.
