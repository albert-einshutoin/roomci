# プロトコルコンフォーマンスレジストリ

このレジストリは、`roomci` における protocol-conformance 言語の正本です。

`roomci` は certification-grade conformance を謳いません。protocol 行が `conformance_subset` を使用できるのは、実装済みサブセットが文書化され、black-box で検証され、重要な非サポート挙動がすべて明示されている場合のみです。

## ステータス用語

| 用語 | 意味 |
|---|---|
| `scenario_model` | 挙動が YAML シナリオ run 内で実行可能 |
| `serve_endpoint` | 挙動が `roomci serve` 経由で外部から駆動または観測可能 |
| `external_client_tested` | black-box client または Compose service が endpoint を実行 |
| `conformance_subset` | 公式仕様参照に対して文書化された wire-level サブセットが実装されている |
| `contract_profile` | adapter-contract フィールドと dry-run fixture が、wire 挙動なしで評価者供給の gateway mapping を記述 |
| `future_profile` | 将来候補の protocol profile が存在するが、実装は謳わない |
| `unsupported` | 意図的に未実装 |

## レジストリ

| プロトコル | 公式リファレンス | 現在のステータス | 実装サブセット | 検証 | 非対応 / 拒否された挙動 |
|---|---|---|---|---|---|
| MQTT 3.1.1 | OASIS MQTT Specification, https://mqtt.org/mqtt-specification/ | `conformance_subset`, `serve_endpoint`, `external_client_tested` | `CONNECT`, `CONNACK`, QoS0 `PUBLISH`, QoS0 `SUBSCRIBE`/`SUBACK` for configured state topics, retained replay to subscribers, UTF-8 topic names, JSON object payloads, configured command/state topic mapping, retained-state observation through HTTP reports | `cargo test -p roomci-serve --lib external_mqtt`; `cargo test -p roomci-cli --test cli standard_mqtt_client_publishes_retained_state_through_serve`; `make protocol-smoke-mqtt` | QoS1/QoS2 wire acks, unsubscribe, wildcard generality beyond the documented subset, sessions, will messages, keepalive enforcement, auth/TLS, clustering, persistence |
| MQTT 5.0 | OASIS Standard MQTT v5.0, https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html | `unsupported` | なし | `mqtt_connect_with_unsupported_level_is_rejected` | プロトコルレベル `5` は文書化された `CONNACK` 不可プロトコルバージョンを受け取る。MQTT 5 プロパティはパースされない |
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

## リリースルール

README、docs、release note で protocol について "compatible"、"conformant"、"supports"、"implements" などの語を使用する claim は、1 つの registry 行と 1 つの verification command を指す必要があります。

行が `future_profile` または `unsupported` の場合、プロダクト文言はそれを直接明記する必要があります。
