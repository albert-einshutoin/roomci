# Hardware-to-Docker CI use cases

This guide shows how teams with real hardware in different locations can turn
field captures into deterministic Docker CI scenarios. The goal is not to put
the real device in CI. The goal is to capture the contract the device exposed
and replay it as a testable behavior model.

## TDD workflow

1. Capture one hardware behavior from an engineer-owned setup: MQTT
   topic/payload, Modbus register map, dry-contact alert, or network fault.
2. Declare the customer-facing contract in
   `adapter-contracts/examples/hardware_ci_multi_engineer_lab.yaml`.
3. Add a failing scenario test in `crates/roomci-core/src/tests.rs` that names
   the expected replay fixture.
4. Add the scenario under `examples/` and make it pass locally with
   `cargo test -p roomci-core hardware_ci`.
5. Run the same use cases inside Docker with `make hardware-ci-usecases-smoke`.
6. Upload JSON, Markdown, JUnit, timeline, and observability artifacts from CI.

## Use case patterns

| Pattern | Scenario | Real-world owner | Captured surface | CI signal |
|---|---|---|---|---|
| Room fleet MQTT replay | `examples/hardware_ci_mqtt_room_fleet.yaml` | Lighting and climate engineers | MQTT command/state topics, retained payloads, required fields | Two independent room devices update deterministic retained state |
| Modbus plus BMS commissioning | `examples/hardware_ci_modbus_bms_commissioning.yaml` | Facilities and commissioning engineers | Modbus holding/input registers, dry-contact alert, ticket acknowledgement | Register scale/write behavior and critical alert workflow pass in one report |
| Mixed-protocol regression | `examples/hardware_ci_mixed_protocol_regression.yaml` | Platform QA across multiple hardware owners | MQTT gateway state, Modbus VAV setpoint, BMS smoke alert, network isolation | One Docker run proves cross-team captures still compose |

## Docker CI path

Run the dedicated Docker gate:

```bash
make hardware-ci-usecases-smoke
```

That target builds the `hardware-ci-usecases` Compose service, validates the
adapter contract, validates all three scenario files, then runs each scenario
inside the roomci container with report outputs in `reports/`.

GitHub Actions also runs the same Compose service in
`.github/workflows/smart-home-ci.yml` under the Docker scenario suite.

## Presentation outline

1. Problem: real-device QA is split across teams, desks, labs, and customer
   sites, so regressions are hard to reproduce before release.
2. Principle: CI should replay hardware contracts, not require every real
   device to be online in CI.
3. Workflow: capture one behavior, write the failing test, encode the contract,
   replay it in Docker, and keep the report artifact.
4. Pattern 1: MQTT room fleet replay for lighting and thermostat engineers.
5. Pattern 2: Modbus/BMS commissioning replay for facilities engineers.
6. Pattern 3: mixed-protocol regression for platform QA and release gates.
7. Boundary: roomci is a behavioral emulator; hardware-in-the-loop remains a
   separate final confidence layer.

## Non-goals

- It does not certify a full wire-protocol implementation.
- It does not remove the need for hardware-in-the-loop or customer-site tests.
- It does not store production credentials, customer secrets, or private device
  dumps in the public repo.
