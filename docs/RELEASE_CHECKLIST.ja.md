# リリースチェックリスト

`roomci` を OSS プロダクトとして提示する前、または企業に評価を依頼する前に、このチェックリストを使用してください。

## 必須 Gate

| Gate | コマンド / 証拠 | 期待結果 |
|---|---|---|
| Clean checkout build | `cargo metadata --format-version 1 --no-deps` | workspace が `roomci-serve` を含むすべての crate を解決する |
| Rust formatting | `cargo fmt --all --check` | formatting diff なし |
| Rust linting | `cargo clippy --workspace --all-targets -- -D warnings` | warning なし |
| Rust tests | `cargo test --workspace --all-targets` | すべての workspace test が pass |
| Docs build | `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` | API docs が warning なしで build される |
| Coverage floor | `cargo tarpaulin --workspace --engine llvm --fail-under 80` | coverage が 80% 以上を維持する |
| Scenario validation | `cargo run -p roomci-cli -- validate examples/*.yaml` | 公開 example が validate される |
| Adapter contract validation | `cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml` | adapter template と example contract が validate される |
| Scenario run reports | `cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --report-json reports/local_first.json --report-md reports/local_first.md --junit reports/local_first.xml` | JSON、Markdown、JUnit report が生成される |
| Docker image | `docker build -t roomci:verify .` | repository ファイルから image が build される |
| Docker scenario run | `docker run --rm -v "$PWD/examples:/scenarios:ro" roomci:verify run /scenarios/local_first_cloud_outage.yaml` | container 化 scenario が pass |
| Compose PoC | `make compose-poc` | external controller が HTTP 経由で `roomci serve` を駆動し report を書き出す |
| MQTT protocol smoke | `make protocol-smoke-mqtt` | standard MQTT client/library path が `roomci serve` に publish し、HTTP report 経由で state を観測する |
| Modbus protocol smoke | `make protocol-smoke-modbus` | Modbus TCP client path が選択 register を read/write し、HTTP report 経由で state を観測する |
| Adapter sample smoke | `make adapter-samples-smoke` | Go と TypeScript adapter sample が HTTP、MQTT、Modbus TCP 経由で `roomci serve` に対して実行される |
| Python SDK smoke | `make python-sdk-smoke` | Python reference client が HTTP、MQTT、Modbus TCP 経由で `roomci serve` を駆動する |
| Developer experience smoke | `make developer-experience-smoke` | Python SDK smoke と `roomci debug` JSON/Markdown artifact が pass |
| Protocol profile smoke | `make protocol-profile-smoke` | Matter、BACnet、KNX、OPC UA contract-profile example と dry-run scenario が validate される |
| VSCode authoring assets | `make vscode-assets-check` | ローカル editor package JSON、snippet、task、schema copy、command reference が validate される |
| S Tier evidence smoke | `make s-tier-evidence-smoke` | JSON report、timeline JSON、timeline NDJSON、observability JSON、run id、trace metadata が生成・検証される |
| Protocol evidence map | `make protocol-evidence` | 検証済み protocol claim が command と non-goal doc にマップされる |
| Full local CI approximation | `make verify` | Docker と Compose チェックを含む CI と同じ practical gate set を実行する |

## 生成 Artifact

- `reports/` は git から意図的に除外されている。
- `target/` は git から意図的に除外されている。
- `make verify`、`make compose-poc`、`docker-demo` で作成された Docker image はローカル build artifact である。

## 公開 Claim ルール

- README badge は実際の repository gate または静的 local-truth claim を指すこと。
- README の test 数と coverage は、直近成功した `cargo test --workspace --all-targets` と `cargo tarpaulin --workspace --engine llvm --fail-under 80` の実行結果と一致すること。
- その組織の実際の topic schema、register map、auth 前提、BMS contract、受入基準なしに、private customer 互換性を主張しないこと。
- timeline または observability artifact を hosted observability、OpenTelemetry、Grafana、InfluxDB、Prometheus、本番 incident-response 実装と説明しないこと。これらは export contract のみである。
- Python reference client を supported production SDK または PyPI package と説明しないこと。
- behavior-model support と wire-protocol support を説明する際は [`PROTOCOL_SUPPORT_MATRIX.md`](PROTOCOL_SUPPORT_MATRIX.md) を source of truth とすること。
- protocol specification reference、conformance subset claim、verification command、明示的 rejection には [`PROTOCOL_CONFORMANCE_REGISTRY.md`](PROTOCOL_CONFORMANCE_REGISTRY.md) を source of truth とすること。
- claim が registry row と verification command にマップされない限り、プロトコルに対して "compatible"、"conformant"、"supports"、"implements" を使わないこと。
- Matter、BACnet、KNX、OPC UA は B Tier `contract_profile` entry のみである。wire-level support、full protocol implementation、certification evidence と説明しないこと。
- ローカル VSCode authoring bundle は Marketplace extension、language server、graphical debugger ではない。
