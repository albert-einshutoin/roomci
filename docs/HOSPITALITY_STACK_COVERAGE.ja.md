# ホスピタリティドメインパック カバレッジ

ホスピタリティは再利用可能な domain pack であり、特定企業の private compatibility claim ではありません。正本は [`PRODUCT_GUIDE.md`](PRODUCT_GUIDE.md) です。

## 現在カバーしている内容

- ローカル MQTT command/state 挙動。
- エッジ primary/secondary failover。
- 実用的な Modbus TCP register 挙動。
- DALI-like 照明シーンチェック。
- Contact I/O および BMS alert エビデンス。
- バックアップ WAN およびネットワークセグメンテーション fault profile。
- 快適性時系列 replay。
- 安全な intercom/relay エビデンス mock。
- ゲスト影響レポート。

## Mock または Contract のみ

- Intercom および access-control event は scenario evidence のみです。
- BMS contact ingestion は contract boundary で schema、severity、replay ID、signature を検証します。
- 企業固有の topic map、payload schema、register map、identity、auth/TLS expectation、acceptance criteria は adapter contract として供給する必要があります。

## スコープ外

- 特定ホスピタリティ operator の full private-stack compatibility。
- cloud、network、intercom、access-control、BMS、notification、CAD/BIM、observability、identity platform の vendor emulator。
- 実際の unlock authority、physical safety authority、production operation。
