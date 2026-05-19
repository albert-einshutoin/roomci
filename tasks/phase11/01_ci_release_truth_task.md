# Task 01 — CI and Release Truthfulness Gate

## Goal

Make public quality claims match the actual repository so a serious evaluator does not lose trust before running the product.

## Why This Matters

The README currently positions `roomci` as CI-ready and references GitHub Actions quality gates. For an OSS product trying to become a default emulator choice, visible and reproducible quality gates are part of the product, not housekeeping.

## Implementation Scope

- Add or restore the referenced GitHub Actions workflow.
- Ensure workflow names, badge paths, README claims, and actual files match.
- Make `make verify` the same local approximation as CI.
- Add release checklist coverage for:
  - clean checkout
  - Docker build
  - Compose PoC
  - reports generated
  - docs build
  - coverage floor
- Ensure generated reports and temporary artifacts are ignored or documented.

## Acceptance Criteria

- README badges and CI text point to real files.
- `make verify` and the CI workflow run equivalent gates.
- A fresh clone can follow README Quick Start without missing files.
- `tasks/status.md` and phase status files do not overclaim completion.
