# 17. Docker および CI 設計

## Docker ファーストである理由

`roomci` の目的は、実機や現場環境なしにスマートホームの依存関係を再現可能にすることです。

Docker により次が可能になります。

- ローカルでの再現
- CI 実行
- サービスエミュレーション方式のローカル再現
- ホストマシンからの分離
- 決定論的な起動
- バックエンド／モバイルテストとの容易な統合

## イメージ

### MVP: 単一イメージ

```txt
ghcr.io/OWNER/roomci:latest
```

`OWNER` は公開用のプレースホルダーです。公開イメージが存在してから、実際の GitHub Container Registry 名前空間に置き換えてください。

単一イメージには次が含まれます。

- シナリオランナー
- サービスモード設定チェッカー
- MQTT 挙動シミュレーター
- エッジエミュレーター
- デバイスモック
- BMS 運用モック

### 将来: 分割イメージ

```txt
roomci-core
roomci-edge
roomci-mqtt
roomci-modbus
roomci-dali
roomci-ops
roomci-network
```

過剰設計を避けるため、最初は 1 つのイメージから始めます。

## Docker 実行例

シナリオモード:

```bash
docker run --rm \
  -v "$PWD/examples:/scenarios" \
  -v "$PWD/reports:/reports" \
  ghcr.io/OWNER/roomci:latest \
  run /scenarios/local_first_cloud_outage.yaml \
  --report-md /reports/roomci.md \
  --report-json /reports/roomci.json \
  --junit /reports/roomci.xml \
  --timeline-json /reports/roomci.timeline.json \
  --timeline-ndjson /reports/roomci.timeline.ndjson \
  --observability-json /reports/roomci.observability.json
```

サービスモード設定チェック:

```bash
docker run --rm \
  -v "$PWD/examples:/scenarios" \
  ghcr.io/OWNER/roomci:latest \
  serve --config /scenarios/local_first_cloud_outage.yaml --check
```

Serve モードのブラックボックス PoC:

```bash
make compose-poc
```

これは `roomci serve` を 1 つの Compose サービスとして起動し、`examples/controllers/http_poc_controller.sh` を別の外部コントローラーサービスとして実行します。コントローラーは HTTP のみで `roomci` と通信し、`reports/` 配下に JSON、Markdown、JUnit レポートを書き出します。

## Docker Compose パターン

```yaml
services:
  roomci-serve:
    image: ghcr.io/OWNER/roomci:latest
    command: serve --config /scenarios/generic_mqtt_retained_state.yaml --host 0.0.0.0 --port 8080 --mqtt-port 1883 --allow-non-loopback
    volumes:
      - ../examples:/scenarios:ro

  external-controller:
    image: ghcr.io/OWNER/roomci:latest
    entrypoint: ["/bin/sh"]
    command: /controllers/http_poc_controller.sh
    environment:
      ROOMCI_URL: http://roomci-serve:8080
      REPORT_DIR: /reports
    depends_on:
      - roomci-serve
    volumes:
      - ../examples/controllers:/controllers:ro
      - ../reports:/reports

  scenario-smoke:
    image: ghcr.io/OWNER/roomci:latest
    command: run /scenarios/local_first_cloud_outage.yaml --junit /reports/roomci.xml
    volumes:
      - ../examples:/scenarios
      - ../reports:/reports
```

## GitHub Actions パターン

```yaml
name: smart-home-ci

on:
  pull_request:

jobs:
  roomci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run roomci scenarios
        run: |
          docker run --rm \
            -v $PWD/examples:/scenarios \
            -v $PWD/reports:/reports \
            ghcr.io/OWNER/roomci:latest \
            run /scenarios/local_first_cloud_outage.yaml \
            --junit /reports/roomci.xml \
            --report-md /reports/roomci.md \
            --report-json /reports/roomci.json \
            --timeline-json /reports/roomci.timeline.json \
            --timeline-ndjson /reports/roomci.timeline.ndjson \
            --observability-json /reports/roomci.observability.json

      - name: Verify release evidence
        run: |
          make protocol-evidence
          make adapter-samples-smoke
          make s-tier-evidence-smoke
```

## CI 出力

必須:

- 終了コード 0/1
- Markdown サマリー
- JSON レポート
- JUnit XML
- timeline JSON
- timeline NDJSON
- observability JSON

有用:

- GitHub Step Summary
- アップロードされたアーティファクト
- シナリオごとのタイムライン
- trace/run 相関 ID
- アーティファクトレベルの observability カウンター

## レポート内容

レポートには次を含める必要があります。

- シナリオ名
- 合格／不合格
- タイムライン
- 失敗したアサーション
- ゲストへの影響
- 運用への影響
- 想定される現場原因
- 推奨確認項目
- runbook へのリンク
