# Design Principles

## Guest Impact First

`roomci` treats smart-home failures as stay-experience risks. Assertions and reports should explain guest-visible impact, not only internal state mismatch.

## Field Failures Before Feature Completeness

The product prioritizes failure modes that are hard to reproduce outside the site: cloud outage, edge failover, contact alarms, scene partial failure, access drift, and commissioning gaps.

## Behavioral Simulation Over Protocol Completeness

The MVP models protocol behavior at the QA contract level. It does not attempt full KNX, DALI, BACnet, SIP, Matter, or Modbus compatibility.

## CI Reproducibility Over Hardware Fidelity

The primary target is local and CI execution without real devices. Deterministic virtual time is more valuable than emulating every hardware edge case.

## Operations Are Product Surface

BMS alerts, Slack notifications, phone escalation, tickets, and runbooks are part of the product experience because they determine how quickly a guest-impacting issue is handled.

## Commissioning Knowledge Should Become Code

Room/device declarations, register maps, scenes, contacts, and access-control expectations should become executable scenarios instead of remaining only in field notes.

## Local-first Reliability Matters

The cloud can fail without breaking the stay experience if local control, local MQTT, and edge routing keep working.

## Do Not Claim Internal Compatibility

`roomci` models common hospitality smart-home and building-automation patterns. It must not claim access to or compatibility with any organization's private internal systems.

## Small External Engine

The tool should stay useful as an external QA binary and Docker image. It should complement production stacks rather than replace them.
