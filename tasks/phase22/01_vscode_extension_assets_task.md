# Task 01: VSCode Extension Assets

## Goal

Add local VSCode assets that make scenario authoring easier without requiring a
marketplace extension.

## Scope

- Create `tools/vscode-roomci/`.
- Include:
  - `package.json`
  - YAML schema association for `*.roomci.yaml` and `examples/*.yaml`
  - snippets for scenario, MQTT contract, Modbus register, BMS alert, comfort
    reading, and assertion blocks
  - tasks for `roomci validate`, `roomci run`, `make verify`, and `make
    protocol-evidence`
  - README with local install instructions
- Add validation target:
  - `make vscode-assets-check`

## Acceptance Criteria

- All JSON assets parse with `python3 -m json.tool`.
- Snippets reference existing schema fields.
- Tasks reference existing commands.
- Docs state this is a local editor asset bundle, not a published extension.

## Test Commands

```bash
make vscode-assets-check
rg -n "roomci validate|roomci run|make verify|protocol-evidence" tools/vscode-roomci
```

## Out Of Scope

- Publishing to VSCode Marketplace.
- Implementing a language server.
- Implementing a graphical debugger.
