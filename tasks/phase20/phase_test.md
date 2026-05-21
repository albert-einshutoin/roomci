# Phase 20 Test Plan

Phase 20 is complete when A Tier developer-experience artifacts are runnable
from a clean checkout and documented for external evaluators.

## Required Gates

- `make verify`
- `make adapter-samples-smoke`
- `make protocol-evidence`

## New Phase 20 Gates

- Python SDK smoke target runs against `roomci serve`.
- Scenario debugger explain command emits deterministic JSON and Markdown.
- VSCode extension assets validate as JSON and reference existing commands.
- Developer-experience docs link to exact examples and smoke targets.

## Acceptance Criteria

- A Python evaluator can run one script and drive HTTP, MQTT, and Modbus paths.
- A scenario author can identify why a failed assertion failed without reading
  Rust code.
- Editor assets improve authoring without becoming required for the CLI.
- All new claims are tied to tests, smoke commands, or explicit non-goals.
