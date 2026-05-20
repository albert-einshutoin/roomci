# Phase 11 Status — Integration-ready Emulator Platform

## Phase Status

`done`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_ci_release_truth_task.md` | `done` | Codex | `.github/workflows/smart-home-ci.yml`; `Makefile`; `docs/RELEASE_CHECKLIST.md`; README quality gates; `make verify` | Public CI/release claims now point to real workflow, local gate, ignored artifacts, and release checklist |
| `02_protocol_support_matrix_task.md` | `done` | Codex | `docs/PROTOCOL_SUPPORT_MATRIX.md`; `docs/README.md`; README link | Behavior model, serve endpoint, external-client-tested, conformance-subset, and unsupported levels are explicit per domain |
| `03_adapter_contract_kit_task.md` | `done` | Codex | `adapter-contracts/templates/company_adapter_contract.yaml`; `adapter-contracts/examples/*.yaml`; `docs/ADAPTER_CONTRACT_KIT.md`; `roomci adapter validate`; `validates_adapter_contract_examples`; `adapter_validate_accepts_shipped_contracts` | Companies can fill MQTT, Modbus, BMS, edge, device, auth, and acceptance details and validate them without runtime code edits |
| `04_external_protocol_depth_task.md` | `done` | Codex | `POST /external/bms/contact`; `external_bms_contact_*`; `external_events_survive_run_boundary`; `standard_mqtt_client_publishes_retained_state_through_serve`; `make protocol-smoke`; `docs/EXTERNAL_PROTOCOL_DEPTH.md`; `docs/PROTOCOL_CONFORMANCE_REGISTRY.md` | BMS/contact endpoint, MQTT standard-client interoperability, external MQTT run-boundary state, and protocol smoke are covered; retained subscriber replay remains an explicit unsupported MQTT boundary |
| `05_customer_poc_packs_task.md` | `done` | Codex | `poc-packs/*.md`; `make poc-generic-mqtt`; `make poc-hospitality`; `make poc-building-automation`; `make poc-bms-ops`; README links | Four one-command PoC packs now identify scenarios, adapter contracts, reports, acceptance checks, and customer-specific replacement inputs |
| `06_developer_experience_task.md` | `done` | Codex | `docs/INTEGRATION_ONBOARDING.md`; HTTP API table; troubleshooting table; client snippets; README link | New evaluators can pick a PoC pack, validate an adapter, start serve mode, drive HTTP/MQTT, and collect reports without reading Rust code |
| `07_category_positioning_task.md` | `done` | Codex | `docs/CATEGORY_READINESS.md`; README link; docs index | Category comparison now frames where roomci wins and does not win against real-device staging, brokers, mocks, Home Assistant, cloud emulators, and HIL |
| `08_notahotel_evaluation_path_task.md` | `done` | Codex | `docs/NOT_A_HOTEL_EVALUATOR_GUIDE.md`; `adapter-contracts/examples/hospitality_local_first_room.yaml`; `docs/README.md` | hospitality path now states current demos, required private inputs, adapter field mapping, non-claims, and scoring rubric |
| `09_dual_track_positioning_task.md` | `done` | Codex | `docs/DUAL_TRACK_POSITIONING.md`; `docs/PRODUCT_POSITIONING.md`; `docs/README.md` | Industry track and hospitality track are explicit, with bounded copy candidates and overclaim avoidance |
| `10_evaluation_evidence_pack_task.md` | `done` | Codex | `docs/EVALUATION_EVIDENCE_PACK.md`; `make verify`; PoC pack targets; protocol docs links | Evidence pack lists commands, expected artifacts, scorecards, unsupported features, and next adoption work |
| `11_adoption_maximization_review_task.md` | `done` | Codex | `adoption_maximization_review.md`; `make compose-poc`; badge URL check; serve-runtime grep checks | Review scored current evaluator readiness and kept Phase 11 open with concrete blockers |
| `12_mqtt_retained_run_boundary_task.md` | `done` | Codex | `external_mqtt_final_state` and `external_mqtt_retained_state` overlays; `drain_external_overlay_into` merges MQTT state; `rendered_report_view` and `/state` include preserved state; tests `external_mqtt_retained_state_is_visible_in_state_before_run`, `external_mqtt_retained_state_survives_run_boundary`, and CLI black-box `external_mqtt_publish_updates_retained_state_through_serve` | MQTT retained messages published by external clients are visible immediately through `/state` and survive `/run`, matching BMS/contact evidence model |

## Blockers

- Real hospitality compatibility remains blocked until actual MQTT topics, payload schemas, Modbus/register maps, BMS/webhook contracts, auth model, and acceptance criteria are supplied.
- This blocker should not block the generic product. It should be handled by adapter contracts and PoC packs.

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Public CI/release claims match repository | `done` | `.github/workflows/smart-home-ci.yml`; `make verify`; `docs/RELEASE_CHECKLIST.md`; README quality gates |
| Protocol support matrix exists | `done` | `docs/PROTOCOL_SUPPORT_MATRIX.md` |
| Adapter contract templates validate | `done` | `cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml`; `cargo test --workspace --all-targets`; unit and CLI tests |
| Standard MQTT client interoperability tested | `done` | `standard_mqtt_client_publishes_retained_state_through_serve`; `make protocol-smoke-mqtt`; Compose protocol smoke uses paho-mqtt |
| External MQTT retained state survives `/run` | `done` | `external_mqtt_final_state` and `external_mqtt_retained_state` overlays; tests assert `/state` before `/run`, `/state` after `/run`, `/reports/latest.json`, and CLI black-box publish behavior |
| Second external protocol endpoint works | `done` | `POST /external/bms/contact`; `external_bms_contact_updates_state_and_timeline`; `external_bms_contact_rejects_invalid_payloads`; `external_bms_contact_sanitizes_source_and_message`; `external_events_survive_run_boundary`; BMS webhook PoC controller script asserts each step |
| Customer PoC packs run from clean checkout | `done` | PoC pack Make targets generate ignored `reports/poc_*` artifacts |
| hospitality evaluator checklist exists | `done` | `docs/NOT_A_HOTEL_EVALUATOR_GUIDE.md` |
| Generic IoT/SmartHome evaluator checklist exists | `done` | `docs/GENERIC_SMARTHOME_EVALUATOR_CHECKLIST.md` |
| Dual-track positioning is documented | `done` | `docs/DUAL_TRACK_POSITIONING.md` |
| Evaluation evidence pack exists | `done` | `docs/EVALUATION_EVIDENCE_PACK.md` |
| Adoption maximization review completed | `done` | `adoption_maximization_review.md` |

## Current Recommendation

Phase 11 is complete. Retained subscriber replay is intentionally not implemented and is tracked as an unsupported MQTT boundary in the conformance registry rather than a blocker for integration-ready evaluation.
