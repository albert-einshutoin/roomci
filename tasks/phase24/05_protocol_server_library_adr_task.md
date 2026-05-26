# Task 05: Protocol Server Library ADR

## Status

`done`

## Problem

HTTP, MQTT, and Modbus serve paths are intentionally small subsets, but they
are hand-written. The project needs an explicit decision on when to keep that
approach and when to adopt libraries such as `hyper`/`axum`, MQTT broker
components, or Modbus crates.

## Scope

- Write an ADR comparing current hand-written subsets with library-backed
  alternatives.
- Decide per protocol whether to keep, replace, or defer.
- Do not migrate protocols until the ADR sets acceptance criteria.

## Acceptance Criteria

- The decision is documented in `docs/adr/`.
- The ADR distinguishes CI contract-emulator needs from production-server
  semantics.
- Follow-up implementation tasks are created only for decisions that are
  promoted.

## Evidence

- `docs/adr/0001-serve-protocol-server-strategy.md`
