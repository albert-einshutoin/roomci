# Phase 13 Goal — Protocol Compliance Track

## Goal

Move `roomci` from "protocol-shaped PoC endpoints" toward official-spec-backed, black-box-verifiable protocol subsets.

The target product claim after this phase is:

```txt
roomci implements documented conformance subsets for selected open building/IoT protocols, verifies them with standard clients in Docker, and clearly rejects or defers every behavior outside those subsets.
```

This phase should strengthen evaluator trust without turning `roomci` into a full broker, PLC, BMS, or certification suite.

## Strategy

Protocol depth should be earned in this order:

1. **MQTT 3.1.1** — already has a serve endpoint and should become the first spec-backed conformance subset.
2. **Modbus TCP / Application Protocol** — high value for building automation and practical enough to implement as a limited wire endpoint.
3. **BMS webhook / HTTP contract** — not a global standard, but should be treated as a versioned adapter contract with explicit validation.
4. **BACnet / OPC UA / Matter / Zigbee / Thread / KNX** — track as official-spec references and adapter-boundary candidates before implementing any wire endpoint.

The product should prefer narrow, verified subsets over broad unsupported claims.

## Official Specification References

| Technology | Reference | URL |
|---|---|---|
| MQTT | OASIS MQTT Specification | https://mqtt.org/mqtt-specification/ |
| MQTT 5.0 | OASIS Standard MQTT v5.0 | https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html |
| Modbus | Modbus Specifications | https://www.modbus.org/modbus-specifications |
| Modbus Application Protocol | Function Code / PDU / Exception definitions | https://www.modbus.org/file/secure/modbusprotocolspecification.pdf |
| BACnet | ANSI/ASHRAE Standard 135 | https://www.ashrae.org/technical-resources/bookstore/bacnet |
| BACnet 135-2024 | Published-version notice | https://bacnet.org/news/ansi-ashrae-135-2024-now-published/ |
| OPC UA | OPC UA Specification | https://opcfoundation.org/developer-tools/specifications-unified-architecture |
| OPC UA Online Reference | Searchable official reference | https://reference.opcfoundation.org/ |
| Zigbee | CSA Zigbee Specification | https://csa-iot.org/all-solutions/zigbee/ |
| Zigbee Revision 23 | Zigbee Specification PDF | https://csa-iot.org/wp-content/uploads/2023/04/05-3474-23-csg-zigbee-specification-compressed.pdf |
| Thread | Thread Specification | https://threadgroup.org/ThreadSpec |
| KNX | KNX Specifications | https://support.knx.org/hc/en-us/articles/360000040999-KNX-Specifications |
| Matter | CSA Matter Standard | https://csa-iot.org/all-solutions/matter/ |
| Matter developer overview | Google Home Developers | https://developers.home.google.com/matter/overview?hl=ja |

## In Scope

- Add a protocol conformance registry that records:
  - official specification source
  - supported version
  - implemented subset
  - unsupported behavior
  - explicit rejection behavior
  - black-box verification tool
  - Docker/Compose test command
- Upgrade MQTT 3.1.1 support from "minimal ingress" to "documented conformance subset".
- Add standard-client MQTT smoke tests using an existing MQTT client/library or CLI, not only hand-written packets.
- Add a Modbus TCP serve endpoint subset backed by Modbus Application Protocol function-code and exception behavior.
- Add Docker/Compose protocol smoke tests that can be run without local Rust tooling.
- Keep docs and README claims synchronized with the actual conformance subset.
- Track BACnet, OPC UA, Zigbee, Thread, KNX, and Matter as future protocol profiles with explicit non-goals until a narrow subset is selected.

## Out of Scope

- Certification-grade conformance for any protocol.
- Full MQTT broker semantics, clustering, persistence, ACLs, TLS, or MQTT 5 properties.
- Full Modbus device/vendor emulation or serial RTU electrical behavior.
- Full BACnet, OPC UA, Zigbee, Thread, KNX, or Matter stacks in this phase.
- Vendor-specific or private customer adapters without supplied contracts.
- Replacing specialized protocol conformance suites.

## Exit Criteria

- `docs/PROTOCOL_SUPPORT_MATRIX.md` distinguishes `conformance_subset` from `scenario_model` with spec references and verification commands.
- MQTT 3.1.1 subset is verified by at least one standard external client in tests or Compose.
- Modbus TCP subset exists or is explicitly blocked with a documented technical reason and next implementation step.
- Every implemented wire endpoint has explicit tests for supported packets and unsupported/rejected packets.
- Docker/Compose protocol smoke tests run from a clean checkout.
- README and protocol docs avoid claiming full conformance beyond the implemented subset.
