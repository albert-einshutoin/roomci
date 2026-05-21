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
| Phase 15 | `done` | Evaluator friction removal: retained subscribe loop, practical Modbus depth, reproducible Docker smoke, serve/core/scenario maintainability, and protocol evidence automation | None | `phase15/phase_status.md` |
| Phase 16 | `done` | Roadmap triage for optional depth with promote/defer/non-goal decisions recorded | None | `phase16/phase_status.md` |
| Phase 17 | `done` | Promoted contract depth: safe intercom/relay mock, network/control-panel profiles, BMS hardening, comfort time-series, adapter samples | None | `phase17/phase_status.md` |
| Phase 18 | `done` | Release-candidate sample and evidence hardening after Phase 17 | None | `phase18/phase_status.md` |
| Phase 19 | `done` | S Tier observability and CI evidence completion | None | `phase19/phase_status.md` |
| Phase 20 | `done` | A Tier developer experience completion for Python SDK, scenario debugger, and adoption workflow docs | VSCode assets deferred to Phase 22 | `phase20/phase_status.md` |
| Phase 21 | `done` | B Tier protocol profile completion: Matter, BACnet, KNX, and OPC UA contract profiles | Real customer maps needed before any profile is promoted beyond contract_profile | `phase21/phase_status.md` |
| Phase 22 | `todo` | Deferred VSCode editor authoring assets | None | `phase22/phase_status.md` |

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

The final self-review created Phase 15. Phases 15, 16, 17, and 18 are now complete. The current product is best treated as a release-candidate pre-adoption PoC surface, not a production controller or vendor emulator.

- **Phase 15** hardened the release-candidate surface without expanding into a full smart-home stack emulator.
- **Phase 16** recorded previously implied roadmap items and forced explicit promote/defer/reject decisions before implementation breadth expands.
- **Phase 17** implemented the promoted contract-depth work.
- **Phase 18** made adapter samples, Phase 17 evidence, and public scorecards mechanically checkable.
- **Phase 19** closed the remaining S Tier gaps around GitHub Actions parity,
  timeline exports, trace metadata, observability artifacts, and evaluator CI
  documentation.
- **Phase 20** completed the A Tier developer-experience surface for Python
  automation and scenario debugging without reading Rust internals.
- **Phase 21** completed the B Tier protocol-profile surface for Matter,
  BACnet, KNX, and OPC UA as scoped contract profiles, not full protocol
  implementations.
- **Phase 22** tracks the intentionally deferred VSCode editor authoring
  assets.

See `tasks/backlog_inventory.md` for the current split between implemented work, already taskified implementation work, newly taskified roadmap work, and intentional non-goals.

See `tasks/strategy_alignment.md` for the current strategy mapped to task status.

See `tasks/product_goal.md` for the current product goal and tier completion model.
