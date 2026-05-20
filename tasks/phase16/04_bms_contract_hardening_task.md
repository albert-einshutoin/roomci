# Task 04: BMS Contract Hardening

## Why

The BMS/contact endpoint is intentionally lightweight. Docs mention stricter validation, HMAC signatures, replay protection, schema versioning, retry semantics, Content-Type enforcement, and severity enums as deferred work.

## Acceptance Criteria

- Decide which hardening features belong in adapter contracts and which, if any, belong in `roomci serve`.
- Define acceptance criteria for HMAC/replay/schema-version validation if promoted.
- Keep real Slack/Zoom/PagerDuty/Twilio calls disabled by default.
- Update external protocol docs so evaluators do not confuse the current endpoint with a production webhook implementation.
