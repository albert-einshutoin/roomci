# プロトコルコンフォーマンスレジストリ

This registry is the source of truth for protocol-conformance language in `roomci`.

`roomci` does not claim certification-grade conformance. A protocol row can only use `conformance_subset` when the implemented subset is documented, black-box verified, and every important unsupported behavior is explicit.

## Status Terms

| Term | Meaning |
|---|---|
| `scenario_model` | Behavior is executable inside YAML scenario runs |
| `serve_endpoint` | Behavior is externally drivable or observable through `roomci serve` |
| `external_client_tested` | A black-box client or Compose service exercises the endpoint |
| `conformance_subset` | A documented wire-level subset is implemented against an official specification reference |
| `contract_profile` | Adapter-contract fields and dry-run fixtures describe evaluator-supplied gateway mappings without wire behavior |
| `future_profile` | A candidate future protocol profile exists, but no implementation is claimed |
| `unsupported` | Intentionally not implemented |

## Registry

| Protocol | Official Reference | Current Status | Implemented Subset | Verification | Unsupported / Rejected Behavior |
|---|---|---|---|---|---|
| MQTT 3.1.1 | OASIS MQTT Specification, https://mqtt.org/mqtt-specification/ | `conformance_subset`, `serve_endpoint`, `external_client_tested` | `CONNECT`, `CONNACK`, QoS0 `PUBLISH`, QoS0 `SUBSCRIBE`/`SUBACK` for configured state topics, retained replay to subscribers, UTF-8 topic names, JSON object payloads, configured command/state topic mapping, retained-state observation through HTTP reports | `cargo test -p roomci-serve --lib external_mqtt`; `cargo test -p roomci-cli --test cli standard_mqtt_client_publishes_retained_state_through_serve`; `make protocol-smoke-mqtt` | QoS1/QoS2 wire acks, unsubscribe, wildcard generality beyond the documented subset, sessions, will messages, keepalive enforcement, auth/TLS, clustering, persistence |
| MQTT 5.0 | OASIS Standard MQTT v5.0, https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html | `unsupported` | None | `mqtt_connect_with_unsupported_level_is_rejected` | Protocol level `5` receives documented `CONNACK` unacceptable protocol version; MQTT 5 properties are not parsed |
| Modbus TCP / Application Protocol | Modbus Specifications, https://www.modbus.org/modbus-specifications; Modbus Application Protocol PDF, https://www.modbus.org/file/secure/modbusprotocolspecification.pdf | `conformance_subset`, `serve_endpoint`, `external_client_tested` | MBAP header, unit id mapping, read contiguous holding registers (`0x03`), read contiguous input registers (`0x04`), write single register (`0x06`), exception responses for unsupported function, illegal address/value, read-only write | `cargo test -p roomci-serve --lib modbus_tcp`; `make protocol-smoke-modbus` | Serial RTU timing/electrical behavior, full function-code coverage, vendor-specific maps, multi-register writes |
| BMS/contact HTTP contract | roomci adapter contract docs | `serve_endpoint`, `external_client_tested` | `POST /external/bms/contact`, validation errors, sanitized observation state, timeline/report evidence, run-boundary preservation | `cargo test -p roomci-serve --lib external_bms_contact`; `examples/controllers/bms_webhook_poc_controller.sh` | Production BMS APIs, HMAC/replay protection, retries, real Slack/phone/ticket calls unless a supplied adapter implements them |
| DALI-like lighting | roomci DALI strategy docs | `scenario_model` | Fixture levels, named scenes, dropped command faults, partial scene failure evidence | `dali_scene_partial_failure` tests and scenario run | Full DALI bus implementation, electrical/gateway certification, photometric validation |
| Contact I/O | roomci device model docs | `scenario_model`, `serve_endpoint` via BMS/contact | Contact state changes, safety-alert source mapping, report evidence | contact and BMS tests | Real relay I/O, debounce/timing certification, physical safety |
| BACnet | ANSI/ASHRAE Standard 135, https://www.ashrae.org/technical-resources/bookstore/bacnet; 135-2024 notice, https://bacnet.org/news/ansi-ashrae-135-2024-now-published/ | `contract_profile` | Adapter-contract fields for device id, object type/instance, property, expected value, and event class; dry-run scenario evidence only | `make protocol-profile-smoke`; `cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/bacnet_contract_profile.yaml` | No BACnet/IP endpoint, object services, COV, BBMD, routing, or certification claims |
| OPC UA | OPC UA specifications, https://opcfoundation.org/developer-tools/specifications-unified-architecture; online reference, https://reference.opcfoundation.org/ | `contract_profile` | Adapter-contract fields for endpoint, namespace, node id, browse name, attribute, expected value, and event type; dry-run scenario evidence only | `make protocol-profile-smoke`; `cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/opcua_contract_profile.yaml` | No OPC UA server, NodeSet/address-space compliance, security policy, certificates, subscriptions, or certification claims |
| Zigbee | CSA Zigbee, https://csa-iot.org/all-solutions/zigbee/; Revision 23 PDF | `future_profile` | None | None yet | No radio stack, coordinator, clusters, commissioning, or certification claims |
| Thread | Thread Specification, https://threadgroup.org/ThreadSpec | `future_profile` | None | None yet | No mesh networking, border router, commissioning, or radio behavior |
| KNX | KNX specifications, https://support.knx.org/hc/en-us/articles/360000040999-KNX-Specifications | `contract_profile` | Adapter-contract fields for gateway, group address, datapoint type, direction, expected value, function, and room; dry-run scenario evidence only | `make protocol-profile-smoke`; `cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/knx_group_address_profile.yaml` | No KNX bus, group-address wire endpoint, ETS import, KNX/IP tunneling/routing, telegram timing, or device certification |
| Matter | CSA Matter, https://csa-iot.org/all-solutions/matter/; Google Home overview, https://developers.home.google.com/matter/overview?hl=ja | `contract_profile` | Adapter-contract fields for gateway, device id, endpoint id, cluster, attribute, command, and expected state; dry-run scenario evidence only | `make protocol-profile-smoke`; `cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/matter_gateway_profile.yaml` | No Matter fabric, commissioning, CASE/PASE, Thread/Wi-Fi transport, Matter SDK integration, controller compatibility, or certification claims |

## Release Rule

Any README, docs, or release note claim using words such as "compatible", "conformant", "supports", or "implements" for a protocol must point to one registry row and one verification command.

If the row is `future_profile` or `unsupported`, the product copy must say so directly.
