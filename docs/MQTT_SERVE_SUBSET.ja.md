# MQTT Serve サブセット

`roomci serve --mqtt-port <port>` は、pre-adoption PoC テスト向けの最小 MQTT 3.1.1 ingress を起動します。

これは production broker でも conformance suite でもありません。外部 controller が実際の MQTT 形状の TCP endpoint 経由で command payload を publish でき、`roomci` が既存 HTTP state/report API 経由で retained-state 挙動を記録するために存在します。

この MQTT ingress と併用する HTTP 観測 API は [`HTTP_SERVE_BEHAVIOR.md`](HTTP_SERVE_BEHAVIOR.md) に記載されています。

## サポート

- MQTT 3.1.1 `CONNECT`
- 成功応答の `CONNACK`
- QoS 0 `PUBLISH`
- 設定済み state topics 向け QoS 0 `SUBSCRIBE` / `SUBACK`
- 一致する MQTT subscriber への retained message replay
- `PINGREQ` / `PINGRESP`
- `DISCONNECT`
- UTF-8 topic names
- JSON object payloads
- 設定済み command/state topic mapping における 1 つの `{device_id}` placeholder
- MQTT replay および次による retained-state 観測:
  - `GET /state`
  - `GET /timeline`
  - `GET /reports/latest.json`
  - `GET /reports/latest.md`
  - `GET /reports/latest.junit.xml`

## 非サポート

- QoS 1 / QoS 2 の wire-level acknowledgements
- `UNSUBSCRIBE`
- 正確に設定された state topics と `+` placeholder を超える wildcard 一般性
- sessions、will messages、keepalive enforcement
- TLS、ACLs、authentication、clustering、persistence
- MQTT 5 properties

## CONNECT 検証

MQTT ingress が受け付けるのは次のみです。

- protocol name: `MQTT`
- protocol level: `4` (MQTT 3.1.1)

サポート外の protocol name または level は `CONNACK` return code `0x01` (`unacceptable protocol version`) を受け取り、TCP 接続はクローズされます。例: MQTT 3.1 の `MQIsdp` および MQTT 5 の protocol level `5` は拒否されます。

## Contract Config

シナリオファイルで MQTT connection contracts を宣言できます。

```yaml
mqtt:
  local:
    retained: true
  contracts:
    - name: generic_device_retained_state
      adapter: mqtt_v3_qos0_subset
      command_topic: fleet/demo/site/lab/device/{device_id}/command
      state_topic: fleet/demo/site/lab/device/{device_id}/state
      device_id_from_topic: placeholder:{device_id}
      payload:
        required_fields: [online, sample_interval_seconds]
```

検証では、サポート外 adapter、欠落 topic mapping、重複 command mapping、サポート外 device-id 抽出戦略を拒否します。

## 実際の統合入力

この PoC を実際の顧客または vendor に適応するには、integrator が次を提供する必要があります。

- command topics
- state topics
- payload schemas および required fields
- QoS、retained、reconnect、session の期待値
- auth/TLS/network の前提
- pass/fail acceptance criteria
- Modbus が関与する場合は Modbus register maps
- 運用フローが関与する場合は BMS webhook/API contracts

実際の integration contract なしに、特定組織の互換性を謳いません。
