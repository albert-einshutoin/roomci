# Task 04 — BMS/Ops Contract Depth

## Goal

Deepen BMS and operations coverage around the safety/guest-impact flows that make hospitality smart-home QA different from generic web QA.

## Implementation Scope

- Expand BMS/ops contracts around:
  - safety alerts
  - sauna/emergency button/contact alerts
  - Slack-like notification evidence
  - phone-call escalation evidence
  - runbook URL
  - ticket lifecycle
  - acknowledgement and recovery
- Keep real Slack, Zoom Phone, PagerDuty, Grafana, and InfluxDB integrations disabled by default.
- Make BMS outputs useful in JSON, Markdown, and JUnit reports.

## Acceptance Criteria

- A BMS/ops evaluator can see how their alert contract maps into `roomci`.
- Reports include actionable operations evidence, not just pass/fail.
- Docs distinguish BMS contract emulation from a production BMS product.
