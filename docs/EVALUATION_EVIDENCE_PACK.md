# Evaluation Evidence Pack

This pack gives evaluators command-level evidence instead of pitch language.

## Run Everything

```bash
make verify
make protocol-smoke
make adapter-samples-smoke
make s-tier-evidence-smoke
make developer-experience-smoke
make protocol-profile-smoke
make protocol-evidence
```

This runs formatting, clippy, tests, docs, coverage, scenario validation,
adapter contract validation, Docker image build, Docker scenario execution,
Compose scenario smoke, the external-controller Compose PoC, protocol smoke,
adapter sample smoke, and the claim evidence checker.

Current release-candidate evidence:

- `make verify` is the full local CI approximation.
- `make protocol-smoke` runs standard-client MQTT and Modbus checks.
- `make adapter-samples-smoke` runs Go and TypeScript adapter samples against
  `roomci serve` over HTTP, MQTT, and Modbus TCP.
- `make s-tier-evidence-smoke` generates and validates report JSON, timeline
  JSON, timeline NDJSON, observability JSON, run correlation, and trace
  metadata.
- `make developer-experience-smoke` runs the Python reference client and
  generates debugger JSON/Markdown artifacts.
- `make protocol-profile-smoke` validates Matter, BACnet, KNX, and OPC UA
  contract-profile examples and dry-run scenario fixtures.
- `make protocol-evidence` checks the claim registry, including Phase 17
  intercom/relay, network/control-panel, BMS hardening, comfort time-series,
  adapter sample evidence, S Tier artifact evidence, and A Tier developer
  experience evidence, and B Tier protocol-profile evidence.

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
| Hospitality smart-home local-first | 86/100 | MQTT topics/payloads, edge behavior, Modbus maps, BMS contracts, auth/network assumptions, acceptance criteria | Medium | Private compatibility, real auth/TLS, vendor endpoints, real safety authorization | Go if adapter fields map cleanly to your specs and reports are useful to smart-home QA |
| Generic IoT/MQTT edge-device fleet | 82/100 | Topic templates, required payload fields, device identity, retained/QoS expectations | Low-medium | Full broker semantics, QoS1/QoS2 wire ACKs, MQTT 5 | Go if contract-level MQTT testing plus adapter samples are enough for CI preflight |
| Building automation / Modbus / BMS | 78/100 | Register maps, scaling, access modes, contact mappings, BMS workflow | Medium | Vendor-specific devices, production BMS integration, BACnet/KNX/OPC UA runtime profiles | Go for behavioral CI checks; no-go for protocol conformance certification |
| Protocol-profile exploration | 70/100 | Matter endpoint/cluster maps, BACnet object maps, KNX group-address maps, OPC UA node maps, acceptance criteria | Medium | Wire endpoints, certification, subscriptions, transport/security behavior | Go if contract-profile dry-run evidence is useful before gateway integration; no-go if packet-level protocol behavior is required |
| CI/platform engineering | 86/100 | Container policy, report ingestion needs, CI runtime budget | Low | Hosted artifacts, release automation, packaged binaries | Go if `make verify`, Docker smoke, and JUnit reports fit the platform workflow |

## Protocol Evidence

- MQTT serve subset: [`MQTT_SERVE_SUBSET.md`](MQTT_SERVE_SUBSET.md)
- HTTP serve behavior: [`HTTP_SERVE_BEHAVIOR.md`](HTTP_SERVE_BEHAVIOR.md)
- External protocol depth: [`EXTERNAL_PROTOCOL_DEPTH.md`](EXTERNAL_PROTOCOL_DEPTH.md)
- Protocol support matrix: [`PROTOCOL_SUPPORT_MATRIX.md`](PROTOCOL_SUPPORT_MATRIX.md)
- Adapter contract kit: [`ADAPTER_CONTRACT_KIT.md`](ADAPTER_CONTRACT_KIT.md)
- B Tier protocol profiles: [`B_TIER_PROTOCOL_PROFILES.md`](B_TIER_PROTOCOL_PROFILES.md)
- Adapter SDK samples: [`ADAPTER_SDK_SAMPLES.md`](ADAPTER_SDK_SAMPLES.md)
- Python reference client: [`PYTHON_SDK.md`](PYTHON_SDK.md)
- Developer workflow: [`DEVELOPER_WORKFLOW.md`](DEVELOPER_WORKFLOW.md)
- S Tier evidence guide: [`S_TIER_EVIDENCE_GUIDE.md`](S_TIER_EVIDENCE_GUIDE.md)
- Claim evidence manifest: [`protocol-evidence.json`](protocol-evidence.json)

## Unsupported Features Disclosed

- Production MQTT broker behavior.
- Full MQTT 5, QoS1/QoS2 wire semantics, TLS, ACLs, persistence, clustering.
- Full Modbus TCP server beyond the documented practical subset.
- Full DALI, KNX, BACnet, Matter, OPC UA, SIP, or BMS implementation.
- Real unlock authorization, physical relay control, phone calls, Slack posts,
  ticket writes, cloud calls, or electrical safety validation.
- Drop-in compatibility with any private customer environment without their supplied contracts.
- Production SDK guarantees or PyPI packaging for the Python reference client.
- VSCode Marketplace extension publishing.

## Next Adoption Work

1. Convert a real customer spec into an adapter contract.
2. Add a black-box external client for the highest-priority missing protocol surface.
3. Add customer-specific PoC acceptance criteria.
4. Keep claims synchronized with [`PROTOCOL_SUPPORT_MATRIX.md`](PROTOCOL_SUPPORT_MATRIX.md).
5. Validate whether a B Tier contract profile is useful with a real evaluator
   before promoting it to a runtime endpoint.
