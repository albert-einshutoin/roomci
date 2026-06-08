# 04. Local-first MQTT アーキテクチャ

## なぜ MQTT が中核なのか

`roomci` は MQTT を二次的なアダプターではなく、主要な通信基盤としてモデル化すべきである。

local-first スマートホームアーキテクチャが MQTT を使う理由:

- ローカルデバイスとコントローラはクラウド可用性なしで動作できる
- retained メッセージが状態同期を簡素化する
- QoS 1 が少なくとも1回の配信をサポートし、リトライ挙動をプロトコル層に移す
- 再接続するクライアントが最新の既知状態を復旧できる
- ローカルとクラウドのブリッジパターンがテストしやすい

## 目標トポロジ

```txt
+-------------------+        MQTT         +---------------------+
| iPad Controller   | <-----------------> | Local MQTT Broker   |
+-------------------+                     +----------+----------+
                                                     |
                                                     | MQTT subscribe/publish
                                                     v
                                          +---------------------+
                                          | Edge Server         |
                                          | Home Controller     |
                                          +----------+----------+
                                                     |
                                                     | Device protocols
                                                     v
                                     +-------------------------------+
                                     | DALI / KNX / Modbus / Contact |
                                     +-------------------------------+

Local MQTT Broker
   |
   | bridge / outbound client
   v
Cloud MQTT Broker Mock / AWS IoT Core-like endpoint
```

## MQTT トピック規約

推奨デフォルトのトピック形式:

```txt
house/{house_id}/room/{room_id}/device/{device_id}/command
house/{house_id}/room/{room_id}/device/{device_id}/state
house/{house_id}/room/{room_id}/scene/{scene_id}/command
house/{house_id}/room/{room_id}/scene/{scene_id}/state
house/{house_id}/bms/alert/{alert_id}
house/{house_id}/ops/ticket/{ticket_id}
```

## Retained 状態モデル

すべてのデバイスは retained 状態トピックを公開すべきである。

例:

```json
{
  "device_id": "living_light_01",
  "power": true,
  "brightness": 60,
  "updated_at": "2026-01-01T00:00:00Z"
}
```

シナリオアサーション:

```yaml
assert:
  mqtt:
    topic: house/minakami/room/living/device/living_light_01/state
    retained:
      power: true
      brightness: 60
```

## QoS 1 重複シミュレーション

MQTT QoS 1 はメッセージが複数回配信されうることを意味する。デバイスロジックとエッジルーティングは重複に安全であるべきである。

シナリオ例:

```yaml
faults:
  - at: T+1s
    target: mqtt.local
    type: duplicate_delivery
    topic: house/minakami/room/sauna/device/sauna_01/command
    count: 2
```

期待される挙動:

- デバイスコマンドは意味的にべき等である
- 副作用が二重カウントされない
- 最終的な retained 状態が正しい

## 再接続復旧

シナリオ:

```yaml
steps:
  - at: T
    client: ipad_controller
    disconnect: true
  - at: T+10s
    mqtt_publish:
      topic: house/minakami/room/living/device/light/state
      retain: true
      payload:
        power: true
  - at: T+20s
    client: ipad_controller
    reconnect: true
  - at: T+21s
    assert:
      client: ipad_controller
      received_retained_state:
        topic: house/minakami/room/living/device/light/state
        payload:
          power: true
```

## クラウド障害シナリオ

クラウド MQTT がオフラインでも、コアのゲスト体験は継続すべきである。

```yaml
scenario:
  name: local_first_cloud_outage

faults:
  - at: T+10s
    target: mqtt.cloud
    type: offline

steps:
  - at: T+15s
    mqtt_publish:
      client: ipad_controller
      topic: house/minakami/room/living/device/light/command
      payload:
        power: true
  - at: T+16s
    assert:
      mqtt:
        topic: house/minakami/room/living/device/light/state
        retained:
          power: true
  - at: T+20s
    assert:
      guest_experience: unaffected
```

## クラウドブリッジモデル

クラウドブリッジは次をモデル化すべきである:

- ローカルからクラウドへの状態転送
- クラウドからローカルへのコマンド転送
- フィードバック用の永続キュー
- 損失/リトライ挙動
- クラウド復旧後の遅延同期

これは AWS IoT Core 完全互換でなくても、AWS IoT Core ライクにできる。
