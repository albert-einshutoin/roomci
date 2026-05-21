# Phase 17 Status: Promoted Contract Depth

Status: `done`

## Tasks

| Task | Status | Notes |
|---|---|---|
| 01 Intercom and relay safe mock | `done` | Scenario-only `intercom` step, safe relay evidence assertion, executable example, and docs added. |
| 02 Network and control-panel fault profiles | `done` | Added network/firewall/local-broker/control-panel fault profiles, BMS evidence assertion, example, and docs. |
| 03 BMS contract hardening | `done` | Adapter schema/content-type/severity/HMAC/replay validation plus serve severity and replay-id negative tests added. |
| 04 Comfort time-series domain pack | `done` | Added sensor-reading steps, comfort time-series assertion, zone final-state evidence, executable example, and docs. |
| 05 Adapter SDK samples | `done` | Added Go and TypeScript sample clients, Lua-like hook pseudocode, evaluator docs, and docs index links. |

## Current Blocker

None. Phase 17 implementation is complete. The remaining follow-up work is tracked in Phase 18.

## Verification

- `make verify` passed after Phase 17 implementation.
- New Phase 17 executable examples are included in `Makefile` scenario lists.
- Go/TypeScript adapter samples are documented, but Go tooling was not available in this local environment for sample compilation.
