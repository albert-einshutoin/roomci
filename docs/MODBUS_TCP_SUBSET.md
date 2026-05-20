# Modbus TCP Serve Subset

`roomci serve --modbus-port <port>` starts a minimal Modbus TCP endpoint for pre-adoption building-automation PoC tests.

This is not a production PLC, gateway, or full Modbus conformance suite. It exists so an external Modbus TCP client can read and write a declared register map while `roomci` records state through HTTP reports.

## Supported

- Modbus TCP MBAP header parsing.
- Protocol id `0`.
- Unit id mapping from scenario `modbus.devices[].unit_id`.
- Function code `0x03`: read holding register.
- Function code `0x04`: read input register.
- Function code `0x06`: write single holding register.
- Quantity `1` for read requests.
- Register address convention:
  - direct scenario address if present, such as `40001`
  - conventional zero-based Modbus client address, such as `0` mapping to `40001` for holding registers
  - conventional zero-based input address, such as `0` mapping to `30001` for input registers
- HTTP observation through:
  - `GET /state`
  - `GET /reports/latest.json`
  - `GET /reports/latest.md`
  - `GET /reports/latest.junit.xml`

## Exception Responses

| Condition | Exception |
|---|---|
| Unsupported function code | `0x01` illegal function |
| Unknown unit id or register address | `0x02` illegal data address |
| Invalid quantity or read-only write | `0x03` illegal data value |

## Not Supported

- Serial RTU behavior or electrical timing.
- Full Modbus function-code coverage.
- Multi-register writes.
- Coils/discrete-input wire functions.
- Vendor-specific device behavior.
- Electrical commissioning or safety validation.

## Example

Scenario:

```yaml
modbus:
  devices:
    - id: floor_heating_01
      unit_id: 1
      holding_registers:
        40001:
          name: target_temperature
          type: decimal_0_1
          value: 230
```

Serve:

```bash
roomci serve --config examples/protocol_conformance_smoke.yaml --modbus-port 5020
```

Smoke:

```bash
make protocol-smoke-modbus
```

## Real Integration Inputs

To adapt this endpoint to a real building-automation system, the integrator must provide:

- unit ids
- register addresses
- register access mode
- scale and unit metadata
- legal value ranges
- acceptance criteria
- fault and alert semantics
