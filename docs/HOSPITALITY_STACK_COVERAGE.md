# Hospitality Smart Home Stack Coverage

`roomci` is not a NOT A HOTEL full-stack emulator. It is a contract-first local/CI emulator for hospitality-grade smart-home and building-automation QA.

The useful product boundary is the middle of the stack: local MQTT, edge control, device protocol behavior, network/control-panel fault handling, BMS operations, comfort automation, and report evidence.

## Coverage Tiers

| Tier | Meaning |
|---|---|
| Cover now | Executable today in scenarios, `roomci serve`, PoC packs, or reports |
| Cover next | Should become executable because it strengthens the core QA journey |
| Mock / contract only | Should be represented through adapter contracts, mocks, or evidence outputs, not real integrations |
| Future profile | Worth tracking, but only after a narrow subset is selected |
| Out of scope | Should not be implemented by `roomci` |

## Coverage Map

| Stack Area | Reported Technologies | Coverage Tier | roomci Position |
|---|---|---|---|
| Local MQTT control | MQTT, local broker, cloud broker, retained messages, QoS 1 expectations | Cover now / cover next | MQTT retained-state behavior and MQTT 3.1.1 serve ingress exist today, including QoS0 publish and retained subscribe/replay for configured state topics. QoS1/reconnect/session boundaries remain outside the current subset. |
| Edge / home control server | Edge server, home control server, local-first fallback, primary/secondary edge | Cover now | Edge routing, failover, and local-first outage scenarios are core product coverage. |
| Device protocols | Modbus, DALI, KNX-like behavior, contact I/O, HVAC, lighting, locks, sauna, bath | Cover now / cover next | Modbus, DALI-like lighting, contact I/O, and comfort/HVAC are scenario models. Modbus TCP is the next wire-level endpoint. KNX remains a future profile. |
| BMS / operations | Self-built BMS, Slack, Zoom Phone, runbook, ticket, comments, recovery | Cover now / cover next | BMS/ops alerts, phone-like escalation, ticket state, runbook URL, and recovery evidence are core. Real Slack/Zoom/PagerDuty calls stay disabled by default. |
| Network / WAN | UniFi, YAMAHA, Starlink, VLAN, VPN, firewall, L3 switch, Wi-Fi roaming | Cover now / cover next | WAN outage and backup-link behavior exist. VLAN/firewall/segment failures should be modeled as fault profiles, not vendor device emulators. |
| Control panel / electrical | 24V power, UPS, redundant PSU, circuit protector, edge computer redundancy, industrial switches | Cover next / mock | Phase 16 promoted these into deterministic QA fault profiles with BMS evidence. `roomci` must not claim electrical safety validation. |
| Comfort sensors / HVAC | temperature, humidity, occupancy-zone sensors, ceiling-zone sensors, discomfort index, HVAC auto mode | Cover now / cover next | Comfort automation exists. The next step is clearer sensor-zone assumptions and field-readable report evidence. |
| Access / intercom | DoorBird, ONVIF, SIP, DTMF, Twilio SIP Trunk, UniFi Door Access, Okta/LDAP, electric lock panels | Mock / contract only | Access drift and intercom/relay safe-mock scenarios are safe QA coverage. Intercom events record PIN/call/relay evidence only. Real unlock authorization is out of scope. |
| Cloud / backend | Google Cloud, Cloud Run, Firestore, Cloud VPN, AWS IoT Core, IoT Rule, SQS, Lambda, Cloudflare Workers | Mock / contract only | Cloud systems are integration assumptions or adapter contracts. `roomci` should not become a cloud emulator. |
| Applications and APIs | Go, TypeScript, Swift, Lua, Protocol Buffers, gRPC, REST, iPad app, Web API, PMS | Mock / contract only / out of scope | Production app languages are not emulator targets. Protocol payloads and acceptance criteria are adapter inputs. PMS and booking flows are out of scope. |
| Monitoring / data | Grafana, InfluxDB, time-series DB, Grafana Alerting | Mock / contract only | Reports and exported evidence should be enough for CI. Real observability stack replacement is out of scope. |
| Design / construction | Rebro, CAD, UniFi Design Center, BuddyBoard, draw.io, KiCad, JLCPCB, Blender, 3D printing | Out of scope | These are design/construction tools, not local/CI QA emulator surfaces. |
| Broader standards | BACnet, OPC UA, Zigbee, Thread, Matter, KNX | Future profile / adapter-contract only / runtime non-goal | Phase 16 deferred BACnet/IP and OPC UA, kept KNX as adapter-contract only unless real group maps are supplied, and kept Matter/Zigbee/Thread as runtime non-goals without gateway contracts. |

## Core QA Journey

The product should optimize for this journey:

```txt
local controller / external client
  -> MQTT command/state contract
  -> edge server routing or failover
  -> device protocol behavior
  -> network or control-panel fault
  -> BMS/ops escalation
  -> CI-ready JSON/Markdown/JUnit evidence
```

This is the part of a hospitality smart-home stack that is both valuable to NOT A HOTEL-like teams and reusable for other smart-home, IoT, and building-automation companies.

## Non-goals

- Full NOT A HOTEL private compatibility.
- Full cloud-platform replacement for Google Cloud, AWS IoT, Cloudflare, or customer backends.
- Vendor emulators for UniFi, DoorBird, Twilio, Zoom Phone, Slack, Grafana, InfluxDB, Jamf, or YAMAHA.
- Real lock authorization or physical access-control decisions.
- Physical electrical safety validation for wiring, high/low voltage circuits, lightning protection, UPS hardware, or breakers.
- CAD/BIM/design-tool workflow emulation.

## Customer Inputs

Any real company evaluation still requires customer-supplied:

- MQTT topics and payload schemas.
- Modbus register maps and scale/unit conventions.
- BMS/webhook contracts.
- Edge failover and local-first timing expectations.
- Network and control-panel fault semantics.
- Auth/TLS assumptions.
- Acceptance criteria and report requirements.
