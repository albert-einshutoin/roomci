# Hospitality Domain Pack Coverage

Hospitality is a reusable domain pack, not a private-company compatibility
claim. The canonical description now lives in
[`PRODUCT_GUIDE.md`](PRODUCT_GUIDE.md).

## Covered Today

- Local MQTT command/state behavior.
- Edge primary/secondary failover.
- Practical Modbus TCP register behavior.
- DALI-like lighting scene checks.
- Contact I/O and BMS alert evidence.
- Backup WAN and network segmentation fault profiles.
- Comfort time-series replay.
- Safe intercom/relay evidence mocks.
- Guest-impact reports.

## Mocked or Contract-Only

- Intercom and access-control events are scenario evidence only.
- BMS contact ingestion validates schema, severity, replay IDs, and signatures
  at the contract boundary.
- Company-specific topic maps, payload schemas, register maps, identity fields,
  auth/TLS expectations, and acceptance criteria must be supplied as adapter
  contracts.

## Out of Scope

- Full private-stack compatibility for any hospitality operator.
- Vendor emulators for cloud, network, intercom, access-control, BMS,
  notification, CAD/BIM, observability, or identity platforms.
- Real unlock authority, physical safety authority, or production operation.
