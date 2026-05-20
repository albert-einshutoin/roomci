# Phase 14 Status — Hospitality Smart Home QA Core Coverage

## Phase Status

`todo`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_stack_coverage_map_task.md` | `todo` | Codex | None yet | Classify reported hospitality stack into cover now, cover next, mock/contract only, future profile, and out of scope |
| `02_core_qa_journey_task.md` | `todo` | Codex | Existing scenarios are separate proof points | Create or document an end-to-end journey connecting MQTT, edge, device protocol, fault, BMS/ops, and report evidence |
| `03_control_panel_fault_profiles_task.md` | `todo` | Codex | Existing fault docs mention control-panel faults | Add 24V UPS, redundant power, breaker isolation, and edge-computer failover as QA fault profiles, not electrical safety claims |
| `04_bms_ops_contract_depth_task.md` | `todo` | Codex | Existing BMS scenario and `/external/bms/contact` endpoint | Strengthen BMS/ops contracts around safety alerts, runbooks, phone/slack evidence, ticket state, and recovery |
| `05_comfort_sensor_hvac_profile_task.md` | `todo` | Codex | Existing comfort automation scenario | Expand comfort profile around occupied-zone sensors, ceiling-zone sensors, humidity, discomfort index, and override behavior |
| `06_access_intercom_boundary_task.md` | `todo` | Codex | Existing access drift scenario and future intercom docs | Define DoorBird/ONVIF/SIP/DTMF/UniFi access as contract/mock boundaries, not production access-control emulation |
| `07_hospitality_core_readiness_review_task.md` | `todo` | Codex | None yet | Review whether the product reads as a focused QA contract emulator rather than an unfocused stack clone |

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Stack coverage map exists | `todo` | None yet |
| Core QA journey is runnable or clearly documented | `todo` | Existing scenarios cover pieces but not one explicit journey |
| Control-panel fault profiles are scoped safely | `todo` | Existing docs mention control-panel faults but no coverage tier |
| BMS/ops contract depth is explicit | `todo` | Existing BMS mock and endpoint need product-level coverage mapping |
| Comfort/HVAC profile is tied to sensor assumptions | `todo` | Existing comfort scenario covers baseline behavior |
| Access/intercom boundary is explicit | `todo` | Access drift exists; DoorBird/SIP/DTMF/ONVIF boundary needs clearer coverage |
| Out-of-scope stack areas are documented | `todo` | Some docs say non-goals, but not mapped to the reported stack |

## Current Recommendation

Start with Task 01 before adding more implementation. Without a coverage map, the product can drift into chasing a long technology list. The strongest target is the core QA journey: local MQTT plus edge plus device protocol behavior plus network/control fault plus BMS/ops evidence.
