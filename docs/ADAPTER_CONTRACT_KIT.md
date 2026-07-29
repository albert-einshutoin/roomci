# Adapter Contract Kit

Adapter contracts describe company-specific protocol details without hard-coding those details into `roomci` runtime code.

Use them when a company wants to evaluate `roomci` against its own MQTT topics, Modbus register maps, BMS alert flows, edge-controller commands, device identity model, auth assumptions, and acceptance criteria.

## Files

- Template: [`adapter-contracts/templates/company_adapter_contract.yaml`](../adapter-contracts/templates/company_adapter_contract.yaml)
- Generic MQTT example: [`adapter-contracts/examples/generic_mqtt_edge_device.yaml`](../adapter-contracts/examples/generic_mqtt_edge_device.yaml)
- Hospitality local-first example: [`adapter-contracts/examples/hospitality_local_first_room.yaml`](../adapter-contracts/examples/hospitality_local_first_room.yaml)
- Building automation / BMS example: [`adapter-contracts/examples/building_automation_bms.yaml`](../adapter-contracts/examples/building_automation_bms.yaml)
- Acceptance evidence mapping example: [`adapter-contracts/mappings/acceptance_evidence_mapping.yaml`](../adapter-contracts/mappings/acceptance_evidence_mapping.yaml)

## Validate

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml
```

Validate every shipped adapter contract:

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml
```

Validate an acceptance-to-evidence mapping against its scenario:

```bash
cargo run -p roomci-cli -- adapter validate \
  adapter-contracts/mappings/acceptance_evidence_mapping.yaml \
  --scenario examples/generic_mqtt_retained_state.yaml
```

## Acceptance Evidence Mapping

Existing scalar `acceptance.criteria` entries remain unchanged. Add
`acceptance.mappings` only when a criterion needs a stable id and explicit
evidence references:

```yaml
acceptance:
  criteria:
    - A command updates retained state.
  report_formats: [json, junit]
  mappings:
    - id: retained-state-synchronized
      criterion: A command updates retained state.
      assertions:
        - scenario: generic_mqtt_retained_state
          assertion: retained_state_updated
      artifacts: [json, junit]
```

The scenario assertion declares the referenced name:

```yaml
assertions:
  - at: T+1s
    name: retained_state_updated
    mqtt:
      topic: fleet/demo/site/lab/device/env_sensor_01/state
      retained:
        online: true
```

Mapping validation fails on missing scenarios or assertion names, duplicate or
unsafe ids, unsupported artifact kinds, and artifacts absent from
`report_formats`. JSON evidence stores the stable name as `reference_id`;
Markdown and JUnit show it alongside the runtime diagnostic name.

`adapter validate` proves that the references resolve and that the artifact
kind can be generated. It does not claim that a report artifact has already
been produced; run the mapped scenario to create evidence.

## Required Customer Inputs

For MQTT:

- topic templates
- device-id extraction rule
- required and optional JSON payload fields
- optional field constraints: `type`, `enum`, `minimum`, and `maximum`
- retained-state expectation
- QoS/session expectation

### Typed MQTT payload fields

`required_fields` remains compatible with existing contracts. Add
`optional_fields` and `fields` only when the evaluator needs contract-level
JSON value checks:

```yaml
payload:
  required_fields: [power]
  optional_fields: [brightness, mode]
  fields:
    power:
      type: boolean
    brightness:
      type: integer
      minimum: 0
      maximum: 100
    mode:
      type: string
      enum: [eco, comfort]
```

Supported types are `string`, `integer`, `number`, `boolean`, `object`, and
`array`. Numeric ranges are available only for `integer` and `number`. Every
entry under `fields` must be classified as required or optional. Unknown
payload fields remain allowed.

Write exact whole-number boundaries as integer literals. Floating-point
boundary notation is limited to ±9,007,199,254,740,991 so comparisons cannot
silently lose integer precision.

This is intentionally a small adapter-contract vocabulary. It validates the
payload values used by scenario and serve-mode MQTT publishes, but does not
implement nested schemas, pattern matching, composition, or a full JSON Schema
engine.

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
