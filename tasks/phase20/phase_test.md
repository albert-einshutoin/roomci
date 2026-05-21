# Phase 20 Test Plan

Phase 20 is complete when A Tier developer-experience artifacts are runnable
from a clean checkout and documented for external evaluators. VSCode editor
assets are excluded from Phase 20 and tracked in Phase 22.

## Required Gates

- `make verify`
- `make adapter-samples-smoke`
- `make protocol-evidence`

## New Phase 20 Gates

- Python SDK smoke target runs against `roomci serve`.
- Scenario debugger explain command emits deterministic JSON and Markdown.
- Developer-experience docs link to exact examples and smoke targets.

## Acceptance Criteria

- A Python evaluator can run one script and drive HTTP, MQTT, and Modbus paths.
- A scenario author can identify why a failed assertion failed without reading
  Rust code.
- All new claims are tied to tests, smoke commands, or explicit non-goals.
