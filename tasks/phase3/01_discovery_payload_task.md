# Task 01 — Discovery Payload Generator

## Objective

Generate Home Assistant MQTT Discovery-like config payloads from roomci devices.

## Implementation Scope

- Add adapter module for HA-like discovery.
- Implement component mapping:
  - `smart_lock -> lock`
  - `light -> light`
  - `climate -> climate`
  - `cover -> cover`
  - `occupancy_sensor -> binary_sensor`
  - `motion_sensor -> binary_sensor`
  - `temperature_sensor -> sensor`
  - `humidity_sensor -> sensor`
- Include state, command, and availability topic fields.

## Acceptance Criteria

- Payloads include stable `unique_id`.
- Topic references match Phase 2 topic builders.
- Unsupported devices produce explicit skipped output, not malformed config.

## References

- `docs/04_protocol_adapters.md`
- `docs/08_adapter_implementation_plan.md`
