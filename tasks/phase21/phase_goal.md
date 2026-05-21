# Phase 21 Goal: B-Tier Protocol Profile Completion

Phase 21 completes the B Tier protocol-profile surface.

The goal is not full protocol implementation. The goal is to make Matter,
BACnet, KNX, and OPC UA evaluable as honest profile prototypes: contract
fixtures, schema validation, examples, docs, and evidence outputs that show how
a company would map real gateway contracts later.

## B Tier Completion Definition

`roomci` reaches B Tier protocol-profile readiness when:

- Matter has a prototype profile for gateway/device-state contract mapping
- BACnet has a profile for object/property/event-style contracts
- KNX has a profile for group-address style contracts
- OPC UA has a profile for node/attribute/event-style contracts
- all profiles validate as adapter contracts
- all profiles have example scenarios or dry-run evidence
- docs state exact supported fields, required customer inputs, and non-goals
- no profile claims certification or production wire compatibility

## Explicit Non-Goals

- Full Matter controller/fabric/commissioning behavior.
- BACnet/IP endpoint, BBMD, COV, routing, or certification.
- KNX bus endpoint, ETS import, device certification, or telegram timing.
- OPC UA server endpoint, subscriptions, security policy, or address-space
  certification.
- Zigbee or Thread runtime work in this phase.
