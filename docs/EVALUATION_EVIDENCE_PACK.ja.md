# 評価証拠パック

このパックは、evaluator に pitch 文ではなくコマンドレベルの証拠を提供します。

## すべて実行

```bash
make verify
make protocol-smoke
make adapter-samples-smoke
make s-tier-evidence-smoke
make developer-experience-smoke
make protocol-profile-smoke
make poc-report-artifact-check
make vscode-assets-check
make protocol-evidence
```

これにより、formatting、clippy、tests、docs、coverage、scenario validation、アダプターコントラクト validation、Docker image build、Docker scenario 実行、Compose scenario smoke、external-controller Compose PoC、protocol smoke、adapter sample smoke、claim evidence checker が実行されます。

現在の release-candidate evidence:

- `make verify` はローカル CI 近似のフルセットです。
- `make protocol-smoke` は standard-client による MQTT と Modbus チェックを実行します。
- `make adapter-samples-smoke` は Go と TypeScript adapter sample を HTTP、MQTT、Modbus TCP 経由で `roomci serve` に対して実行します。
- `make s-tier-evidence-smoke` は report JSON、timeline JSON、timeline NDJSON、observability JSON、run correlation、trace metadata を生成・検証します。
- `make developer-experience-smoke` は Python reference client を実行し、debugger JSON/Markdown artifact を生成します。
- `make protocol-profile-smoke` は Matter、BACnet、KNX、OPC UA の contract-profile example を検証し、scenario ごとの report artifact を書き出します。
- `make poc-report-artifact-check` は、複数 scenario ファイルと 1 つの report 出力セットを組み合わせた PoC recipe を拒否します。
- `make vscode-assets-check` は scenario 作成向けのローカル VSCode schema association、snippet、task、schema sync を検証します。
- `make protocol-evidence` は claim registry をチェックします。Phase 17 の intercom/relay、network/control-panel、BMS hardening、comfort time-series、adapter sample evidence、S Tier artifact evidence、A Tier developer experience evidence、B Tier protocol-profile evidence を含みます。

## PoC パック

| パック | コマンド | Report Artifact |
|---|---|---|
| Generic MQTT edge-device fleet | `make poc-generic-mqtt` | `reports/poc_generic_mqtt_*.{json,md,xml}` |
| Core QA scenario suite | `make poc-core-qa` | `reports/poc_core_qa_*.{json,md,xml}` |
| Hospitality smart home local-first room | `make poc-hospitality` | `reports/poc_hospitality_*.{json,md,xml}` |
| Building automation / BMS | `make poc-building-automation` | `reports/poc_building_*.{json,md,xml}` |
| BMS / operations escalation | `make poc-bms-ops` | `reports/poc_bms_ops_*.{json,md,xml}` |

生成された `reports/` ファイルは git から意図的に除外されています。

## スコアカード

| 評価レンズ | 現在の適合度 | 顧客から必要な入力 | 統合工数 | 不足している深度 | Go / No-go 基準 |
|---|---:|---|---|---|---|
| Hospitality smart-home local-first | 86/100 | MQTT topic/payload、edge 動作、Modbus map、BMS contract、auth/network 前提、受入基準 | Medium | プライベート互換性、本番 auth/TLS、vendor endpoint、本番 safety authorization | adapter フィールドが spec にきれいにマップでき、report が smart-home QA に有用なら Go |
| Generic IoT/MQTT edge-device fleet | 82/100 | topic テンプレート、必須 payload フィールド、デバイス ID、retained/QoS 期待値 | Low-medium | フル broker セマンティクス、QoS1/QoS2 wire ACK、MQTT 5 | contract-level MQTT テストと adapter sample で CI preflight に足りるなら Go |
| Building automation / Modbus / BMS | 78/100 | register map、scaling、アクセスモード、contact mapping、BMS workflow | Medium | vendor 固有デバイス、本番 BMS 統合、BACnet/KNX/OPC UA runtime profile | 動作 CI チェックなら Go。プロトコル conformance 認証なら no-go |
| Protocol-profile exploration | 70/100 | Matter endpoint/cluster map、BACnet object map、KNX group-address map、OPC UA node map、受入基準 | Medium | wire endpoint、certification、subscription、transport/security 動作 | gateway 統合前に contract-profile dry-run evidence が有用なら Go。packet-level プロトコル動作が必要なら no-go |
| CI/platform engineering | 86/100 | container policy、report 取り込み要件、CI runtime budget | Low | hosted artifact、release automation、packaged binary | `make verify`、Docker smoke、JUnit report が platform workflow に合うなら Go |

## Protocol Evidence

- MQTT serve subset: [`MQTT_SERVE_SUBSET.md`](MQTT_SERVE_SUBSET.md)
- HTTP serve behavior: [`HTTP_SERVE_BEHAVIOR.md`](HTTP_SERVE_BEHAVIOR.md)
- External protocol depth: [`EXTERNAL_PROTOCOL_DEPTH.md`](EXTERNAL_PROTOCOL_DEPTH.md)
- Protocol support matrix: [`PROTOCOL_SUPPORT_MATRIX.md`](PROTOCOL_SUPPORT_MATRIX.md)
- Adapter contract kit: [`ADAPTER_CONTRACT_KIT.md`](ADAPTER_CONTRACT_KIT.md)
- 評価者向けインテークキット: [`EVALUATOR_INTAKE_KIT.ja.md`](EVALUATOR_INTAKE_KIT.ja.md)
- B Tier protocol profiles: [`B_TIER_PROTOCOL_PROFILES.md`](B_TIER_PROTOCOL_PROFILES.md)
- Adapter SDK samples: [`ADAPTER_SDK_SAMPLES.md`](ADAPTER_SDK_SAMPLES.md)
- Python reference client: [`PYTHON_SDK.md`](PYTHON_SDK.md)
- Developer workflow: [`DEVELOPER_WORKFLOW.md`](DEVELOPER_WORKFLOW.md)
- S Tier evidence guide: [`S_TIER_EVIDENCE_GUIDE.md`](S_TIER_EVIDENCE_GUIDE.md)
- Claim evidence manifest: [`protocol-evidence.json`](protocol-evidence.json)

## 非対応機能の開示

- 本番 MQTT broker 動作。
- フル MQTT 5、QoS1/QoS2 wire セマンティクス、TLS、ACL、persistence、clustering。
- 文書化された practical subset を超える Modbus TCP server。
- フル DALI、KNX、BACnet、Matter、OPC UA、SIP、BMS 実装。
- 本物の unlock authorization、物理 relay 制御、電話、Slack 投稿、ticket 書き込み、cloud 呼び出し、電気 safety validation。
- 顧客が contract を提供しない限り、任意のプライベート顧客環境への drop-in 互換性。
- Python reference client の本番 SDK 保証または PyPI packaging。
- VSCode Marketplace extension の公開。
- ローカル VSCode authoring bundle の language server または graphical debugger 動作。

## 次の導入作業

1. 実際の顧客 spec をアダプターコントラクトに変換する。
2. 優先度の高い不足 protocol surface 向けに black-box external client を追加する。
3. 顧客固有の PoC 受入基準を追加する。
4. claim を [`PROTOCOL_SUPPORT_MATRIX.md`](PROTOCOL_SUPPORT_MATRIX.md) と同期し続ける。
5. runtime endpoint に昇格する前に、B Tier contract profile が実 evaluator に有用かを検証する。
