# Task 04: Observability Artifact Profile

## Finding

Docs mention Grafana-like metrics and time-series exports, but the current
implementation only emits reports. That is honest, but S Tier readiness needs a
small observability artifact profile so evaluators can see how `roomci` output
would be ingested without expecting a hosted observability stack.

## Scope

- Add an observability export renderer that produces deterministic JSON with:
  - counters by `event_type`
  - counters by assertion status
  - scenario result
  - run id
  - scenario name
  - timeline event count
  - failed assertion count
  - impact levels observed
- Add CLI flag:
  - `--observability-json <path>`
- Add serve endpoint:
  - `GET /observability/latest.json`
- Add docs explaining how this maps to Grafana/Influx/Prometheus ingestion
  without emulating those systems.

Example output:

```json
{
  "schema_version": "roomci.observability.v1",
  "run_id": "local_first_cloud_outage",
  "scenario_name": "local_first_cloud_outage",
  "result": "passed",
  "timeline_event_count": 7,
  "assertion_count": 2,
  "failed_assertion_count": 0,
  "events_by_type": {
    "event": 1,
    "mqtt_retained_state_updated": 2
  },
  "assertions_by_status": {
    "passed": 2,
    "failed": 0
  },
  "impact_levels": []
}
```

## Acceptance Criteria

- `roomci-report` has an observability JSON renderer.
- CLI writes `--observability-json`.
- Serve mode exposes `/observability/latest.json`.
- Release checklist includes the observability export gate.
- Docs state that Grafana, InfluxDB, Prometheus, and OTel remain external
  systems.

## Test Commands

```bash
cargo test -p roomci-report observability
cargo test -p roomci-cli --test cli observability
cargo test -p roomci-serve --lib observability
```

```bash
cargo run -p roomci-cli -- run examples/bms_sauna_emergency_alert.yaml \
  --observability-json reports/bms.observability.json
python3 - <<'PY'
import json
from pathlib import Path
data = json.loads(Path("reports/bms.observability.json").read_text())
assert data["schema_version"] == "roomci.observability.v1"
assert data["scenario_name"] == "bms_sauna_emergency_alert"
assert data["timeline_event_count"] > 0
assert "events_by_type" in data
assert "assertions_by_status" in data
PY
```

## Out Of Scope

- Influx line protocol in this phase.
- Prometheus endpoint.
- Grafana dashboard files.
- Alert routing or notification side effects.
