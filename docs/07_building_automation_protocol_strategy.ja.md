# 07. ビルディングオートメーションプロトコル戦略

## 戦略

`roomci` はすべてのプロトコルを完全実装しようとすべきではない。ローカル QA、コミッショニング、障害モードテストに必要なレベルでプロトコル挙動をエミュレートすべきである。

## プロトコル優先度

| プロトコル / システム | 優先度 | 根拠 |
|---|---:|---|
| MQTT | v0.1 | ローカル/クラウド通信モデルの中核 |
| Modbus TCP | v0.1 | レジスタマップ検証と機器制御 |
| DALI-like | v0.1 | 照明シーン品質と部分障害テスト |
| Contact I/O | v0.1 | アラーム、リレー、安全、インターコム/ゲート制御 |
| KNX-like | v0.1.5 | レガシー/標準バスモデルとスループットシミュレーション |
| BACnet-like | v0.2 | HVAC/BMS スタイルのオブジェクトモデル |
| SIP / DTMF | v0.3 | インターコムとスタッフ呼び出しワークフロー |
| ONVIF-like | v0.3 | ドアカメラ / NVR イベントモデル |
| Matter / CSA Aliro | Future | ウォッチモードと将来の標準化 |
| Home Assistant discovery | Appendix | 一般的な IoT 統合、低優先度 |

## モックレベル定義

### Level 0: 概念モデル

デバイスタイプと状態のみをモデル化する。

### Level 1: コマンド/状態モデル

コマンドと状態遷移をモデル化するが、プロトコルフレームはモデル化しない。

### Level 2: プロトコルライクなインターフェース

エミュレータは実プロトコル概念に似たインターフェースを公開する。

例:

- Modbus holding registers
- DALI fixture addresses
- KNX group addresses

### Level 3: ワイヤプロトコルサブセット

エミュレータは実ワイヤプロトコルのサブセットを話す。

慎重に使用する。

### Level 4: 完全互換

v0.x の目標ではない。

## 推奨 MVP レベル

| 領域 | レベル |
|---|---:|
| MQTT | Level 3 程度（ブローカー挙動経由） |
| Modbus TCP | Level 2 または Level 3 サブセット |
| DALI | Level 2 |
| Contact I/O | Level 1 |
| KNX | Level 2 |
| BMS alerts | Level 1 |
| SIP/DTMF | 当初は Level 1 |

## なぜ早期に完全互換性を避けるのか？

完全互換性には、プロトコルの深いエッジケース、認証上の懸念、ベンダー固有の差異、広範なテストが必要になる。`roomci` の目標はスマートホーム QA の洞察を示すことであり、産業用プロトコルスタックと競合することではない。

## アダプターインターフェース

プロトコルアダプターは次を実装すべきである:

```rust
trait DeviceAdapter {
    fn apply_command(&mut self, command: Command) -> Result<DeviceEvent, DeviceError>;
    fn read_state(&self) -> DeviceState;
    fn inject_fault(&mut self, fault: Fault) -> Result<(), FaultError>;
    fn recover(&mut self, target: RecoveryTarget) -> Result<(), RecoveryError>;
}
```

## フィールドデバッグ出力

すべてのアダプターはフィールド向けのヒントを提供すべきである。

例:

```txt
DALI fixture failed to reach expected level.
Possible field causes:
- wrong fixture address
- fixture not assigned to group
- gateway timeout
- wiring issue
```
