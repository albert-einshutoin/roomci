# 05. エッジサーバーエミュレータ

## 目的

エッジサーバー / ホームコントロールサーバーエミュレータは、シミュレートされたスマートホームのローカル中枢である。

ローカル MQTT ブローカーからコマンドを受信し、デバイス固有のプロトコルアクションにマッピングし、retained 状態を更新し、local-first フォールバック挙動を処理する。

## 責務

- ローカル MQTT コマンドトピックを購読する
- コマンドをデバイスアダプターにルーティングする
- retained 状態更新を公開する
- 自動化ロジックを実行する
- クラウドが利用不可のときにローカルのみモードを処理する
- プライマリ/セカンダリのフェイルオーバーをシミュレートする
- ヘルスエンドポイントを公開する
- テスト用の任意 HTTP 制御 API を公開する

## エッジサーバーモデル

```yaml
edge:
  id: edge_main
  mode: active
  redundancy:
    enabled: true
    secondary: edge_backup
  mqtt:
    local_broker: mqtt://local-broker:1883
    cloud_broker: mqtt://cloud-broker:1883
```

## コマンドルーティング例

入力 MQTT コマンド:

```json
{
  "target": "living_scene",
  "action": "activate",
  "value": "welcome"
}
```

エッジサーバーはこれをプロトコルコマンドに展開する:

```txt
DALI fixture D411S10 -> set_level 60
DALI fixture D411S11 -> set_level 40
KNX group 1/0/1 -> write true
HVAC living -> set target_temperature 24
```

## エッジフェイルオーバー

シナリオ:

```yaml
scenario:
  name: edge_server_failover

faults:
  - at: T+10s
    target: edge.primary
    type: power_lost

assertions:
  - at: T+12s
    target: edge.secondary
    condition: active
  - at: T+15s
    target: mqtt.local
    condition: available
  - at: T+20s
    target: guest_experience
    condition: unaffected
```

## ローカルのみモード

クラウドがオフラインのとき:

- ローカル MQTT は利用可能なまま
- エッジサーバーはローカルデバイス制御を継続する
- クラウドブリッジは状態をバッファするか pending としてマークする
- 運用レポートはクラウド同期の劣化を示すが、ゲスト操作は機能し続ける

## ヘルスエンドポイント

推奨 HTTP エンドポイント:

```txt
GET  /health
GET  /edge/status
POST /edge/faults
POST /edge/recover
GET  /edge/events
```

## Lua フック（将来）

将来バージョンでは、Lua 自動化エンジンを使うホスピタリティまたはビルディングオートメーションプラットフォーム向けに、制御ロジック用 Lua フックをサポートできる。

例:

```yaml
hooks:
  on_scene_activate: scripts/welcome_scene.lua
```

Lua の例:

```lua
function on_scene_activate(ctx)
  ctx:dali_set_level("D411S10", 60)
  ctx:dali_set_level("D411S11", 40)
  ctx:modbus_write("floor_heating_01", 40001, 245)
  ctx:contact_output("relay_welcome", true)
end
```

## Go バックエンドサンプル（将来）

Go サンプルは、実デバイスが存在しないことを知らなくても、バックエンドサービスが HTTP/MQTT 経由で `roomci` と対話する方法を示すべきである。
