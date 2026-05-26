# Task 01: README Interview Demo Link

## Status

`done`

## Problem

The top-level README linked to `docs/INTERVIEW_DEMO.md`, but that file was not
committed. This creates immediate friction and reduces trust for an evaluator
following the strongest-demo path.

## Scope

- Add `docs/INTERVIEW_DEMO.md`.
- Keep the document command-first and aligned with existing product boundaries.
- Do not expand product claims or protocol support claims.

## Acceptance Criteria

- `test -f docs/INTERVIEW_DEMO.md` passes.
- The walkthrough references runnable demo, failure-report, and Compose PoC
  paths.
- The walkthrough links to protocol-boundary docs instead of making new
  unsupported claims.
