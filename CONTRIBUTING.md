# Contributing to roomci

Thanks for contributing. Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).
The [Japanese developer workflow](docs/DEVELOPER_WORKFLOW.ja.md) and
[Evaluator Intake Kit](docs/EVALUATOR_INTAKE_KIT.md) provide additional context.

## Reports and proposals

Check [existing issues](https://github.com/albert-einshutoin/roomci/issues)
before opening a [bug report, feature request, or evaluator PoC consultation](https://github.com/albert-einshutoin/roomci/issues/new/choose).
For a PoC, start with the six first-contact questions in the
[Evaluator Intake Kit](docs/EVALUATOR_INTAKE_KIT.md); collect detailed customer
specifications only after the target and acceptance owner are identified.

Public issues and pull requests must not contain credentials, private keys,
personal information, unpublished customer configurations, or confidential
logs. Redact examples; mark unavailable facts `Not verified / 未確認` rather than
guessing. You can ask about a PoC without providing sensitive material.
For suspected vulnerabilities, follow [Security](SECURITY.md) and do not post
details or secrets in a public issue. GitHub Discussions is not enabled; use
issues for public, non-sensitive questions.

## Development

`main` is the only long-lived integration and release branch. Start a short-lived
branch from the latest `main` and open a pull request against `main`.

Use the current stable Rust toolchain with `rustfmt` and `clippy`. The repository
does not declare or verify a minimum supported Rust version; do not infer one
from the edition. Docker Compose is needed for real-broker and third-party SUT
integration tests. `make verify` also needs Make, Docker, and `cargo-tarpaulin`.
See [Dependency Security Policy](docs/DEPENDENCY_POLICY.md) for the lockfile,
RustSec gate, and `serde_yaml` compatibility hold.

The Cargo workspace contains:

| Crate | Role |
|---|---|
| `roomci-cli` | CLI and executable entry point |
| `roomci-scenario` | Scenario schema and validation |
| `roomci-core` | Deterministic scenario execution |
| `roomci-device-model` | Device state model |
| `roomci-edge` | Edge behavior model |
| `roomci-mqtt` | MQTT behavior model |
| `roomci-ops` | Operations behavior model |
| `roomci-report` | Reports and evidence output |
| `roomci-serve` | HTTP/MQTT service mode |

The `examples/`, `adapter-contracts/`, `schemas/`, `compose/`, `docs/`, and
`tools/` directories contain sample scenarios, contracts, schemas, Docker
integration assets, documentation, and editor assets respectively.

For a small change, run the relevant crate tests and formatter first. For
example:

```bash
cargo fmt --all --check
cargo test -p roomci-scenario
```

Select tests for the changed behavior and its consumers. The full local Rust
gate is:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
```

CI's
`smart-home-ci.yml` additionally runs dependency audit, coverage, Docker and
Compose scenarios, external MQTT recovery, Node-RED, release evidence, and the
Action self-test. `make verify` is a broader local gate with coverage, examples,
Docker, and Compose; it is not a prerequisite for every report or small change.

Choose the relevant evaluation path from the
[README's three paths](README.md#choose-an-evaluation-path):
internal model, reference external MQTT SUT, or Node-RED third-party SUT. The
linked guides provide prerequisites and evidence boundaries. Public baseline
results do not establish compatibility with a customer SUT.

## Golden reports

`crates/roomci-core/tests/golden/` pins scenario `RunReport` JSON;
`crates/roomci-report/tests/golden/` pins representative renderers. Do not
regenerate goldens for a behavior-preserving refactor. If a contract change is
intentional, explain it and review the diff separately, then regenerate only
the affected goldens:

```bash
UPDATE_GOLDEN=1 cargo test -p roomci-core --test golden_reports
UPDATE_GOLDEN=1 cargo test -p roomci-report --test golden_renders
```

A new `examples/*.yaml` scenario needs its corresponding golden. Include the
golden diff in the PR evidence.

## Pull requests

Keep commits focused, describe the problem and scope, and complete the PR
template. Report executed checks with results and evidence. Mark checks that
do not apply separately from checks not run, with a reason. State which of the
three evaluation paths supplied evidence; do not extend protocol or customer
compatibility claims beyond that evidence.

Contributions are licensed under the repository's [Apache License 2.0](LICENSE).
