# 15. シナリオ仕様

## 概要

シナリオは、再現可能なスマートホーム現場状況を記述します。

シナリオは次を定義する必要があります。

- デバイス
- WAN・edge モデル設定
- MQTT ブローカー
- 外部入力
- コミッショニングメタデータ
- 運用／BMS ルール
- タイムラインステップ
- 障害（faults）
- アサーション
- 説明用メタデータ（`scenario.description`、`scenario.tags`）

## トップレベル構造

トップレベルの `version` を検証し、現在は `0.1` 系のみ受け付けます。

```yaml
version: "0.1"
scenario:
  name: local_first_cloud_outage
  description: Verify local controls survive cloud outage.
  tags: [mqtt, local-first, outage]

mqtt: {}
devices: []
inputs: {}
commissioning: {}
alerts: []
faults: []
steps: []
assertions: []
```

トップレベル、`scenario`、fault の未知キーはパース時に拒否します。従来の
`environment`、`network`、`future_milestone`、`report`、`scenario.clock`、
fault の `severity` は実行を制御しておらず、現在は受け付けません。実行用
シナリオから削除してください。説明には `scenario.description` と
`scenario.tags`、レポート形式の指定には CLI フラグを使います。自由形式の
MQTT payload やデバイス状態マップ内の同名キーはデータとして引き続き受理します。
任意セクションの省略時の動作は従来どおりです。

## 時間モデル

記号的な相対時間を使用します。

```txt
T
T+1s
T+5m
T-30m
```

シナリオランナーはこれを仮想時間に変換します。

## 障害宣言

内部モデルでは global と step 内の fault `duration` による時間回復は未対応です。
整数に `ms`、`s`、`m`、`h` を付けた値は場所付きの
`UnsupportedFaultDuration` で実行前に拒否します。書式不正の文字列は
`InvalidDuration`、明示的な `null` や文字列以外はパースエラーです。
`duration` を省略した fault は従来どおり持続し、シリアライズと
`GET /scenario` の出力でもキーを省略するため、再読込・検証できます。
`mqtt.local.enabled: false`、`mqtt.*.retained: false`、未知の構造化された
`mqtt` キーも拒否します。実broker試験の契約は
[外部MQTT復旧試験](EXTERNAL_MQTT_RECOVERY.md)を参照してください。

障害はグローバルに宣言できます。

```yaml
faults:
  - at: T+10s
    target: mqtt.cloud
    type: offline
```

またはステップ内に記述できます。

```yaml
steps:
  - at: T+10s
    fault:
      target: mqtt.cloud
      type: offline
```

## アサーション

アサーションは次をサポートする必要があります。

- デバイス状態
- MQTT 保持メッセージ
- 運用通知
- ネットワーク到達性
- 快適性メトリクス
- アクセス制御ドリフト
- コミッショニングチェックリスト生成
- チケット状態
- ゲストへの影響

例:

```yaml
assertions:
  - at: T+20s
    target: guest_experience
    condition: unaffected
```

アクセス制御ドリフトのアサーションは `inputs.identity_group` と `inputs.access_system_group` を比較し、シナリオが意図的に古いアクセスユーザーを検出した場合に合格します。

```yaml
inputs:
  identity_group:
    - alice@example.com
  access_system_group:
    - alice@example.com
    - retired@example.com

assertions:
  - at: T
    assert:
      access_control_drift: detected
```

コミッショニングチェックリストのアサーションは、宣言された部屋デバイスをカウントし、現場確認項目を生成できる場合に合格します。

```yaml
commissioning:
  site: minakami
  rooms:
    - id: living
      devices:
        - D411S10
        - floor_heating_01

assertions:
  - at: T
    assert:
      commissioning_checklist: generated
```

## 例: ローカルファーストシナリオ

```yaml
version: "0.1"
scenario:
  name: local_first_cloud_outage
  tags: [mqtt, local-first]

mqtt:
  local:
    retained: true
  cloud:
    enabled: true

devices:
  - id: living_light
    type: light
    protocol: dali
    state:
      power: false
      brightness: 0

faults:
  - at: T+10s
    target: mqtt.cloud
    type: offline

steps:
  - at: T+15s
    mqtt_publish:
      client: ipad_controller
      topic: house/minakami/room/living/device/living_light/command
      payload:
        power: true
        brightness: 60

assertions:
  - at: T+16s
    mqtt:
      topic: house/minakami/room/living/device/living_light/state
      retained:
        power: true
        brightness: 60
  - at: T+20s
    guest_experience: unaffected
```

## シナリオタグ

推奨タグ:

```txt
mqtt
local-first
modbus
dali
bms
ops
network
comfort
commissioning
control-panel
intercom
access-control
```

## レポート出力

形式は `roomci run` の CLI フラグ（`--markdown`、`--json`、`--junit`、
`--report-dir`）で指定します。シナリオ YAML 内のレポート設定は未対応です。
