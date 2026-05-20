# Phase 14 Test Plan

## Quality Gates

- Coverage map exists and classifies the reported hospitality stack without claiming private compatibility.
- Core QA journey has a runnable scenario or PoC pack that includes:
  - MQTT command/state contract
  - edge routing or failover
  - device protocol behavior such as Modbus, DALI-like lighting, contact I/O, or HVAC
  - network or control-panel fault
  - BMS/ops escalation
  - JSON, Markdown, and JUnit evidence
- Contract/mock-only technologies have explicit boundaries:
  - SIP/DTMF/ONVIF/access control are not presented as production-compatible.
  - Slack/Zoom Phone/PagerDuty/Grafana/InfluxDB are represented as evidence outputs or contracts, not real integrations by default.
  - AWS/GCP/Cloudflare appear as integration assumptions or future adapters, not local cloud replacements.
- Product docs explain why full CAD, construction, physical electrical validation, and real lock authorization are out of scope.
- `make poc-hospitality`, `make poc-building-automation`, and `make poc-bms-ops` remain passing after coverage changes.
- If a new core QA journey target is added, it runs from a clean checkout and writes report artifacts under `reports/`.

## Done Means

Phase 14 is done when a hospitality or building-automation evaluator can read the coverage map and immediately understand:

- what `roomci` already exercises,
- what can be configured through contracts,
- what is intentionally only mocked,
- what is future protocol work,
- and why this is a smart-home QA contract emulator rather than a full hospitality technology-stack emulator.
