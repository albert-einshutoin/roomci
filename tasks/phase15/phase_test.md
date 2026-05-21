# Phase 15 Test Plan

Phase 15 is complete only when all of these gates pass:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`
- `cargo tarpaulin --workspace --engine llvm --fail-under 80`
- `make protocol-smoke`
- `make compose-poc`
- A new protocol evidence check proves that every `verified` protocol claim in the support matrix links to an executable test or smoke target.
- A source-size review confirms `roomci-serve`, `roomci-core`, and `roomci-scenario` are each split below the 800-line maintainability target, or the phase status records a justified exception.

## Review Gate

Before marking Phase 15 done, perform a release-candidate self-review focused on:

- evaluator first-run time,
- protocol claim truthfulness,
- runtime maintainability,
- Docker reproducibility,
- whether the remaining gaps belong in docs, tasks, or implementation.
