# Task 06 — Release Metadata, CHANGELOG, and Working README Badges

## Goal

Close the "public surface vs reality" gap that the strict review of Phase 9 + 10 surfaced: workspace `Cargo.toml` is missing publish-ready metadata, there is no `CHANGELOG.md` at the repo root, and the README badges point to a GitHub URL that returns 404. None of these are blockers for an internal demo, all of them are visible to an external evaluator within thirty seconds of opening the repo.

## Why This Matters

`roomci` positions itself as a portfolio piece and as an OSS PoC product candidate. An evaluator who clicks a CI badge and gets a 404, or runs `cargo package` and gets "manifest has no description", treats those as signals that the product is less polished than the README implies. This task fixes those signals.

## Implementation Scope

### Workspace metadata

- Add to root `Cargo.toml` under `[workspace.package]`:
  - `description = "Local-first Smart Home QA & Operations Emulator for CI"`
  - `keywords = ["iot", "smart-home", "mqtt", "emulator", "ci"]` (max 5)
  - `categories = ["simulation", "command-line-utilities", "development-tools::testing"]`
  - `readme = "README.md"`
- Confirm every workspace member crate's `Cargo.toml` inherits these via `description.workspace = true` etc., or sets per-crate values where it makes sense.
- Verify with `cargo package -p roomci-cli --list` and `cargo package -p roomci-serve --list` (after Task 01 lands) that packaging succeeds without metadata warnings.

### CHANGELOG.md

- Create `CHANGELOG.md` at the repo root.
- Use the Keep a Changelog format (`## [0.x.y] - YYYY-MM-DD`, with `Added`/`Changed`/`Fixed`/`Removed`/`Security` sections).
- Backfill one entry per shipped phase from Phase 0 through Phase 12. Use `tasks/phase{N}/phase_status.md` evidence rows to populate each entry. Each phase entry should be short — one to five bullets per category, focused on user-visible behavior, not internal refactors.
- Add a `## [Unreleased]` section at the top for in-flight work.

### README badges

- Run `gh repo view <owner>/<repo>` (or `curl -I` the badge URL) to confirm the current 404.
- Pick one of:
  - **Option A — Make the repo public** (preferred if the user agrees): `gh repo create`, push `main`, push the existing GitHub Actions workflow, confirm CI runs, confirm badge resolves. Update README badge URLs to whatever the final canonical URL becomes.
  - **Option B — Replace the badges** if the public repo cannot be created: remove or swap to local-only badges (e.g. a static shield linking to the CHANGELOG, a license badge backed by the in-repo LICENSE file, and a coverage badge backed by a generated `coverage.svg` checked into `docs/`).
- Either way: every badge on the README must resolve. No 404s.

### Status alignment

- Update `tasks/status.md` to reflect Phase 12's existence and (when this task lands) its completion of the release-plumbing portion.

## Acceptance Criteria

- `cargo metadata --format-version 1 | jq -e '.packages[] | select(.name == "roomci-cli") | .description'` returns a non-null string.
- `cargo package -p roomci-cli --list` and `cargo package -p roomci-serve --list` succeed with zero metadata warnings.
- `CHANGELOG.md` exists, follows Keep a Changelog format, and references every phase from Phase 0 through Phase 12.
- Every badge URL in the README returns HTTP 200 (verified with `curl -I`).
- `tasks/status.md` lists Phase 12 with the correct status.

## Out of Scope

- Actually running `cargo publish`. Metadata readiness is the goal; publishing is a release decision left to the user.
- Adding a `release-please` or `semantic-release` pipeline. Manual maintenance of `CHANGELOG.md` is acceptable for the current phase cadence.
- Rewriting the README beyond badge URLs.

## Evidence

- `curl -sI <each badge URL> | head -1` shows `HTTP/2 200` for every badge.
- `head -30 CHANGELOG.md` shows the Keep a Changelog header and the `[Unreleased]` section.
- `grep -c '^## \[' CHANGELOG.md` returns at least 14 (one `[Unreleased]` plus one per phase 0–12).
- `cargo package -p roomci-cli --list` log shows no `warning: manifest has no ...` lines.
