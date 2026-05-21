# 製品ポジショニング

## What roomci Is

`roomci` is a Docker-friendly QA contract emulator for MQTT-based edge, smart-home, and building-automation systems.

It lets teams describe expected behavior as YAML scenarios, run those scenarios locally or in CI, and collect timeline, assertion, JSON, Markdown, and JUnit reports. The current runtime is a behavioral emulator: it models the contract that a controller, edge service, device adapter, or operations workflow should satisfy without requiring real devices or a site visit.

The core product category is:

```txt
MQTT / edge / device QA contract emulator for CI
```

## Who It Is For

- MQTT-based edge-device teams that need repeatable command/state contract tests.
- Smart-home and hospitality engineering teams that need local-first failure tests.
- Building-automation teams that need practical Modbus, lighting, contact, and BMS behavior checks.
- Facility-ops and commissioning teams that want field knowledge encoded as executable scenarios.
- CI owners who need pass/fail artifacts instead of manual environment checks.

## What It Is Not

`roomci` is not:

- a production MQTT broker
- a production smart-home controller
- a protocol conformance suite
- a full KNX, DALI, BACnet, Matter, SIP, or Modbus implementation
- a real BMS, Slack, phone, ticketing, or cloud integration

The purpose is not to prove wire-level protocol compliance. The purpose is to make behavioral contracts testable before a device, edge service, or operations flow reaches a real environment.

## なぜ Not a Production MQTT Broker

Production MQTT brokers need full session semantics, authorization, TLS, clustering, persistence, operational metrics, and protocol-version compatibility. `roomci` does not try to replace that layer.

Instead, it models the contract that usually matters in CI:

- command topics produce expected state topics
- retained state is available after updates
- duplicate delivery is idempotent
- local operation remains available during upstream failure
- timelines and assertions explain what changed

When Phase 10 adds a real serve runtime, MQTT compatibility should still be scoped as an adapter surface for tests, not a broker replacement.

## なぜ Not a Protocol Conformance Suite

Protocol conformance suites validate precise protocol behavior. `roomci` validates product behavior around those protocols.

For example:

- DALI-like scene tests check whether a scene reaches expected fixture levels.
- Modbus tests check whether a declared register map behaves as expected.
- BMS/ops tests check whether a critical contact results in notification, escalation, ticket, and runbook state.

That is intentionally narrower and more product-focused than full conformance.

## Hospitality as a Domain Pack

Hospitality smart home is the strongest current domain pack because it combines local-first control, field commissioning, BMS alerting, comfort automation, access drift, and guest-impact reporting.

It is not the whole product. The reusable core is the same for other MQTT/edge/building-automation teams:

```txt
scenario -> emulator models -> assertions -> CI reports
```

The hospitality domain pack should be read as a concrete, high-signal demonstration of why this category matters: local-first control, commissioning, BMS alerting, and guest-impact focus.

For public positioning, keep the two tracks explicit:

- Industry track: IoT / SmartHome / building-automation QA contract emulator for local and CI testing.
- Hospitality track: local-first hospitality smart-home domain pack that demonstrates the kind of reliability work that hospitality engineering teams need to make repeatable.

See [`DUAL_TRACK_POSITIONING.md`](DUAL_TRACK_POSITIONING.md) for copy candidates and claim boundaries.
