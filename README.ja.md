# roomci

[![CI](https://img.shields.io/badge/ci-make%20verify-blue.svg)](#quality-gates)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Coverage](https://img.shields.io/badge/coverage-84.51%25-green.svg)](#quality-gates)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

📖 **[日本語](README.ja.md)** | **[English](README.md)**

**MQTT / Edge / デバイス QA コントラクト エミュレーター for CI**

`roomci` は Docker 対応の **QA コントラクト エミュレーター**で、MQTT ベースのエッジ、スマートホーム、建築オートメーションシステム向けです。コマンド/状態コントラクト、ローカルファースト障害モード、デバイスアダプターの動作、コミッショニングチェック、オペレーション エスカレーションを再現可能なローカル環境および CI シナリオに変換します。

これは **動作エミュレーター**です：コントローラー、エッジサービス、デバイスアダプター、またはオペレーションフローが満たすべき動作を検証します。実デバイスやオンサイト環境は不要です。

設計は **LocalStack に似たエミュレーター**ですが、MQTT/edge/デバイス QA 向けです：

- LocalStack：ローカルおよび CI テスト用にクラウドサービスをエミュレート
- roomci：ローカルおよび CI テスト用にデバイス、エッジ、MQTT、オペレーション コントラクトをエミュレート

## タグライン

> エッジデバイスとスマートホームのフィールド障害をユーザーが経験する前に再現する。

## 存在意義

MQTT およびエッジデバイスの品質は、単一のコマンドが機能するかどうかではありません。状態同期、保持メッセージ、ローカルファースト制御、エッジフェイルオーバー、デバイスアダプターの動作、およびオペレーション エスカレーションが現実的な障害モードに耐えられるかどうかです。

ホスピタリティスマートホームは最も強力に含まれるドメインパックです。障害がゲストエクスペリエンスに直接影響するためです：クラウド停止、ライティング シーンドリフト、アクセス許可ドリフト、快適性自動化、BMS エスカレーション。`roomci` はこのドメイン特有の深さをモデル化しつつ、再利用可能な QA コントラクト エミュレーターのアプリケーションとしてフレーム化します。

`roomci` はこれらの障害モードを、ローカル環境、Docker、または CI で実行できる再現可能なシナリオに変換します。

## 最強のデモを試す

ローカルファースト停止シナリオを詳細なタイムラインで実行：

```bash
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --verbose
```

次に、意図的な障害レポートデモを実行：

```bash
cargo run -p roomci-cli -- run examples/dali_scene_partial_failure.yaml \
  --report-md reports/dali_scene_partial_failure.md
```

期待される結果：`roomci run` は終了コード `1` で終了し、ゲスト影響レポートを書き込みます。

レポート抜粋：

```text
Result: failed
scene_consistency:welcome
DALI-like scene consistency violation: D411S10 expected level 60, actual 0
Guest impact: Lighting scene did not match intended guest ambience.
```

完全なインタビューウォークスルーについては、[`docs/INTERVIEW_DEMO.md`](docs/INTERVIEW_DEMO.md) を参照してください。プロダクト ポジショニングについては、[`docs/PRODUCT_POSITIONING.md`](docs/PRODUCT_POSITIONING.md)、[`docs/DOMAIN_PACKS.md`](docs/DOMAIN_PACKS.md)、[`docs/GENERIC_MQTT_CONTRACTS.md`](docs/GENERIC_MQTT_CONTRACTS.md) を参照してください。
プロダクト境界については、[`docs/HOSPITALITY_STACK_COVERAGE.md`](docs/HOSPITALITY_STACK_COVERAGE.md) および [`docs/CORE_QA_JOURNEY.md`](docs/CORE_QA_JOURNEY.md) を参照してください。プロトコル サブセット主張については、[`docs/PROTOCOL_CONFORMANCE_REGISTRY.md`](docs/PROTOCOL_CONFORMANCE_REGISTRY.md) を参照してください。

評価者オンボーディングの場合は、[`docs/INTEGRATION_ONBOARDING.md`](docs/INTEGRATION_ONBOARDING.md)、[`docs/EVALUATION_EVIDENCE_PACK.md`](docs/EVALUATION_EVIDENCE_PACK.md)、[`docs/CATEGORY_READINESS.md`](docs/CATEGORY_READINESS.md) を使用してください。

## クイックスタート

```bash
# フルキュレーションデモ
make demo

# ホスピタリティスマートホームドメインデモ
make demo-hospitality

# 汎用 MQTT コントラクトデモ
make demo-generic-mqtt

# 外部コントローラーコンテナを使用したブラックボックス serve モード PoC
make compose-poc

# フル ローカル検証
make verify

# ワークスペースをビルド
cargo build --workspace --release

# シナリオを検証
cargo run --release -- validate examples/local_first_cloud_outage.yaml

# 単一シナリオを実行してレポートを生成
cargo run --release -- run examples/local_first_cloud_outage.yaml \
  --report-json reports/local_first.json \
  --report-md   reports/local_first.md \
  --junit       reports/local_first.xml

# 複数シナリオを実行して終了コードを集約
cargo run --release -- run \
  examples/local_first_cloud_outage.yaml \
  examples/modbus_floor_heating.yaml \
  examples/bms_sauna_emergency_alert.yaml \
  examples/access_permission_drift.yaml \
  examples/commissioning_checklist.yaml

# ドライラン（検証のみ、実行しない）
cargo run --release -- run --dry-run examples/comfort_auto_mode.yaml

# 詳細なタイムライン出力
cargo run --release -- run --verbose examples/edge_server_failover.yaml

# サービスモード設定チェック
cargo run --release -- serve --config examples/local_first_cloud_outage.yaml --check

# localhost HTTP コントロール/レポート API を起動
cargo run --release -- serve --config examples/generic_mqtt_retained_state.yaml --port 8080

# HTTP に外部 PoC クライアント用の最小限の MQTT 入力を追加
cargo run --release -- serve \
  --config examples/generic_mqtt_retained_state.yaml \
  --port 8080 \
  --mqtt-port 1883

# または Docker イメージを使用
docker build -t roomci:latest .
docker run --rm -v "$PWD/examples:/scenarios:ro" roomci:latest \
  run /scenarios/starlink_failover.yaml
```

## デモパス

| デモ | 対象者 | コマンド |
|---|---|---|
| キュレーション フル デモ | 一般的なプロダクト レビュー | `make demo` |
| ホスピタリティ ドメインパック | ホスピタリティとスマートホーム エンジニアリングチーム | `make demo-hospitality` |
| 汎用 MQTT コントラクト | MQTT / edge デバイス プラットフォームチーム | `make demo-generic-mqtt` |
| Serve モード PoC | 外部コントローラー統合形状を検証するチーム | `make compose-poc` |

## 合格デモシナリオ

| シナリオ | 説明 |
|---|---|
| `examples/generic_mqtt_retained_state.yaml` | 汎用 MQTT コマンド/状態コントラクトは、ホスピタリティ命名なしで保持状態を更新します。 |
| `examples/generic_mqtt_duplicate_delivery.yaml` | 重複 MQTT 配信は、保持されたコマンド/状態動作に対してべき等のままです。 |
| `examples/local_first_cloud_outage.yaml` | iPad → ローカル MQTT → エッジ → デバイスは、クラウド ブローカーがオフラインの間も機能します。保持状態は停止期間中生き残ります。 |
| `examples/edge_server_failover.yaml` | プライマリ エッジの電源が切れます。スタンバイが昇格され、次のコマンドをルーティングします。 |
| `examples/modbus_floor_heating.yaml` | 床暖房設定値は Modbus レジスターに 0.1 °C 精度で到達します。 |
| `examples/bms_sauna_emergency_alert.yaml` | サウナ過温度コンタクトが開きます。BMS は Slack、電話、チケット ランブックにエスカレートします。 |
| `examples/starlink_failover.yaml` | 設定された予算内での Starlink への WAN フェイルオーバー。 |
| `examples/comfort_auto_mode.yaml` | 不快指数が HVAC 自動モードを駆動します。ユーザーオーバーライドは尊重されます。 |
| `examples/comfort_timeseries_replay.yaml` | 決定論的なセンサーゾーン読み取りが不快指数エビデンスを更新します。 |
| `examples/access_permission_drift.yaml` | ID グループに存在しなくなった古いアクセス システム ユーザーを検出します。 |
| `examples/commissioning_checklist.yaml` | ルームとデバイス宣言からフィールド コミッショニング チェックを生成します。 |
| `examples/intercom_relay_safe_mock.yaml` | PIN/通話/リレー エビデンスは、実際のロック制御なしでキャプチャされます。 |
| `examples/network_control_panel_fault_profiles.yaml` | ネットワーク、ファイアウォール、UPS、回路、冗長 PSU 障害プロファイルが BMS エビデンスを発行します。 |

これらのシナリオは `roomci run` で合格することが想定されています。

## プロトコル プロフィール フィクスチャ

これらの B Tier フィクスチャはカスタマー提供のプロトコル マップを `contract_profile` メタデータとして検証します。これらはワイヤプロトコル エンドポイントまたは認証エビデンスではありません。

| シナリオ | 説明 |
|---|---|
| `examples/matter_gateway_profile.yaml` | Matter ゲートウェイ エンドポイント/クラスター/属性/コマンド マッピングをコントラクト エビデンスとして。 |
| `examples/bacnet_contract_profile.yaml` | BACnet オブジェクト/プロパティ/イベント マッピングをコントラクト エビデンスとして。 |
| `examples/knx_group_address_profile.yaml` | KNX グループアドレス/データポイント/機能マッピングをコントラクト エビデンスとして。 |
| `examples/opcua_contract_profile.yaml` | OPC UA ノード/属性/イベント マッピングをコントラクト エビデンスとして。 |

`make protocol-profile-smoke` で実行します。

## 障害レポート デモシナリオ

| シナリオ | 説明 |
|---|---|
| `examples/dali_scene_partial_failure.yaml` | DALI シーン有効化（1 つのフィクスチャが欠落）。`roomci run` は 0 以外で終了し、ゲスト影響障害レポートを生成します。 |

## レポート

`roomci run` は同一実行から CI フレンドリーなレポートおよびエビデンス形式を生成します：

- `--report-json <path>` — フル マシン可読実行レポート（タイムライン、アサーション、最終状態、保持 MQTT メッセージ）。
- `--report-md <path>` — ゲスト影響フレーミングを備えた人間が読み可能な Markdown サマリー。
- `--junit <path>` — CI ダッシュボード用 JUnit XML（GitHub Actions、GitLab CI、Jenkins）。
- `--timeline-json <path>` — ログ/検索/トレース取り込み用の安定タイムライン イベント エクスポート。
- `--timeline-ndjson <path>` — 改行区切りタイムライン イベント エクスポート。
- `--observability-json <path>` — 外部監視可能性取り込み用の決定論的カウンターおよび実行サマリー。
- `--run-id <id>` — JSON、タイムライン、監視可能性アーティファクトで使用される安定実行相関 ID。

`.github/workflows/smart-home-ci.yml` からの CI アーティファクトは、すべての合格デモシナリオおよび意図的な障害レポート デモのレポート、タイムライン、監視可能性形式をアップロードします。
[`docs/S_TIER_EVIDENCE_GUIDE.md`](docs/S_TIER_EVIDENCE_GUIDE.md) で コピー&ペースト評価者パスを参照してください。

Python 自動化およびシナリオ デバッグについては、[`docs/DEVELOPER_WORKFLOW.md`](docs/DEVELOPER_WORKFLOW.md) を参照してください。

Matter、BACnet、KNX、OPC UA コントラクト プロフィール境界については、[`docs/B_TIER_PROTOCOL_PROFILES.md`](docs/B_TIER_PROTOCOL_PROFILES.md) を参照してください。

## CLI リファレンス

```text
roomci run <scenarios...>
  --report-json <path>   最後のシナリオの JSON レポートを書き込む
  --report-md   <path>   最後のシナリオの Markdown レポートを書き込む
  --junit       <path>   最後のシナリオの JUnit XML レポートを書き込む
  --timeline-json <path> 最後のシナリオの安定タイムライン JSON を書き込む
  --timeline-ndjson <path>
                         安定改行区切りタイムライン JSON を書き込む
  --observability-json <path>
                         監視可能性サマリー JSON を書き込む
  --run-id <id>          エクスポートされたアーティファクトの実行相関 ID を設定
  --verbose              すべてのタイムライン イベントを出力
  --quiet                シナリオ単位の詳細を抑制
  --dry-run              検証のみ。実行しない

roomci validate <scenarios...>
  1 つ以上のシナリオ ファイルを読み込んで検証し、実行しない

roomci debug <scenario>
  --debug-json <path>    決定論的デバッグ JSON を書き込む
  --debug-md   <path>    決定論的デバッグ Markdown を書き込む

roomci serve --config <scenario> --check
  長時間実行プロセスを開始せずにサービスモード構成を検証

roomci serve --config <scenario>
  localhost バウンド HTTP コントロール/レポート API を起動

roomci serve --config <scenario> --mqtt-port <port>
  最小限の MQTT 3.1.1 CONNECT + QoS0 PUBLISH PoC 入力も起動
```

終了コード：`0`（すべてのシナリオ合格）、`1`（1 つ以上のアサーション失敗）、`2`（読み込み、検証、または実行時エラー）。

## Serve モード HTTP API

`roomci serve` はデフォルトで `127.0.0.1:8080` で起動します。`--allow-non-loopback` が渡されない限り、ループバック以外のホストを拒否します。

現在の HTTP エンドポイント：

- `GET /health`
- `GET /scenario`
- `GET /state`
- `GET /timeline`
- `GET /timeline.export.json`
- `GET /timeline.ndjson`
- `POST /fault`
- `POST /run`
- `POST /finish`
- `GET /reports/latest`
- `GET /reports/latest.json`
- `GET /reports/latest.md`
- `GET /reports/latest.junit.xml`
- `GET /observability/latest.json`

これはローカル PoC 統合サーフェスであり、プロダクション API または MQTT ブローカー置き換えではありません。

## Serve モード MQTT PoC

`roomci serve --mqtt-port <port>` は外部 PoC クライアント用の最小限の MQTT 形状の入力を有効にします。MQTT 3.1.1 `CONNECT` および JSON オブジェクト ペイロード付き QoS0 `PUBLISH` をサポートします。発行されたトピックは、シナリオ ファイル内の `mqtt.contracts` と照合され、マッチするペイロードは HTTP 状態およびレポート エンドポイントで表示される保持状態を更新します。

例：コントラクト：

```yaml
mqtt:
  contracts:
    - name: generic_device_retained_state
      adapter: mqtt_v3_qos0_subset
      command_topic: fleet/demo/site/lab/device/{device_id}/command
      state_topic: fleet/demo/site/lab/device/{device_id}/state
      device_id_from_topic: placeholder:{device_id}
      payload:
        required_fields: [online, sample_interval_seconds]
```

サポートされるサブセットは [サポートされるサブセット: [`docs/MQTT_SERVE_SUBSET.md`](docs/MQTT_SERVE_SUBSET.md) で文書化されています。統合チェックリストは [`docs/PRE_ADOPTION_POC_CHECKLIST.md`](docs/PRE_ADOPTION_POC_CHECKLIST.md) にあります。

`roomci serve --modbus-port <port>` は外部 PoC クライアント用の最小限の Modbus TCP サブセットを有効にします。MBAP、読み取りホールディング/入力レジスター、単一レジスター書き込み、および文書化された例外応答をサポートします。サポートされるサブセットは [`docs/MODBUS_TCP_SUBSET.md`](docs/MODBUS_TCP_SUBSET.md) で文書化されています。

ブラックボックス PoC パスは：

```bash
make compose-poc
```

そのコマンドは Docker Compose で `roomci serve` を起動し、別のコントローラー サービスとして `examples/controllers/http_poc_controller.sh` を実行し、HTTP API を駆動し、`reports/` の下に JSON、Markdown、JUnit レポートを書き込みます。

## 品質ゲート

`.github/workflows/smart-home-ci.yml` は以下を強制します：

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`
- `cargo doc --workspace --no-deps`（`RUSTDOCFLAGS=-D warnings`）
- `cargo tarpaulin --workspace --fail-under 80`

現在の測定値：**100 テスト** 合格、**84.51%** ライン カバレッジ。

リリースまたは会社評価ビルドを提示する前に [`docs/RELEASE_CHECKLIST.md`](docs/RELEASE_CHECKLIST.md) を使用してください。動作モデル サポートとワイヤプロトコル サポートの真実の源として [`docs/PROTOCOL_SUPPORT_MATRIX.md`](docs/PROTOCOL_SUPPORT_MATRIX.md) を使用してください。

会社固有のアダプター コントラクトは実行時作業の前に検証できます：

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml
```

評価者 PoC パックはワンコマンド ターゲットとして利用可能です：

```bash
make poc-generic-mqtt
make poc-core-qa
make poc-hospitality
make poc-building-automation
make poc-bms-ops
```

## コア概念

再利用可能なコアは QA コントラクト エミュレーターです：

- シナリオ定義
- MQTT コマンド/状態および保持状態コントラクト
- エッジ ルーティングおよびフェイルオーバー動作
- 障害インジェクション
- デバイスおよびオペレーション モデル
- JSON、Markdown、JUnit レポート
- Cargo、Docker、Compose 経由の CI 実行

ホスピタリティ スマートホームは、そのコアの上の高シグナル ドメインパックです。これらは以下を結合します：

- iPad / モバイル コントローラー
- ローカル MQTT ブローカー
- エッジ サーバー / ホーム コントロール サーバー
- クラウド MQTT ブローカーおよびクラウド イベント パイプライン
- DALI / KNX / Modbus / コンタクト I/O / HVAC / ライティング デバイス
- BMS アラート、Slack 通知、電話 エスカレーション、ランブック
- ネットワーク セグメンテーション、WAN フェイルオーバー、Starlink スタイル バックアップ パス
- コミッショニング、フィールド QA、メンテナンス、継続運用

`roomci` はこれらの依存関係を CI で再現可能にして、ユーザーまたはゲストが経験する前にチームが障害シナリオをテストできるようにします。

## 主なユースケース

1. **汎用 MQTT コントラクト QA** — コマンド/状態トピック、保持状態、重複配信、ローカル ブローカー可用性を検証します。
2. **ローカルファースト コントロール QA** — ローカル コントローラー → ローカル MQTT → エッジ → デバイス コントロールがクラウド不可の間機能することを検証します。
3. **コミッショニング QA** — フィールド コミッショニング チェックを再利用可能な YAML シナリオに変換します。
4. **建築オートメーション プロトコル シミュレーション** — DALI 型ライティング、Modbus レジスター、コンタクト I/O アラート、HVAC、KNX 型レガシー バス動作。
5. **BMS / オペレーション アラート シミュレーション** — 緊急アラートおよび復旧に対する Slack / 電話通話 / チケット / ランブック フローを検証します。
6. **ネットワーク 障害 シミュレーション** — ISP 停止、Starlink スタイル フェイルオーバー、VLAN 分離問題、ローカル のみの運用。
7. **快適性自動化 シミュレーション** — 不快指数ターゲット、HVAC 自動モード、ユーザー オーバーライド、ルーム固有の調整。

## これではないもの

`roomci` は以下を意図していません：

- プロダクション MQTT ブローカーまたは MQTT ブローカー置き換え
- プロトコル適合性スイート
- フル KNX、DALI、BACnet、Matter 実装
- プロダクション スマートホーム コントローラー置き換え
- リアル BMS プロダクト
- プロダクショングレード SIP / PBX 実装

目標は、MQTT/edge/デバイス コントラクト、スマートホーム QA、フィールド障害、コミッショニング、オペレーション フローをローカル環境および CI 環境でテスト可能にするために十分な動作をモデル化することです。

## ディレクトリ構造

```txt
roomci/
  crates/                    Rust ワークスペース
    roomci-cli/              CLI エントリポイント（バイナリ）
    roomci-core/             シナリオ ランナー（仮想時間、アサーション）
    roomci-mqtt/             ローカル + クラウド MQTT 動作モデル
    roomci-edge/             冗長 edge サーバー エミュレーター
    roomci-device-model/     Modbus、DALI ライティング、コンタクト I/O
    roomci-fault/            障害スケジューリング プリミティブ
    roomci-ops/              BMS / Slack / 電話 / ランブック エスカレーション
    roomci-report/           JSON / Markdown / JUnit レンダラー
    roomci-scenario/         YAML シナリオ ローダー + バリデーター
  examples/                  汎用 MQTT、ホスピタリティ、建築オートメーション、障害レポート デモ
  schemas/scenario.schema.json   シナリオ ファイル用 JSON Schema
  docs/                      アーキテクチャ、プロトコル、シナリオ仕様
  tasks/                     フェーズごとのビルド ログ
  .github/workflows/         CI パイプライン
  Dockerfile                 マルチステージ イメージ（バイナリのみ）
```

## ポジショニング

`roomci` は再利用可能な MQTT/edge/建築オートメーション QA コントラクト エミュレーターとしてポジショニングされています。最も強力にシップされるドメインパックはホスピタリティ スマートホームです：ローカルファースト、MQTT 駆動、エッジサーバー ベースのシステムで、デバイス制御、建築オートメーション プロトコル、フィールド コミッショニング、BMS オペレーション、ネットワーク信頼性をゲスト ステイ エクスペリエンス モデルに統合します。

プロダクトは、いかなる特定の会社のプライベート実装にも依存しません。実統合には、チームの実際の MQTT トピック、ペイロード スキーマ、デバイス/レジスター マップ、BMS/webhook コントラクト、認証モデル、および受け入れ基準が必要になります。
