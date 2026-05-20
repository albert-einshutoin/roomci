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
| Phase 11 | `in_progress` | External protocol depth after completed PoC/evidence/onboarding/positioning work | Real customer/vendor specs are required only for compatibility claims, not for generic platform progress | `phase11/phase_status.md` |
| Phase 12 | `done` | Serve runtime hardening, health semantics, MQTT CONNECT validation, and release plumbing | None | `phase12/phase_status.md` |

## Update Rules

- Update this file when a phase changes status.
- Update the matching `phase_status.md` when a task or quality gate changes status.
- Use `review` when implementation exists but verification or acceptance criteria remain incomplete.
- Use `blocked` only with a concrete blocker.

## Current Recommendation

Phase 10 turned the contract emulator into a pre-adoption PoC product: a localhost-bound service that external clients can connect to, drive, observe, and collect CI-ready reports from without requiring private hospitality evaluator protocol details.

Phase 12 is complete. Phase 11 is now the only remaining active productization track:

- **Phase 11** expands the external surface so `roomci` becomes integration-ready for real company evaluations — adapter contract kit, protocol support matrix, customer PoC packs, second external protocol surface.
- **Next priority:** Phase 11 Task 12 MQTT retained-state run-boundary preservation, then the remaining Task 04 MQTT interoperability/replay scope.

Where Phase 12 landed first (release plumbing, runtime safety), Phase 11 benefits because every Phase 11 deliverable is shipped on a more credible runtime and a more honest public surface.
