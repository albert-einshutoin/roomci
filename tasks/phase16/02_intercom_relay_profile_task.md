# Task 02: Intercom And Relay Profile

## Why

Docs mention PIN, relay pulse, staff call, DoorBird/SIP/DTMF-like flows, and Twilio/phone failures. These are important to hospitality QA but can become safety-sensitive if modeled as real unlock behavior.

## Acceptance Criteria

- Define a safe contract/mock profile for intercom and relay events.
- Include PIN accepted/rejected, relay pulse requested, staff call attempted, fallback path, and failure evidence if they fit the product boundary.
- Explicitly keep real unlock authorization out of scope.
- Decide whether this belongs in a scenario-only model, a serve endpoint, or adapter-contract fixtures.
- Create implementation tasks only for the safe subset.
