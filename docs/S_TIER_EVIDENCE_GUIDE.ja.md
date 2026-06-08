# S Tier エビデンスガイド

このガイドは、リリース候補の QA コントラクトエミュレーターとして `roomci` を評価するための、コピー&ペースト可能な手順です。

S Tier エビデンスとは、ホスト型可観測性、本番オーケストレーション、プロトコル認証を主張せずに、アダプター CI、タイムラインエクスポート、トレースメタデータ、可観測性アーティファクト、GitHub Actions 統合を証明できることを意味します。

## ローカルゲート

完全なローカル近似を実行する:

```bash
make verify
```

S Tier アーティファクトゲートに焦点を当てて実行する:

```bash
make s-tier-evidence-smoke
```

期待されるアーティファクト:

| アーティファクト | パス | 目的 |
|---|---|---|
| 完全 JSON レポート | `reports/phase19-smoke.json` | `schema_version` と `run_id` を含む完全な実行レポート |
| Markdown レポート | `reports/phase19-smoke.md` | 評価者向けの人間が読める要約 |
| JUnit レポート | `reports/phase19-smoke.xml` | CI テストレポート |
| タイムライン JSON | `reports/phase19-smoke.timeline.json` | 安定した `roomci.timeline.v1` イベント配列 |
| タイムライン NDJSON | `reports/phase19-smoke.timeline.ndjson` | 1 タイムラインイベント 1 行 |
| 可観測性 JSON | `reports/phase19-smoke.observability.json` | 決定論的カウンターと実行要約 |

## アダプター CI

実行:

```bash
make adapter-samples-smoke
```

Docker Compose で `roomci serve` を起動し、Go と TypeScript のアダプターサンプルを HTTP、MQTT、Modbus TCP サーフェスに対して実行します。

## 主張エビデンス

実行:

```bash
make protocol-evidence
```

`docs/protocol-evidence.json` の主張が、実際のドキュメント、例、テスト、リリースゲートコマンドを指していることを検証します。

## GitHub Actions

リポジトリワークフロー `.github/workflows/smart-home-ci.yml` は次を実行します:

- Rust 品質ゲート
- Docker シナリオスイート
- Phase 17 シナリオアーティファクト
- タイムライン JSON / NDJSON アーティファクト
- 可観測性 JSON アーティファクト
- プロトコル/ドメインエビデンスチェック
- アダプターサンプルスモーク

ワークフローは `reports/` をアーティファクトとしてアップロードします。

## アーティファクトコントラクト

タイムラインイベントは `roomci.timeline.v1` を使用します:

```json
{
  "schema_version": "roomci.timeline.v1",
  "run_id": "phase19-smoke",
  "scenario_name": "local_first_cloud_outage",
  "trace_id": "roomci-trace-phase19-smoke",
  "span_id": "roomci-span-phase19-smoke-000000",
  "parent_span_id": null,
  "sequence": 0,
  "at": "T+0s",
  "event_type": "event",
  "target": null,
  "message": "..."
}
```

可観測性アーティファクトは `roomci.observability.v1` を使用します:

```json
{
  "schema_version": "roomci.observability.v1",
  "run_id": "phase19-smoke",
  "scenario_name": "local_first_cloud_outage",
  "result": "passed",
  "timeline_event_count": 7,
  "assertion_count": 2,
  "failed_assertion_count": 0,
  "events_by_type": {},
  "assertions_by_status": {
    "passed": 2,
    "failed": 0
  },
  "impact_levels": []
}
```

## 境界

これらのアーティファクトはエクスポートコントラクトのみです。`roomci` は Grafana、InfluxDB、Prometheus、OpenTelemetry Collector、Jaeger、本番アラート、本番オーケストレーション、本番インシデント対応システムを実行しません。
