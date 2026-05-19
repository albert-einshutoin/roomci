# Phase 8 Goal — Public Release and Interview Polish

## Goal

Close the remaining public-release and interview-readiness gaps found during the product review.

Phase 7 made `roomci` technically strong enough to demo. Phase 8 makes the public surface honest, consistent, and safe to show outside the local workspace.

## In Scope

- Fix license metadata and repository files so README badges match the actual package.
- Separate passing demo scenarios from the intentional failure-report demo.
- Add and document `roomci serve --check` as the current service-mode validation entrypoint.
- Refresh README quality claims against current command output.
- Review NOT A HOTEL-related wording, examples, and generated design notes before public publishing.
- Add a release-readiness check that proves the README Quick Start and demo commands still work.

## Exit Criteria

- `LICENSE` exists and matches the workspace `MIT` license metadata, or README/Cargo metadata are changed to the chosen license state.
- README does not present failing scenarios as passing demos.
- `roomci run` succeeds for every scenario listed as a passing demo.
- Any intentional failure-report demo is clearly separated and verified as expected to fail.
- `roomci serve --check` validates a service-mode scenario config and is covered by tests.
- README test count, coverage, and quality-gate claims match current verified output.
- Public-facing NOT A HOTEL wording avoids implying access to private/internal systems.
- Release-readiness commands are recorded in `tasks/phase8/phase_status.md`.
