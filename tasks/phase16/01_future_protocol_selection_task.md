# Task 01: Future Protocol Selection

## Why

BACnet, KNX, Matter, OPC UA, Zigbee, and Thread are visible in the support registry as future profiles, but they should not all become implementation targets by default.

## Acceptance Criteria

- Rank each protocol by evaluator value, implementation tractability, maintenance cost, and fit with the contract-emulator product boundary.
- For each protocol, choose one outcome:
  - promote to a concrete later-phase implementation task,
  - keep as adapter-contract only,
  - or keep as a documented non-goal.
- Identify the first narrow subset if promoted.
- Update the protocol registry and support matrix with the decision.

## Default Bias

Prefer BACnet/IP or KNX only if there is a concrete evaluator use case. Prefer not to implement radio stacks such as Zigbee/Thread directly unless a simulated gateway profile is enough.
