# Product Guide

This is the canonical product-positioning guide for `roomci`.

## Product Category

`roomci` is a CI-first QA contract emulator for MQTT-based edge-device,
smart-home, and building-automation systems.

It is useful when a team needs to test behavior before real devices, real
sites, or customer-specific integrations are available:

```txt
scenario -> contract emulator -> assertions -> timeline/report evidence
```

## Primary Users

- Edge-device and controller teams that need repeatable MQTT command/state
  contract tests.
- Smart-home teams that need local-first and failure-mode checks.
- Building-automation teams that need practical Modbus, lighting, contact I/O,
  comfort, BMS, and operations behavior coverage.
- CI owners who need deterministic artifacts rather than manual staging notes.
- Field/commissioning teams that want common failure knowledge encoded as
  executable scenarios.

## What It Does

`roomci` provides:

- YAML scenarios for expected device, edge, network, and operations behavior;
- a virtual-time runner for deterministic local and CI execution;
- a localhost `serve` mode for external controller PoCs;
- MQTT 3.1.1 and Modbus TCP practical subsets for standard-client smoke tests;
- adapter contracts for company-specific topic maps, payload schemas, register
  maps, identity fields, auth expectations, and acceptance criteria;
- JSON, Markdown, JUnit, timeline, observability, and protocol-evidence
  artifacts.

## What It Is Not

`roomci` is not:

- a production MQTT broker;
- a production smart-home controller;
- a protocol certification or conformance suite;
- a full BACnet, KNX, Matter, OPC UA, DALI, SIP, or Modbus implementation;
- a vendor cloud, BMS, intercom, access-control, network, notification, CAD, or
  identity-platform emulator;
- a private compatibility layer for any specific company without supplied
  contracts.

## Domain Packs

The reusable core is industry-wide:

```txt
MQTT / edge / device / operations QA contract emulator for CI
```

Domain packs provide concrete examples on top of that core.

| Domain Pack | Purpose | Current Maturity |
|---|---|---|
| Generic MQTT edge | Command/state, retained state, serve-mode client checks | Strong |
| Building automation | Modbus TCP subset, lighting behavior, contact I/O, BMS alerts | Strong for contract tests |
| Hospitality smart-home | Local-first control, comfort automation, access/intercom safe mocks, guest-impact reports | Strong demonstration pack |
| Future protocol profiles | Matter, BACnet, KNX, OPC UA adapter-contract templates | Contract profile only |

Hospitality remains a useful domain pack because it compresses many hard
failure modes into one evaluator journey: local-first operation, edge failover,
network outage, device drift, comfort behavior, BMS escalation, and field
reporting. It should not be read as a claim of compatibility with any private
hospitality operator or vendor stack.

## Category Fit

Use `roomci` when:

- pre-device or pre-site QA evidence is valuable;
- failures cross controller code, edge services, devices, and operations;
- private integration details can be represented as contracts;
- CI artifacts matter to both software and field teams;
- the team wants a reusable harness instead of one-off mocks.

Do not use `roomci` as the final gate when:

- certification-grade protocol conformance is required;
- electrical timing, sensor noise, or hardware commissioning is the core risk;
- the team cannot provide protocol contracts or acceptance criteria;
- production authorization, real lock control, or safety authority is required.

## Comparison

| Alternative | Where It Wins | Where roomci Wins |
|---|---|---|
| Real-device staging | Highest physical fidelity | Faster local/CI feedback and deterministic failure reproduction |
| Generic MQTT broker | Real broker semantics | Device, edge, operations, reports, and acceptance criteria in one harness |
| Ad hoc scripts | Fast one-off tests | Reusable contracts, typed validation, PoC packs, and reports |
| Home automation platforms | Broad ecosystem integrations | CI-focused contracts and no home-server dependency |
| Cloud IoT emulators | Cloud API coverage | Local-first, edge, device, BMS, Modbus, contact, and field-failure modeling |
| Hardware-in-the-loop | Timing and electrical realism | Earlier pre-HIL coverage and cheaper regression loops |

## Evaluator Journey

1. Run `make demo` or `make poc-core-qa`.
2. Inspect JSON, Markdown, JUnit, timeline, and observability artifacts.
3. Fill an adapter contract with real topic maps, payload fields, register maps,
   auth expectations, and acceptance criteria.
4. Drive `roomci serve` with a standard MQTT/Modbus/HTTP client or the sample
   adapters.
5. Use the report evidence to decide which gaps require real hardware, a
   customer contract, or a future protocol profile.

## Boundary Statement

The strongest claim `roomci` should make is:

```txt
roomci makes edge-device, smart-home, and building-automation behavior
reproducible as local and CI contract tests before real devices or sites are
available.
```

It should not claim full protocol conformance, vendor compatibility, production
orchestration, or private-stack emulation.

