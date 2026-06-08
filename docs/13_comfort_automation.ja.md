# 13. 快適性自動化

## roomci に快適性自動化が含まれる理由

スマートホームの品質は、コマンドが実行されるかどうかだけではありません。ゲストがデバイスを操作しなくても空間が快適になるかどうかが重要です。

`roomci-comfort` は次をモデル化します。

- 温度
- 湿度
- 不快指数（discomfort index）
- 天井 vs リビングエリアのセンサー
- HVAC 自動モード
- ユーザー上書きイベント
- 部屋ごとのチューニング

## 不快指数

デフォルトの計算式:

```txt
DI = 0.81 * temperature + 0.01 * humidity * (0.99 * temperature - 14.3) + 46.3
```

## センサーモデル

```yaml
sensors:
  ceiling:
    temperature: 30.5
    humidity: 62.1
  living_area:
    temperature: 28.5
    humidity: 49.4
```

## 決定論的時系列リプレイ

Phase 17 は、決定論的 CI リプレイ用の実行可能なセンサー読み取りステップを追加します。これはハードウェアや熱シミュレーションではありません。下流の快適性自動化がエビデンスとして利用できる、センサーゾーン観測のコントラクトレベルのリプレイです。

```yaml
steps:
  - at: T
    sensor_reading:
      target: living_area
      zone: living
      temperature: 25.0
      humidity: 55.0
      occupancy: true
  - at: T+30s
    sensor_reading:
      target: ceiling_area
      zone: ceiling
      temperature: 27.0
      humidity: 50.0
      occupancy: false
assertions:
  - at: T+31s
    assert:
      comfort_timeseries: observed
```

各読み取りは、温度、湿度、在室、ゾーン、不快指数、および単純な振動フラグを含む最終状態の `comfort.<target>` を更新します。

## 快適性ターゲット

部屋ごとのターゲット:

```yaml
comfort:
  target_discomfort_index: 74
  acceptable_range:
    min: 72
    max: 76
```

## HVAC 自動制御シナリオ

```yaml
scenario:
  name: comfort_auto_mode

sensors:
  ceiling:
    temperature: 30.5
    humidity: 62.1
  living_area:
    temperature: 28.5
    humidity: 49.4

comfort:
  target_discomfort_index: 74
  acceptable_range:
    min: 72
    max: 76

steps:
  - at: T
    automation:
      type: hvac_auto_mode
      room: living
  - at: T+30m
    assert:
      target: living_area.discomfort_index
      condition: between 72 and 76
  - at: T+2h
    assert:
      target: user_override
      condition: false
```

## ユーザー上書きモデル

ユーザー上書きは、自動モードが快適性の期待と一致しない可能性を示すシグナルです。

```yaml
user_events:
  - at: T+25m
    type: hvac_manual_override
    new_temperature: 25
```

アサーション:

```yaml
assert:
  user_override_count: 0
```

## 湿度スパイクシナリオ

```yaml
faults:
  - at: T+20m
    target: living_area.humidity
    type: spike
    value: 75
    duration: 10m
```

システムは、制御アルゴリズムが振動せずに滑らかに反応するかどうかをテストする必要があります。

## レポート例

```txt
FAIL comfort_auto_mode

Comfort target:
- target DI: 74
- acceptable: 72..76

Actual:
- DI at T+30m: 78.2

User behavior:
- manual override occurred at T+35m

Possible causes:
- target DI too low/high for room
- sensor placement mismatch
- HVAC internal sensor differs from living-area sensor
- humidity spike not handled
```

## 将来: 外部データリプレイ

将来版では実際の時系列データを取り込み、CI 内で条件をリプレイできます。

```bash
roomci replay-comfort data/fukuoka_doma_2025-09.csv
```
