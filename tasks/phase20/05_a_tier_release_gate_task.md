# Task 05: A Tier Release Gate

## Goal

Make A Tier claims mechanically checkable.

## Scope

- Extend `docs/protocol-evidence.json` or add `docs/product-evidence.json` for
  developer-experience claims.
- Add checker support for:
  - Python SDK smoke target
  - debugger command tests
  - VSCode asset validation
  - docs links
- Add Makefile target:
  - `make developer-experience-smoke`
- Include this target in `make verify` only if runtime stays acceptable.
  Otherwise document it as a release-candidate optional gate.

## Acceptance Criteria

- A Tier claims fail CI if referenced examples/docs/targets disappear.
- Release checklist contains A Tier gates.
- Evidence pack distinguishes S Tier evidence from A Tier developer-experience
  evidence.

## Test Commands

```bash
make developer-experience-smoke
python3 scripts/protocol_evidence_check.py
```

## Out Of Scope

- Marketplace publishing.
- PyPI publishing.
- Hosted SDK documentation.
