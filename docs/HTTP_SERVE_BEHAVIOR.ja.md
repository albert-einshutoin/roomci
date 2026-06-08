# HTTP Serve 動作

`roomci serve` は外部 PoC コントローラー向けに localhost HTTP API を公開します。

この API は意図的に小さく、内部は同期的ですが、受理された各 HTTP 接続は独立して処理されるため、遅いクライアントが他のクライアントをブロックしません。

## 接続モデル

- 受理された各 HTTP 接続ごとに 1 つのワーカースレッドを spawn する。
- 実行中の HTTP 接続は 32 件で上限が設定される。
- 上限に達すると、サーバーは次の内容で `HTTP 503` を返す:

```json
{"error":"too_many_connections"}
```

## タイムアウト

受理された各 HTTP 接続には次が設定される:

- 読み取りタイムアウト: 2 秒
- 書き込みタイムアウト: 2 秒

クライアントが TCP 接続を開いて完全な HTTP リクエストを送らない場合、ワーカーは読み取りタイムアウト後にその接続を閉じる。他のクライアントは引き続き処理される。

## スコープ

これはローカル PoC ランタイムであり、本番の公開 HTTP API ではありません。外部コントローラーが `/health` を呼び、障害を注入し、`/finish` を呼び、レポートをダウンロードする短命の CI / 開発者評価フローを想定しています。

追加の外部入力エンドポイント:

- `POST /external/bms/contact` は運用 PoC 向けに BMS/接点イベント JSON を受理する。詳細は [`EXTERNAL_PROTOCOL_DEPTH.md`](EXTERNAL_PROTOCOL_DEPTH.md)。

`POST /external/bms/contact` は次の堅牢化フィールドを受理する:

- `severity`: 任意。`info`、`warning`、`critical`、`emergency` のいずれかである必要がある
- `schema_version`: 任意の文字列。エビデンスにコピーされる
- `replay_id`: 任意の文字列。同一 serve プロセス内で再利用されると `HTTP 409` で拒否される

本番 HMAC 検証は意図的にローカル serve ランタイム外です。
HMAC メタデータはアダプターコントラクトで宣言してください。

## 実行リクエスト

`POST /run` は現在のシナリオ設定をスナップショットし、シナリオ実行中は serve 状態ロックを解放し、実行後に新しいレポートを書き戻す。

`/run` リクエストは同時に 1 件だけ実行できる。別の `/run` が実行中に到着すると、`roomci` は次の内容で `HTTP 409` を返す:

```json
{"error":"run_in_progress","message":"a scenario run is already in progress"}
```

内部 serve 状態ミューテックスが poisoned の場合、HTTP ルートはリスナーを panic させず `HTTP 500` と `error: "serve_state_poisoned"` を返す。

### 外部観測オーバーレイ

`POST /run` は `latest_report` を新しい `RunReport` で置き換える。実行間（`POST /fault`、`POST /external/bms/contact`、外部 MQTT publish 経由）に発行されたタイムラインイベントがその置き換えで黙って上書きされないよう、serve ランタイムは小さな外部観測オーバーレイを保持する:

- `state.external_observation_timeline` — 前回の `/run` 以降に外部エンドポイントが記録した `TimelineEvent`。次の `/run` 成功境界でドレインされ、`latest_report.timeline` に追加される。
- `state.external_observations` — サニタイズされた source をキーにした BMS 観測。次の `/run` 成功境界でドレインされ、`external.bms.` プレフィックス付きキーで `latest_report.final_state` にマージされる。

オーバーレイはキューイング中も可視:

- `GET /state` は `external_observations` フィールド配下でオーバーレイを公開する。
- `GET /timeline` は `latest_report.timeline` とオーバーレイの和集合を返す。
- `GET /timeline.export.json` は同じ和集合を、`run_id`、`trace_id`、`span_id`、sequence フィールド付きの安定した `roomci.timeline.v1` イベントとして返す。
- `GET /timeline.ndjson` は同じ安定タイムラインイベントを改行区切り JSON として返す。
- `GET /reports/latest.json|md|junit.xml` は同じ和集合をレンダリングするため、CI コンシューマーは JSON でも人間が読める Markdown でも外部イベントを見られる。
- `GET /observability/latest.json` はイベントカウンター、アサーションカウンター、シナリオ結果、影響レベルを含む決定論的 `roomci.observability.v1` アーティファクトを返す。

トレースフィールドはローカル / CI 相関向けの決定論的アーティファクトメタデータです。OpenTelemetry 実装ではなく、本番システム間の分散トレーシングを意味しません。

### 並行外部入力

serve ランタイムは `/run`、`/state`、外部 BMS/接点入力、MQTT publish オーバーレイ、レポート取得が重なる場合でもレポートを内部整合させます。ローカル serve 状態ロックが受理する順序を超えた、独立に受理された外部リクエスト間のリアルタイム順序は保証しません。

保証されること:

- リクエストが panic したり serve 状態を poison したりしない;
- `GET /state` とレポートエンドポイントは整合したスナップショットを返す;
- 成功した `/run` 前に受理されたオーバーレイはドレインされ、そのレポートにマージされる;
- その境界後に受理されたオーバーレイは次のレポート用にキューイングされたまま残る。

保証されないこと:

- 並行外部クライアント間のウォールクロック順序;
- 本番イベントストリーム順序、exactly-once 配信、分散トランザクションセマンティクス。

## ヘルスセマンティクス

`GET /health` は serve ライフサイクル状態を返す:

| ステータス | HTTP | 意味 |
|---|---:|---|
| `idle` | 200 | サービスは初期化済みで、外部コントローラーの完了または実行を待っている |
| `running` | 200 | `POST /run` リクエストが実行中 |
| `passed` | 200 | 直近の `/run` または `/finish` が合格レポートで完了した |
| `failed` | 503 | 直近の `/run` または `/finish` が不合格レポートで完了した |

レスポンスには `scenario`、`result`、`latest_report_id`、`serve_version` も含まれる。`latest_report_id` は現在のレポート `run_id` と一致する。
