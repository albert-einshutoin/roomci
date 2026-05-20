# Task 03 — Control-panel Fault Profiles

## Goal

Represent control-panel and electrical-infrastructure concerns as QA fault profiles without pretending to validate physical electrical safety.

## Implementation Scope

- Add or document fault profiles for:
  - 24V power supply degradation
  - UPS battery degradation
  - redundant power path failure
  - breaker / circuit protector isolation
  - edge-computer primary failure and secondary takeover
  - industrial switch or local network segment failure
- Ensure these profiles feed reports and BMS/ops evidence.
- Clearly state that `roomci` does not validate wiring, high/low voltage safety, lightning protection, or electrical code compliance.

## Acceptance Criteria

- Control-panel concerns are visible as software/ops QA scenarios.
- Physical safety claims are explicitly out of scope.
- At least one profile is tied to BMS/ops notification evidence.
