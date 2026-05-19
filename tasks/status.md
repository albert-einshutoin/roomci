# roomci Task Status

## Phase Status

| Phase | Status | Current Focus | Blocking Issue | Status File |
|---|---|---|---|---|
| Phase 0 | `done` | Latest scenario contract, CLI, reports | None | `phase0/phase_status.md` |
| Phase 1 | `done` | Local MQTT retained-state model | None | `phase1/phase_status.md` |
| Phase 2 | `done` | Edge server emulator | None | `phase2/phase_status.md` |
| Phase 3 | `done` | Modbus, DALI-like, contact I/O mocks | None | `phase3/phase_status.md` |
| Phase 4 | `done` | BMS/ops alert mock | None | `phase4/phase_status.md` |
| Phase 5 | `done` | Docker and CI packaging | None | `phase5/phase_status.md` |
| Phase 6 | `done` | Reliability depth and future integrations | None | `phase6/phase_status.md` |
| Phase 7 | `done` | Production readiness (docs, coverage, CLI, CI) | None | `phase7/phase_status.md` |
| Phase 8 | `done` | Public release and interview polish | None | `phase8/phase_status.md` |
| Phase 9 | `done` | Generic MQTT contract positioning | None | `phase9/phase_status.md` |
| Phase 10 | `done` | Pre-adoption PoC productization with HTTP serve, black-box controller E2E, MQTT QoS0 ingress, configurable contracts, and PoC docs | None | `phase10/phase_status.md` |
| Phase 11 | `todo` | Integration-ready emulator platform for company-specific IoT, SmartHome, building-automation, and NOT A HOTEL-style PoCs | Real customer/vendor specs are required only for compatibility claims, not for generic platform progress | `phase11/phase_status.md` |
| Phase 12 | `todo` | Serve runtime hardening (HTTP concurrency, read timeout, mutex panic safety, `/run` lock scope, `/health` semantics, MQTT CONNECT validation) and release plumbing (workspace Cargo metadata, CHANGELOG, working README badges) | README badge URLs depend on the GitHub repository being pushed public; everything else is unblocked | `phase12/phase_status.md` |

## Update Rules

- Update this file when a phase changes status.
- Update the matching `phase_status.md` when a task or quality gate changes status.
- Use `review` when implementation exists but verification or acceptance criteria remain incomplete.
- Use `blocked` only with a concrete blocker.

## Current Recommendation

Phase 10 turned the contract emulator into a pre-adoption PoC product: a localhost-bound service that external clients can connect to, drive, observe, and collect CI-ready reports from without requiring private NOT A HOTEL protocol details.

Phase 11 and Phase 12 should run in parallel:

- **Phase 11** expands the external surface so `roomci` becomes integration-ready for real company evaluations — adapter contract kit, protocol support matrix, customer PoC packs, second external protocol surface.
- **Phase 12** hardens the existing surface the strict review of Phase 9 + 10 flagged — HTTP per-connection concurrency and read timeout, mutex panic safety, `/run` lock scope, `/health` semantics, MQTT CONNECT validation, plus release plumbing (workspace Cargo metadata, CHANGELOG, working README badges).

Where the two phases intersect (adapter dispatch), Phase 12 defers to Phase 11's `03_adapter_contract_kit_task.md`. Where Phase 12 lands first (release plumbing, runtime safety), Phase 11 benefits because every Phase 11 deliverable is shipped on a more credible runtime and a more honest public surface.
