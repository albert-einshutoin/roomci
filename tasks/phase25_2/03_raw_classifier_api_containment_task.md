# Task 03: Raw Classifier API Containment

## Status

`done`

## Problem

Borrowed raw classifier helpers still exist as public APIs next to the owned
validated model. This creates two semantic models that can drift when a new
step, fault, or assertion is added.

## Scope

- Make the owned validated model the primary API.
- Contain raw classifier helpers inside the scenario crate or mark them as
  compatibility-only.
- Remove `roomci-core` reliance on raw classifiers.
- Preserve any public compatibility path only with clear documentation.

## Implementation Checklist

- [x] Search for `typed_step_kind`, `typed_fault_kind`, and
  `typed_assertion_kind` usage outside `roomci-scenario`.
- [x] Convert internal validation to construct owned validated variants
  directly where practical.
- [x] Reduce helper visibility to `pub(crate)` if no external use remains.
- [x] If public visibility must remain, add deprecation or compatibility docs.
- [x] Add a regression test that validates a known form through
  `ValidatedScenario`, not the raw helper.
- [x] Confirm `cargo doc --no-deps` is not required because public API text was
  not expanded; classifier visibility was narrowed to `pub(crate)`.

## Acceptance Criteria

- There is one primary path for promoted scenario semantics.
- New promoted behavior is added through owned validated types.
- Raw helpers cannot silently become the default runtime path again.
