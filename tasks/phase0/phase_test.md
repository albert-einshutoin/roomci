# Phase 0 Test Plan

## Quality Gates

- Parser tests cover latest scenario shape.
- CLI integration tests cover `--report-md` and `--report-json`.
- Runner tests cover local cloud outage with local MQTT unaffected.
- Report tests cover field-oriented failure details.
- Legacy example support is either tested or explicitly deprecated.

## Required Test Cases

1. `examples/local_first_cloud_outage.yaml` validates.
2. `mqtt.cloud` offline does not block local MQTT command handling.
3. MQTT retained assertion checks expected state payload.
4. `guest_experience: unaffected` passes when local command succeeds.
5. Report files are written with `--report-md`, `--report-json`, and `--junit`.

## Done Means

Phase 0 is done when the latest docs' v0.1 release command works locally without Docker.
