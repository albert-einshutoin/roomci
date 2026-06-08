# 開発者ワークフロー

このガイドは、QA contract emulator として `roomci` を評価・拡張するための実践的な手順です。

## 初回実行

```bash
cargo run -p roomci-cli -- validate examples/*.yaml
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml
```

ローカル gate 全体を実行する:

```bash
make verify
```

## アダプターコントラクト

テンプレートと同梱 example を検証する:

```bash
cargo run -p roomci-cli -- adapter validate \
  adapter-contracts/templates/company_adapter_contract.yaml \
  adapter-contracts/examples/*.yaml
```

アダプターコントラクトは、顧客固有の MQTT topic、payload schema、register map、アラート severity、認証前提、デバイス ID、受入基準に使用します。

## Python 自動化

evaluator が通常の Python テストコードから `roomci serve` を駆動したい場合は、Python reference client を使用します:

```bash
make python-sdk-smoke
```

client surface と手動コマンドは [`PYTHON_SDK.md`](PYTHON_SDK.md) を参照してください。

## シナリオデバッグ

シナリオが失敗し、著者が決定論的な実行証拠を必要とする場合は `roomci debug` を使用します:

```bash
cargo run -p roomci-cli -- debug examples/dali_scene_partial_failure.yaml \
  --debug-json reports/dali.debug.json \
  --debug-md reports/dali.debug.md
```

JSON artifact は `roomci.debug.v1` を使用し、実行順序、解決済み virtual-time ラベル、累積 state diff、assertion 結果、失敗原因、推奨チェックを含みます。

## エディターでの作成

VSCode アセットは Phase 20 から意図的に延期されています。そのタスクが実装されるまで、次を使用してください:

```bash
cargo run -p roomci-cli -- validate examples/<scenario>.yaml
cargo run -p roomci-cli -- debug examples/<scenario>.yaml
```

## CI artifact のレビュー

report、timeline、observability artifact には S Tier evidence path を使用します:

```bash
make s-tier-evidence-smoke
```

Python と debugger evidence には A Tier developer-experience gate を使用します:

```bash
make developer-experience-smoke
```

## 境界

これらのワークフローはローカル evaluator 向けです。本番 SDK、PyPI package、marketplace extension、hosted observability backend、本番 orchestration レイヤー、プロトコル認証を主張するものではありません。
