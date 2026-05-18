# Task 03 — CLI Report Flags

## Objective

Align CLI flags with latest docs and keep existing flags as aliases.

## Implementation Scope

- Add `--report-md` alias for Markdown.
- Add `--report-json` alias for JSON.
- Keep `--markdown` and `--json` as compatibility aliases.
- Update CLI tests.

## Acceptance Criteria

- Latest command in `docs/02_product_requirements.md` works.
- Compose command uses supported flags.
- CLI help shows report flags clearly.
