# Task 07: API Deprecation and Cleanup

## Status

`done`

## Problem

Compatibility constructors and raw helper APIs are useful during migration, but
internal code should not continue relying on them once the validated model is
available.

## Scope

- Remove internal usage of deprecated or silent constructors.
- Keep public compatibility APIs only when they are intentionally supported.
- Make unsafe/raw paths visibly named as raw, unchecked, or compatibility-only.
- Update docs or changelog notes if public API expectations change.

## Implementation Checklist

- [x] Search for internal calls to compatibility constructors and raw helpers.
- [x] Add or update tests that use the validated constructor as the primary path.
- [x] Replace internal compatibility calls with fallible validated conversion.
- [x] Mark remaining compatibility APIs with clear deprecation text or boundary
  documentation.
- [x] Ensure `cargo clippy --workspace --all-targets -- -D warnings` has no
  deprecation or dead-code fallout.
- [x] Record the final search command and result in `phase_status.md`.

## Acceptance Criteria

- Internal runtime/scenario code uses validated constructors by default.
- Compatibility APIs are either removed, deprecated, or explicitly documented.
- No warning-suppression is needed to keep the workspace clean.
