# Hospitality Domain Pack Coverage

Hospitality は再利用可能な domain pack であり、特定企業の private
compatibility claim ではありません。正本は
[`PRODUCT_GUIDE.md`](PRODUCT_GUIDE.md) です。

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

- Intercom と access-control events は scenario evidence のみです。
- BMS contact ingestion は schema、severity、replay ID、signature を
  contract boundary で検証します。
- Company-specific な topic map、payload schema、register map、identity、
  auth/TLS expectation、acceptance criteria は adapter contract として
  供給する必要があります。

## Out of Scope

- 特定 hospitality operator の full private-stack compatibility。
- Cloud、network、intercom、access-control、BMS、notification、CAD/BIM、
  observability、identity platform の vendor emulator。
- Real unlock authority、physical safety authority、production operation。
