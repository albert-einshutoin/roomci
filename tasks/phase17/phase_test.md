# Phase 17 Test Plan

Phase 17 is complete only when promoted profiles have executable examples, tests, and public docs.

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`
- `make verify`
- Example/profile docs do not claim vendor compatibility or physical safety validation.
