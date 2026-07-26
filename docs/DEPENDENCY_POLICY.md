# Dependency Security Policy

`roomci` treats its resolved `Cargo.lock` and GitHub Actions as supply-chain
inputs. This policy keeps updates reviewable while avoiding a noisy PR queue.

## Automated Gates

- Dependabot checks Cargo and GitHub Actions dependencies weekly against
  `main`. Minor and patch updates are grouped by ecosystem; major updates stay
  separate for explicit compatibility review.
- CI installs `cargo-audit` with `--locked` and runs `cargo-audit audit --deny
  warnings`. The `-D warnings` flag is fail-closed for every RustSec warning
  category, including unsound and yanked dependencies. The external binary is
  invoked directly so repository-local Cargo aliases cannot shadow the audit.
- RustSec exceptions are permitted only when no patched version can be
  selected, the affected path is demonstrably unreachable, and a short expiry,
  accountable owner, and tracking issue are all documented in the reviewed
  exception. A patched dependency must be updated instead of ignored.

## Resolved RustSec Evidence

| Advisory | Resolution evidence | Tracking |
|---|---|---|---|
| `RUSTSEC-2026-0190` (`anyhow`, unsound `Error::downcast_mut`) | On 2026-07-26, `cargo-audit audit --deny warnings` failed for lockfile version `1.0.102`. The advisory patches `>= 1.0.103`, so `Cargo.lock` was minimally updated to `anyhow 1.0.104`; the same deny-warnings command now passes with no ignore configuration. | [#8](https://github.com/albert-einshutoin/roomci/issues/8) |

There is no active RustSec exception. `cargo-audit audit --deny warnings` is the
complete CI gate and must remain green without an advisory ignore file.

## Local Tool Bundles

`tools/vscode-roomci/package.json` is private, has no third-party npm
dependencies, and therefore intentionally has no lockfile or npm audit step.
If a runtime or development dependency is added, the same PR must add a
committed lockfile, a weekly npm Dependabot entry, and an `npm audit` release
gate. A metadata-only editor bundle must not silently become an unmonitored
JavaScript supply-chain surface.

## `serde_yaml` Compatibility Hold

`serde_yaml 0.9.34` remains a short-term dependency because scenario YAML is a
public authoring contract. A replacement can change scalar typing, duplicate
keys, tags, anchors, diagnostics, and the accepted/rejected input set even when
ordinary examples still parse.

Evaluate a migration candidate only in a dedicated PR. It may be accepted only
when all of these conditions hold:

1. Existing scenario and adapter golden artifacts are unchanged, or every
   intentional change is reviewed as a public contract migration.
2. property tests cover equivalent YAML/JSON state conversion and malformed
   inputs without panics or non-deterministic output.
3. An explicit accept/reject parity corpus proves valid files remain valid and
   invalid files remain rejected, including duplicate keys, tags, anchors,
   numeric scalars, and public error locations.
4. `cargo test --workspace --all-targets`, `cargo-audit audit --deny warnings`, and
   the release checklist pass after the old dependency is removed.

The migration trigger is a RustSec advisory affecting `serde_yaml`, a
maintained candidate with documented compatibility evidence, or an upstream
Rust/toolchain compatibility break. Until then, this hold protects scenario
semantics while Dependabot provides routine update review.
