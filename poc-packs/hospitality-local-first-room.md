# Hospitality Local-first Room PoC Pack

Run:

```bash
make poc-hospitality
```

## Includes

- Adapter contract: `adapter-contracts/examples/hospitality_local_first_room.yaml`
- Scenarios:
  - `examples/local_first_cloud_outage.yaml`
  - `examples/edge_server_failover.yaml`
  - `examples/modbus_floor_heating.yaml`
  - `examples/bms_sauna_emergency_alert.yaml`
  - `examples/commissioning_checklist.yaml`
- Reports:
  - `reports/poc_hospitality_local_first.{json,md,xml}`
  - `reports/poc_hospitality_edge_failover.{json,md,xml}`
  - `reports/poc_hospitality_modbus.{json,md,xml}`
  - `reports/poc_hospitality_bms.{json,md,xml}`
  - `reports/poc_hospitality_commissioning.{json,md,xml}`

## Acceptance Checklist

- Local-first control remains available during upstream outage.
- Edge failover happens inside the modeled window.
- Modbus scaling and write access behave as expected.
- Critical contact creates BMS/ops evidence.
- Commissioning checks produce report artifacts.

## Replace With Real Spec

- Real NOT A HOTEL MQTT topics and payload schemas.
- Real edge command/state transitions.
- Real device/register maps and scaling rules.
- Real BMS/webhook contracts.
- Auth, network, and acceptance criteria.

This pack is NOT A HOTEL-style. It does not claim private NOT A HOTEL compatibility.
