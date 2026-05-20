# Task 03: Network And Control-Panel Fault Profiles

## Why

Hospitality stack coverage marks VLAN/firewall/segment failures and control-panel/electrical faults as important next coverage, but they need precise QA semantics.

## Acceptance Criteria

- Define candidate scenario profiles for:
  - VLAN or segment isolation,
  - firewall policy drift,
  - local broker unreachable but edge still alive,
  - 24V power degradation,
  - UPS low/battery degraded,
  - circuit-protector trip,
  - redundant PSU or edge-computer failover evidence.
- Mark which are executable QA checks versus documentation-only assumptions.
- Explicitly state that physical electrical safety validation is out of scope.
- Promote high-value profiles to later implementation tasks.
