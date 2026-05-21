# Phase 19 Test Plan

Phase 19 is complete when the S Tier evidence surface is reproducible from a
clean checkout and useful to external CI consumers.

## Required Gates

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`
- `make protocol-evidence`
- `make adapter-samples-smoke`
- `make verify`

## New Phase 19 Gates

- A focused test proves every timeline export event has:
  - `schema_version`
  - `run_id`
  - `scenario_name`
  - `sequence`
  - `at`
  - `event_type`
  - `target`
  - `message`
- A CLI integration test writes the new timeline export artifact and validates
  it with `serde_json`.
- A serve-mode test fetches the new timeline export endpoint and verifies it
  uses the same timeline union as `/timeline`.
- A CI workflow validation check proves GitHub Actions includes:
  - adapter sample smoke
  - protocol evidence check
  - Phase 17 scenario artifacts
  - uploaded JSON / Markdown / JUnit / timeline artifacts

## Acceptance Criteria

- The public docs explain observability as export artifacts, not a hosted
  observability system.
- The GitHub Actions examples can be copied by another repo with only scenario
  path edits.
- The release checklist includes the new S Tier evidence gates.
