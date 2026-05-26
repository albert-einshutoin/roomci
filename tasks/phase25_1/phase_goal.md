# Phase 25.1 Goal: Rust Type Boundary Maximization

## Goal

Maximize Rust's language advantages before Phase 26 by moving flexible YAML,
string identifiers, and untyped value maps to explicit validated domain types at
the scenario/runtime boundary.

## Why This Phase Exists

Phase 25 closed public-surface freshness. Phase 26 expands adapter contract
generalization. Before adding more contract surface area, the Rust core should
make invalid states harder to represent internally:

- raw `serde_yaml::Value` and `serde_json::Value` should stay at parse/report
  boundaries;
- device, topic, scene, fixture, broker, edge, contact, and alert identities
  should not be interchangeable strings in runtime logic;
- known step, fault, assertion, and condition variants should be represented by
  enums and newtypes with exhaustive matching;
- domain config should be projected once into typed runtime inputs instead of
  repeatedly reading nested maps.

## In Scope

- Keep the external YAML scenario format backward compatible.
- Introduce a validated owned scenario model between raw serde structs and
  runtime execution.
- Add identity/topic newtypes with parsing and validation errors.
- Promote known step, fault, assertion, and condition forms into owned enums.
- Convert domain config that runtime reads into typed projection structs.
- Reduce direct raw-map access in core runtime modules.
- Add focused regression tests for invalid IDs, topics, conditions, and config.
- Preserve current reports and public CLI behavior unless validation errors are
  intentionally made earlier and clearer.

## Out of Scope

- Changing the user-facing scenario YAML schema.
- Implementing Phase 26 adapter payload/topic/assertion features.
- Adding customer-specific protocol compatibility.
- Adding production authentication, TLS, broker semantics, or certified protocol
  stacks.
- Rewriting report JSON as a typed public API.

## Exit Criteria

- Scenario loading has a clear raw-to-validated boundary.
- Core runtime logic consumes validated domain structures for promoted behavior.
- Raw `BTreeMap<String, serde_yaml::Value>` and `serde_json::Value` access is
  limited to parse, validation, projection, or report boundaries.
- Existing examples validate and run unchanged.
- New malformed-input tests fail at validation with actionable errors.
- Phase 26 can start without increasing the untyped surface area.
