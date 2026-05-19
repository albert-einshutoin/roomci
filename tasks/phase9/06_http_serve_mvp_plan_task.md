# Task 06 — HTTP Serve MVP Plan

## Objective

Define the next implementation step for turning the PC into a localhost-bound virtual device/system without overclaiming MQTT broker compatibility.

## Acceptance Criteria

- Document an HTTP-first `serve` MVP plan.
- Include endpoints:
  - `GET /health`
  - `GET /scenario`
  - `GET /state`
  - `GET /timeline`
  - `POST /run`
  - `POST /fault`
  - `GET /reports/latest`
- Explain why HTTP comes before full MQTT broker compatibility.
- Explain what would be required for MQTT-compatible serve mode later.
- Keep this as a plan unless explicitly implementing it in a future phase.
