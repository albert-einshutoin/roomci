# roomci documentation index

This directory is the canonical home for roomci design notes. Documents are
numbered so they read top-to-bottom as a small book, but each one stands on
its own.

## Orientation

| # | Document | What it covers |
|---|---|---|
| 00 | [Executive summary](00_executive_summary.md) | One-page overview of roomci, who it is for, and how it is used |
| Positioning | [Product positioning](PRODUCT_POSITIONING.md) | QA contract emulator category, audience, and scope boundaries |
| Domains | [Domain packs](DOMAIN_PACKS.md) | Core emulator modules and reusable domain packs |
| MQTT | [Generic MQTT contracts](GENERIC_MQTT_CONTRACTS.md) | Generic MQTT command/state examples and current supported subset |
| MQTT serve | [MQTT serve subset](MQTT_SERVE_SUBSET.md) | Minimal MQTT 3.1.1 CONNECT + QoS0 PUBLISH ingress for PoC tests |
| Modbus serve | [Modbus TCP subset](MODBUS_TCP_SUBSET.md) | Minimal Modbus TCP read/write endpoint for PoC tests |
| Protocols | [Protocol support matrix](PROTOCOL_SUPPORT_MATRIX.md) | Source of truth for behavior models, serve endpoints, tested external surfaces, and non-goals |
| Protocols | [Protocol conformance registry](PROTOCOL_CONFORMANCE_REGISTRY.md) | Official references, implemented subsets, black-box verification commands, and explicit rejections |
| Coverage | [Hospitality stack coverage](HOSPITALITY_STACK_COVERAGE.md) | What roomci covers, mocks, defers, or rejects from a hospitality smart-home stack |
| Journey | [Core QA journey](CORE_QA_JOURNEY.md) | End-to-end local MQTT, edge, device, fault, BMS, comfort, and evidence path |
| Adapters | [Adapter contract kit](ADAPTER_CONTRACT_KIT.md) | Templates, examples, and validation for company-specific protocol contracts |
| Evidence | [Evaluation evidence pack](EVALUATION_EVIDENCE_PACK.md) | Commands, PoC packs, scorecards, report artifacts, and unsupported-feature disclosure |
| Onboarding | [Integration onboarding](INTEGRATION_ONBOARDING.md) | 15-minute path, HTTP API, troubleshooting, and client snippets |
| Category | [Category readiness](CATEGORY_READINESS.md) | Comparison against real-device staging, brokers, mocks, Home Assistant, cloud emulators, and HIL |
| Positioning | [Dual-track positioning](DUAL_TRACK_POSITIONING.md) | Industry-wide product story plus hospitality domain-pack story |
| Principles | [Design principles](DESIGN_PRINCIPLES.md) | Product philosophy and scope boundaries |

## Product

| # | Document | What it covers |
|---|---|---|
| 02 | [Product requirements](02_product_requirements.md) | Personas, jobs-to-be-done, and success metrics for the QA emulator |
| 18 | [MVP roadmap](18_mvp_roadmap.md) | Phase plan from Phase 0 (contract) through Phase 7 (production readiness) |

## Architecture

| # | Document | What it covers |
|---|---|---|
| 03 | [Architecture](03_architecture.md) | Crate boundaries, runtime model, virtual-time clock |
| 04 | [Local-first MQTT architecture](04_local_first_mqtt_architecture.md) | Retained state, QoS1, reconnect, local vs cloud broker |
| 05 | [Edge server emulator](05_edge_server_emulator.md) | Primary/secondary redundancy and failover model |
| 06 | [Device model](06_device_model.md) | Per-device state, command capability matrix |
| 11 | [Network and failover](11_network_and_failover.md) | WAN failure, backup link activation, comfort-during-outage |

## Protocols and integrations

| # | Document | What it covers |
|---|---|---|
| 07 | [Building automation protocol strategy](07_building_automation_protocol_strategy.md) | Why Modbus/DALI/BACnet/KNX and how they fit together |
| 08 | [Modbus strategy](08_modbus_strategy.md) | Register-map shape, read-only enforcement, decimal types |
| 09 | [DALI lighting strategy](09_dali_lighting_strategy.md) | Scene targets, per-fixture levels, command-drop faults |
| 10 | [BMS operations emulation](10_bms_operations_emulation.md) | Alert pipeline, Slack/phone escalation, runbook URLs |
| 14 | [Intercom and access control](14_intercom_and_access_control.md) | Future Aiphone / access-control integration |

## Scenarios and faults

| # | Document | What it covers |
|---|---|---|
| 12 | [Control panel fault model](12_control_panel_fault_model.md) | iPad/controller faults the emulator can inject |
| 13 | [Comfort automation](13_comfort_automation.md) | Pre-arrival climate, comfort metric, automation contract |
| 15 | [Scenario spec](15_scenario_spec.md) | YAML format reference for `examples/*.yaml` |
| 16 | [Fault injection](16_fault_injection.md) | Catalog of faults, targeting rules, end conditions |

## Delivery

| # | Document | What it covers |
|---|---|---|
| 17 | [Docker / CI design](17_docker_ci_design.md) | Container layout and GitHub Actions integration |
| Serve | [HTTP serve MVP plan](HTTP_SERVE_MVP_PLAN.md) | Planned localhost-bound control/report API before MQTT-compatible serve mode |
| Serve | [HTTP serve behavior](HTTP_SERVE_BEHAVIOR.md) | Current HTTP connection, timeout, and overload behavior |
| Serve | [External protocol depth](EXTERNAL_PROTOCOL_DEPTH.md) | MQTT subset boundary and BMS/contact external endpoint |
| PoC | [Pre-adoption PoC checklist](PRE_ADOPTION_POC_CHECKLIST.md) | Integration checklist for external protocol contracts and acceptance criteria |
| PoC | [Generic SmartHome evaluator checklist](GENERIC_SMARTHOME_EVALUATOR_CHECKLIST.md) | Evaluation checklist for generic IoT, SmartHome, edge-device, and building-automation teams |
| Release | [Release checklist](RELEASE_CHECKLIST.md) | Reproducible gates for CI, Docker, Compose, reports, docs, and coverage |
| 20 | [Appendix: future integrations](20_appendix_future_integrations.md) | Out-of-scope work and where it would live |

For the API-level reference, run `cargo doc --no-deps --open` from the repo
root; every public type and function in the workspace is documented.
