# 14. Intercom and Access Control

## Scope

Intercom and access control are important but should not be MVP-heavy.

v0.1 should include:

- PIN check-in event model
- contact relay output
- staff-call event mock

v0.3 may include:

- DoorBird-like API
- SIP call mock
- DTMF relay trigger
- Twilio outage simulation
- Zoom Phone mock
- local PBX fallback
- ONVIF-like camera event
- PoE speaker webhook mock

## Check-in model

```yaml
checkin:
  guest_id: guest_001
  pin_code: "123456"
  valid_from: 2026-01-01T15:00:00+09:00
  valid_until: 2026-01-02T11:00:00+09:00
```

## Safe mock flow

The executable scope is a scenario-only safe mock. It records PIN decisions,
relay pulse requests, staff-call attempts, and fallbacks as evidence, but it
never performs real unlock authorization or controls physical locks.

```yaml
steps:
  - at: T
    intercom:
      id: front_gate
      event: pin_check
      outcome: accepted
  - at: T+1s
    intercom:
      id: front_gate
      event: relay_pulse
      outcome: requested
  - at: T+5s
    intercom:
      id: front_gate
      event: staff_call
      outcome: failed
      fallback: manual_staff_callback
assertions:
  - at: T+6s
    assert:
      intercom_relay: safe_evidence
```

## Contact relay output

```yaml
contacts:
  outputs:
    - id: gate_relay
      type: pulse
      duration_ms: 1000
```

## Staff call model

```yaml
staff_call:
  source: intercom.main_gate
  route: online_concierge
  fallback: local_pbx
```

## DTMF future

DTMF can be modeled as an input event that triggers a relay.

```yaml
steps:
  - at: T
    call:
      id: staff_call_001
      dtmf: "1"
  - at: T+1s
    assert:
      contact:
        id: gate_relay
        pulsed: true
```

## Twilio outage scenario future

```yaml
faults:
  - at: T
    target: twilio_sip_trunk
    type: offline

assertions:
  - at: T+5s
    target: local_pbx
    condition: used_as_fallback
```

## Access control drift scenario

A lightweight module can emulate UniFi/Okta style access drift by comparing identity-group membership with access-system users.

```yaml
scenario:
  name: access_permission_drift

inputs:
  identity_group:
    - alice@example.com
    - bob@example.com
  access_system_group:
    - alice@example.com
    - bob@example.com
    - retired@example.com

assertions:
  - at: T
    assert:
      access_control_drift: detected
```

Report:

```txt
PASS access_permission_drift

Unexpected access user:
- retired@example.com

Risk:
Former member may still unlock protected space.
```
