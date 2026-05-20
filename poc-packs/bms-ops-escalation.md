# BMS / Operations Escalation PoC Pack

Run:

```bash
make poc-bms-ops
```

## Includes

- Adapter contracts:
  - `adapter-contracts/examples/building_automation_bms.yaml`
  - `adapter-contracts/examples/hospitality_local_first_room.yaml`
- Scenarios:
  - `examples/bms_sauna_emergency_alert.yaml`
  - `examples/access_permission_drift.yaml`
- Reports:
  - `reports/poc_bms_ops_alert.{json,md,xml}`
  - `reports/poc_bms_ops_access_drift.{json,md,xml}`

## Acceptance Checklist

- Critical alert creates notification, escalation, ticket, and runbook evidence.
- Access drift creates a guest or operations risk signal.
- Reports are usable by platform, field, and operations reviewers.

## Replace With Real Spec

- Real BMS event/webhook schema.
- Notification routing and escalation policy.
- Ticket lifecycle states.
- Access-control source of truth and stale-user criteria.
