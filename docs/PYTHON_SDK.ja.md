# Python Reference Client

`examples/adapters/python-http-mqtt-modbus` contains a small Python reference
client for evaluator automation.

It is not a production SDK. It does not provide auth, TLS policy, async IO,
retry budgets, version negotiation, or PyPI packaging.

## Smoke Gate

Run the reproducible Docker smoke:

```bash
make python-sdk-smoke
```

The smoke starts `roomci serve` with `examples/protocol_conformance_smoke.yaml`
and then runs the Python sample against:

- HTTP BMS/contact event submission
- MQTT 3.1.1 QoS 0 command publish
- Modbus TCP holding-register write/read
- HTTP state, timeline, finish, and latest report fetch

## Manual Run

Start `roomci serve`:

```bash
cargo run -p roomci-cli -- serve \
  --config examples/protocol_conformance_smoke.yaml \
  --mqtt-port 1883 \
  --modbus-port 1502
```

Run the sample in another shell:

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

The MQTT and Modbus examples live in `smoke.py` because they depend on
standard external client libraries, not the HTTP reference client itself.
