# Task 04: OPC UA Contract Profile

## Goal

Add an OPC UA contract profile for node/attribute/event-style QA without
implementing an OPC UA server or security policy stack.

## Scope

- Add adapter contract example:
  - `adapter-contracts/examples/opcua_contract_profile.yaml`
- Add scenario example or dry-run fixture:
  - `examples/opcua_contract_profile.yaml`
- Model:
  - endpoint label
  - namespace
  - node id
  - browse name
  - attribute
  - expected value
  - event type
- Document required customer inputs and non-goals.

## Acceptance Criteria

- OPC UA profile validates.
- Docs state this is not an OPC UA server.
- No subscription, security policy, certificate, address-space, or compliance
  claim is made.

## Test Commands

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/opcua_contract_profile.yaml
cargo run -p roomci-cli -- validate examples/opcua_contract_profile.yaml
make protocol-evidence
```

## Out Of Scope

- OPC UA server endpoint.
- Subscriptions.
- Security policies and certificates.
- Address-space compliance.
