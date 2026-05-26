# Task 01: Validated Scenario Boundary

## Status

`done`

## Problem

The external scenario format needs to remain flexible, but core runtime code
should not operate directly on raw serde-loaded structs and loosely typed maps.
Without a validated boundary, later adapter contract work will keep expanding
the untyped surface.

## Scope

- Keep the current raw scenario structs as the YAML compatibility layer.
- Add an owned validated scenario model in the scenario crate.
- Convert raw scenario data into the validated model before runtime execution.
- Return structured validation errors instead of silently defaulting unknown
  runtime-relevant data.

## Implementation Checklist

- [x] Add a failing test that loads an existing example and converts it into a
  validated model.
- [x] Add a failing test that malformed raw scenario data is rejected before
  execution.
- [x] Create a `ValidatedScenario` entry point near the existing scenario model.
- [x] Implement raw-to-validated conversion with `TryFrom` or an equivalent
  fallible constructor.
- [x] Wire `roomci-core` execution through the validated scenario path.
- [x] Keep public CLI commands compatible with current YAML files.
- [x] Record evidence commands in `phase_status.md`.

## Acceptance Criteria

- Existing scenarios validate and run unchanged.
- Runtime execution does not need to classify known scenario semantics from raw
  YAML data on the hot path.
- Validation failures name the invalid field and the rejected value.
