# Dual-track Positioning

`roomci` should be read through two connected tracks.

## Industry Track

Primary category:

```txt
IoT / SmartHome / building-automation QA contract emulator for local and CI testing
```

Primary mechanism:

```txt
company-specific specs -> adapter contracts -> local emulator -> JSON/Markdown/JUnit evidence
```

This track is for MQTT edge-device teams, smart-home teams, building-automation teams, field QA teams, and CI owners. The product does not depend on knowing any single company's private implementation.

Good copy:

- "Turn private IoT and smart-home protocol contracts into repeatable local and CI tests."
- "Let external controllers drive a local emulator, then collect CI-ready reports."
- "Model behavior that matters before testing against real devices or a real site."
- "Bring your MQTT topics, Modbus maps, BMS contracts, and pass/fail rules."

Avoid:

- "Full MQTT broker."
- "Full Modbus/DALI/BACnet conformance."
- "Production BMS."
- "Drop-in compatibility."

## Hospitality Track

Strongest domain pack:

```txt
local-first hospitality smart home
```

This track proves domain insight: local MQTT, edge failover, Modbus-style equipment, DALI-like lighting scenes, contact I/O, BMS escalation, WAN failover, comfort automation, access drift, and commissioning reports.

Good copy:

- "Hospitality local-first scenarios show the kind of reliability work a NOT A HOTEL-style smart-home team needs to make repeatable."
- "The demo is not a private implementation clone; it is a contract-first path toward a serious PoC once real specs are supplied."
- "Hospitality is the sharpest domain pack, not the entire product."

Avoid:

- "We reproduced NOT A HOTEL."
- "Compatible with NOT A HOTEL systems."
- "Ready to connect to production rooms."

## Why The Two Tracks Reinforce Each Other

The industry track prevents the product from looking like a narrow interview demo. It makes `roomci` understandable to any company with private IoT, SmartHome, edge, or building-automation contracts.

The hospitality track prevents the product from looking generic. It demonstrates concrete, high-stakes reliability problems: guest-facing control, local-first operation, field commissioning, access drift, and operations response.

Together, the positioning is:

```txt
roomci is a contract-first emulator for IoT and SmartHome QA, with a strong hospitality local-first domain pack that makes NOT A HOTEL-style evaluation credible without overclaiming compatibility.
```

## First-read README Candidate

```txt
roomci is a contract-first local/CI emulator for IoT, SmartHome, edge-device, and building-automation QA. Teams bring their MQTT topics, Modbus maps, BMS contracts, edge behavior, and acceptance criteria; roomci validates those contracts, runs local scenarios, exposes HTTP/MQTT PoC endpoints, and emits JSON, Markdown, and JUnit evidence.

The strongest shipped domain pack is hospitality local-first smart home: cloud outage, local MQTT, edge failover, Modbus-style equipment control, DALI-like scene checks, contact I/O emergencies, BMS escalation, WAN failover, comfort automation, access drift, and commissioning reports.
```
