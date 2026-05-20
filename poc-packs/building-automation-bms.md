# Building Automation / BMS PoC Pack

Run:

```bash
make poc-building-automation
```

## Includes

- Adapter contract: `adapter-contracts/examples/building_automation_bms.yaml`
- Scenarios:
  - `examples/modbus_floor_heating.yaml`
  - `examples/bms_sauna_emergency_alert.yaml`
- Reports:
  - `reports/poc_building_modbus.{json,md,xml}`
  - `reports/poc_building_bms_contact.{json,md,xml}`

## Acceptance Checklist

- Adapter contract validates.
- Modbus writable registers accept expected values.
- Contact-driven critical alert produces operations evidence.
- JSON, Markdown, and JUnit reports are generated.

## Replace With Real Spec

- Device ids and unit ids.
- Register addresses, types, access modes, scales, and units.
- Contact source mappings.
- BMS severity, routing, ticket lifecycle, and acknowledgement expectations.
