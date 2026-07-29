# 統合オンボーディング

このパスは、自分のリポジトリで `roomci` を評価する platform、IoT、smart-home、building-automation エンジニア向けです。

公開例の値を置き換える前に、[評価者向けインテークキット](EVALUATOR_INTAKE_KIT.ja.md)
を完了してください。不足している顧客所有情報は `blocked` のままとし、公開 fixture
から推測しません。

## 30 秒パス（clone 不要）

リリース済みの `roomci` バイナリをインストールした後、現在のリポジトリに実行可能な
retained-state smoke scenario を生成します。

```bash
roomci init --ci github
roomci validate roomci/smoke.yaml
roomci run roomci/smoke.yaml --verbose
```

任意の workflow は `albert-einshutoin/roomci@v0.1.1` を使い、report を
`roomci-reports` に出力します。`init` は全生成対象を事前確認し、どれか一つでも
既存なら何も書かずに案内付きで終了します。すべての生成物を置き換える意思がある場合
だけ `--force` を使ってください。生成する VS Code YAML 設定は既存設定とマージせず、
リポジトリ設定を暗黙に変更しないようにしています。

バイナリのインストールとアップグレード確認は [`RELEASING.ja.md`](RELEASING.ja.md) を
参照してください。

## 15 分パス

1. PoC パックを選ぶ:
   - Generic MQTT: `make poc-generic-mqtt`
   - Hospitality local-first: `make poc-hospitality`
   - Building automation / BMS: `make poc-building-automation`
   - BMS / operations: `make poc-bms-ops`
2. `adapter-contracts/examples/` 配下で、そのパックが使うアダプターコントラクトを確認する。
3. `adapter-contracts/templates/company_adapter_contract.yaml` をコピーする。
4. topic テンプレート、device id、register map、alert route、auth 前提、受入基準を、実際の非本番 spec に置き換える。
5. コントラクトを検証する:

```bash
cargo run -p roomci-cli -- adapter validate path/to/your_adapter_contract.yaml
```

6. HTTP/MQTT PoC surface を起動する:

```bash
cargo run -p roomci-cli -- serve --config examples/generic_mqtt_retained_state.yaml --port 8080 --mqtt-port 1883
```

7. controller、test client、または script から駆動する。
8. 次を収集する:
   - `GET /state`
   - `GET /timeline`
   - `GET /reports/latest.json`
   - `GET /reports/latest.md`
   - `GET /reports/latest.junit.xml`

## HTTP Serve API

| Method | Path | 目的 |
|---|---|---|
| `GET` | `/health` | `idle`、`running`、`passed`、`failed` ステータスを読み取る |
| `GET` | `/scenario` | 読み込まれた scenario メタデータを確認する |
| `GET` | `/state` | 現在の emulator state を確認する |
| `GET` | `/timeline` | 発行された event を確認する |
| `POST` | `/fault` | 実行中 state に fault を注入する |
| `POST` | `/run` | 読み込まれた scenario を実行する |
| `POST` | `/finish` | 確定し latest report を作成する |
| `GET` | `/reports/latest.json` | JSON report |
| `GET` | `/reports/latest.md` | Markdown report |
| `GET` | `/reports/latest.junit.xml` | JUnit report |

timeout、overload、health、run-lock セマンティクスは [`HTTP_SERVE_BEHAVIOR.md`](HTTP_SERVE_BEHAVIOR.md) を参照してください。

## トラブルシューティング

| 症状 | 想定原因 | 対処 |
|---|---|---|
| Port already in use | HTTP または MQTT port の競合 | `--port 0`、`--mqtt-port 0` を使うか、別 port を選ぶ |
| MQTT publish is ignored | topic が `mqtt.contracts[].command_topic` と一致しない | adapter/scenario の topic テンプレートと `{device_id}` placeholder を更新する |
| MQTT publish is rejected | JSON payload が不正または必須 field が不足 | アダプターコントラクトの `payload.required_fields` に合わせる |
| `CONNACK` returns `0x01` | MQTT client の protocol name が `MQTT`、level が `4` でない | 現在の serve subset では MQTT 3.1.1 を使用する |
| Modbus request returns exception `0x02` | unit id または register address が scenario map にない | `modbus.devices[].unit_id` と register address を更新する |
| Modbus write returns exception `0x03` | register が read-only、または value/quantity が subset 外 | 設定済み holding register に 1 value だけ書き込む |
| `/health` returns HTTP 503 | latest report が failed | 失敗証拠として `/reports/latest.md` と `/timeline` を読む |
| Docker/Compose fails | Docker daemon、image build、または port の問題 | `docker build -t roomci:verify .` を実行し、続けて `make compose-poc` |
| Report seems too narrow | scenario の assertion が少なすぎる | アダプターコントラクトに受入基準を追加し、scenario assertion にマップする |

## Client Snippet

HTTP health:

```bash
curl -fsS http://127.0.0.1:8080/health
```

fault を注入する:

```bash
curl -fsS -X POST http://127.0.0.1:8080/fault \
  -H 'content-type: application/json' \
  -d '{"target":"mqtt.cloud","type":"offline"}'
```

Markdown evidence を収集する:

```bash
curl -fsS http://127.0.0.1:8080/reports/latest.md > reports/latest.md
```
