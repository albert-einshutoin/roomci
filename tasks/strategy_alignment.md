# Strategy Alignment

This file maps the current product strategy to task status.

## Things We Should Do

| Strategy Item | Task State | Where It Lives |
|---|---|---|
| Make the NOT A HOTEL-like core QA journey the center of the product demo | `done`, with selected depth still `todo` | Phase 14 defines the core journey; `make poc-core-qa` is the demo path. Control-panel fault depth is triaged in Phase 16 Task 03. |
| Local MQTT | `done`, with subscribe depth `todo` | Phase 1, Phase 10, Phase 13; Phase 15 Task 01 covers retained subscribe/replay. |
| Edge server | `done` | Phase 2 and Phase 14. |
| Modbus | `done`, with practical subset depth `todo` | Phase 3 scenario model and Phase 13 Modbus TCP subset; Phase 15 Task 02 expands the practical subset. |
| DALI-like | `done` as scenario model | Phase 3 and Phase 14. Full DALI wire behavior is not claimed. |
| Contact I/O | `done` as scenario/BMS input surface | Phase 3, Phase 4, Phase 11, Phase 14. |
| BMS alert | `done`, with webhook hardening decision `todo` | Phase 4 and Phase 11; Phase 16 Task 04 decides HMAC/replay/schema hardening. |
| Network failover | `done`, with segment/firewall depth `todo` | Phase 6 and Phase 14; Phase 16 Task 03 covers VLAN/firewall/segment profiles. |
| Comfort automation | `done`, with time-series depth `todo` | Phase 6 and Phase 14; Phase 16 Task 05 covers sensor-zone/time-series replay. |
| Control panel fault | `done` as scoped QA profile, executable depth `todo` | Phase 14 scoped this safely; Phase 16 Task 03 decides which profiles become executable. |
| Keep protocols official-spec-backed and subset-scoped | `done`, with next depth `todo` | Phase 13 registry and release checklist; Phase 15 Task 05 automates evidence. |
| MQTT 3.1.1 subset | `done`, with subscribe depth `todo` | Phase 13; Phase 15 Task 01. |
| Modbus TCP subset | `done`, with multi-register/coil decision `todo` | Phase 13; Phase 15 Task 02. |
| MQTT 5 unsupported boundary | `done` | Phase 13 and protocol conformance registry. |
| BACnet / OPC UA / KNX / Matter future profiles | `done` as registry rows, selection `todo` | Phase 13 registry; Phase 16 Task 01 ranks and promotes/defers future protocols. |
| Move company-specific differences into adapter contracts | `done`, with hardening decisions `todo` | Phase 11 adapter contract kit and onboarding docs; Phase 16 Task 04 covers BMS hardening boundaries. |
| MQTT topic | `done` | Adapter contracts and MQTT scenario contracts. |
| Payload schema | `done` at required-field level, deeper schema remains future hardening | Adapter contracts; Phase 16 Task 04 if webhook schema versioning is promoted. |
| Register map | `done` | Adapter contracts and Modbus model. |
| Alert severity | `done` as accepted/sanitized field, stricter enum decision `todo` | BMS/contact endpoint; Phase 16 Task 04. |
| Auth/TLS | `done` as explicit customer input / unsupported runtime boundary | Onboarding, release checklist, MQTT subset docs. |
| Acceptance criteria | `done` | Adapter contracts, PoC packs, evaluator docs. |
| Device identity | `done` | Adapter contracts and MQTT placeholder mapping. |
| Keep NOT A HOTEL-specific language inside the hospitality domain pack/evaluator path | `done` | Phase 8, Phase 9, Phase 11, Phase 14. Core positioning is Smart Home / Building Automation QA Contract Emulator. |
| Ship multiple domain packs | `done` | Generic MQTT edge, hospitality smart home, building automation, and BMS ops PoC packs exist in Phase 11. |
| Verify with standard clients and black-box Docker smoke | `done` | Phase 13 standard MQTT/Modbus smoke; Phase 15 Task 03 replaced runtime package install with a pinned protocol-smoke image build. |

## Things We Should Not Do

| Non-Goal | Task State | Where It Lives |
|---|---|---|
| Recreate the full NOT A HOTEL stack | `documented non-goal` | Phase 14 and backlog inventory. |
| Become AWS/GCP/Cloudflare/UniFi/DoorBird/Twilio vendor emulators | `documented non-goal` | Phase 14 and backlog inventory. |
| Implement BACnet/Matter/KNX/OPC UA all at once | `documented non-goal`; selection `todo` | Phase 13 future profiles; Phase 16 Task 01 forces ranking before implementation. |
| Claim full protocol conformance | `blocked by release rule` | Phase 13 registry and release checklist. |
| Enter physical safety or real lock-control territory | `documented non-goal` | Phase 14, Phase 16 Task 02, Phase 16 Task 03, and backlog inventory. |

## Current Reading

The strategy is taskified.

The implementation-complete layer is Phase 0-14. The release-candidate hardening layer is Phase 15. The optional breadth and future-profile decision layer is Phase 16. Anything outside those layers is either an explicit customer input or a documented non-goal.
