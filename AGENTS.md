# Repository verification commands

| Purpose | Command |
|---|---|
| Focused external MQTT unit tests | `cargo test -p roomci-cli --bin roomci external_mqtt` |
| Strict scenario validation | `cargo test -p roomci-scenario unsupported_broker_options_and_fault_duration_fail_before_run` |
| Real broker, proxy and separate SUT | `bash examples/external-mqtt-recovery/run.sh` (Docker Compose required) |
| Full local Rust gates | `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets && RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` |
| Hosted CI | `smart-home-ci.yml`: quality-gates, roomci, external-mqtt-recovery, release-evidence, action-self-test |

`ci-pr` selective execution is not implemented. Run the full local gates for shared crates, Cargo manifests, Docker/Compose or workflow changes. The external MQTT Compose suite is the authority for the reference SUT's real-network recovery result; internal model tests do not prove it.
