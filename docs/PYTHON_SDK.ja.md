# Python Reference Client

`examples/adapters/python-http-mqtt-modbus` には、evaluator 自動化向けの小さな Python reference client が含まれています。

これは本番 SDK ではありません。auth、TLS policy、async IO、retry budget、version negotiation、PyPI packaging は提供しません。

## Smoke Gate

再現可能な Docker smoke を実行する:

```bash
make python-sdk-smoke
```

smoke は `examples/protocol_conformance_smoke.yaml` で `roomci serve` を起動し、次に対して Python サンプルを実行します:

- HTTP BMS/contact event 送信
- MQTT 3.1.1 QoS 0 command publish
- Modbus TCP holding-register write/read
- HTTP state、timeline、finish、latest report の取得

## 手動実行

`roomci serve` を起動する:

```bash
cargo run -p roomci-cli -- serve \
  --config examples/protocol_conformance_smoke.yaml \
  --mqtt-port 1883 \
  --modbus-port 1502
```

別シェルでサンプルを実行する:

```bash
ROOMCI_HTTP=http://127.0.0.1:8080 \
ROOMCI_MQTT=127.0.0.1:1883 \
ROOMCI_MODBUS=127.0.0.1:1502 \
python3 examples/adapters/python-http-mqtt-modbus/smoke.py
```

## Client Surface

```python
from roomci_sdk import RoomciClient

client = RoomciClient("http://127.0.0.1:8080")
client.health()
client.state()
client.timeline()
client.post_bms_contact("contact.sauna_emergency_button", "on")
client.finish()
client.latest_report_json()
```

MQTT と Modbus の例は `smoke.py` にあります。これらは HTTP reference client 自体ではなく、標準的な外部 client library に依存するためです。
