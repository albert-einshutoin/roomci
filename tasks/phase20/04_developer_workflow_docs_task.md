# Task 04: Developer Workflow Docs

## Goal

Create a single developer-experience guide that explains when to use CLI,
serve mode, Python SDK, scenario debugger, and VSCode assets.

## Scope

- Add `docs/DEVELOPER_WORKFLOW.md`.
- Link it from README and `docs/README.md`.
- Include paths for:
  - first run
  - adapter contract validation
  - Python SDK smoke
  - scenario debugging
  - VSCode authoring
  - CI artifact review
- Include explicit non-goals around production SDKs and marketplace publishing.

## Acceptance Criteria

- New guide contains exact commands.
- Commands map to existing Makefile targets or CLI commands.
- README has one concise link to the guide.
- No wording suggests production SDK guarantees.

## Test Commands

```bash
rg -n "DEVELOPER_WORKFLOW|python-sdk-smoke|roomci debug|vscode-assets-check" README.md docs Makefile tasks
```

## Out Of Scope

- Marketing landing page.
- Screenshots.
- Hosted documentation.
