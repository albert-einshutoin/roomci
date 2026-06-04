# Task 03: Provenance and Changelog Cleanup

## Status

`done`

## Problem

`NOTICE.md` is the publication-facing provenance note, and the changelog's many
same-day phase entries can be misread as fabricated calendar releases.

## Scope

- Make the provenance note concrete about generated/design-note origins,
  third-party names, and private-compatibility boundaries.
- Clarify that pre-1.0 changelog entries are phase checkpoints, not published
  calendar releases.

## Acceptance Criteria

- `NOTICE.md` can be read without guessing what the project is claiming.
- `CHANGELOG.md` explains the phase-based history before the version entries.
