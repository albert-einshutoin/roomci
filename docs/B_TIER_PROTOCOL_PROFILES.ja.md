# B ティアプロトコルプロファイル

B Tier profile は、Matter、BACnet、KNX、OPC UA を `contract_profile` fixture として評価可能にします。

これらは wire-protocol 実装でも certification エビデンスでもありません。有用なプロダクト claim はより狭いものです。評価者が protocol-facing gateway map を持ち込み、実 gateway、controller、または field device が CI で利用可能になる前に adapter contract としてエンコードできます。

## Gate

```bash
make protocol-profile-smoke
```

これは 4 つの adapter contract と 4 つの dry-run scenario fixture を検証します。

- `adapter-contracts/examples/matter_gateway_profile.yaml`
- `adapter-contracts/examples/bacnet_contract_profile.yaml`
- `adapter-contracts/examples/knx_group_address_profile.yaml`
- `adapter-contracts/examples/opcua_contract_profile.yaml`
- `examples/matter_gateway_profile.yaml`
- `examples/bacnet_contract_profile.yaml`
- `examples/knx_group_address_profile.yaml`
- `examples/opcua_contract_profile.yaml`

## Matter Gateway Profile

サポートフィールド:

- `gateway`
- `device_id`
- `endpoint_id`
- `cluster`
- `attribute`
- `command`
- `expected_state`
- `acceptance`

必要な顧客入力:

- gateway mapping
- device identity mapping
- endpoint ids
- cluster and attribute names or ids
- command names
- expected state and acceptance criteria

非目的:

- Matter fabric
- commissioning
- CASE/PASE
- Thread or Wi-Fi transport behavior
- Matter SDK integration
- certification

## BACnet Contract Profile

サポートフィールド:

- `device_id`
- `object_type`
- `object_instance`
- `property`
- `expected_value`
- `event_class`
- `acceptance`

必要な顧客入力:

- BACnet device identity
- object type and instance
- property name or id
- expected value
- event or alarm class, when relevant
- acceptance criteria

非目的:

- BACnet/IP endpoint
- object service implementation
- COV subscriptions
- BBMD
- routing
- certification

## KNX Group-Address Profile

サポートフィールド:

- `gateway`
- `group_address`
- `datapoint_type`
- `direction`
- `expected_value`
- `function`
- `room`
- `acceptance`

必要な顧客入力:

- gateway label
- group-address map
- datapoint type
- direction
- expected value
- room/device/function mapping
- acceptance criteria

非目的:

- ETS import
- KNX/IP tunneling or routing
- telegram timing
- bus behavior
- device certification

## OPC UA Contract Profile

サポートフィールド:

- `endpoint`
- `namespace`
- `node_id`
- `browse_name`
- `attribute`
- `expected_value`
- `event_type`
- `acceptance`

必要な顧客入力:

- endpoint label
- namespace
- node id
- browse name
- attribute
- expected value
- event type, when relevant
- acceptance criteria

非目的:

- OPC UA server endpoint
- subscriptions
- security policies
- certificates
- NodeSet or address-space compliance
- certification

## 有用性の境界

これらの profile は、チームが実ハードウェアや現場アクセスの前に gateway-level contract を表現できる場合に有用です。packet-level protocol 挙動、公式 certification、timing、transport security、radio 挙動、vendor-specific device stack が評価に必要な場合は有用ではありません。
