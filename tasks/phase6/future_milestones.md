# Phase 6 Future Milestones

Phase 6 does not make every future integration part of the MVP runtime. It records post-MVP work as explicit milestones so future tasks can be split without blurring the current product boundary.

## v0.2 — Reliability and Commissioning Depth

| Milestone | Scope | Validation Artifact |
|---|---|---|
| WAN / Starlink failover | Backup WAN activation and ops notification | `examples/starlink_failover.yaml` runs |
| Comfort automation | Discomfort index target and no user override | `examples/comfort_auto_mode.yaml` runs |
| Commissioning checklist | Generate field-engineer checklist from room/device declarations | `examples/commissioning_checklist.yaml` runs |

## v0.3 — Access and Intercom

| Milestone | Scope | Validation Artifact |
|---|---|---|
| Access-control drift | Compare identity source against access-system users | `examples/access_permission_drift.yaml` runs |
| Intercom / relay mock | PIN, relay pulse, staff call, local fallback | Planned scenario schema extension |
| DoorBird / SIP / DTMF-like profiles | Model vendor-like intercom failure modes | Planned protocol profile |

## Out of MVP

- Full SIP/PBX behavior.
- Real vendor API compatibility.
- Real Matter / CSA Aliro conformance.
- Production access-control authorization.
