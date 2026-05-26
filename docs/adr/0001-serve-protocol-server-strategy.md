# ADR 0001: Serve Protocol Server Strategy

## Status

Accepted

## Context

`roomci serve` exposes local HTTP, MQTT, and Modbus TCP surfaces so external
controllers can drive pre-adoption PoCs without real devices or sites.

Those surfaces are intentionally scoped contract-emulator subsets:

- HTTP is a localhost control/report API.
- MQTT accepts MQTT 3.1.1 CONNECT and QoS0 PUBLISH/SUBSCRIBE behavior needed by
  the documented PoC contract.
- Modbus TCP supports the documented practical read/write subset for configured
  registers.

The current implementations are hand-written. That keeps the binary small and
the behavior easy to pin to CI evidence, but it also creates maintenance risk
if the project drifts toward production-server or certification-grade protocol
claims.

## Decision

Keep the current hand-written protocol subsets for Phase 24.

Adopt protocol/server libraries only when the product need crosses one of
these thresholds:

- HTTP needs routing middleware, TLS, auth, streaming, or a stable public API.
- MQTT needs broker semantics beyond the documented PoC subset, such as QoS1,
  QoS2, retained-session behavior, ACLs, persistence, MQTT 5, or clustering.
- Modbus needs broader function-code coverage, device timing semantics,
  multi-client fidelity, or certification-style behavior.

Until then, the hand-written paths remain acceptable because `roomci` is a
local CI contract emulator, not a production server stack.

## Protocol Decisions

| Surface | Decision | Reason |
|---|---|---|
| HTTP | Keep hand-written subset | The API is localhost-only, small, synchronous, and covered by behavior tests. |
| MQTT | Keep hand-written subset | Current value is standard-client PoC ingress for configured topics, not broker replacement. |
| Modbus TCP | Keep hand-written subset | Current value is configured register contract evidence, not full device simulation. |

## Consequences

- Continue adding bounded parser, malformed input, and concurrency regression
  tests around the current subsets.
- Keep public docs explicit that these are local PoC surfaces.
- Do not claim full protocol implementation or conformance unless a registry row
  maps to a verification command.
- Revisit this ADR before adding any protocol depth that requires production
  server semantics.

## Follow-Up

No protocol migration task is promoted from this ADR.

Future customer-independent work should focus on adapter-contract expression,
payload validation, topic mapping diagnostics, and evaluator intake rather than
replacing protocol servers prematurely.
