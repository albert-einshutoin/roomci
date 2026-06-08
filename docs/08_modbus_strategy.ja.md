# 08. Modbus 戦略

## なぜ Modbus が重要か

Modbus は MVP に含めるべきである。実ビル機器へのマッピングがよく、次をサポートするからだ:

- 床暖房制御
- 電力メーター
- バスシステム
- 詳細なエラーコード取得
- マルチドロップデバイス通信
- ベンダーやパートナーとのレジスタマップベース統合
- 既存 Modbus ツールによるフィールドデバッグ

## スコープ

v0.1 は RTU ライクなメンタルモデルを持つ **Modbus TCP モック** をサポートすべきである。

v0.1 は完全なシリアル/RS485 実装は不要だが、YAML で RTU スタイルのデバイスに対応するユニット ID とレジスタマップをモデル化できるようにすべきである。

## アドレスタイプ

4 つの Modbus データカテゴリをサポートする:

| タイプ | アクセス | 幅 | 用途 |
|---|---|---|---|
| Coils | 読み書き | 1 bit | ON/OFF コマンド |
| Discrete Inputs | 読み取り専用 | 1 bit | 状態入力 |
| Input Registers | 読み取り専用 | 16 bit | センサー値 |
| Holding Registers | 読み書き | 16 bit | 設定とコマンド |

## レジスタマップ例

```yaml
modbus:
  devices:
    - id: floor_heating_01
      host: 0.0.0.0
      port: 5020
      unit_id: 1
      holding_registers:
        40001:
          name: target_temperature
          type: decimal_0_1
          value: 245
          unit: celsius
          readable_value: 24.5
      input_registers:
        30001:
          name: current_temperature
          type: decimal_0_1
          value: 238
          unit: celsius
          readable_value: 23.8
      discrete_inputs:
        10001:
          name: error
          value: false
      coils:
        1:
          name: power
          value: true
```

## レジスタ検証

`roomci` は次を検証すべきである:

- 重複したレジスタアドレス
- 無効な型範囲
- 読み取り専用レジスタへの書き込み試行
- マルチレジスタ値のエンディアン宣言
- スケーリング定義の欠落
- パートナー向け説明の欠落

## マルチレジスタ値

例:

```yaml
40010:
  name: accumulated_power_wh
  type: uint32
  registers: 2
  byte_order: big_endian
  word_order: little_endian
```

## 障害

Modbus 固有の障害:

```yaml
faults:
  - target: modbus.floor_heating_01
    type: timeout
  - target: modbus.power_meter_01
    type: wrong_unit_id
  - target: modbus.bath_01.40021
    type: stuck_value
    value: 410
  - target: modbus.floor_heating_01
    type: exception_response
    code: illegal_data_address
```

## デモ: 床暖房

```yaml
scenario:
  name: modbus_floor_heating_0_1_degree_control

steps:
  - at: T
    modbus_write:
      device: floor_heating_01
      register: 40001
      value: 245
  - at: T+1s
    assert:
      modbus:
        device: floor_heating_01
        register: 40001
        readable_value: 24.5
```

## レポート例

```txt
PASS modbus_floor_heating_0_1_degree_control

Verified:
- Unit ID 1 responded
- Holding register 40001 accepted target temperature 24.5°C
- Input register 30001 remained readable

コミッショニングメモ:
- レジスタマップはシナリオの期待と整合している。
```
