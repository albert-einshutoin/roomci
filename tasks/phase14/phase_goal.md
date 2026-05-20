# Phase 14 Goal — Hospitality Smart Home QA Core Coverage

## Goal

Make `roomci` explicitly target the useful middle of a hospitality-grade smart-home stack:

```txt
NOT A HOTEL の全スタック emulator ではなく、
NOT A HOTEL 的な Smart Home / Building Automation QA の中核フローを
再現できる contract emulator
```

This phase defines exactly what `roomci` should cover, what it should only mock, and what it should not attempt to emulate.

## Product Boundary

`roomci` should cover system behavior where software, edge control, device protocols, network failure, BMS operations, and field QA meet.

It should not cover every application, cloud service, vendor device, CAD tool, or physical construction workflow used by a real hospitality operator.

## Coverage Tiers

| Tier | Meaning | Examples |
|---|---|---|
| Core coverage | Should be executable in scenarios and/or `roomci serve` | Local MQTT, edge server, Modbus, DALI-like lighting, contact I/O, BMS alerts, WAN failover, comfort automation |
| Contract/mock coverage | Should be represented as adapter contracts, mocks, or fault profiles | SIP/DTMF, ONVIF, DoorBird, UniFi access, Slack/Zoom Phone, Cloud Run/AWS IoT feedback, UPS/control-panel health |
| Future profile | Should be documented as candidate future depth | BACnet, OPC UA, KNX, Matter, Thread, Zigbee, vendor gateways |
| Out of scope | Should not be implemented by `roomci` | PMS, full cloud platform emulation, CAD/Rebro/KiCad/Blender, real lock authorization, production BMS, physical electrical safety |

## In Scope

- Add a stack coverage map that classifies the reported hospitality technologies into:
  - cover now
  - cover next
  - mock/contract only
  - future profile
  - out of scope
- Define the "core QA journey" that an evaluator should be able to run:
  1. local controller publishes MQTT command
  2. edge server routes command while cloud is degraded
  3. device protocol model updates state
  4. network or control-panel fault is injected
  5. BMS/ops escalation is triggered when safety/comfort threshold is crossed
  6. report captures timeline, retained state, alerts, recovery, and field-facing evidence
- Add scenario/PoC pack gaps for:
  - local MQTT + edge + device + BMS end-to-end flow
  - control-panel / 24V UPS / breaker-isolation fault profiles
  - comfort sensor and HVAC auto-mode profile
  - access/intercom contract mock boundary
  - network/VLAN/failover evidence
- Keep every hospitality-specific claim framed as public-research-inspired or customer-supplied-contract-ready, not private compatibility.

## Out of Scope

- Emulating NOT A HOTEL private production systems.
- Implementing PMS, booking, owner app, or full check-in systems.
- Replacing Google Cloud, AWS IoT, Cloudflare, UniFi, DoorBird, Twilio, Zoom Phone, Slack, Grafana, InfluxDB, or Jamf.
- Physical electrical safety validation for 24V power, UPS, breakers, high/low voltage circuits, or lightning protection.
- Real access control or lock authorization decisions.
- CAD/BIM/design-tool workflows such as Rebro, UniFi Design Center, KiCad, Blender, or 3D printing.

## Exit Criteria

- A coverage map states which reported hospitality technologies `roomci` should cover, mock, defer, or reject.
- At least one end-to-end hospitality-like QA journey connects MQTT, edge, device protocol behavior, network/control fault, BMS/ops escalation, and report evidence.
- Docs clearly explain why `roomci` is not a full hospitality-stack emulator.
- Tasks and README point evaluators toward the core QA journey, not a broad technology checklist.
