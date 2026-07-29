# アダプターコントラクトキット

アダプターコントラクトは、企業固有のプロトコル詳細を `roomci` のランタイムコードにハードコードせずに記述するためのものです。

企業が自社の MQTT topic、Modbus register map、BMS アラートフロー、edge-controller コマンド、デバイス ID モデル、認証前提、受入基準に対して `roomci` を評価したい場合に使用します。

template を編集する前に、[評価者向けインテークキット](EVALUATOR_INTAKE_KIT.ja.md)
で顧客所有の最小入力を収集してください。公開例が示すのは文書化された roomci の
baseline であり、非公開環境や vendor との互換性ではありません。

## ファイル

- テンプレート: [`adapter-contracts/templates/company_adapter_contract.yaml`](../adapter-contracts/templates/company_adapter_contract.yaml)
- Generic MQTT 例: [`adapter-contracts/examples/generic_mqtt_edge_device.yaml`](../adapter-contracts/examples/generic_mqtt_edge_device.yaml)
- Hospitality local-first 例: [`adapter-contracts/examples/hospitality_local_first_room.yaml`](../adapter-contracts/examples/hospitality_local_first_room.yaml)
- Building automation / BMS 例: [`adapter-contracts/examples/building_automation_bms.yaml`](../adapter-contracts/examples/building_automation_bms.yaml)
- 受入基準と証跡のマッピング例: [`adapter-contracts/mappings/acceptance_evidence_mapping.yaml`](../adapter-contracts/mappings/acceptance_evidence_mapping.yaml)

## 検証

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/generic_mqtt_edge_device.yaml
```

同梱されているすべてのアダプターコントラクトを検証する:

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml
```

受入基準から証跡へのマッピングを scenario と照合する場合:

```bash
cargo run -p roomci-cli -- adapter validate \
  adapter-contracts/mappings/acceptance_evidence_mapping.yaml \
  --scenario examples/generic_mqtt_retained_state.yaml
```

## 受入基準と証跡のマッピング

既存の `acceptance.criteria` の文字列形式はそのまま利用できます。安定
ID と明示的な証跡参照が必要な criterion にだけ
`acceptance.mappings` を追加します。

```yaml
acceptance:
  criteria:
    - A command updates retained state.
  report_formats: [json, junit]
  mappings:
    - id: retained-state-synchronized
      criterion: A command updates retained state.
      assertions:
        - scenario: generic_mqtt_retained_state
          assertion: retained_state_updated
      artifacts: [json, junit]
```

参照される scenario assertion には `name` を宣言します。

```yaml
assertions:
  - at: T+1s
    name: retained_state_updated
    mqtt:
      topic: fleet/demo/site/lab/device/env_sensor_01/state
      retained:
        online: true
```

存在しない scenario/assertion、重複または安全でない ID、未対応の
artifact、`report_formats` に未宣言の artifact は検証エラーになります。
JSON 証跡では安定名を `reference_id` として保持し、Markdown/JUnit では
runtime の診断名と併記します。

`adapter validate` が保証するのは参照の解決と artifact 種別を生成できる
ことです。artifact が既に生成済みであるとは主張しません。実際の証跡は
対応する scenario を実行して作成してください。

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
