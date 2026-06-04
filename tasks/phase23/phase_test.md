# Phase 23 Test Plan

Phase 23 is complete when:

- `Cargo.toml`, `README*`, `LICENSE`, and `NOTICE.md` agree on
  `Apache-2.0`.
- Public docs no longer depend on the stale `albert-einshutoin` namespace.
- Public docs avoid vendor-specific hospitality name-dropping unless the text
  is explicitly explaining a non-goal or generic integration class.
- `docs/PRODUCT_GUIDE.md` is the canonical positioning guide and the older
  overlapping positioning docs point there.
- Phase 24 exists for code-architecture hardening work that should not be
  rushed into the trust cleanup.
- `cargo fmt --all --check`, `cargo test --workspace --all-targets`, and
  `make protocol-evidence` pass.
