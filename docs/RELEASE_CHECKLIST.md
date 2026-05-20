# Release Checklist

Use this checklist before presenting `roomci` as an OSS product or before asking a company to evaluate it.

## Required Gates

| Gate | Command / Evidence | Expected Result |
|---|---|---|
| Clean checkout build | `cargo metadata --format-version 1 --no-deps` | Workspace resolves all crates, including `roomci-serve` |
| Rust formatting | `cargo fmt --all --check` | No formatting diff |
| Rust linting | `cargo clippy --workspace --all-targets -- -D warnings` | No warnings |
| Rust tests | `cargo test --workspace --all-targets` | All workspace tests pass |
| Docs build | `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` | API docs build without warnings |
| Coverage floor | `cargo tarpaulin --workspace --engine llvm --fail-under 80` | Coverage remains at or above 80% |
| Scenario validation | `cargo run -p roomci-cli -- validate examples/*.yaml` | Public examples validate |
| Adapter contract validation | `cargo run -p roomci-cli -- adapter validate adapter-contracts/templates/company_adapter_contract.yaml adapter-contracts/examples/*.yaml` | Adapter template and example contracts validate |
| Scenario run reports | `cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --report-json reports/local_first.json --report-md reports/local_first.md --junit reports/local_first.xml` | JSON, Markdown, and JUnit reports are generated |
| Docker image | `docker build -t roomci:verify .` | Image builds from repository files |
| Docker scenario run | `docker run --rm -v "$PWD/examples:/scenarios:ro" roomci:verify run /scenarios/local_first_cloud_outage.yaml` | Containerized scenario passes |
| Compose PoC | `make compose-poc` | External controller drives `roomci serve` over HTTP and writes reports |
| Full local CI approximation | `make verify` | Runs the same practical gate set as CI, including Docker and Compose checks |

## Generated Artifacts

- `reports/` is intentionally ignored by git.
- `target/` is intentionally ignored by git.
- Docker images created by `make verify`, `make compose-poc`, or `docker-demo` are local build artifacts.

## Public Claim Rules

- README badges must point to real repository gates or static local-truth claims.
- README test count and coverage must match the latest successful `cargo test --workspace --all-targets` and `cargo tarpaulin --workspace --engine llvm --fail-under 80` run.
- Do not claim private customer compatibility, including NOT A HOTEL compatibility, without their actual topic schemas, register maps, auth assumptions, BMS contracts, and acceptance criteria.
- Use [`PROTOCOL_SUPPORT_MATRIX.md`](PROTOCOL_SUPPORT_MATRIX.md) as the source of truth when describing behavior-model support versus wire-protocol support.
