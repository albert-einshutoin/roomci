# Task 03: Docker Protocol Smoke Image

## Why

The protocol smoke currently installs Python packages at container runtime. That is acceptable for an internal check but weak for a public evaluator because first-run speed and network dependency can vary.

## Acceptance Criteria

- Add a pinned protocol-smoke image or Compose build stage.
- Remove runtime `pip install` from the smoke command.
- Keep dependencies version-pinned.
- Update README or evidence docs with the stable command.
- Verify `make protocol-smoke` and `make verify`.
