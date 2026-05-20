# Task 02: Modbus Practical Subset

## Why

The current Modbus TCP runtime supports MBAP framing, functions 03/04/06, one-register reads, one-register writes, unit mapping, and exception responses. Real controller smoke tests commonly need multi-register reads and sometimes coil/discrete input probes.

## Acceptance Criteria

- Support multi-register reads for functions 03 and 04 over contiguous configured registers.
- Decide and document whether coils/discrete inputs belong in the next subset.
- Add exception tests for invalid quantity, gaps, unknown unit IDs, and read-only writes.
- Extend the Python/pymodbus Docker smoke to cover the new subset.
- Keep `make protocol-smoke` passing.

## Out of Scope

- Modbus RTU serial timing.
- Vendor-specific function codes.
- Full conformance certification.
