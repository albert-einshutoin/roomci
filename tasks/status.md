# roomci Task Status

## Phase Status

| Phase | Status | Current Focus | Blocking Issue | Status File |
|---|---|---|---|---|
| Phase 0 | `done` | Latest scenario contract, CLI, reports | None | `phase0/phase_status.md` |
| Phase 1 | `todo` | Local MQTT retained-state model | None | `phase1/phase_status.md` |
| Phase 2 | `todo` | Edge server emulator | Phase 1 not complete | `phase2/phase_status.md` |
| Phase 3 | `todo` | Modbus, DALI-like, contact I/O mocks | Phase 0/1 not complete | `phase3/phase_status.md` |
| Phase 4 | `todo` | BMS/ops alert mock | Contact I/O mock not complete | `phase4/phase_status.md` |
| Phase 5 | `todo` | Docker and CI packaging | Runtime behavior not complete | `phase5/phase_status.md` |
| Phase 6 | `todo` | Reliability depth and future integrations | MVP demos not complete | `phase6/phase_status.md` |

## Update Rules

- Update this file when a phase changes status.
- Update the matching `phase_status.md` when a task or quality gate changes status.
- Use `review` when implementation exists but verification or acceptance criteria remain incomplete.
- Use `blocked` only with a concrete blocker.

## Current Recommendation

Start Phase 1 by expanding the in-memory local MQTT retained-state model into the first-class MQTT module described in `docs/04_local_first_mqtt_architecture.md`.
