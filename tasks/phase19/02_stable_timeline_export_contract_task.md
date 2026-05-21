# Task 02: Stable Timeline Export Contract

## Finding

`roomci` has timeline data in:

- CLI verbose output
- full JSON report
- Markdown report
- `GET /timeline`
- `GET /reports/latest.json`

That is useful, but it is not a stable timeline-specific contract. Evaluators
who want to feed `roomci` into CI dashboards, log search, traces, or data
pipelines currently have to parse the full report shape or scrape Markdown.

## Scope

- Add a dedicated timeline export in `roomci-report`.
- Add CLI output flags:
  - `--timeline-json <path>` for a JSON array
  - `--timeline-ndjson <path>` for one event per line
- Add serve endpoints:
  - `GET /timeline.export.json`
  - `GET /timeline.ndjson`
- Keep the exported fields stable and minimal:

```json
{
  "schema_version": "roomci.timeline.v1",
  "run_id": "local_first_cloud_outage",
  "scenario_name": "local_first_cloud_outage",
  "sequence": 0,
  "at": "T+0s",
  "event_type": "event",
  "target": null,
  "message": "cloud MQTT broker unavailable"
}
```

## Acceptance Criteria

- `roomci-report` exposes timeline JSON and NDJSON renderers.
- CLI can write timeline JSON and NDJSON artifacts.
- Serve mode can return timeline JSON and NDJSON.
- Existing full JSON / Markdown / JUnit reports remain unchanged unless Task 03
  intentionally adds trace metadata.
- Exports include external observation overlay events in serve mode, matching
  existing `/timeline` behavior.

## Test Commands

```bash
cargo test -p roomci-report timeline
cargo test -p roomci-cli --test cli timeline
cargo test -p roomci-serve --lib timeline
```

```bash
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml \
  --timeline-json reports/local_first.timeline.json \
  --timeline-ndjson reports/local_first.timeline.ndjson
python3 - <<'PY'
import json
from pathlib import Path
items = json.loads(Path("reports/local_first.timeline.json").read_text())
assert items
for index, item in enumerate(items):
    assert item["schema_version"] == "roomci.timeline.v1"
    assert item["sequence"] == index
    assert item["scenario_name"] == "local_first_cloud_outage"
lines = Path("reports/local_first.timeline.ndjson").read_text().splitlines()
assert len(lines) == len(items)
for line in lines:
    assert json.loads(line)["schema_version"] == "roomci.timeline.v1"
PY
```

## Out Of Scope

- OpenTelemetry SDK dependency.
- Hosted log ingestion.
- Changing scenario semantics.
