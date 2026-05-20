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
| Phase 13 | `todo` | Official-spec-backed protocol conformance subsets for MQTT and Modbus first, with future protocol profiles tracked honestly | Full certification-grade conformance remains out of scope | `phase13/phase_status.md` |
| Phase 14 | `done` | Hospitality smart-home QA core coverage: local MQTT, edge, device protocols, network/control faults, BMS/ops, comfort, and safe access/intercom boundaries | NOT A HOTEL private compatibility and full-stack emulation remain out of scope | `phase14/phase_status.md` |

## Update Rules

- Update this file when a phase changes status.
- Update the matching `phase_status.md` when a task or quality gate changes status.
- Use `review` when implementation exists but verification or acceptance criteria remain incomplete.
- Use `blocked` only with a concrete blocker.

## Current Recommendation

Phase 10 turned the contract emulator into a pre-adoption PoC product: a localhost-bound service that external clients can connect to, drive, observe, and collect CI-ready reports from without requiring private hospitality evaluator protocol details.

Phase 12 is complete. Phase 11 is the remaining active productization track, while Phase 13 and Phase 14 define the next two quality bars:

- **Phase 11** expands the external surface so `roomci` becomes integration-ready for real company evaluations — adapter contract kit, protocol support matrix, customer PoC packs, second external protocol surface.
- **Next priority:** close the remaining Phase 11 Task 04 MQTT interoperability/replay scope, then move into Phase 13's protocol conformance track.
- **Phase 13** should then turn selected wire endpoints into official-spec-backed conformance subsets, starting with MQTT 3.1.1 and Modbus TCP.
- **Phase 14** should keep the product from becoming a broad stack clone by defining the hospitality-like core QA journey and explicit out-of-scope boundaries.

Where Phase 12 landed first (release plumbing, runtime safety), Phase 11 benefits because every Phase 11 deliverable is shipped on a more credible runtime and a more honest public surface.
