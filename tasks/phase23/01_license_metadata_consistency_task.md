# Task 01: License Metadata Consistency

## Status

`done`

## Problem

The public license surface is inconsistent: repository docs and `LICENSE` use
Apache 2.0, while `Cargo.toml` still declares MIT. Stale task/changelog text
also mentions MIT.

## Scope

- Set Cargo workspace license metadata to `Apache-2.0`.
- Update stale release/task text that still claims MIT.
- Keep the fix limited to metadata and documentation; do not change runtime
  code.

## Acceptance Criteria

- `rg 'license = "MIT"|MIT license' Cargo.toml README.md README.ja.md CHANGELOG.md tasks/phase8 -S`
  returns no active public claim.
- `cargo metadata --format-version 1 --no-deps` reports `Apache-2.0` package
  license metadata.

