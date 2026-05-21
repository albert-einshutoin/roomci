# Phase 21 Status: B-Tier Protocol Profile Completion

Status: `done`

## Tasks

| Task | Status | Notes |
|---|---|---|
| 01 Matter prototype profile | `done` | Added Matter gateway contract profile with endpoint/cluster/attribute/command fields and explicit non-goals. |
| 02 BACnet contract profile | `done` | Added BACnet object/property/event contract profile without BACnet/IP endpoint claims. |
| 03 KNX contract profile | `done` | Added KNX group-address contract profile without ETS import or bus timing claims. |
| 04 OPC UA contract profile | `done` | Added OPC UA node/attribute/event profile without server/security/subscription claims. |
| 05 B Tier evidence and docs gate | `done` | Added `make protocol-profile-smoke`, support matrix, conformance registry, evidence checker, and evaluator docs. |

## Current Blocker

None. Real customer protocol maps remain the next validation input before any
profile should be promoted beyond `contract_profile`.
