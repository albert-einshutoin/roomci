# HTTP Serve MVP プラン

## 目的

完全な MQTT 互換 serve モードが存在する前に、外部テストが駆動できる localhost バインドの仮想システムへ `roomci serve` を変換する。

この文書は Phase 10 serve モードの方向性を記録します。最初の実装ステップでは、`roomci serve --config <scenario>` から localhost バインドの HTTP 制御/レポート API を起動し、`--check` は設定を検証して終了します。

## なぜ HTTP を先にするか

完全な MQTT ブローカー互換性より先に HTTP を提供するのは、CI ハーネスにプロトコルリスクを抑えた決定論的な制御/観測サーフェスを与えるためです。

HTTP API はシナリオメタデータ、現在の状態、タイムライン、障害注入、レポートを公開でき、既存の内部ランナーが正本のままです。これにより、ブローカー互換性を主張せずに次の実装ステップが有用になります。

## 提案エンドポイント

| メソッド | パス | 目的 |
|---|---|---|
| `GET` | `/health` | エミュレーターが稼働中であること、バージョン/シナリオ ID を確認する |
| `GET` | `/scenario` | 読み込まれたシナリオメタデータと設定済みコントラクトを返す |
| `GET` | `/state` | 現在のデバイス、MQTT 保持、エッジ、運用、障害状態を返す |
| `GET` | `/timeline` | 記録されたタイムラインイベントを返す |
| `POST` | `/run` | サービスモードで設定済みシナリオを実行または進める |
| `POST` | `/fault` | 既存の `target` と `type` 形式で障害を注入する |
| `GET` | `/reports/latest` | コンテンツネゴシエーションまたは format クエリで最新レポートを返す |

有用なレポートバリアント:

```txt
GET /reports/latest?format=json
GET /reports/latest?format=markdown
GET /reports/latest?format=junit
```

現在の実装は明示的なレポートパスも公開しています:

```txt
GET /reports/latest.json
GET /reports/latest.md
GET /reports/latest.junit.xml
```

## これが可能にすること

外部テストは次を実行できます:

1. `roomci serve --config scenario.yaml` を起動する。
2. `GET /health` をポーリングする。
3. コントローラーロジックを駆動するか `POST /run` を呼ぶ。
4. `POST /fault` で障害を注入する。
5. `GET /timeline` と `GET /state` を読む。
6. CI 向けに `GET /reports/latest` を収集する。

現在のブラックボックス PoC は `make compose-poc` を使用します。Docker Compose が `roomci serve` を起動し、HTTP API だけを使う別のコントローラープロセスとして `examples/controllers/http_poc_controller.sh` を実行します。

## MQTT 互換 serve モードの境界

現在の `--mqtt-port` サーフェスは狭い PoC サブセットをサポートします: MQTT 3.1.1 `CONNECT` と JSON オブジェクトペイロード付き QoS0 `PUBLISH`。設定済み `mqtt.contracts` 経由で保持状態を更新し、結果は HTTP 状態/レポート API から観測します。

完全なブローカー互換性には依然として次が必要です:

- サポートされる MQTT サブセットの文書
- TLS、ACL、クラスタリング、完全な MQTT コンフォーマンス周りの明確な非目標
- 外部サブスクリプションが必要な場合の MQTT サブスクライバーリプレイ
- 顧客コントラクトで必要な場合の QoS1/QoS2 ワイヤーレベル動作

プロダクトの主張は「本番 MQTT ブローカー」ではなく「QA コントラクトエミュレーター」のままにすべきです。
