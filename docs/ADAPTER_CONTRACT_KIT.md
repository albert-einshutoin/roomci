# Adapter Contract Kit

Adapter contracts describe company-specific protocol details without hard-coding those details into `roomci` runtime code.

Use them when a company wants to evaluate `roomci` against its own MQTT topics, Modbus register maps, BMS alert flows, edge-controller commands, device identity model, auth assumptions, and acceptance criteria.

## Files

- Template: [`adapter-contracts/templates/company_adapter_contract.yaml`](../adapter-contracts/templates/company_adapter_contract.yaml)
- Generic MQTT example: [`adapter-contracts/examples/generic_mqtt_edge_device.yaml`](../adapter-contracts/examples/generic_mqtt_edge_device.yaml)
- Hospitality local-first example: [`adapter-contracts/examples/hospitality_local_first_room.yaml`](../adapter-contracts/examples/hospitality_local_first_room.yaml)
- Building automation / BMS example: [`adapter-contracts/examples/building_automation_bms.yaml`](../adapter-contracts/examples/building_automation_bms.yaml)

## Validate

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml
```

Validate every shipped adapter contract:

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml
```

## Required Customer Inputs

For MQTT:

- topic templates
- device-id extraction rule
- required JSON payload fields
- retained-state expectation
- QoS/session expectation

For Modbus:

- device id and unit id
- register address
- register type
- scale and unit
- read/write access mode

For BMS / operations:

- alert source
- severity
- schema version
- content type, currently `application/json`
- severity enum, such as `info`, `warning`, `critical`, `emergency`
- optional HMAC metadata: header, `hmac-sha256`, and secret reference
- replay window in seconds
- notification channels
- ticket lifecycle
- acknowledgement expectation

For edge controllers:

- command source
- command target
- expected state transition

For evaluation:

- non-production auth assumptions
- pass/fail acceptance criteria
- expected report formats

## Boundary

The adapter contract kit does not make `roomci` compatible with a private system by itself. It makes missing private details explicit, validates that a supplied contract is structurally usable, and gives the emulator a stable input format for future adapter dispatch work.
