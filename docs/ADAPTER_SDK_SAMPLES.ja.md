# アダプターSDKサンプル

`roomci` には first-party SDK は不要です。有用な統合コントラクトは、公開されているローカル surface です:

- HTTP: `POST /external/bms/contact`
- MQTT: MQTT 3.1.1 CONNECT および QoS 0 PUBLISH/SUBSCRIBE subset
- Modbus: Modbus TCP holding-register read/write subset

サンプルクライアントは [`examples/adapters`](../examples/adapters/) にあります:

- Go サンプル: standard-library HTTP、raw MQTT packet、raw Modbus TCP request
- TypeScript サンプル: Node `fetch` と `net`
- Python reference client: HTTP helper および paho-mqtt / pymodbus smoke

## コマンド

```bash
cargo run -p roomci-cli -- serve \
  --config examples/protocol_conformance_smoke.yaml \
  --mqtt-port 1883 \
  --modbus-port 1502
```

```bash
go run ./examples/adapters/go-http-mqtt-modbus
```

```bash
npx tsx examples/adapters/typescript-http-mqtt-modbus/index.ts
```

```bash
python3 examples/adapters/python-http-mqtt-modbus/smoke.py
```

Go と TypeScript は次で検証します:

```bash
make adapter-samples-smoke
```

Python は次で検証します:

```bash
make python-sdk-smoke
```

Python サンプルの詳細は [`PYTHON_SDK.md`](PYTHON_SDK.md) を参照してください。

## Lua-like Hook 疑似コード

Lua-like hook を使う embedded または gateway チーム向けに、hook は小さく保ち、プライベート mapping はアダプターコントラクトに委譲します:

```lua
local event = {
  source = "contact.sauna_emergency_button",
  state = "on",
  severity = "critical",
  schema_version = "bms.alert.v1",
  replay_id = os.time() .. "-sauna-emergency"
}

http.post("http://127.0.0.1:8080/external/bms/contact", json.encode(event), {
  ["content-type"] = "application/json"
})
```

gateway、BMS、embedded platform ごとに Lua runtime が異なるため、意図的に疑似コードのままにしています。ポータブルな部分は contract フィールドです。
