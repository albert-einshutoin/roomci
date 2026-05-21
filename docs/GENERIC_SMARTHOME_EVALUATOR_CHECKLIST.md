# Generic IoT / SmartHome Evaluator Checklist

Use this checklist when evaluating `roomci` for a non-hospitality smart-home, IoT, edge-device, or building-automation team.

## What To Bring

- MQTT command/state topic templates.
- MQTT payload schema and required fields.
- Modbus unit ids, register addresses, access mode, scale, and units.
- Device identity and room/site naming conventions.
- BMS/webhook event contract if operations flows are in scope.
- Network or edge-failover assumptions.
- Optional B Tier protocol maps:
  - Matter gateway endpoint/cluster/attribute/command maps.
  - BACnet object/property maps.
  - KNX group-address and datapoint maps.
  - OPC UA namespace/node/attribute maps.
- Pass/fail acceptance criteria and required report artifacts.

## What To Run First

```bash
make poc-generic-mqtt
make poc-building-automation
make protocol-smoke
make adapter-samples-smoke
make protocol-profile-smoke
make protocol-evidence
```

If you want to wire a controller-shaped client instead of only running Make
targets, start from the small Go and TypeScript samples in
[`examples/adapters`](../examples/adapters/).

## What To Inspect

- `docs/PROTOCOL_SUPPORT_MATRIX.md`
- `docs/PROTOCOL_CONFORMANCE_REGISTRY.md`
- `docs/protocol-evidence.json`
- `adapter-contracts/templates/company_adapter_contract.yaml`
- generated reports under `reports/`

## Fit Scorecard

| Dimension | Good Fit Signal | Not a Fit Signal |
|---|---|---|
| MQTT command/state QA | Topics and payloads can be mapped into contracts | You need a production broker replacement |
| Building automation QA | Register maps and alert flows can be represented | You need electrical commissioning certification |
| Local/CI workflow | Docker/Compose evidence is useful | Only real-device HIL is acceptable |
| Operations evidence | Reports and mocked escalation are enough for pre-adoption QA | Real Slack/phone/ticket side effects are required by default |
| Safety-sensitive access flows | Safe scenario evidence for intercom/relay is enough | Real unlock authorization or relay control must be validated |
| Comfort and field faults | Deterministic time-series, network, and control-panel profiles are useful | Thermal physics, electrical timing, or live network chaos is required |
| Protocol depth | Narrow conformance subsets are acceptable | Full protocol certification is required |
| B Tier protocol profiles | Gateway/object/group-address/node maps can be expressed as contracts | You need Matter/BACnet/KNX/OPC UA wire endpoints or certification |

## Boundary

`roomci` is a QA contract emulator. It is not a cloud platform emulator, production BMS, protocol certification suite, or physical device replacement.
