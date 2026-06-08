# Modbus TCP Serve サブセット

`roomci serve --modbus-port <port>` は、pre-adoption ビルディングオートメーション PoC テスト向けの最小 Modbus TCP endpoint を起動します。

これは production PLC、gateway、または full Modbus conformance suite ではありません。外部 Modbus TCP client が宣言済み register map を読み書きでき、`roomci` が HTTP レポート経由で state を記録するために存在します。

## サポート

- Modbus TCP MBAP header parsing。
- Protocol id `0`。
- シナリオ `modbus.devices[].unit_id` からの unit id mapping。
- Function code `0x03`: read holding registers。
- Function code `0x04`: read input registers。
- Function code `0x06`: write single holding register。
- 連続 read リクエスト向け quantity `1..=125`。
- Register address 規約:
  - 直接シナリオ address（例: `40001`）
  - 従来の zero-based Modbus client address（例: holding registers では `0` が `40001` にマップ）
  - 従来の zero-based input address（例: input registers では `0` が `30001` にマップ）
- 次による HTTP 観測:
  - `GET /state`
  - `GET /reports/latest.json`
  - `GET /reports/latest.md`
  - `GET /reports/latest.junit.xml`

## Exception Responses

| Condition | Exception |
|---|---|
| Unsupported function code | `0x01` illegal function |
| Unknown unit id or register address | `0x02` illegal data address |
| Invalid quantity, malformed request payload, or read-only write | `0x03` illegal data value |

## 非サポート

- Serial RTU 挙動または electrical timing。
- Full Modbus function-code coverage。
- Multi-register writes。
- Coils/discrete-input wire functions。
- Vendor-specific device 挙動。
- Electrical commissioning または safety validation。

## 例

Scenario:

```yaml
modbus:
  devices:
    - id: floor_heating_01
      unit_id: 1
      holding_registers:
        40001:
          name: target_temperature
          type: decimal_0_1
          value: 230
```

Serve:

```bash
roomci serve --config examples/protocol_conformance_smoke.yaml --modbus-port 5020
```

Smoke:

```bash
make protocol-smoke-modbus
```

## 実際の統合入力

この endpoint を実際のビルディングオートメーションシステムに適応するには、integrator が次を提供する必要があります。

- unit ids
- register addresses
- register access mode
- scale and unit metadata
- legal value ranges
- acceptance criteria
- fault and alert semantics
