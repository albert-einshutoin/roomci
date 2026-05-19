# Phase 10 Test Plan

## Quality Gates

- `roomci serve --config <scenario> --check` validates and exits.
- `roomci serve --config <scenario>` starts localhost endpoints and shuts down cleanly.
- HTTP API integration tests cover health, scenario, state, timeline, fault injection, finish, and report retrieval.
- External MQTT/client-driven retained-state scenario passes without using internal runner APIs.
- Docker Compose black-box E2E produces JSON, Markdown, and JUnit reports.
- Failure-path E2E exits non-zero and emits actionable failure reports.
- Docs and README state supported protocol subset and non-goals.
- Existing `roomci run` scenario mode remains backward-compatible.

## Done Means

Phase 10 is done when a developer can run one command to start `roomci` as a local emulator, run a separate client against it, and collect CI-ready reports without relying on private vendor protocol knowledge.
