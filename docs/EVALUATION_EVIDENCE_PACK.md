# Evaluation Evidence Pack

This pack gives evaluators command-level evidence instead of pitch language.

## Run Everything

```bash
make verify
make protocol-smoke
```

This runs formatting, clippy, tests, docs, coverage, scenario validation, adapter contract validation, Docker image build, Docker scenario execution, Compose scenario smoke, and the external-controller Compose PoC.

Current measured baseline:

- 100 workspace tests
- 84.51% line coverage
- Docker and Compose PoC pass through `make verify`
- MQTT and Modbus protocol smoke pass through `make protocol-smoke`

## PoC Packs

| Pack | Command | Report Artifacts |
|---|---|---|
| Generic MQTT edge-device fleet | `make poc-generic-mqtt` | `reports/poc_generic_mqtt_*.{json,md,xml}` |
| Hospitality smart home local-first room | `make poc-hospitality` | `reports/poc_hospitality_*.{json,md,xml}` |
| Building automation / BMS | `make poc-building-automation` | `reports/poc_building_*.{json,md,xml}` |
| BMS / operations escalation | `make poc-bms-ops` | `reports/poc_bms_ops_*.{json,md,xml}` |

Generated `reports/` files are intentionally ignored by git.

## Scorecards

| Evaluation Lens | Current Fit | Required Customer Inputs | Integration Effort | Missing Depth | Go / No-go Criteria |
|---|---:|---|---|---|---|
| Hospitality smart-home local-first | 78/100 | MQTT topics/payloads, edge behavior, Modbus maps, BMS contracts, auth/network assumptions, acceptance criteria | Medium | Private compatibility, real auth/TLS, richer external endpoints | Go if adapter fields map cleanly to your specs and reports are useful to smart-home QA |
| Generic IoT/MQTT edge-device fleet | 76/100 | Topic templates, required payload fields, device identity, retained/QoS expectations | Low-medium | Full broker semantics, QoS1/QoS2 wire ACKs, MQTT 5 | Go if contract-level QoS0 testing is enough for CI preflight |
| Building automation / Modbus / BMS | 68/100 | Register maps, scaling, access modes, contact mappings, BMS workflow | Medium | Modbus TCP serve endpoint, vendor-specific devices, production BMS integration | Go for behavioral CI checks; no-go for protocol conformance certification |
| CI/platform engineering | 82/100 | Container policy, report ingestion needs, CI runtime budget | Low | Hosted artifacts, release automation, packaged binaries | Go if `make verify` and JUnit reports fit the platform workflow |

## Protocol Evidence

- MQTT serve subset: [`MQTT_SERVE_SUBSET.md`](MQTT_SERVE_SUBSET.md)
- HTTP serve behavior: [`HTTP_SERVE_BEHAVIOR.md`](HTTP_SERVE_BEHAVIOR.md)
- External protocol depth: [`EXTERNAL_PROTOCOL_DEPTH.md`](EXTERNAL_PROTOCOL_DEPTH.md)
- Protocol support matrix: [`PROTOCOL_SUPPORT_MATRIX.md`](PROTOCOL_SUPPORT_MATRIX.md)
- Adapter contract kit: [`ADAPTER_CONTRACT_KIT.md`](ADAPTER_CONTRACT_KIT.md)

## Unsupported Features Disclosed

- Production MQTT broker behavior.
- Full MQTT 5, QoS1/QoS2 wire semantics, TLS, ACLs, persistence, clustering.
- Full Modbus TCP server.
- Full DALI, KNX, BACnet, Matter, SIP, or BMS implementation.
- Drop-in compatibility with any private customer environment without their supplied contracts.

## Next Adoption Work

1. Convert a real customer spec into an adapter contract.
2. Add a black-box external client for the highest-priority missing protocol surface.
3. Add customer-specific PoC acceptance criteria.
4. Keep claims synchronized with [`PROTOCOL_SUPPORT_MATRIX.md`](PROTOCOL_SUPPORT_MATRIX.md).
