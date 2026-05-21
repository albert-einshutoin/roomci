# Phase 22 Test Plan

Phase 22 is complete when the editor asset bundle is usable from a clean
checkout without publishing anything.

## Required Gates

- `make vscode-assets-check`
- `python3 -m json.tool tools/vscode-roomci/package.json`
- `rg -n "roomci validate|roomci run|make verify|protocol-evidence" tools/vscode-roomci`

## Acceptance Criteria

- all JSON assets parse
- snippets reference existing scenario fields
- tasks reference existing commands
- docs state this is a local editor asset bundle, not a published extension
