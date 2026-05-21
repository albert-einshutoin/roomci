# Adapter Samples

These samples show how an external controller can exercise `roomci serve`
without linking against roomci crates.

They are intentionally small and stay at the reference-client level:

- Go: [`go-http-mqtt-modbus/main.go`](go-http-mqtt-modbus/main.go)
- TypeScript: [`typescript-http-mqtt-modbus/index.ts`](typescript-http-mqtt-modbus/index.ts)
- Python: [`python-http-mqtt-modbus`](python-http-mqtt-modbus)

## Start roomci

Use a scenario that includes MQTT devices and Modbus registers:

```bash
cargo run -p roomci-cli -- serve \
  --config examples/protocol_conformance_smoke.yaml \
  --mqtt-port 1883 \
  --modbus-port 1502
```

## Run the Go sample

```bash
ROOMCI_HTTP=http://127.0.0.1:8080 \
ROOMCI_MQTT=127.0.0.1:1883 \
ROOMCI_MODBUS=127.0.0.1:1502 \
go run ./examples/adapters/go-http-mqtt-modbus
```

## Run the TypeScript sample

```bash
ROOMCI_HTTP=http://127.0.0.1:8080 \
ROOMCI_MQTT=127.0.0.1:1883 \
ROOMCI_MODBUS=127.0.0.1:1502 \
npx tsx examples/adapters/typescript-http-mqtt-modbus/index.ts
```

## Run the Python sample

```bash
ROOMCI_HTTP=http://127.0.0.1:8080 \
ROOMCI_MQTT=127.0.0.1:1883 \
ROOMCI_MODBUS=127.0.0.1:1502 \
python3 examples/adapters/python-http-mqtt-modbus/smoke.py
```

The samples cover:

- HTTP BMS/contact event submission with `schema_version` and `replay_id`
- MQTT 3.1.1 CONNECT plus QoS 0 PUBLISH
- Modbus TCP read holding registers

The reproducible smoke target compiles/runs both samples against
`roomci serve`:

```bash
make adapter-samples-smoke
make python-sdk-smoke
```

They are not production SDKs. Treat them as adapter contract examples and copy
only the parts that match your private topic, payload, register, and auth
requirements.
