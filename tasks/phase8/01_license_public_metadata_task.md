# Task 01 — License and Public Metadata

## Objective

Fix public metadata inconsistencies before publishing the repository.

## Acceptance Criteria

- Add a real `LICENSE` file if the project remains MIT licensed.
- Ensure `Cargo.toml`, README badge, and repository files agree on the license.
- Keep or update `LICENSE-NOTE.txt` so it explains only the source/provenance caution, not a missing license state.
- Verify README license link resolves to an existing file.

## Review Findings

- `Cargo.toml` declares `license = "MIT"`.
- README links the MIT badge to `LICENSE`.
- The repository currently has `LICENSE-NOTE.txt`, but no `LICENSE` file.
