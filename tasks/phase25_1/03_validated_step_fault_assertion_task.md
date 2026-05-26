# Task 03: Validated Step, Fault, and Assertion Model

## Status

`done`

## Problem

Known steps, faults, and assertions should be executable enum variants rather
than repeated string classification. Borrowed classifier helpers improved the
boundary, but owned validated variants are the next step before more adapter
contract features are added.

## Scope

- Promote known step kinds into owned validated variants.
- Promote known fault kinds into owned validated variants.
- Promote known assertion kinds into owned validated variants.
- Keep unknown extension behavior only where it is explicitly part of the
  public contract.

## Implementation Checklist

- [x] Add tests that unknown promoted step/fault/assertion forms fail with clear
  validation errors.
- [x] Add tests that existing known forms convert into owned enum variants.
- [x] Move execution dispatch to exhaustive matches over validated variants.
- [x] Keep compatibility helpers only at the raw scenario boundary.
- [x] Remove duplicate runtime string matching after validated variants exist.
- [x] Update task evidence with targeted `cargo test` commands.

## Acceptance Criteria

- Promoted runtime behavior uses exhaustive enum matching.
- Unknown runtime-relevant forms fail before execution unless explicitly marked
  as extension data.
- Adding a new promoted step/fault/assertion requires updating the typed model.
