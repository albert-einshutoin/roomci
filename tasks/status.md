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
| Phase 11 | `done` | External protocol depth after completed PoC/evidence/onboarding/positioning work | Real customer/vendor specs are required only for compatibility claims, not for generic platform progress | `phase11/phase_status.md` |
| Phase 12 | `done` | Serve runtime hardening, health semantics, MQTT CONNECT validation, and release plumbing | None | `phase12/phase_status.md` |
| Phase 13 | `done` | Official-spec-backed protocol conformance subsets for MQTT and Modbus first, with future protocol profiles tracked honestly | Full certification-grade conformance remains out of scope | `phase13/phase_status.md` |
| Phase 14 | `done` | Hospitality smart-home QA core coverage: local MQTT, edge, device protocols, network/control faults, BMS/ops, comfort, and safe access/intercom boundaries | NOT A HOTEL private compatibility and full-stack emulation remain out of scope | `phase14/phase_status.md` |
| Phase 15 | `todo` | Evaluator friction removal: retained subscribe loop, practical Modbus depth, reproducible Docker smoke, serve maintainability, and protocol evidence automation | None | `phase15/phase_status.md` |

## Update Rules

- Update this file when a phase changes status.
- Update the matching `phase_status.md` when a task or quality gate changes status.
- Use `review` when implementation exists but verification or acceptance criteria remain incomplete.
- Use `blocked` only with a concrete blocker.

## Current Recommendation

Phase 10 turned the contract emulator into a pre-adoption PoC product: a localhost-bound service that external clients can connect to, drive, observe, and collect CI-ready reports from without requiring private hospitality evaluator protocol details.

Phases 11-14 are complete. The product now has the integration surface, runtime hardening, protocol subset evidence, and hospitality-like core QA boundary needed for a serious pre-adoption evaluation.

- **Phase 11** completed the integration-ready evaluator surface.
- **Phase 13** completed official-spec-backed MQTT and Modbus conformance subsets.
- **Phase 14** completed the hospitality-like core QA boundary and full-stack non-goals.

The final self-review created Phase 15. Next work should remove evaluator friction before adding more protocol breadth.

- **Phase 15** should harden the current release-candidate surface rather than expanding into a full smart-home stack emulator.
