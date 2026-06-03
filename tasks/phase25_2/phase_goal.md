# Phase 25.2 Goal: Validated Graph Completion and Audit Gate

## Goal

Close the remaining Rust-maximization gaps found in the Phase 25.1 self-review
before Phase 26 expands the adapter contract surface.

## Why This Phase Exists

Phase 25.1 moved scenario execution through `ValidatedScenario`, owned enums,
newtypes, typed conditions, and runtime config projection. The follow-up review
showed that the implementation is stronger but not yet maximized:

- `cargo audit` fails because `roomci-cli` dev-depends on `rumqttc`, which
  currently pulls vulnerable `rustls-webpki` and unmaintained
  `rustls-pemfile`.
- `roomci-core` receives `ValidatedScenario` but still reaches back into
  `ScenarioFile` raw config to rebuild subsystem models.
- raw borrowed classifier APIs remain public and can drift from the owned
  validated model.
- promoted `ops` and `automation` behavior still uses map/key string dispatch.
- several target values remain broad `String` fields instead of domain target
  enums.

## In Scope

- Make dependency audit status explicit and green.
- Remove avoidable runtime dependency on `ValidatedScenario::raw()` for promoted
  subsystem execution inputs.
- Contain or deprecate raw borrowed classifier APIs so validated owned types are
  the primary path.
- Promote known `ops` and `automation` forms into typed variants.
- Split promoted command, fault, assertion, and sensor targets into domain
  target enums where behavior already depends on target shape.
- Preserve the public YAML scenario format and current CLI/report behavior.
- Document intentional raw boundaries that remain for YAML compatibility,
  extension data, subsystem adapters, and report output.

## Out of Scope

- Phase 26 adapter payload/topic/acceptance feature work.
- Customer-specific protocol compatibility.
- Replacing all dynamic report JSON with a public typed report API.
- Full protocol stack rewrites or certification claims.
- Removing intentionally dynamic extension fields that are not promoted runtime
  behavior.

## Exit Criteria

- `cargo audit` passes or the dependency risk is removed from the dependency
  graph.
- Core runtime no longer needs raw scenario maps for promoted subsystem inputs.
- Raw classifier helpers are no longer part of the normal public path.
- Promoted `ops` and `automation` behavior executes through typed variants.
- Promoted targets use domain enums instead of free-form strings.
- All Phase 25.2 quality gates pass and Phase 26 can start without widening raw
  string/value handling.
