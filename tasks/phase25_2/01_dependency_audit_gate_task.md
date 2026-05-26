# Task 01: Dependency Audit Gate

## Status

`done`

## Problem

`cargo audit` currently fails because `roomci-cli` dev-depends on `rumqttc`,
which pulls vulnerable `rustls-webpki 0.102.8` and unmaintained
`rustls-pemfile 2.2.0`. Even though this is a dev-dependency path, it blocks a
clean Rust quality gate.

## Scope

- Resolve the RustSec findings without weakening integration coverage.
- Prefer removing the risky dependency from tests if an in-repo TCP MQTT smoke
  client is enough.
- If upgrading is viable, prove the new dependency graph no longer contains the
  advisory path.
- Keep MQTT integration tests deterministic.

## Implementation Checklist

- [x] Confirm the current advisory path with `cargo audit`.
- [x] Inspect `crates/roomci-cli/tests/cli.rs` usage of `rumqttc`.
- [x] Choose removal, replacement, or upgrade based on actual test needs.
- [x] Update tests and dependency metadata.
- [x] Run `cargo update` only if dependency resolution requires it.
- [x] Verify `cargo audit` passes.
- [x] Record the final dependency tree evidence in `phase_status.md`.

## Acceptance Criteria

- `cargo audit` passes.
- MQTT integration coverage remains present.
- No dependency warning is ignored without a written reason.
