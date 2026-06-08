# 06. デバイスモデル

## 設計目標

デバイスモデルは QA シナリオに十分な表現力を持ちつつ、完全な物理的忠実度は目指さない。

デバイスは次を持つ必要がある:

- 安定した ID
- タイプ
- プロトコルアダプター
- 現在の状態
- コマンドインターフェース
- テレメトリ/状態トピック
- 障害モード
- フィールドメタデータ

## デバイススキーマ

```yaml
devices:
  - id: living_light_01
    type: light
    protocol: dali
    area: living
    metadata:
      fixture_address: D411S10
      floor: 1
      drawing_ref: E-201
    state:
      power: false
      brightness: 0
```

## コアデバイスタイプ

| タイプ | 目的 | MVP |
|---|---|---|
| light | 照明とシーン | Yes |
| hvac | エアコン / 暖房 | Yes |
| floor_heating | 温度制御暖房 | Yes |
| contact_input | アラーム、センサー、ドライ接点 | Yes |
| contact_output | リレー、解錠、スピーカートリガー | Yes |
| curtain | カーテン / ブラインド制御 | Should |
| sauna | サウナ温度 / 緊急アラート | Should |
| bath | バス / 冷水浴温度制御 | Should |
| power_meter | 電力量計測 | Should |
| humidity_sensor | 快適性自動化 | Should |
| temperature_sensor | 快適性自動化 | Should |
| intercom | インターコム API（将来） | Later |

## デバイス状態の原則

- 状態は JSON にシリアライズ可能であること。
- 状態は retained MQTT メッセージとして公開可能であること。
- 状態更新にはタイムスタンプを付与すること。
- デバイスエラーにはゲストへの影響とフィールドデバッグのヒントの両方を含めること。

## 例: HVAC

```yaml
- id: hvac_living
  type: hvac
  protocol: modbus
  state:
    power: true
    mode: cool
    target_temperature: 24.0
    internal_temperature: 26.0
    error_code: null
```

## 例: 接点入力

```yaml
- id: sauna_emergency_button
  type: contact_input
  protocol: dry_contact
  state:
    value: off
    criticality: emergency
    location: sauna_room
```

## 例: 接点出力

```yaml
- id: gate_relay
  type: contact_output
  protocol: relay
  state:
    value: off
    pulse_duration_ms: 1000
```

## フィールドメタデータ

フィールドメタデータは重要である。障害レポートが現場でのデバッグに役立つためだ。

推奨メタデータ:

```yaml
metadata:
  house: setouchi_villa_01
  area: living
  panel: CP-01
  terminal_block: TB-12
  drawing_ref: E-201
  runbook_url: https://example.com/runbooks/living-light
  owner_team: smart-home
```

## デバイス障害モデル

一般的なデバイス障害:

- offline
- command_drop
- delayed_response
- stale_state
- wrong_mapping
- error_code
- flapping
- power_lost
- degraded
- partial_failure

## ゲスト影響レベル

失敗したアサーションはすべて影響度を推定すべきである:

| レベル | 意味 |
|---|---|
| none | ゲストへの影響は想定されない |
| low | 軽微な遅延または劣化 |
| medium | 目立つが回復可能な障害 |
| high | ゲスト体験に大きな影響 |
| critical | 安全 / アクセス / 緊急事態の問題 |
