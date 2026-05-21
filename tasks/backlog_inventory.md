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

## Intentionally Not Taskified As Implementation

These should stay documented non-goals unless a future product decision reverses them:

| Area | Reason |
|---|---|
| Full NOT A HOTEL private compatibility | Requires private topics, payloads, register maps, BMS contracts, auth/TLS assumptions, and acceptance criteria. |
| Production MQTT broker replacement | Outside the QA contract emulator scope. |
| Full Modbus RTU electrical/serial behavior | Requires serial timing and hardware-level validation. |
| Full BACnet, KNX, Matter, OPC UA, Zigbee, Thread certification | Certification-grade protocol stacks are not the current product. |
| Real lock authorization or physical access decisions | Safety-critical and outside local CI emulation. |
| Cloud platform emulation for AWS, GCP, Cloudflare, Twilio, Slack, Zoom, Grafana, InfluxDB, UniFi, DoorBird, YAMAHA, Jamf | These should remain contracts, mocks, or evidence outputs rather than vendor replacements. |
| CAD/BIM/construction tooling | Not a QA emulator surface. |
