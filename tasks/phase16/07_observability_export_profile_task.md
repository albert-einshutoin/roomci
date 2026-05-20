# Task 07: Observability Export Profile

## Why

Docs mention Grafana, InfluxDB, time-series DBs, and alerting. roomci should not become an observability platform, but evaluator adoption may improve if reports can be exported into familiar formats.

## Acceptance Criteria

- Decide whether to support an Influx line protocol export, Grafana-friendly JSON, or no additional observability output.
- Keep JSON/Markdown/JUnit as the primary evidence formats unless a concrete evaluator need justifies more.
- If promoted, define a later implementation task and release claim boundary.
