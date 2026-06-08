# 外部プロトコル深度

`roomci serve` には現在、外部から駆動可能な 2 つのサーフェスがあります:

- コマンド/状態コントラクト PoC 向けの MQTT 3.1.1 サブセット
- 運用/接点 I/O PoC 向けの HTTP BMS/接点イベントエンドポイント

## MQTT サブセット境界

現在サポートされる MQTT serve サブセット:

- プロトコル名 `MQTT`
- プロトコルレベル `4`（MQTT 3.1.1）
- QoS0 `PUBLISH`
- JSON オブジェクトペイロード
- `mqtt.contracts` によるトピックマッチング
- HTTP `/state` とレポートによる保持状態の観測

現在非対応の MQTT 動作:

- QoS1/QoS2 のワイヤー確認応答
- MQTT サブスクライバーへの保持リプレイ
- サブスクリプション処理
- MQTT 5 プロパティ
- TLS、ACL、認証、クラスタリング、永続化

詳細は [`MQTT_SERVE_SUBSET.md`](MQTT_SERVE_SUBSET.md)。

## BMS / 接点の外部エンドポイント

エンドポイント:

```txt
POST /external/bms/contact
```

ペイロード:

```json
{
  "source": "contact.sauna_emergency_button",
  "state": "on",
  "severity": "critical"
}
```

必須フィールド: `source`（文字列）、`state`（文字列）。任意: `severity`（文字列）。未知のフィールドは無視されます。

### 検証レスポンス

| ボディ形状 | HTTP | `error` |
|---|---:|---|
| `source` と `state` が文字列の有効な JSON オブジェクト | 202 | （該当なし、`accepted:true` を返す） |
| `source` が欠落（または文字列以外の `source`） | 400 | `missing_source` |
| `state` が欠落（または文字列以外の `state`） | 400 | `missing_state` |
| JSON としてパースできないボディ | 400 | `invalid_json` |

`severity` は **列挙型検証されません** — 任意の文字列が受理され、サニタイズされた形式で保存されます。`application/json` の Content-Type ヘッダーは **強制されません**。ハンドラーはリクエストの Content-Type に関係なくボディをパースします。より厳密な検証は、ランタイムをライブラリ非依存に保つため Adapter Contract Kit（Phase 11 Task 03）に延期されています。

### サニタイズ

外部クライアントは `source`、`state`、`severity` に敵対的な値を渡せます。MQTT デバイス ID 名前空間との衝突防止と Markdown レンダリングの安全性のため:

- `source`、`state`、`severity` は保存前にサニタイズされます。`[A-Za-z0-9._:/-]` 外の文字は `_` に置換されます。空値は `unknown` になります。
- 外部入力から構築されたタイムラインメッセージの制御文字（改行、キャリッジリターン、その他の ASCII 制御バイト）はスペースに置換され、Markdown レポートが乗っ取られないようにします。

202 レスポンスはサニタイズされた値をエコーするため、コントローラーはサーバーが保存した内容を正確に確認できます。

### serve 状態への影響

- サニタイズされた `source` を `target` とする `event_type: external_bms_contact_observed` のタイムラインイベントを追加し、外部観測オーバーレイにキューイングします。
- 観測を `external_observations[<サニタイズされた source>]` に保存し、`GET /state` がデバイスの保持状態とは別の専用の観測可能バケットとして公開します。
- `POST /run` を跨いで存続します: 実行成功の境界でオーバーレイのタイムラインイベントは新しい `latest_report.timeline` に追加され、各観測は `external.bms.<サニタイズされた source>` キーで `latest_report.final_state` にマージされます。この時点でオーバーレイはドレインされます。

これが BMS エンドポイントをマルチステップ PoC に有用にするコントラクトです: 外部コントローラーは BMS イベントを post し、任意で `POST /run` をトリガーしても、レンダリングされた JSON/Markdown/JUnit レポートにそのイベントが残ります。

### 例クライアント

```bash
ROOMCI_URL=http://127.0.0.1:8080 examples/controllers/bms_webhook_poc_controller.sh
```

例のコントローラーは各ステップがサーバーに到達することをアサートします: 202 レスポンス、`/state` の `external_observations` バケット、`/timeline` の event_type、`/finish` の `finished:true`、レンダリングされた `latest.md` 内のイベントの存在。

## 境界

BMS/接点エンドポイントは本番 BMS webhook 実装ではありません。接点/運用イベントがシナリオモードのステップと同じタイムライン/レポートモデルを更新できることを証明する必要がある外部テストクライアント向けの PoC 入力サーフェスです。堅牢化された webhook 機能（HMAC 署名、リプレイ保護、スキーマバージョニング、リトライセマンティクス、Content-Type の強制、severity の列挙型）は Adapter Contract Kit に延期され、ランタイム自体のスコープ外です。
