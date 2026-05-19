# Task 06 — Developer Experience and Integration Onboarding

## Goal

Make `roomci` easy for a busy platform, IoT, or smart-home engineer to evaluate without reading the whole codebase.

## Implementation Scope

- Add a concise integration guide:
  - choose a PoC pack
  - fill a contract template
  - run local emulator
  - connect external service/client
  - inject faults
  - collect reports
  - map failures back to production acceptance criteria
- Add troubleshooting docs for:
  - port conflicts
  - malformed MQTT payloads
  - missing required fields
  - unsupported protocol features
  - Docker/Compose failures
  - report interpretation
- Add machine-readable API docs for serve-mode HTTP endpoints.
- Add example client snippets for common evaluator paths.

## Acceptance Criteria

- A new developer can reach a successful external-controller PoC in under 15 minutes from a clean checkout.
- Errors include enough context to fix contract/config mistakes.
- Docs do not require knowing the Rust implementation to use the emulator.
- The onboarding path makes `roomci` feel like a product, not only a portfolio repo.
