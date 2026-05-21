# Phase 16 Status: Roadmap Triage For Optional Depth

Status: `done`

## Tasks

| Task | Status | Notes |
|---|---|---|
| 01 Future protocol selection | `done` | BACnet/IP and OPC UA deferred; KNX adapter-contract only; Matter/Zigbee/Thread non-goals for runtime without gateway contracts. |
| 02 Intercom and relay profile | `done` | Promoted safe scenario-only mock profile; real unlock authorization remains out of scope. |
| 03 Network/control-panel fault profiles | `done` | Promoted deterministic QA fault profiles for Phase 17. |
| 04 BMS contract hardening | `done` | Promoted adapter/runtime boundary hardening for Phase 17. |
| 05 Comfort time-series profile | `done` | Promoted deterministic domain-pack replay for Phase 17. |
| 06 Adapter SDK samples | `done` | Promoted small Go/TypeScript examples and docs-only Lua-like guidance for Phase 17. |
| 07 Observability export profile | `done` | Deferred; JSON/Markdown/JUnit remain primary unless a concrete evaluator needs Influx/Grafana export. |

## Current Blocker

None. Phase 16 is complete; promoted implementation work is tracked in Phase 17.
