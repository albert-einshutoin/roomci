# アダプターコントラクトキット

アダプターコントラクトは、企業固有のプロトコル詳細を `roomci` のランタイムコードにハードコードせずに記述するためのものです。

企業が自社の MQTT topic、Modbus register map、BMS アラートフロー、edge-controller コマンド、デバイス ID モデル、認証前提、受入基準に対して `roomci` を評価したい場合に使用します。

## ファイル

- テンプレート: [`adapter-contracts/templates/company_adapter_contract.yaml`](../adapter-contracts/templates/company_adapter_contract.yaml)
- Generic MQTT 例: [`adapter-contracts/examples/generic_mqtt_edge_device.yaml`](../adapter-contracts/examples/generic_mqtt_edge_device.yaml)
- Hospitality local-first 例: [`adapter-contracts/examples/hospitality_local_first_room.yaml`](../adapter-contracts/examples/hospitality_local_first_room.yaml)
- Building automation / BMS 例: [`adapter-contracts/examples/building_automation_bms.yaml`](../adapter-contracts/examples/building_automation_bms.yaml)

## 検証

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml
```

同梱されているすべてのアダプターコントラクトを検証する:

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml
```

## 顧客から必要な入力

MQTT の場合:

- topic テンプレート
- device-id 抽出ルール
- 必須 JSON payload フィールド
- retained state の期待値
- QoS / session の期待値

Modbus の場合:

- device id と unit id
- register アドレス
- register タイプ
- scale と unit
- read/write アクセスモード

BMS / operations の場合:

- alert source
- severity
- schema version
- content type（現在は `application/json`）
- severity enum（例: `info`, `warning`, `critical`, `emergency`）
- 任意の HMAC メタデータ: header、`hmac-sha256`、secret 参照
- replay window（秒）
- notification channel
- ticket lifecycle
- acknowledgement の期待値

edge controller の場合:

- command source
- command target
- 期待される state transition

evaluation の場合:

- 非本番環境の認証前提
- pass/fail 受入基準
- 期待される report 形式

## 境界

アダプターコントラクトキット単体では、プライベートシステムとの `roomci` 互換性は実現しません。不足しているプライベート詳細を明示し、提供されたコントラクトが構造的に利用可能かを検証し、将来のアダプター dispatch 作業向けに安定した入力形式をエミュレーターに与えます。
