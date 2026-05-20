# Task 04 — Modbus TCP Conformance Subset

## Goal

Add a narrow Modbus TCP wire endpoint subset so building-automation evaluators can use standard Modbus tools against `roomci serve`, instead of only running internal scenario-model steps.

## Specification Sources

- Modbus Specifications: https://www.modbus.org/modbus-specifications
- Modbus Application Protocol: https://www.modbus.org/file/secure/modbusprotocolspecification.pdf

## Implementation Scope

- Add a `roomci serve` Modbus TCP port option or adapter endpoint.
- Parse MBAP headers and route requests into the existing `ModbusModel`.
- Start with a deliberately small function-code subset:
  - read holding registers
  - read input registers
  - write single register
  - optionally write multiple registers if the existing model supports it cleanly
- Return Modbus exception responses for:
  - unsupported function code
  - illegal data address
  - illegal data value
  - writes to read-only registers
- Map customer register definitions from existing adapter contracts or scenario config.
- Add black-box tests with a standard Modbus TCP client/tool once selected.

## Acceptance Criteria

- A standard Modbus TCP client/tool can read at least one configured register from `roomci serve`.
- A standard client/tool can write a configured writable register and observe the update in `/state` or report output.
- Unsupported function codes and invalid addresses return documented exception responses.
- Read-only register writes are rejected at the wire endpoint and in the internal model.
- Support matrix and conformance registry mark Modbus as `conformance_subset` only for the implemented function codes.
