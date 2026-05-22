# Backlog Inventory

This file separates what is already implemented, what is already taskified, and what was only implied by docs until Phase 16.

## Implemented And Taskified

These are implemented and tracked as `done` in phase status files:

| Area | Status | Evidence |
|---|---|---|
| Scenario runner, validation, JSON/Markdown/JUnit reports | `done` | Phase 0 |
| Local MQTT retained-state behavior in scenario mode | `done` | Phase 1 |
| Edge routing and edge failover | `done` | Phase 2 |
| Modbus scenario model, DALI-like lighting model, contact I/O model | `done` | Phase 3 |
| BMS/ops alert model with Slack-like, phone-like, ticket, and runbook evidence | `done` | Phase 4 |
| Docker image, Compose smoke, CI-oriented verification | `done` | Phase 5 |
| WAN failover, comfort automation, access drift, commissioning checklist | `done` | Phase 6 |
| Docs, tests, CLI, coverage, and release-readiness hardening | `done` | Phases 7-8 |
| Generic MQTT / edge / device QA positioning | `done` | Phase 9 |
| `roomci serve` HTTP PoC surface and external controller E2E | `done` | Phase 10 |
| Adapter contracts, PoC packs, evaluator docs, evidence pack, positioning review | `done` | Phase 11 |
| Serve runtime hardening and release metadata | `done` | Phase 12 |
| MQTT 3.1.1 and Modbus TCP conformance subsets with Docker protocol smoke | `done` | Phase 13 |
| Hospitality smart-home QA core coverage boundary | `done` | Phase 14 |
| Evaluator friction removal: MQTT retained subscribe, practical Modbus depth, protocol evidence automation, and source module extraction | `done` | Phase 15 |
| Roadmap triage for future protocol and optional-depth items | `done` | Phase 16 |
| Promoted contract depth: intercom/relay safe mock, infrastructure fault profiles, BMS hardening, comfort time-series, adapter samples | `done` | Phase 17 |
| Release-candidate evidence hardening | `done` | Phase 18 |
| S Tier observability and CI evidence completion | `done` | Phase 19 |
| A Tier Python and debugger developer experience | `done` | Phase 20 |
| B Tier protocol profiles for Matter, BACnet, KNX, and OPC UA | `done` | Phase 21 |
| Local VSCode authoring assets | `done` | Phase 22 |

## Already Taskified For Implementation

These were implemented in Phase 15 and remain here as traceability links:

| Area | Task |
|---|---|
| MQTT retained subscribe/replay path | `tasks/phase15/01_mqtt_retained_subscribe_loop_task.md` |
| Modbus practical subset beyond one-register operations | `tasks/phase15/02_modbus_practical_subset_task.md` |
| Reproducible protocol smoke image | `tasks/phase15/03_docker_protocol_smoke_image_task.md` |
| `roomci-serve` protocol module extraction | `tasks/phase15/04_serve_protocol_module_extraction_task.md` |
| Machine-checkable evidence for protocol claims | `tasks/phase15/05_protocol_evidence_automation_task.md` |
| `roomci-core` module extraction for 800-line maintainability target | `tasks/phase15/06_core_module_extraction_task.md` |
| `roomci-scenario` module extraction for 800-line maintainability target | `tasks/phase15/07_scenario_module_extraction_task.md` |

## Newly Taskified From Implied Docs

These were implied by roadmap/docs and were decided in Phase 16:

| Area | Task |
|---|---|
| Future standards selection for BACnet, KNX, Matter, OPC UA, Zigbee, Thread | `tasks/phase16/01_future_protocol_selection_task.md` |
| Intercom, relay, PIN, staff-call, SIP/DTMF-like mock boundaries | `tasks/phase16/02_intercom_relay_profile_task.md` |
| Network segmentation and control-panel fault profiles | `tasks/phase16/03_network_control_panel_fault_profiles_task.md` |
| BMS webhook hardening boundaries such as HMAC, replay, schema versioning, severity enum, Content-Type | `tasks/phase16/04_bms_contract_hardening_task.md` |
| Comfort sensor-zone and time-series replay profile | `tasks/phase16/05_comfort_timeseries_profile_task.md` |
| SDK/sample integration path for Go backends and Lua-like automation hooks | `tasks/phase16/06_adapter_sdk_samples_task.md` |
| Observability export profile such as Influx line protocol or Grafana-friendly artifacts | `tasks/phase16/07_observability_export_profile_task.md` |

## Promoted And Implemented In Phase 17

These are the Phase 16 items promoted into concrete implementation tasks and completed in Phase 17:

| Area | Task |
|---|---|
| Safe intercom and relay mock profile | `tasks/phase17/01_intercom_relay_safe_mock_task.md` |
| Network and control-panel fault profiles | `tasks/phase17/02_network_control_panel_fault_profiles_task.md` |
| BMS contract hardening | `tasks/phase17/03_bms_contract_hardening_task.md` |
| Comfort time-series domain pack | `tasks/phase17/04_comfort_timeseries_domain_pack_task.md` |
| Adapter SDK samples | `tasks/phase17/05_adapter_sdk_samples_task.md` |

## Newly Taskified After Phase 17 Self-Review

| Area | Task |
|---|---|
| Adapter sample compile/run verification | `tasks/phase18/01_adapter_sample_ci_task.md` |
| Phase 17 evidence registry expansion | `tasks/phase18/02_phase17_evidence_registry_task.md` |
| Domain-pack release scorecard refresh | `tasks/phase18/03_release_scorecard_refresh_task.md` |

## Promoted And Implemented In Phase 19

| Area | Task |
|---|---|
| GitHub Actions parity for S Tier gates | `tasks/phase19/01_github_actions_parity_task.md` |
| Stable timeline export contract | `tasks/phase19/02_stable_timeline_export_contract_task.md` |
| Trace metadata and run correlation | `tasks/phase19/03_trace_metadata_and_run_correlation_task.md` |
| Observability artifact profile | `tasks/phase19/04_observability_artifact_profile_task.md` |
| Evaluator CI documentation pack | `tasks/phase19/05_evaluator_ci_documentation_pack_task.md` |

## Newly Taskified For A Tier Completion

| Area | Task |
|---|---|
| Python SDK package and smoke | `tasks/phase20/01_python_sdk_package_task.md` |
| Scenario debugger explain command | `tasks/phase20/02_scenario_debugger_explain_task.md` |
| Developer workflow docs | `tasks/phase20/04_developer_workflow_docs_task.md` |
| A Tier release gate | `tasks/phase20/05_a_tier_release_gate_task.md` |

## Deferred From Phase 20 And Implemented In Phase 22

| Area | Task |
|---|---|
| VSCode extension assets | `tasks/phase22/01_vscode_extension_assets_task.md` |

## Newly Taskified For B Tier Completion

| Area | Task |
|---|---|
| Matter prototype profile | `tasks/phase21/01_matter_prototype_profile_task.md` |
| BACnet contract profile | `tasks/phase21/02_bacnet_contract_profile_task.md` |
| KNX contract profile | `tasks/phase21/03_knx_contract_profile_task.md` |
| OPC UA contract profile | `tasks/phase21/04_opcua_contract_profile_task.md` |
| B Tier evidence and docs gate | `tasks/phase21/05_b_tier_evidence_docs_gate_task.md` |

These Phase 21 tasks are now implemented as `contract_profile` evidence, not
wire-protocol implementations.

## Intentionally Not Taskified As Implementation

These should stay documented non-goals unless a future product decision reverses them:

| Area | Reason |
|---|---|
| Full private hospitality-operator compatibility | Requires private topics, payloads, register maps, BMS contracts, auth/TLS assumptions, and acceptance criteria. |
| Production MQTT broker replacement | Outside the QA contract emulator scope. |
| Full Modbus RTU electrical/serial behavior | Requires serial timing and hardware-level validation. |
| Full BACnet, KNX, Matter, OPC UA, Zigbee, Thread certification | B Tier profiles may be taskified as contract profiles, but certification-grade protocol stacks remain outside the product. |
| Real lock authorization or physical access decisions | Safety-critical and outside local CI emulation. |
| Vendor platform emulation for cloud, phone, notification, observability, network, intercom, access-control, or device-management systems | These should remain contracts, mocks, or evidence outputs rather than vendor replacements. |
| CAD/BIM/construction tooling | Not a QA emulator surface. |
