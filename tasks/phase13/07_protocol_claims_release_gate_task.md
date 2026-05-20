# Task 07 — Protocol Claims Release Gate

## Goal

Prevent README/docs from drifting into unsupported protocol-conformance claims as the product becomes more ambitious.

## Implementation Scope

- Add a release checklist section for protocol claims.
- Add a lightweight verification script or documented manual gate that scans README/docs for risky phrases:
  - "full MQTT broker"
  - "Modbus compatible" without subset qualifier
  - "BACnet/OPC UA/Matter support" without implementation evidence
  - "conformant" without registry evidence
- Make the support matrix and conformance registry the required source of truth.
- Update docs to use terms consistently:
  - `scenario_model`
  - `serve_endpoint`
  - `external_client_tested`
  - `conformance_subset`
  - `unsupported`

## Acceptance Criteria

- Release checklist includes protocol-claim review.
- Any protocol claim in README can be traced to a registry row and test evidence.
- Unsupported protocols are described as future profiles or adapter candidates, not current features.
