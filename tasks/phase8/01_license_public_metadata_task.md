# Task 01 — License and Public Metadata

## Objective

Fix public metadata inconsistencies before publishing the repository.

## Acceptance Criteria

- Add a real `LICENSE` file for the selected public license.
- Ensure `Cargo.toml`, README badge, and repository files agree on the license.
- Keep or update `NOTICE.md` so it explains only the source/provenance caution, not a missing license state.
- Verify README license link resolves to an existing file.

## Review Findings

- `Cargo.toml` declares `license = "Apache-2.0"`.
- README links the MIT badge to `LICENSE`.
- The repository keeps the Apache-2.0 `LICENSE` as the canonical license file.
