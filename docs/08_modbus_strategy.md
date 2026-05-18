# 08. Modbus Strategy

## Why Modbus matters

Modbus should be part of the MVP because it maps well to real building equipment and supports:

- floor heating control
- power meters
- bath systems
- detailed error-code acquisition
- multi-drop device communication
- register-map based integration with vendors and partners
- field debugging through existing Modbus tools

## Scope

v0.1 should support **Modbus TCP mock** with an RTU-like mental model.

v0.1 does not need full serial/RS485 implementation, but the YAML should allow modeling unit IDs and register maps that correspond to RTU-style devices.

## Address types

Support the four Modbus data categories:

| Type | Access | Width | Use |
|---|---|---|---|
| Coils | read/write | 1 bit | ON/OFF commands |
| Discrete Inputs | read-only | 1 bit | status inputs |
| Input Registers | read-only | 16 bit | sensor values |
| Holding Registers | read/write | 16 bit | settings and commands |

## Example register map

```yaml
modbus:
  devices:
    - id: floor_heating_01
      host: 0.0.0.0
      port: 5020
      unit_id: 1
      holding_registers:
        40001:
          name: target_temperature
          type: decimal_0_1
          value: 245
          unit: celsius
          readable_value: 24.5
      input_registers:
        30001:
          name: current_temperature
          type: decimal_0_1
          value: 238
          unit: celsius
          readable_value: 23.8
      discrete_inputs:
        10001:
          name: error
          value: false
      coils:
        1:
          name: power
          value: true
```

## Register validation

`roomci` should validate:

- duplicated register addresses
- invalid type ranges
- write attempts to read-only registers
- endianness declarations for multi-register values
- missing scaling definitions
- missing partner-facing descriptions

## Multi-register values

Example:

```yaml
40010:
  name: accumulated_power_wh
  type: uint32
  registers: 2
  byte_order: big_endian
  word_order: little_endian
```

## Faults

Modbus-specific faults:

```yaml
faults:
  - target: modbus.floor_heating_01
    type: timeout
  - target: modbus.power_meter_01
    type: wrong_unit_id
  - target: modbus.bath_01.40021
    type: stuck_value
    value: 410
  - target: modbus.floor_heating_01
    type: exception_response
    code: illegal_data_address
```

## Demo: floor heating

```yaml
scenario:
  name: modbus_floor_heating_0_1_degree_control

steps:
  - at: T
    modbus_write:
      device: floor_heating_01
      register: 40001
      value: 245
  - at: T+1s
    assert:
      modbus:
        device: floor_heating_01
        register: 40001
        readable_value: 24.5
```

## Report example

```txt
PASS modbus_floor_heating_0_1_degree_control

Verified:
- Unit ID 1 responded
- Holding register 40001 accepted target temperature 24.5°C
- Input register 30001 remained readable

Commissioning note:
- Register map appears consistent with scenario expectations.
```
