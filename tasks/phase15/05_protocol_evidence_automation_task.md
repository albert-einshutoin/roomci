# Task 05: Protocol Evidence Automation

## Why

The protocol registry and support matrix are now honest, but they are manually synchronized with tests. Public trust improves if verified claims are backed by a small machine-checkable evidence map.

## Acceptance Criteria

- Add a checked evidence manifest for protocol support claims.
- Each `verified` protocol subset maps to at least one command, test name, or smoke target.
- Unsupported/future protocols map to explicit non-goal docs.
- Add a CI-friendly command that fails when a verified claim has no evidence.
- Update the release checklist to require this command.
