# Phase 0 Goal — Latest Scenario Contract, CLI, Reports

## Goal

Move the implementation from the legacy smart-room scenario shape to the latest `roomci-docs-latest` contract.

The first success target is:

```bash
roomci validate examples/local_first_cloud_outage.yaml
roomci run examples/local_first_cloud_outage.yaml --report-md report.md --report-json report.json --junit report.xml
```

## In Scope

- Scenario parser aligned with `docs/15_scenario_spec.md`.
- Optional `scenario.clock`; symbolic `T` starts at virtual time zero.
- Top-level `devices`, `mqtt`, `faults`, `steps`, `assertions`, and `report` fields.
- CLI aliases: `--report-md`, `--report-json`, and existing `--markdown`, `--json`.
- Local-first cloud outage scenario support.
- JSON, Markdown, and JUnit reports.
- Backward compatibility only where cheap.

## Non-goals

- Real MQTT broker.
- `serve` implementation.
- Docker image build.
- Full Modbus, DALI, contact I/O, BMS, or network simulation.

## Exit Criteria

- Latest `examples/local_first_cloud_outage.yaml` validates.
- Running latest local-first scenario produces reports.
- Scenario result passes when local retained state works despite `mqtt.cloud` outage.
- `cargo test` passes.
- CLI examples in `docs/02_product_requirements.md` and Compose use supported flags.
