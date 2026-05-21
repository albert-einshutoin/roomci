# Phase 19 Goal: S-Tier Observability And CI Evidence Completion

Phase 19 closes the remaining S Tier gaps after Phase 18.

The product already has adapter sample smoke, protocol evidence checks, JSON /
Markdown / JUnit reports, Docker Compose smoke, and a GitHub Actions workflow.
What remains is not more emulator breadth. The remaining S Tier work is making
those signals easy for evaluators to consume in real CI and observability
workflows.

## S Tier Completion Definition

`roomci` reaches S Tier evidence readiness when:

- adapter samples are verified in GitHub Actions, not only by local `make verify`
- Phase 17 scenarios are included in CI examples and artifact uploads
- protocol/domain claim evidence is checked in CI
- timeline events can be exported as a stable machine contract independent of
  the full report shape
- tracing metadata exists for each run and timeline event
- observability export is a thin contract output, not a Grafana/Influx platform
  emulator
- docs show copy-paste GitHub Actions patterns for local repos, Docker users,
  and artifact consumers

## Explicit Non-Goals

- Becoming an observability backend.
- Running Grafana, InfluxDB, OpenTelemetry Collector, Prometheus, or Jaeger.
- Competing with MQTT brokers or production orchestrators.
- Claiming production incident-response authority.
- Adding Matter, BACnet, KNX, or OPC UA runtime scope in this phase.
