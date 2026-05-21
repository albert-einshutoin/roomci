# roomci Product Goal

## Target Position

`roomci` should become the first-choice OSS QA contract emulator for teams
building IoT, SmartHome, hospitality smart-home, edge-device, and building
automation systems.

The product should let an evaluator bring private company contracts such as
MQTT topics, payload schemas, Modbus register maps, BMS alert contracts,
acceptance criteria, and auth/network assumptions, then quickly turn those
contracts into reproducible local and CI evidence.

## Product Promise

`roomci` does not replace production controllers, brokers, gateways, BMS
systems, security authorities, or hardware-in-the-loop rigs.

It provides the pre-adoption and pre-site layer:

- encode expected behavior as scenarios and adapter contracts
- run those scenarios locally, in Docker, or in GitHub Actions
- drive the emulator through standard HTTP, MQTT, and Modbus client paths
- collect reports, timelines, trace metadata, observability artifacts, and JUnit
  evidence
- make unsupported protocol depth and customer-specific unknowns explicit

## Core User Journeys

1. **Evaluator first run**
   - clone the repository
   - run `make verify`
   - inspect generated JSON / Markdown / JUnit / timeline / observability
     artifacts
   - understand what is supported and what is intentionally out of scope

2. **Company adapter PoC**
   - fill an adapter contract with private topics, payloads, register maps, BMS
     schema, identity rules, auth assumptions, and acceptance criteria
   - validate the contract
   - run a PoC pack
   - collect CI evidence that maps directly to the company acceptance criteria

3. **Developer workflow**
   - use SDK samples or SDK packages from Go, TypeScript, and Python
   - debug scenario execution without reading Rust internals
   - use editor support for schema validation, snippets, and task commands

4. **Protocol-profile evaluation**
   - evaluate MQTT and Modbus through implemented official-spec-backed subsets
   - evaluate Matter, BACnet, KNX, and OPC UA through future/prototype profiles
     that remain honest about non-certification scope

## Tier Completion Model

### S Tier: Evidence And CI Surface

S Tier is complete when adapter CI, timeline export, trace metadata,
observability artifacts, and GitHub Actions examples are machine-checkable and
documented. This is tracked in Phase 19.

### A Tier: Developer Experience And Adoption Surface

A Tier is complete when an evaluator can integrate `roomci` without writing raw
curl scripts or reading Rust code:

- Python SDK package
- scenario debugger
- VSCode extension assets
- SDK and debugger CI examples
- copy-paste evaluator workflows

This is tracked in Phase 20.

### B Tier: Future Protocol Profile Surface

B Tier is complete when high-interest smart-home and building-automation
protocols are represented as scoped profile prototypes:

- Matter prototype profile
- BACnet profile
- KNX profile
- OPC UA profile

These profiles should validate contracts and produce useful scenario evidence,
but must not claim full protocol certification or production gateway behavior.
This is tracked in Phase 21.

## Permanent Non-Goals

- MQTT broker competition.
- Full protocol certification.
- Real lock authorization, physical relay authority, or security authority.
- Production orchestration.
- Vendor cloud emulation for AWS, GCP, Cloudflare, UniFi, DoorBird, Twilio,
  Slack, Zoom, Grafana, InfluxDB, Jamf, or similar systems.
- Full NOT A HOTEL private compatibility without supplied private contracts.
