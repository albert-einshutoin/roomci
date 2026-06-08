# 09. DALI ライティング戦略

## なぜ DALI が重要か

照明は建築体験の主要な部分である。DALI ライクなシミュレーションは有用である。照明障害は明らかなシステムクラッシュではなく、品質劣化として体験されることが多いからだ。

モデル化の目標:

- フィクスチャアドレス
- グループ
- シーン
- 調光レベル
- 部分シーン障害
- コマンドスループット
- コミッショニングミス

## スコープ

v0.1 は完全な DALI ワイヤプロトコルではなく、DALI ライクな論理モデルを実装すべきである。

## フィクスチャモデル

```yaml
lighting:
  fixtures:
    - id: D411S10
      protocol: dali
      area: living
      type: line_light
      group: welcome_scene
      level: 0
    - id: D411S11
      protocol: dali
      area: living
      type: downlight
      group: welcome_scene
      level: 0
```

## シーンモデル

```yaml
scenes:
  welcome:
    fixtures:
      D411S10: 60
      D411S11: 40
      D512B01: 20
```

## 一般的な障害モード

| 障害 | 意味 |
|---|---|
| command_drop | フィクスチャがコマンドを受信しない |
| delayed_response | フィクスチャのレベル変化が遅すぎる |
| wrong_address | コマンドが誤ったフィクスチャを対象にする |
| wrong_group | フィクスチャが意図したシーンに含まれない |
| stuck_level | フィクスチャが古い調光レベルのまま |
| gateway_timeout | ブリッジ/ゲートウェイがコマンド実行に失敗する |
| throughput_limit | コマンドが速すぎる、または多すぎる |

## シーン整合性アサーション

```yaml
assert:
  scene: welcome
  consistency: complete
```

すべてのフィクスチャが期待状態に到達したかを確認する。

## デモ: 部分シーン障害

```yaml
scenario:
  name: welcome_scene_partial_failure

faults:
  - target: dali.fixture.D411S10
    type: command_drop

steps:
  - at: T
    command:
      target: scene.welcome
      action: activate
  - at: T+3s
    assert:
      scene: welcome
      consistency: complete
```

## レポート例

```txt
FAIL welcome_scene_partial_failure

Scene consistency violation:
- D411S10 expected level 60, actual 0
- D411S11 expected level 40, actual 40

Guest impact:
Medium

Possible field causes:
- wrong DALI address
- fixture not assigned to scene group
- gateway command timeout
- wiring issue
- scene mapping config mismatch
```

## KNX/DALI スループットシナリオ

一部の導入では、照明コマンドがスループットの低いバスを通過する場合がある。シナリオでシーン遷移の遅延をモデル化できる。

```yaml
faults:
  - target: lighting.bus.main
    type: throughput_limit
    commands_per_second: 20
```

アサーション:

```yaml
assert:
  scene: all_lights_off
  completed_within: 5s
```

## 将来: フロアプランマッピング

将来モジュールでは、フィクスチャをフロアプラン/CAD 座標系にマッピングできる。

```yaml
fixture_map:
  D411S10:
    x: 1200
    y: 3400
    drawing_ref: L-101
```

これにより、フィールドエンジニアがどこを確認すべきかを正確に伝えるコミッショニングレポートをサポートする。
