# Task 03: BMS Contract Hardening

## Why

The BMS/contact endpoint is useful but intentionally lightweight. Evaluators need a clearer contract boundary for schema, severity, auth, and replay assumptions.

## Acceptance Criteria

- Add adapter-contract fields for severity enum, schema version, Content-Type expectation, optional HMAC metadata, and replay window.
- Decide which checks are enforced by `roomci serve` and which remain adapter-contract validation.
- Keep real Slack/Zoom/PagerDuty/Twilio calls disabled by default.
- Add negative tests and docs for rejected malformed/hardening payloads.

## Out of Scope

- Production webhook security guarantees.
- Real notification-provider integrations.
