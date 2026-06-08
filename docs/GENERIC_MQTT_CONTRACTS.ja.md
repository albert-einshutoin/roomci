# 一般的な MQTT コントラクト

## 目的

Generic MQTT シナリオは、ホスピタリティ固有の客室、ゲスト、物件命名なしで、再利用可能な `roomci` core を示します。

現在の MQTT モデルは behavioral emulator です。次の単純な command/state contract を検証します。

```txt
.../device/<device_id>/command -> .../device/<device_id>/state
```

command topic に publish された payload は、edge routing が成功した後、派生 state topic 上の retained state になります。

## 現在サポートされているコントラクト

現在サポート:

- local broker availability state
- retained command/state update 挙動
- scheduled fault による duplicate delivery idempotency
- 宣言済み device id への edge routing
- `{device_id}` 抽出付きの設定可能な `mqtt.contracts` topic mapping
- serve-mode MQTT 3.1.1 `CONNECT` + QoS0 `PUBLISH` PoC ingress
- 同一 run からの JSON、Markdown、JUnit レポート

現在非サポート:

- full MQTT broker conformance
- topic/payload mapping 向けの任意変換言語
- QoS2、session persistence、authorization、TLS、ACLs、clustering
- production broker 置き換え挙動

serve-mode のプロトコル境界は [`MQTT_SERVE_SUBSET.md`](MQTT_SERVE_SUBSET.md) を参照してください。

## 例

### Retained State

`examples/generic_mqtt_retained_state.yaml` は generic edge-device command を publish します。

```txt
fleet/demo/site/lab/device/env_sensor_01/command
```

アサーションは retained state topic を検証します。

```txt
fleet/demo/site/lab/device/env_sensor_01/state
```

実行:

```bash
cargo run -p roomci-cli -- run examples/generic_mqtt_retained_state.yaml --verbose
```

### Duplicate Delivery

`examples/generic_mqtt_duplicate_delivery.yaml` は command topic に `duplicate_delivery` fault を注入し、retained state が意味的に 1 つの最終値を保持することを検証します。

実行:

```bash
cargo run -p roomci-cli -- run examples/generic_mqtt_duplicate_delivery.yaml --verbose
```

両方の generic MQTT 例を実行:

```bash
make demo-generic-mqtt
```

## 設定可能な Topic Mapping

pre-adoption PoC 用途では、シナリオで topic mapping を宣言できます。

```yaml
mqtt:
  contracts:
    - name: generic_device_retained_state
      adapter: mqtt_v3_qos0_subset
      command_topic: fleet/demo/site/lab/device/{device_id}/command
      state_topic: fleet/demo/site/lab/device/{device_id}/state
      device_id_from_topic: placeholder:{device_id}
      payload:
        required_fields: [online, sample_interval_seconds]
```

これにより、vendor 固有の MQTT 命名をコードから切り離し、後から実際の integration contract を供給できます。
