# Phase 11 Goal — Integration-ready Emulator Platform

## Goal

Raise `roomci` from a pre-adoption PoC surface into an integration-ready emulator platform that can become the default first choice for IoT, smart-home, and building-automation teams evaluating local/CI emulation.

The target product claim after this phase is:

```txt
Bring your protocol contracts, topic/register/webhook specs, and acceptance criteria; roomci turns them into a local emulator, fault harness, and CI evidence pack without requiring your production hardware or site network.
```

For hospitality specifically, the goal is not to guess private implementation details. The goal is to make every unknown an explicit adapter/configuration input so their actual MQTT, Modbus, BMS, edge, and operations contracts can be mapped quickly if they choose to evaluate it.

## Strategic Positioning

`roomci` should compete for the same mental slot as cloud service emulators and IoT test harnesses:

- LocalStack-like local dependency for edge/device QA.
- Floci-like practical emulator candidate for IoT and SmartHome workflows.
- Contract-first alternative to fragile staging environments with real devices.
- CI evidence generator for software, field engineering, and operations teams.

The product should be strongest where real systems have unclear or company-specific specifications. Instead of treating unknown specs as a blocker, `roomci` should provide the adapter scaffolding, validation, and test fixtures needed to encode those specs safely.

## Evaluation Maximization

The product should maximize evaluation through a dual-track strategy:

- The first track is industry-wide: `roomci` is a contract-first local/CI emulator for IoT, SmartHome, edge-device, and building-automation QA.
- The second track is hospitality-specific: `roomci` keeps a strong hospitality-focused local-first smart-home path that demonstrates domain insight without claiming private compatibility.

These tracks should reinforce each other. Industry-wide positioning makes `roomci` look like a real product rather than a narrow interview demo. The hospitality path keeps the product relevant to hospitality by showing concrete workflows around local control, edge reliability, Modbus-style equipment, BMS operations, WAN failure, and commissioning evidence.

## What Is Missing Today

- Real external protocol endpoints are shallow: MQTT exists only as a minimal QoS0 publish ingress; Modbus and BMS remain mostly scenario models.
- There is no adapter SDK/template that tells a customer how to encode their actual topics, register maps, webhooks, auth assumptions, and acceptance criteria.
- There is no protocol compatibility matrix that separates behavior emulation, wire compatibility, and production conformance.
- There is no customer-style PoC pack that can be handed to a company and completed by filling in their contract details.
- There is no public benchmark or comparison narrative against common alternatives such as real-device staging, mock scripts, generic MQTT brokers, Home Assistant test setups, or IoT emulator products.
- CI/public release claims need to be backed by visible workflow files, reproducible artifacts, and a release checklist.

## In Scope

- Add integration contract templates for MQTT, Modbus, BMS/webhooks, edge controllers, device state, auth assumptions, and acceptance criteria.
- Add adapter scaffolding so company-specific specs can be mapped without modifying core runtime code.
- Expand external protocol surfaces beyond the current minimal MQTT ingress.
- Add a protocol support matrix that distinguishes:
  - scenario-only behavioral model
  - local endpoint compatibility
  - external client interoperability
  - conformance-tested subset
  - production non-goals
- Add customer-style PoC packs:
  - generic smart-home controller
  - hospitality/hospitality-focused local-first home
  - generic building automation/BMS
  - generic MQTT edge-device fleet
- Add docs that explain exactly what a company must provide to evaluate `roomci`.
- Add comparison/positioning docs so `roomci` is credible as a first-choice emulator for IoT and SmartHome QA.
- Add hospitality-focused evaluator docs that show exactly how their real specs would map into generic adapter contracts.
- Add dual-track positioning so broad market relevance does not dilute hospitality smart-home relevance.
- Add evaluation scorecards and evidence packs for both generic companies and hospitality-focused evaluators.
- Add public release gates that prove docs, CI, Docker, Compose, reports, and examples are reproducible from a clean checkout.

## Out of Scope

- Claiming compatibility with hospitality private systems without their actual contracts.
- Production replacement for MQTT brokers, BMS products, PLCs, or smart-home controllers.
- Full protocol conformance for every protocol in one phase.
- Real calls to customer cloud, Slack, phone, SIP, access-control, device, or BMS systems by default.
- Building vendor-specific closed-source adapters without a supplied contract.

## Exit Criteria

- A new integrator can start from a template, fill in company-specific protocol details, and run a local PoC without changing Rust core code.
- `roomci serve` exposes at least two externally drivable protocol surfaces, not just HTTP plus minimal MQTT.
- The docs clearly show what hospitality would need to provide for a serious evaluation.
- The docs clearly show what any other IoT/SmartHome company would need to provide for a serious evaluation.
- The docs explain why industry-wide positioning increases hospitality relevance instead of weakening it.
- Evaluation evidence lets both generic companies and hospitality-focused teams score product fit without trusting pitch language.
- A clean-checkout verification command proves CLI, Docker, Compose, docs, and reports.
- Public positioning avoids overclaiming while making the product category ambition explicit.
