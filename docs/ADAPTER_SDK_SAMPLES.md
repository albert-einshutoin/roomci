# Adapter SDK Samples

`roomci` does not require a first-party SDK. The useful integration contract is
the public local surface:

- HTTP: `POST /external/bms/contact`
- MQTT: MQTT 3.1.1 CONNECT and QoS 0 PUBLISH/SUBSCRIBE subset
- Modbus: Modbus TCP holding-register read/write subset

The sample clients live in [`examples/adapters`](../examples/adapters/):

- Go sample: standard-library HTTP, raw MQTT packet, raw Modbus TCP request
- TypeScript sample: Node `fetch` and `net`

## Commands

```bash
cargo run -p roomci-cli -- serve examples/protocol_conformance_smoke.yaml --mqtt-port 1883 --modbus-port 1502
```

```bash
go run ./examples/adapters/go-http-mqtt-modbus
```

```bash
npx tsx examples/adapters/typescript-http-mqtt-modbus/index.ts
```

## Lua-like Hook Pseudocode

For embedded or gateway teams that use Lua-like hooks, keep the hook tiny and
delegate private mapping to an adapter contract:

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

This deliberately stays pseudocode because Lua runtimes differ by gateway,
BMS, and embedded platform. The contract fields are the portable part.
