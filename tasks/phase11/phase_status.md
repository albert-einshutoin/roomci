# Phase 11 Status — Integration-ready Emulator Platform

## Phase Status

`todo`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_ci_release_truth_task.md` | `done` | Codex | `.github/workflows/smart-home-ci.yml`; `Makefile`; `docs/RELEASE_CHECKLIST.md`; README quality gates; `make verify` | Public CI/release claims now point to real workflow, local gate, ignored artifacts, and release checklist |
| `02_protocol_support_matrix_task.md` | `done` | Codex | `docs/PROTOCOL_SUPPORT_MATRIX.md`; `docs/README.md`; README link | Behavior model, serve endpoint, external-client-tested, conformance-subset, and unsupported levels are explicit per domain |
| `03_adapter_contract_kit_task.md` | `done` | Codex | `adapter-contracts/templates/company_adapter_contract.yaml`; `adapter-contracts/examples/*.yaml`; `docs/ADAPTER_CONTRACT_KIT.md`; `roomci adapter validate`; `validates_adapter_contract_examples`; `adapter_validate_accepts_shipped_contracts` | Companies can fill MQTT, Modbus, BMS, edge, device, auth, and acceptance details and validate them without runtime code edits |
| `04_external_protocol_depth_task.md` | `todo` | Unassigned | None yet | Add standard MQTT client interoperability and at least one non-MQTT external endpoint |
| `05_customer_poc_packs_task.md` | `done` | Codex | `poc-packs/*.md`; `make poc-generic-mqtt`; `make poc-hospitality`; `make poc-building-automation`; `make poc-bms-ops`; README links | Four one-command PoC packs now identify scenarios, adapter contracts, reports, acceptance checks, and customer-specific replacement inputs |
| `06_developer_experience_task.md` | `todo` | Unassigned | None yet | Make clean-checkout integration onboarding short and product-like |
| `07_category_positioning_task.md` | `todo` | Unassigned | None yet | Position roomci as a first-choice IoT/SmartHome emulator candidate without overclaiming |
| `08_notahotel_evaluation_path_task.md` | `done` | Codex | `docs/NOT_A_HOTEL_EVALUATOR_GUIDE.md`; `adapter-contracts/examples/hospitality_local_first_room.yaml`; `docs/README.md` | NOT A HOTEL path now states current demos, required private inputs, adapter field mapping, non-claims, and scoring rubric |
| `09_dual_track_positioning_task.md` | `done` | Codex | `docs/DUAL_TRACK_POSITIONING.md`; `docs/PRODUCT_POSITIONING.md`; `docs/README.md` | Industry track and hospitality track are explicit, with bounded copy candidates and overclaim avoidance |
| `10_evaluation_evidence_pack_task.md` | `done` | Codex | `docs/EVALUATION_EVIDENCE_PACK.md`; `make verify`; PoC pack targets; protocol docs links | Evidence pack lists commands, expected artifacts, scorecards, unsupported features, and next adoption work |
| `11_adoption_maximization_review_task.md` | `done` | Codex | `adoption_maximization_review.md`; `make compose-poc`; badge URL check; serve-runtime grep checks | Review scored current evaluator readiness and kept Phase 11 open with concrete blockers |

## Blockers

- Real NOT A HOTEL compatibility remains blocked until actual MQTT topics, payload schemas, Modbus/register maps, BMS/webhook contracts, auth model, and acceptance criteria are supplied.
- This blocker should not block the generic product. It should be handled by adapter contracts and PoC packs.

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Public CI/release claims match repository | `done` | `.github/workflows/smart-home-ci.yml`; `make verify`; `docs/RELEASE_CHECKLIST.md`; README quality gates |
| Protocol support matrix exists | `done` | `docs/PROTOCOL_SUPPORT_MATRIX.md` |
| Adapter contract templates validate | `done` | `cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml`; `cargo test --workspace --all-targets`; unit and CLI tests |
| Standard MQTT client interoperability tested | `todo` | None yet |
| Second external protocol endpoint works | `todo` | None yet |
| Customer PoC packs run from clean checkout | `done` | PoC pack Make targets generate ignored `reports/poc_*` artifacts |
| NOT A HOTEL evaluator checklist exists | `done` | `docs/NOT_A_HOTEL_EVALUATOR_GUIDE.md` |
| Generic IoT/SmartHome evaluator checklist exists | `todo` | Covered partially by protocol matrix and adapter kit; dedicated checklist still pending |
| Dual-track positioning is documented | `done` | `docs/DUAL_TRACK_POSITIONING.md` |
| Evaluation evidence pack exists | `done` | `docs/EVALUATION_EVIDENCE_PACK.md` |
| Adoption maximization review completed | `done` | `adoption_maximization_review.md` |

## Current Recommendation

Tasks 01, 02, 03, 05, 08, 09, and 10 are complete. Next, implement Task 06 developer experience and Task 07 category positioning, then continue to external protocol-depth work.
