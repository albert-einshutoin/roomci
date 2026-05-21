# Task 02: Scenario Debugger Explain Command

## Goal

Add a scenario debugging surface that explains execution order, resolved virtual
time, state changes, assertion inputs, and likely failure causes.

## Scope

- Add CLI command:
  - `roomci debug <scenario>`
- Add output flags:
  - `--debug-json <path>`
  - `--debug-md <path>`
- Include:
  - sorted event order
  - resolved `T+...` timestamps
  - state diffs per step
  - assertions evaluated
  - failed assertion detail
  - suggested checks from existing report logic
- Reuse existing scenario validation and runtime code.

## Acceptance Criteria

- Debug output is deterministic.
- Failing scenario debug output explains `dali_scene_partial_failure.yaml`.
- Passing scenario debug output shows state diffs without false failures.
- JSON output is machine-readable for editor/CI consumers.

## Test Commands

```bash
cargo test -p roomci-cli --test cli debug
cargo run -p roomci-cli -- debug examples/dali_scene_partial_failure.yaml \
  --debug-json reports/dali.debug.json \
  --debug-md reports/dali.debug.md
python3 -m json.tool reports/dali.debug.json >/dev/null
```

## Out Of Scope

- Interactive TUI.
- Browser UI.
- Time-travel execution.
