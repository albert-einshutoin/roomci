# Task 06 — Access / Intercom Boundary

## Goal

Define how access control and intercom technologies fit into `roomci` without crossing into unsafe or vendor-specific production emulation.

## Implementation Scope

- Classify DoorBird, ONVIF, SIP, DTMF, Twilio SIP Trunk, Zoom Phone, UniFi Door Access, Okta/LDAP, and electric-lock control panels.
- Keep current access drift scenario as a safe QA model.
- Add optional contract/mock profile for:
  - intercom call event
  - DTMF relay command observed
  - gate/door relay output requested
  - access identity drift detected
- Explicitly exclude real unlock authorization and real physical lock control.

## Acceptance Criteria

- Access/intercom coverage is useful for QA without implying production safety.
- Docs tell evaluators which contract inputs they would need to supply.
- Real access-control authorization remains out of scope.
