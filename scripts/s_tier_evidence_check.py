#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def fail(message):
    print(f"s-tier evidence check failed: {message}", file=sys.stderr)
    sys.exit(1)


def read_json(path):
    try:
        return json.loads(path.read_text())
    except Exception as error:
        fail(f"could not read {path}: {error}")


def main():
    if len(sys.argv) != 2:
        fail("usage: s_tier_evidence_check.py <artifact-prefix>")
    prefix = Path(sys.argv[1])
    report = read_json(prefix.with_suffix(".json"))
    timeline = read_json(prefix.with_suffix(".timeline.json"))
    observability = read_json(prefix.with_suffix(".observability.json"))
    ndjson_path = prefix.with_suffix(".timeline.ndjson")

    if report.get("schema_version") != "roomci.report.v1":
        fail("report schema_version mismatch")
    if report.get("run_id") != "phase19-smoke":
        fail("report run_id mismatch")
    if not isinstance(timeline, list) or not timeline:
        fail("timeline export is empty or not an array")
    first = timeline[0]
    for key in [
        "schema_version",
        "run_id",
        "scenario_name",
        "sequence",
        "at",
        "event_type",
        "target",
        "message",
        "trace_id",
        "span_id",
    ]:
        if key not in first:
            fail(f"timeline event missing {key}")
    if first["schema_version"] != "roomci.timeline.v1":
        fail("timeline schema_version mismatch")
    if first["run_id"] != "phase19-smoke":
        fail("timeline run_id mismatch")
    if len(ndjson_path.read_text().splitlines()) != len(timeline):
        fail("timeline NDJSON line count does not match JSON event count")
    if observability.get("schema_version") != "roomci.observability.v1":
        fail("observability schema_version mismatch")
    if observability.get("run_id") != "phase19-smoke":
        fail("observability run_id mismatch")
    if observability.get("timeline_event_count") != len(timeline):
        fail("observability timeline_event_count mismatch")
    print("s-tier evidence ok")


if __name__ == "__main__":
    main()
