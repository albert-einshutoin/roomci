# Task 01: Intercom And Relay Safe Mock

## Why

Hospitality evaluators need to validate PIN, call, and relay event handling, but `roomci` must not model real unlock authorization or physical access safety.

## Acceptance Criteria

- Add a scenario-only intercom/relay profile for PIN accepted/rejected, relay pulse requested, staff call attempted, fallback path, and failure evidence.
- Keep real unlock authorization explicitly out of scope.
- Add report evidence and examples for accepted, rejected, and fallback paths.
- Update hospitality coverage docs and adapter contract examples.

## Out of Scope

- Real lock control.
- SIP/DTMF/ONVIF/DoorBird/Twilio/Zoom emulation.
