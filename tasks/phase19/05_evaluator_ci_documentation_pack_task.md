# Task 05: Evaluator CI Documentation Pack

## Finding

`docs/17_docker_ci_design.md`, `docs/EVALUATION_EVIDENCE_PACK.md`, and
`.github/workflows/smart-home-ci.yml` exist, but an external evaluator still has
to assemble the recommended CI path from multiple places.

S Tier requires a single evaluator-facing path that says exactly what to copy,
what artifacts to expect, and what claims the artifacts prove.

## Scope

- Add `docs/S_TIER_EVIDENCE_GUIDE.md`.
- Update docs index and README links.
- Include three copy-paste paths:
  - local `make verify`
  - Docker Compose PoC
  - GitHub Actions workflow example
- Include artifact map:
  - full JSON report
  - Markdown summary
  - JUnit XML
  - timeline JSON
  - timeline NDJSON
  - observability JSON
- Include claim map:
  - adapter CI
  - protocol evidence
  - timeline export
  - trace metadata
  - observability artifact
  - GitHub Actions examples

## Acceptance Criteria

- Evaluator guide has exact commands and expected artifact paths.
- README links to the guide from the Quality Gates or Integration section.
- `docs/README.md` links to the guide.
- `docs/RELEASE_CHECKLIST.md` includes the new S Tier artifact checks.
- No wording implies hosted observability, production orchestration, full
  certification, or private customer compatibility.

## Test Commands

```bash
rg -n "S_TIER_EVIDENCE_GUIDE|timeline-json|timeline-ndjson|observability-json|adapter-samples-smoke|protocol-evidence" README.md docs
```

```bash
make protocol-evidence
```

## Out Of Scope

- Creating a marketing landing page.
- Adding screenshots or hosted dashboards.
- Publishing package artifacts.
