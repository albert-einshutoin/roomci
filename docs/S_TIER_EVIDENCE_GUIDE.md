# S Tier Evidence Guide

This guide is the copy-paste path for evaluating `roomci` as a release-candidate
QA contract emulator.

S Tier evidence means the project can prove adapter CI, timeline export, trace
metadata, observability artifacts, and GitHub Actions integration without
claiming hosted observability, production orchestration, or protocol
certification.

## Local Gate

Run the complete local approximation:

```bash
make verify
```

Run the focused S Tier artifact gate:

```bash
make s-tier-evidence-smoke
```

Expected artifacts:

| Artifact | Path | Purpose |
|---|---|---|
| Full JSON report | `reports/phase19-smoke.json` | Complete run report with `schema_version` and `run_id` |
| Markdown report | `reports/phase19-smoke.md` | Human-readable evaluator summary |
| JUnit report | `reports/phase19-smoke.xml` | CI test report |
| Timeline JSON | `reports/phase19-smoke.timeline.json` | Stable `roomci.timeline.v1` event array |
| Timeline NDJSON | `reports/phase19-smoke.timeline.ndjson` | One timeline event per line |
| Observability JSON | `reports/phase19-smoke.observability.json` | Deterministic counters and run summary |

## Adapter CI

Run:

```bash
make adapter-samples-smoke
```

This starts `roomci serve` in Docker Compose and runs the Go and TypeScript
adapter samples against HTTP, MQTT, and Modbus TCP surfaces.

## Claim Evidence

Run:

```bash
make protocol-evidence
```

This verifies that claims in `docs/protocol-evidence.json` point to real docs,
examples, tests, and release-gate commands.

## GitHub Actions

The repository workflow at `.github/workflows/smart-home-ci.yml` runs:

- Rust quality gates
- Docker scenario suite
- Phase 17 scenario artifacts
- timeline JSON / NDJSON artifacts
- observability JSON artifacts
- protocol/domain evidence check
- adapter sample smoke

The workflow uploads `reports/` as an artifact.

## Artifact Contract

Timeline events use `roomci.timeline.v1`:

```json
{
  "schema_version": "roomci.timeline.v1",
  "run_id": "phase19-smoke",
  "scenario_name": "local_first_cloud_outage",
  "trace_id": "roomci-trace-phase19-smoke",
  "span_id": "roomci-span-phase19-smoke-000000",
  "parent_span_id": null,
  "sequence": 0,
  "at": "T+0s",
  "event_type": "event",
  "target": null,
  "message": "..."
}
```

Observability artifacts use `roomci.observability.v1`:

```json
{
  "schema_version": "roomci.observability.v1",
  "run_id": "phase19-smoke",
  "scenario_name": "local_first_cloud_outage",
  "result": "passed",
  "timeline_event_count": 7,
  "assertion_count": 2,
  "failed_assertion_count": 0,
  "events_by_type": {},
  "assertions_by_status": {
    "passed": 2,
    "failed": 0
  },
  "impact_levels": []
}
```

## Boundary

These artifacts are export contracts only. `roomci` does not run Grafana,
InfluxDB, Prometheus, OpenTelemetry Collector, Jaeger, production alerting,
production orchestration, or a production incident-response system.
