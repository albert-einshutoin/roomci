# Phase 10 Status — Pre-adoption PoC Productization

## Phase Status

`done`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_serve_runtime_task.md` | `done` | Codex | `cargo test -p roomci-cli --tests`; `serve_starts_http_runtime_and_exposes_reports` starts `roomci serve --port 0` and calls HTTP endpoints | Replaced placeholder service mode with a localhost runtime; `--check` still exits |
| `02_http_control_report_api_task.md` | `done` | Codex | HTTP endpoints implemented for health, scenario, state, timeline, fault, run/finish, and latest reports | Uses JSON/Markdown/JUnit renderers from existing report model |
| `03_external_mqtt_poc_task.md` | `done` | Codex | `external_mqtt_publish_updates_retained_state_through_serve`; `--mqtt-port` MQTT 3.1.1 CONNECT + QoS0 PUBLISH subset | External client can publish retained-state updates through a real MQTT-shaped TCP endpoint |
| `04_connection_contract_config_task.md` | `done` | Codex | `mqtt.contracts` schema; validation rejects unsupported adapters, missing/ambiguous mappings, and unsupported extraction strategy | Topic mappings, device-id extraction, and required payload fields are configurable |
| `05_external_controller_e2e_task.md` | `done` | Codex | `external_http_controller_script_drives_serve_black_box`; `make compose-poc`; `make verify` | Added separate HTTP controller script and Docker Compose E2E |
| `06_poc_product_docs_task.md` | `done` | Codex | `README.md`; `docs/MQTT_SERVE_SUBSET.md`; `docs/PRE_ADOPTION_POC_CHECKLIST.md` | Documents scenario vs serve modes, protocol boundaries, required vendor inputs, and NOT A HOTEL compatibility boundary |

## Blockers

- None for the generic PoC surface.
- Vendor-specific compatibility remains blocked until a real integration contract is provided.

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Serve runtime starts endpoints | `done` | `serve_starts_http_runtime_and_exposes_reports` integration test |
| Config-check remains non-blocking | `done` | `serve_check_validates_config_without_blocking` integration test |
| HTTP control/report API works | `done` | `/health`, `/finish`, and `/reports/latest.md` covered by integration test; other routes share same router |
| External MQTT/client-driven scenario works | `done` | `external_mqtt_publish_updates_retained_state_through_serve` |
| Docker Compose black-box E2E works | `done` | `make compose-poc` and `make verify` build `roomci-serve` and `external-controller` services |
| Reports generated from external interactions | `done` | Controller writes `external_controller_latest.{json,md,xml}` under `reports/` |
| Docs avoid protocol overclaiming | `done` | README and MQTT subset docs state this is not a production broker or private-system compatibility claim |
| Verification | `done` | `make verify` passes with 84 tests and 84.47% line coverage |

## Current Recommendation

Phase 10 is complete. Remaining compatibility work is product expansion beyond this phase: MQTT subscriber replay, QoS1/QoS2 wire behavior, TLS/auth, and vendor-specific adapters once real contracts are supplied.
