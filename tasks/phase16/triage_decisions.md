# Phase 16 Triage Decisions

Phase 16 is a roadmap decision phase, not an implementation phase. These decisions use Phase 15 evidence: MQTT retained subscribe works, Modbus contiguous register reads work, protocol claims are machine-checked, and the large Rust files have been split below the maintainability target.

## Decisions

| Area | Decision | Reason | Next Task |
|---|---|---|---|
| BACnet/IP | `defer` | High evaluator value, but implementation and maintenance cost are too high without a concrete customer object/service subset. | Keep in registry as `future_profile`; require evaluator-supplied object list before promotion. |
| KNX | `adapter-contract only` | Relevant to hospitality/building automation, but ETS/group-address compatibility should not be guessed. | Add later only as group-address contract fixtures if real maps are supplied. |
| Matter / Zigbee / Thread | `non-goal for runtime` | Radio/fabric/commissioning behavior is outside a local contract emulator. | Keep as documented future/non-goal; use gateway payload contracts if needed. |
| OPC UA | `defer` | Potentially relevant to industrial/building automation, but not central to current smart-home QA journey. | Revisit after Modbus/BMS/control-panel profiles mature. |
| Intercom / relay | `promote safe mock` | Useful for hospitality QA, but real unlock authorization is safety-sensitive. | Phase 17 Task 01. |
| Network/control-panel faults | `promote scenario profiles` | Directly strengthens the core QA journey and NOT A HOTEL-like relevance. | Phase 17 Task 02. |
| BMS hardening | `promote contract/runtime boundary` | Evaluators will expect schema/severity/auth boundaries without real Slack/phone integrations. | Phase 17 Task 03. |
| Comfort time-series | `promote domain-pack profile` | Deterministic replay improves HVAC/sensor QA without hardware emulation. | Phase 17 Task 04. |
| Adapter SDK samples | `promote docs/examples` | Helps adoption without introducing another runtime. | Phase 17 Task 05. |
| Observability export | `defer to evidence export` | JSON/Markdown/JUnit remain primary; Grafana/Influx should not become a platform replacement. | Add only after a concrete evaluator asks for it. |

## Guardrails

- Do not implement full BACnet, KNX, Matter, OPC UA, Zigbee, or Thread stacks without a supplied narrow evaluator contract.
- Do not implement real unlock authorization, SIP/DTMF gateways, or vendor/cloud emulators.
- Promote only deterministic profiles that improve CI evidence and remain contract-first.
