# Task 01 — Workspace and CLI Skeleton

## Objective

Create the Rust workspace and CLI entrypoints that make roomci executable.

## Implementation Scope

- Add workspace crates:
  - `roomci-core`
  - `roomci-device-model`
  - `roomci-scenario`
  - `roomci-fault`
  - `roomci-report`
  - `roomci-cli`
- Add CLI commands:
  - `roomci run <scenario.yaml>`
  - `roomci validate <scenario.yaml>`
- Add structured error handling suitable for CLI and future adapters.
- Add version output and basic help text.

## Acceptance Criteria

- `cargo test` runs for the workspace.
- `roomci --help` lists `run` and `validate`.
- `roomci validate docs/examples/checkin_lock_offline.yaml` performs syntax and schema-level validation.
- CLI errors are human-readable and include the file path when validation fails.

## Risks

- Overbuilding the crate split before behavior exists.
- Letting CLI output become the only report contract.

## References

- `docs/02_architecture.md`
- `docs/05_docker_ci_design.md`
