# Task 01: GitHub Actions Parity For S Tier Gates

## Finding

Local `make verify` now runs adapter samples and protocol evidence checks, but
`.github/workflows/smart-home-ci.yml` is behind the local gate:

- it does not run `make adapter-samples-smoke`
- it does not run `make protocol-evidence`
- its scenario validate/run lists do not include all Phase 17 scenarios
- its uploaded artifact set does not include timeline-specific exports because
  those do not exist yet

This weakens the S Tier claim because an evaluator looking at GitHub Actions
does not see the same evidence that exists locally.

## Scope

- Update `.github/workflows/smart-home-ci.yml`.
- Prefer named jobs over a single opaque `make verify` job so failures are easy
  to diagnose.
- Include these gates:
  - Rust quality gates
  - Docker scenario suite
  - protocol evidence check
  - protocol smoke
  - adapter sample smoke
  - Phase 17 scenario run artifacts
- Keep runtime reasonable. It is acceptable for GitHub Actions to run a
  slightly split form of `make verify` if the release checklist documents the
  difference.

## Acceptance Criteria

- CI runs `make protocol-evidence`.
- CI runs `make adapter-samples-smoke` or an equivalent Compose invocation.
- CI validates and runs:
  - `examples/intercom_relay_safe_mock.yaml`
  - `examples/network_control_panel_fault_profiles.yaml`
  - `examples/comfort_timeseries_replay.yaml`
- CI uploads JSON, Markdown, and JUnit artifacts for all passing scenario
  groups.
- README and release checklist describe the CI gate truthfully.

## Test Commands

```bash
python3 - <<'PY'
from pathlib import Path
text = Path(".github/workflows/smart-home-ci.yml").read_text()
required = [
    "make protocol-evidence",
    "make adapter-samples-smoke",
    "intercom_relay_safe_mock.yaml",
    "network_control_panel_fault_profiles.yaml",
    "comfort_timeseries_replay.yaml",
    "actions/upload-artifact@v4",
]
missing = [item for item in required if item not in text]
if missing:
    raise SystemExit(f"missing CI evidence: {missing}")
PY
```

```bash
make protocol-evidence
make adapter-samples-smoke
```

## Out Of Scope

- Publishing Docker images.
- Adding release automation or tags.
- Adding Matter/BACnet/KNX/OPC UA jobs.
