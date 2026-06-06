# 実機ハードウェアを Docker CI に落とし込むユースケース

このガイドは、複数のエンジニアが別々の実機環境を持っている状況で、
現場の観測結果を決定論的な Docker CI シナリオに変換する方法をまとめます。
CI に実機そのものを載せるのではなく、実機が外に出している
コントラクトを捕まえ、再現可能な動作モデルとしてテストします。

## TDD ワークフロー

1. 各エンジニアの実機環境から 1 つの動作を捕まえる：MQTT topic/payload、
   Modbus register map、dry-contact alert、network fault など。
2. 外部向けの契約を
   `adapter-contracts/examples/hardware_ci_multi_engineer_lab.yaml` に宣言する。
3. `crates/roomci-core/src/tests.rs` に、期待する replay fixture 名だけを
   先に書いた失敗テストを追加する。
4. `examples/` にシナリオを追加し、
   `cargo test -p roomci-core hardware_ci` を GREEN にする。
5. 同じユースケースを `make hardware-ci-usecases-smoke` で Docker 内実行する。
6. CI から JSON、Markdown、JUnit、timeline、observability artifact を保存する。

## ユースケースパターン

| パターン | シナリオ | 実機側の担当者 | 捕まえる面 | CI で見る信号 |
|---|---|---|---|---|
| 部屋単位の MQTT fleet replay | `examples/hardware_ci_mqtt_room_fleet.yaml` | 照明・空調エンジニア | MQTT command/state topic、retained payload、必須 field | 2 種類の部屋デバイスが決定論的な retained state を更新する |
| Modbus + BMS コミッショニング | `examples/hardware_ci_modbus_bms_commissioning.yaml` | 設備・コミッショニングエンジニア | Modbus holding/input register、dry-contact alert、ticket acknowledgement | register scale/write と critical alert workflow が 1 つの report で通る |
| Mixed-protocol regression | `examples/hardware_ci_mixed_protocol_regression.yaml` | 複数実機担当を束ねる Platform QA | MQTT gateway state、Modbus VAV setpoint、BMS smoke alert、network isolation | 複数チームの capture が 1 つの Docker run で合成できる |

## Docker CI 実行パス

専用の Docker gate は次で実行します。

```bash
make hardware-ci-usecases-smoke
```

この target は `hardware-ci-usecases` Compose service を build し、adapter
contract を検証し、3 つの scenario を validate した後、roomci container
内で各 scenario を実行します。report は `reports/` に出力されます。

GitHub Actions でも `.github/workflows/smart-home-ci.yml` の Docker scenario
suite から同じ Compose service を実行します。

## プレゼン骨子

1. 課題：実機 QA はチーム、机上環境、ラボ、顧客現場に分散し、release 前の再現が難しい。
2. 原則：CI に毎回実機を要求するのではなく、実機の contract を replay する。
3. 流れ：実機動作を capture し、失敗テストを書き、contract に落とし、Docker で replay し、report artifact を残す。
4. パターン 1：照明・空調エンジニア向けの MQTT room fleet replay。
5. パターン 2：設備エンジニア向けの Modbus/BMS commissioning replay。
6. パターン 3：Platform QA 向けの mixed-protocol regression。
7. 境界：roomci は behavioral emulator。最終確認としての hardware-in-the-loop は別レイヤーに残す。

## 非ゴール

- full wire-protocol implementation の認証ではない。
- hardware-in-the-loop や顧客現場テストを不要にするものではない。
- production credential、customer secret、private device dump を public repo に置かない。
