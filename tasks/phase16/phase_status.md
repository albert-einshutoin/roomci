# Phase 16 Status: Roadmap Triage For Optional Depth

Status: `todo`

## Tasks

| Task | Status | Notes |
|---|---|---|
| 01 Future protocol selection | `todo` | Rank BACnet, KNX, Matter, OPC UA, Zigbee, and Thread for possible future subsets. |
| 02 Intercom and relay profile | `todo` | Decide the contract/mock depth for PIN, relay pulse, staff call, SIP/DTMF-like flows, and failure modes. |
| 03 Network/control-panel fault profiles | `todo` | Split VLAN/firewall/segment and 24V/UPS/circuit-protector faults into executable QA profile candidates. |
| 04 BMS contract hardening | `todo` | Decide which webhook hardening features belong in adapter contracts versus serve runtime. |
| 05 Comfort time-series profile | `todo` | Decide whether real sensor-zone replay belongs in core or a domain pack. |
| 06 Adapter SDK samples | `todo` | Decide whether Go backend samples and Lua-like automation hooks should ship as examples, adapters, or docs only. |
| 07 Observability export profile | `todo` | Decide whether Influx/Grafana-friendly exports add enough evaluator value to implement. |

## Current Blocker

Phase 15 should run first. Phase 16 is planning/selection work for optional depth.
