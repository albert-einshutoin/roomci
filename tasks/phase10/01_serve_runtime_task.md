# Task 01 — Serve Runtime

## Objective

Replace the current placeholder `serve` behavior with a real localhost-bound runtime that can host test endpoints for external clients.

## Acceptance Criteria

- `roomci serve --config examples/local_first_cloud_outage.yaml --check` still validates and exits successfully.
- `roomci serve --config examples/local_first_cloud_outage.yaml` starts a long-running runtime with graceful shutdown.
- Default bind address is loopback-only.
- Runtime startup validates the scenario and fails fast with actionable errors.
- Runtime owns shared state for scenario metadata, current device/broker state, timeline, assertions, and reports.
- The CLI prints endpoint URLs and the loaded scenario name at startup.
- Unit or integration tests cover config-check behavior and runtime initialization.

## Notes

- Use a small async runtime boundary rather than rewriting the scenario runner wholesale.
- Keep scenario-mode `roomci run` behavior unchanged.
