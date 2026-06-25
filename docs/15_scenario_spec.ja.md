# 15. シナリオ仕様

## 概要

シナリオは、再現可能なスマートホーム現場状況を記述します。

シナリオは次を定義する必要があります。

- 環境（environment）
- デバイス
- ネットワーク
- MQTT ブローカー
- 外部入力
- コミッショニングメタデータ
- 運用／BMS ルール
- タイムラインステップ
- 障害（faults）
- アサーション
- レポートメタデータ

## トップレベル構造

`scenario.version` の値は SemVer として検証され、現在は `0.1` のみ受け付けます。

```yaml
version: "0.1"
scenario:
  name: local_first_cloud_outage
  description: Verify local controls survive cloud outage.
  tags: [mqtt, local-first, outage]

environment: {}
network: {}
mqtt: {}
devices: []
inputs: {}
commissioning: {}
alerts: []
faults: []
steps: []
assertions: []
report: {}
```

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

## レポートフィールド

```yaml
report:
  title: Local-first cloud outage test
  owner: smart-home
  severity_on_failure: high
  output:
    markdown: true
    json: true
    junit: true
```
