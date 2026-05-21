# Task 03: Trace Metadata And Run Correlation

## Finding

Reports and timeline events are deterministic, but they do not carry a formal
run correlation model. CI users can read files by path, but a downstream system
cannot reliably join JSON reports, timeline exports, JUnit output, and
serve-mode health state without inferring identity from filenames.

## Scope

- Add stable run metadata to `RunReport`:
  - `schema_version: "roomci.report.v1"`
  - `run_id`
  - `scenario_name`
  - `generated_by: "roomci"`
- Use a deterministic default `run_id` derived from scenario name for local
  scenario mode.
- Add optional CLI flag:
  - `--run-id <id>`
- Add serve-mode run id propagation:
  - `/health.latest_report_id`
  - `/reports/latest.json`
  - timeline exports
- Add a trace-friendly field to timeline export events:
  - `trace_id`
  - `span_id`
  - `parent_span_id`

These fields should be deterministic strings for CI evidence, not distributed
tracing claims.

## Acceptance Criteria

- JSON report includes `schema_version` and `run_id`.
- Timeline export events include the same `run_id`.
- CLI `--run-id smoke-123` produces artifacts with `run_id = "smoke-123"`.
- Serve mode uses the latest report id consistently across `/health`, reports,
  and timeline exports.
- Docs explicitly state this is trace metadata, not an OpenTelemetry
  implementation.

## Test Commands

```bash
cargo test -p roomci-core run_id
cargo test -p roomci-report trace
cargo test -p roomci-cli --test cli run_id
cargo test -p roomci-serve --lib latest_report_id
```

```bash
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml \
  --run-id phase19-smoke \
  --report-json reports/phase19-smoke.json \
  --timeline-json reports/phase19-smoke.timeline.json
python3 - <<'PY'
import json
from pathlib import Path
report = json.loads(Path("reports/phase19-smoke.json").read_text())
timeline = json.loads(Path("reports/phase19-smoke.timeline.json").read_text())
assert report["schema_version"] == "roomci.report.v1"
assert report["run_id"] == "phase19-smoke"
assert timeline[0]["run_id"] == "phase19-smoke"
assert "trace_id" in timeline[0]
assert "span_id" in timeline[0]
PY
```

## Out Of Scope

- W3C Trace Context propagation over MQTT/HTTP.
- OpenTelemetry collector export.
- Cross-process distributed tracing.
