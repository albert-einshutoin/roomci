# Task 04: Concurrent Serve and Overlay Tests

## Status

`todo`

## Problem

Serve-mode concurrency has basic coverage, but race-sensitive combinations
remain thin: `/run` plus `/state`, external MQTT publish during a run, report
fetches during state mutation, and BMS contact ingestion overlap.

## Scope

- Add integration tests for concurrent serve operations.
- Assert deterministic overlay ordering where the product promises it.
- Document unsupported concurrent semantics where determinism is intentionally
  not promised.

## Acceptance Criteria

- Concurrent operations do not panic or poison shared state.
- Reports remain internally consistent after overlapped external inputs.
- Any non-deterministic behavior is documented as such.

