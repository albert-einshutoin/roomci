# 12. 制御パネル障害モデル

## 制御パネルをモデル化する理由

制御パネルはスマートホームの物理的な心臓部です。照明、HVAC、ロック、サウナ、風呂、センサー、リレー、エッジサーバー、電源システムを接続します。

有用なエミュレーターは、デバイスだけでなく、制御パネル周辺の障害モードもモデル化する必要があります。

## 制御パネルコンポーネント

```yaml
control_panel:
  id: CP-01
  power:
    dc24v:
      supplies:
        - id: psu_01
          status: ok
        - id: psu_02
          status: ok
      redundancy_unit: enabled
      ups:
        id: ups_24v_01
        status: ok
        battery_health: 100
      circuit_protectors:
        - id: cp_light_line_01
          status: closed
        - id: cp_sensor_line_01
          status: closed
  edge_servers:
    primary: edge_primary
    secondary: edge_secondary
```

## 障害

| 障害 | 意味 |
|---|---|
| psu_failure | 24V 電源の 1 台が故障 |
| ups_degraded | UPS バッテリー健全性が閾値未満 |
| circuit_trip | 保護回路がトリップ |
| short_circuit | 1 分岐上の短絡をシミュレート |
| edge_primary_failure | プライマリエッジサーバーが停止 |
| edge_failover_failed | セカンダリが引き継がない |
| module_offline | 制御モジュールの 1 台が到達不能になる |

実行可能な Phase 17 プロファイル:

| Target | Type | Timeline Evidence |
|---|---|---|
| `control_panel.ups` | `battery_degraded` | `control_panel_ups_degraded` |
| `control_panel.circuit_protector.<id>` | `tripped` | `control_panel_circuit_protector_tripped` |
| `control_panel.psu.<id>` | `degraded` | `control_panel_redundant_psu_degraded` |
| `edge.secondary` | `takeover_failed` | `edge_redundancy_takeover_failed` |

これらのプロファイルは QA エビデンス専用です。配線の安全性、ブレーカー動作、物理的な冗長性は検証しません。

## シナリオ: PSU 故障

```yaml
scenario:
  name: psu_redundancy

faults:
  - at: T+10s
    target: control_panel.CP-01.power.dc24v.psu_01
    type: psu_failure

assertions:
  - at: T+11s
    target: control_panel.CP-01.power.dc24v
    condition: powered
  - at: T+12s
    target: ops
    event: psu_failure_alert_sent
```

## シナリオ: 回路保護装置のトリップ

```yaml
scenario:
  name: circuit_protector_isolates_short

faults:
  - at: T
    target: circuit.light_line_01
    type: short_circuit

assertions:
  - at: T+1s
    target: circuit.light_line_01
    condition: tripped
  - at: T+1s
    target: circuit.sensor_line_01
    condition: alive
  - at: T+2s
    target: ops
    event: circuit_trip_alert_sent
```

## シナリオ: エッジフェイルオーバー

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

## レポート要件

制御パネル障害レポートには次を含める必要があります。

- 故障したコンポーネント
- 影響を受けた回路／デバイス
- ゲストへの影響
- 運用への影響
- 想定される現場確認項目
- 冗長性が機能したかどうか
- BMS アラートが発火したかどうか

例:

```txt
PASS edge_server_failover

Fault:
- edge.primary power_lost at T+10s

Redundancy:
- edge.secondary active at T+12s

Guest impact:
- none

Field checks:
- inspect primary edge server power
- verify 24V line
- check UPS/breaker status
```
