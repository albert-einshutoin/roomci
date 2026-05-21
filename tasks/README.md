# roomci Task Plan

This task plan now follows `roomci-docs-latest` as the product source of truth.

The product target is:

```txt
Local-first Smart Home QA & Operations Emulator for CI
```

The current detailed product goal is tracked in
[`product_goal.md`](product_goal.md).

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
| Phase 13 | Protocol compliance track | Selected protocol subsets are tied to official specifications and verified with standard clients/tools |
| Phase 14 | Hospitality smart-home QA core coverage | The product boundary is fixed around core QA flows instead of the full reported technology stack |
| Phase 15 | Evaluator friction removal | The release-candidate evaluation surface is hardened around MQTT/Modbus depth, Docker reproducibility, maintainability, and claim evidence |
| Phase 16 | Roadmap triage for optional depth | Implied future work is explicitly promoted, deferred, or rejected before new breadth is added |
| Phase 17 | Promoted contract depth | Safe intercom/relay, network/control-panel, BMS hardening, comfort time-series, and adapter samples are implemented |
| Phase 18 | Release-candidate evidence hardening | Adapter samples, Phase 17 claims, and evaluator scorecards are mechanically verifiable |
| Phase 19 | S Tier observability and CI evidence | GitHub Actions parity, stable timeline export, trace metadata, observability artifacts, and evaluator CI docs complete the S Tier surface |
| Phase 20 | A Tier developer experience | Python SDK, scenario debugger, VSCode assets, and developer workflow docs make adoption easier |
| Phase 21 | B Tier protocol profiles | Matter, BACnet, KNX, and OPC UA are represented as honest contract profiles without certification claims |

## Progress Management

Use:

- `tasks/status.md` for the top-level phase board.
- `tasks/phase*/phase_status.md` for task-level status, quality gates, blockers, and next action.
- `tasks/backlog_inventory.md` for the implemented / taskified / newly taskified / out-of-scope backlog split.
- `tasks/strategy_alignment.md` for the current product strategy mapped to phase/task status.

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
