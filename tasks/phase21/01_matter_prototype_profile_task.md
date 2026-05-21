# Task 01: Matter Prototype Profile

## Goal

Add a scoped Matter prototype profile that models gateway-level device-state
contracts without claiming Matter controller, fabric, commissioning, or
certification behavior.

## Scope

- Add adapter contract example:
  - `adapter-contracts/examples/matter_gateway_profile.yaml`
- Add scenario example or dry-run fixture:
  - `examples/matter_gateway_profile.yaml`
- Add docs section:
  - supported concepts: endpoint id, cluster id/name, attribute id/name, command
    name, expected state
  - required customer inputs: gateway mapping, device identity, clusters,
    attributes, acceptance criteria
  - non-goals: fabric, commissioning, CASE/PASE, Thread/Wi-Fi transport,
    certification
- Validate through existing adapter contract validation.

## Acceptance Criteria

- Matter profile validates.
- Docs call it a `contract_profile`, not protocol support.
- Evidence checker tracks it as B Tier profile evidence.

## Test Commands

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/matter_gateway_profile.yaml
cargo run -p roomci-cli -- validate examples/matter_gateway_profile.yaml
make protocol-evidence
```

## Out Of Scope

- Matter SDK integration.
- Commissioning.
- Thread border router behavior.
- Certification claims.
