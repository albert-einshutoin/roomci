# Category Readiness

`roomci` should be evaluated as a contract-first emulator for IoT, SmartHome, edge-device, and building-automation QA.

## Comparison

| Alternative | Where It Wins | Where roomci Wins |
|---|---|---|
| Real-device staging environment | Highest physical fidelity | Faster local/CI feedback, lower setup cost, deterministic failure reproduction |
| Generic MQTT broker | Real broker semantics | Device, edge, operations, reports, and acceptance criteria in one harness |
| Ad hoc mock scripts | Quick one-off tests | Reusable contracts, typed validation, PoC packs, reports, and documented scope |
| Home Assistant-based test setup | Broad ecosystem integrations | CI-focused contracts, failure reports, adapter templates, and no home-server dependency |
| Cloud-only IoT emulator | Cloud API coverage | Local-first, edge, device, BMS, Modbus, contact, and field-failure modeling |
| Hardware-in-the-loop | Timing and electrical realism | Earlier pre-HIL coverage, cheaper regression loops, easier unsupported-path disclosure |

## Where roomci Wins Today

- Contract-first local and CI emulation.
- External HTTP and MQTT subset surface for black-box PoCs.
- Modbus TCP practical subset for standard-client smoke checks.
- JSON, Markdown, and JUnit evidence.
- Adapter templates for private company specs.
- Go and TypeScript adapter samples that run against `roomci serve`.
- Hospitality local-first domain pack with concrete field, comfort, network,
  control-panel, access/intercom, and operations risks.
- Explicit support matrix that separates behavior modeling from wire compatibility.

## Where roomci Does Not Win Yet

- Full protocol conformance.
- Hardware timing, electrical behavior, and real device commissioning.
- Production control-plane replacement.
- Unknown private vendor features without supplied contracts.
- Full DALI, KNX, BACnet, Matter, SIP, OPC UA, Zigbee, Thread, or production BMS behavior.
- Vendor cloud, intercom, access-control, networking, or notification platform emulation.

## Evaluator Checklist

Use `roomci` when:

- Your team needs pre-device or pre-site QA evidence.
- Your failures cross software, edge, devices, and operations.
- Your private specs can be represented as contracts.
- CI reports are useful to both software and field teams.
- You want a reusable harness instead of another one-off mock.
- You want a pre-adoption PoC surface that can be driven by standard MQTT,
  Modbus, HTTP, Docker Compose, and small adapter samples.

Do not use `roomci` as the final gate when:

- Certification-grade protocol conformance is required.
- Electrical timing, sensor noise, or physical commissioning is the core risk.
- The team cannot provide protocol contracts or acceptance criteria.

## Category Claim

`roomci` should be a first-choice candidate when the evaluation question is:

```txt
Can we make our IoT, SmartHome, edge, or building-automation behavior reproducible in local and CI before we reach real devices or real sites?
```

It should not be positioned as a replacement for protocol conformance suites, hardware-in-the-loop rigs, or production control systems.
