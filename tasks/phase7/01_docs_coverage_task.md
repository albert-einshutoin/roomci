# Task 01 — Docs and Coverage

## Objective

Document every crate and public API, measure workspace test coverage, and make `docs/` the canonical documentation tree.

## Acceptance Criteria

- Every crate has a `//!` crate-level doc comment and a `///` doc comment on its primary public items.
- `cargo doc --no-deps --workspace` completes with zero warnings.
- `cargo tarpaulin --workspace` runs and reports coverage; baseline is recorded in `tasks/phase7/phase_status.md`.
- `roomci-docs-latest/` is removed after confirming parity with `docs/`.
- `docs/README.md` indexes the canonical docs.
