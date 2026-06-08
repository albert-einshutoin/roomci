# 03. アーキテクチャ

## 高レベルアーキテクチャ

```txt
                +-------------------------+
                |        CI Runner        |
                | GitHub Actions / Local  |
                +-----------+-------------+
                            |
                            v
+-------------------------------------------------------------+
|                         roomci                              |
|                                                             |
|  +-------------------+      +----------------------------+   |
|  | Scenario Runner   |----->| Assertion / Report Engine  |   |
|  +-------------------+      +----------------------------+   |
|             |                                               |
|             v                                               |
|  +-------------------+      +----------------------------+   |
|  | Virtual Time      |----->| Event Timeline             |   |
|  +-------------------+      +----------------------------+   |
|             |                                               |
|             v                                               |
|  +-------------------------------------------------------+   |
|  | Emulation Modules                                     |   |
|  |                                                       |   |
|  | MQTT | Edge | Devices | BMS | Network | Comfort       |   |
|  +-------------------------------------------------------+   |
+-------------------------------------------------------------+
```

## 実行時モード

### 1. シナリオモード

シナリオファイルを実行し、合格/不合格で終了する。

```bash
roomci run examples/local_first_cloud_outage.yaml
```

CI で使用する。

### 2. サービスモード設定チェック

長時間稼働プロセスを起動せずに、シナリオをサービスモード設定として検証する。

```bash
roomci serve --config examples/local_first_cloud_outage.yaml --check
```

バックエンド/モバイルの E2E テストを長時間稼働アダプタープロセスに接続する前に使用する。

### 3. Compose スタックモード

複数のサービスコンテナを起動し、スマートホームスタック全体を再現する。

```bash
docker compose -f compose/docker-compose.yml up
```

サービスエミュレーション方式のローカル再現をデモする際に使用する。

## コアモジュール

### roomci-core

- シナリオパーサー
- 仮想時間エンジン
- イベントタイムライン
- アサーション
- レポート生成

### roomci-mqtt

- ローカルブローカーモデル
- クラウドブローカーモック
- retained メッセージ
- QoS 挙動シミュレーション
- 再接続/復旧シミュレーション

### roomci-edge

- ホームコントロールサーバーエミュレータ
- ローカルコマンドルーティング
- デバイスコマンドアダプター
- local-first フォールバック挙動

### roomci-devices

- Modbus モック
- DALI ライクな照明モック
- KNX ライクなレガシーバス
- 接点 I/O
- HVAC
- カーテン/ブラインド
- サウナ/バス

### roomci-control-panel

- 24V 電源障害
- UPS 劣化
- 回路保護器トリップ
- 冗長エッジフェイルオーバー

### roomci-network

- ローカルネットワーク分断
- WAN 障害
- ISP 劣化
- バックアップ WAN フェイルオーバー
- VLAN セグメンテーションチェック

### roomci-ops

- BMS アラート
- Slack 通知モック
- 電話エスカレーションモック
- チケット状態
- ランブックリンク
- 復旧通知

### roomci-comfort

- 温度/湿度モデル
- 不快指数計算
- HVAC 自動モード
- ユーザー上書きイベント

## データフロー: local-first 運用

```txt
iPad Controller
   |
   | MQTT command
   v
Local MQTT Broker
   |
   | retained command/state
   v
Edge Server / Home Control Server
   |
   | protocol-specific command
   v
Device Adapter
   |
   | resulting state
   v
Local MQTT Broker retained state
   |
   v
iPad Controller receives updated state
```

## データフロー: クラウドブリッジ

```txt
Local MQTT Broker
   |
   v
Edge MQTT Client / Bridge
   |
   v
Cloud MQTT Broker Mock / AWS IoT Core-like endpoint
   |
   v
Rule Router
   |
   v
SQS-like Queue
   |
   v
Cloud Run Callback Mock
```

## 設計方針

### なぜ完全なプロトコル互換性を目指さないのか？

KNX、DALI、BACnet、SIP の完全実装は、面接向け OSS MVP には大きすぎる。`roomci` は QA シナリオに十分な挙動シミュレーションを提供すべきである。

### なぜ Docker-first なのか？

対象ワークフローはローカルおよび CI テストである。Docker により、デバイス、ゲートウェイ、物理配線を必要とせず、決定論的なサービス依存関係を容易に起動できる。

### なぜ YAML シナリオなのか？

コミッショニング知識はしばしば手順的である。YAML はチェックリストを実行可能なシナリオに変換できる。

### なぜ Rust なのか？

`roomci` には次が必要であり、Rust が適している:

- 決定論的なステートマシン
- 並行ネットワークサーバー
- オーバーヘッドの小さい Docker イメージ
- 単一バイナリ配布
- イベント順序、タイムアウト、リトライ、障害の安全な処理

Rust は Go/Lua/Swift/TypeScript の本番スタックの置き換えとして位置づけない。外部 QA エンジンとして使用する。
