# roomci VSCode Local Authoring Assets

This directory is a local editor asset bundle for authoring roomci YAML
scenarios. It is not a published VSCode Marketplace extension.

## What It Provides

- YAML schema association for `*.roomci.yaml` and `examples/*.yaml`
- snippets for scenario, MQTT, Modbus, BMS, comfort, and assertion blocks
- reusable VSCode tasks for validation, scenario runs, full verify, and protocol
  evidence checks

## Local Use

For an extension-development session from this repository:

```bash
code --extensionDevelopmentPath="$(pwd)/tools/vscode-roomci" .
```

For day-to-day local use without publishing or packaging anything, copy
`tools/vscode-roomci/.vscode/tasks.json` and
`tools/vscode-roomci/.vscode/settings.json` into your workspace `.vscode/`
directory. The settings file expects the canonical workspace schema at
`schemas/scenario.schema.json`.

## Tasks

The bundled task file references existing repository commands:

- `roomci validate current file`: `cargo run -p roomci-cli -- validate "${file}"`
- `roomci run current file`: `cargo run -p roomci-cli -- run "${file}"`
- `roomci make verify`: `make verify`
- `roomci protocol evidence`: `make protocol-evidence`

## Validation

```bash
make vscode-assets-check
```

This parses every JSON asset, checks that the bundled schema copy matches
`schemas/scenario.schema.json`, and verifies that snippets/tasks reference
existing roomci fields and commands.
