# Phase 9 Goal — Generic MQTT Contract Positioning

## Goal

Reposition `roomci` from a hospitality-focused smart-home portfolio piece into a generic MQTT / edge / building-automation QA contract emulator while preserving the NOT A HOTEL-facing depth as a hospitality domain pack.

The product should read as useful to smart-home, building-automation, facility-ops, BMS/EMS, and MQTT-based edge-device teams, not only as a NOT A HOTEL interview artifact.

## In Scope

- Update public positioning to a two-layer model:
  - Core: MQTT / edge / device QA contract emulator for CI.
  - Domain packs: hospitality smart home, building automation, BMS/ops, commissioning, access control.
- Add docs for product positioning, generic MQTT contracts, and domain packs.
- Add generic MQTT/device contract examples that are not hospitality-specific.
- Add demo targets for `demo-hospitality` and `demo-generic-mqtt`.
- Keep NOT A HOTEL interview docs as a domain-specific narrative, not the product's only identity.
- Decide the next implementation step for HTTP `serve` MVP without overclaiming MQTT broker compatibility.

## Out of Scope

- Full MQTT broker compatibility.
- Full KNX/DALI/BACnet/SIP/Modbus protocol compatibility.
- Production BMS/controller behavior.
- Vendor-specific API compatibility.

## Exit Criteria

- README describes `roomci` as a generic MQTT / edge / device QA contract emulator with hospitality examples.
- NOT A HOTEL content is preserved under interview/domain docs, not used as the only product frame.
- `docs/PRODUCT_POSITIONING.md`, `docs/GENERIC_MQTT_CONTRACTS.md`, and `docs/DOMAIN_PACKS.md` exist.
- At least two generic MQTT examples exist and run.
- `make demo-hospitality` and `make demo-generic-mqtt` pass.
- `make verify` still passes.
- Docs explicitly explain why `roomci` is not a production broker or protocol conformance suite.
