# Phase 13 Test Plan

## Quality Gates

- Protocol conformance registry exists and lists official specification references for MQTT, MQTT 5, Modbus, BACnet, OPC UA, Zigbee, Thread, KNX, and Matter.
- `docs/PROTOCOL_SUPPORT_MATRIX.md` includes, per protocol:
  - support level
  - spec reference
  - implemented subset
  - black-box verification command
  - unsupported behavior
  - explicit rejection behavior
- MQTT 3.1.1 tests cover:
  - accepted `CONNECT` with protocol name `MQTT`, level `4`
  - rejected MQTT 3.1 `MQIsdp`
  - rejected MQTT 5 protocol level `5`
  - QoS0 JSON `PUBLISH` through a standard client/library
  - malformed packet handling
  - retained state visible through `/state` and reports
- Modbus TCP tests cover the selected subset:
  - MBAP header parsing
  - unit id handling
  - supported function code success response
  - unsupported function code exception response
  - read-only write rejection
  - illegal address / illegal value behavior
- Docker/Compose smoke tests run at least:
  - one standard MQTT client against `roomci serve`
  - one Modbus TCP client/tool against `roomci serve`, once Modbus TCP subset exists
- Docs tests or release checks verify no README/docs claim full protocol conformance unless backed by implementation evidence.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --all-targets`, and `make compose-poc` pass.

## Done Means

Phase 13 is done when an evaluator can inspect a protocol row, see the official spec source, run the Docker-backed black-box test, and understand exactly which protocol behaviors are supported, rejected, or intentionally deferred.
