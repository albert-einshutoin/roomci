# B Tier Protocol Profiles

B Tier profiles make Matter, BACnet, KNX, and OPC UA evaluable as
`contract_profile` fixtures.

They are not wire-protocol implementations and not certification evidence. The
useful product claim is narrower: an evaluator can bring protocol-facing
gateway maps and encode them as adapter contracts before a real gateway,
controller, or field device is available in CI.

## Gate

```bash
make protocol-profile-smoke
```

This validates the four adapter contracts and the four dry-run scenario
fixtures:

- `adapter-contracts/examples/matter_gateway_profile.yaml`
- `adapter-contracts/examples/bacnet_contract_profile.yaml`
- `adapter-contracts/examples/knx_group_address_profile.yaml`
- `adapter-contracts/examples/opcua_contract_profile.yaml`
- `examples/matter_gateway_profile.yaml`
- `examples/bacnet_contract_profile.yaml`
- `examples/knx_group_address_profile.yaml`
- `examples/opcua_contract_profile.yaml`

## Matter Gateway Profile

Supported fields:

- `gateway`
- `device_id`
- `endpoint_id`
- `cluster`
- `attribute`
- `command`
- `expected_state`
- `acceptance`

Required customer inputs:

- gateway mapping
- device identity mapping
- endpoint ids
- cluster and attribute names or ids
- command names
- expected state and acceptance criteria

Non-goals:

- Matter fabric
- commissioning
- CASE/PASE
- Thread or Wi-Fi transport behavior
- Matter SDK integration
- certification

## BACnet Contract Profile

Supported fields:

- `device_id`
- `object_type`
- `object_instance`
- `property`
- `expected_value`
- `event_class`
- `acceptance`

Required customer inputs:

- BACnet device identity
- object type and instance
- property name or id
- expected value
- event or alarm class, when relevant
- acceptance criteria

Non-goals:

- BACnet/IP endpoint
- object service implementation
- COV subscriptions
- BBMD
- routing
- certification

## KNX Group-Address Profile

Supported fields:

- `gateway`
- `group_address`
- `datapoint_type`
- `direction`
- `expected_value`
- `function`
- `room`
- `acceptance`

Required customer inputs:

- gateway label
- group-address map
- datapoint type
- direction
- expected value
- room/device/function mapping
- acceptance criteria

Non-goals:

- ETS import
- KNX/IP tunneling or routing
- telegram timing
- bus behavior
- device certification

## OPC UA Contract Profile

Supported fields:

- `endpoint`
- `namespace`
- `node_id`
- `browse_name`
- `attribute`
- `expected_value`
- `event_type`
- `acceptance`

Required customer inputs:

- endpoint label
- namespace
- node id
- browse name
- attribute
- expected value
- event type, when relevant
- acceptance criteria

Non-goals:

- OPC UA server endpoint
- subscriptions
- security policies
- certificates
- NodeSet or address-space compliance
- certification

## Usefulness Boundary

These profiles are useful when a team can express a gateway-level contract in
advance of real hardware or site access. They are not useful when the evaluation
requires packet-level protocol behavior, official certification, timing,
transport security, radio behavior, or vendor-specific device stacks.
