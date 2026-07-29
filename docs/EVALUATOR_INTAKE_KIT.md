# Evaluator Intake Kit

Use this checklist before adapting `roomci` to a real customer PoC. It turns
customer-owned protocol facts into a reviewable adapter contract, scenario,
and evidence plan without implying support for a private product or vendor.

Never request production credentials, live private keys, unrestricted device
access, or production-only endpoints. Use redacted samples and dedicated
non-production values.

## Intake Decision

| State | What roomci can do |
|---|---|
| No customer specification | Run the public generic MQTT, hospitality, building-automation, and BMS examples; demonstrate report formats and fail-closed validation |
| Partial specification | Validate and model only the supplied fields; record every missing fact as an assumption or blocked acceptance criterion |
| Complete non-production specification | Build a customer-owned adapter/scenario PoC and produce mapped evidence for the agreed acceptance criteria |

Without the customer-owned specification, roomci cannot claim topic, payload,
register, alert, authentication, TLS, timing, safety, or vendor compatibility.
Passing a public fixture proves the documented roomci subset, not compatibility
with a private deployment.

## Required Intake

Use one row per protocol surface. Mark unavailable inputs as `blocked`; do not
guess private values.

| Required input | Minimum customer-owned evidence | Adapter contract mapping | Scenario / PoC mapping |
|---|---|---|---|
| MQTT topics | Redacted command and state topic samples, wildcard/placeholder rules, direction, QoS, retained/session expectations | `mqtt.contracts[].name`, `command_topic`, `state_topic`, `adapter` | `mqtt.contracts[]`, `steps[].mqtt_publish`, `assertions[].mqtt`; run `make poc-generic-mqtt` |
| MQTT payloads | One accepted and one rejected JSON sample per command, required/optional fields, types, enum/range rules | `mqtt.contracts[].payload.required_fields`, `optional_fields`, `fields` | Publish the samples in scenario steps or against `roomci serve`; collect pass/fail reports |
| Device identity | Device-id source, normalization, case sensitivity, allowed examples, unknown-id behavior | `devices[].id`, `mqtt.contracts[].device_id_from_topic`, topic `{device_id}` placement | Use matching and unknown ids in scenarios; validate with `roomci adapter validate CONTRACT` |
| Modbus/register map | Unit id, address, register type, data representation, scale/unit, read/write access, valid range, expected commissioning value | `modbus.devices[].id`, `unit_id`, `registers[].address`, `name`, `type`, `access`, `scale`, `unit` | `modbus.devices[]`, `steps[].modbus_write`, Modbus assertions; run `make poc-building-automation` |
| BMS alert schema | Source id, schema/content type, severity vocabulary, signed headers, replay window, channels, ticket states, redacted accepted/rejected samples | `bms.alerts[].id`, `source`, `schema_version`, `content_type`, `severity`, `severity_enum`, `hmac`, `replay_window_seconds`, `channels`, `ticket_lifecycle` | `alerts`, `contacts`, BMS/contact steps and assertions; run `make poc-bms-ops` |
| Authentication and TLS assumptions | Test auth mode, secret reference names, trust source, client-auth requirement, hostname/SNI rule, TLS version, and whether plaintext localhost is acceptable | `auth` records the supplied non-production assumptions, such as `mode`, `secret_ref`, and TLS notes | Treat these as review notes unless the selected serve subset enforces them; current HTTP/MQTT serve is localhost-oriented and does not prove TLS interoperability |
| Safety boundary | Allowed read/write operations, prohibited actions, non-production target, rate/concurrency limits, emergency stop/rollback owner | `edge.commands[]` limits modeled commands; `devices[]` and Modbus `access` limit modeled targets | Encode only approved mocks and assertions. Never point roomci at production actuators or safety-critical unlock/control paths |
| Report expectations | Required formats, audience, redaction rules, retention/location, run id, and required failure diagnostics | `acceptance.report_formats`; optional `acceptance.mappings[].artifacts` | Choose CLI report flags or PoC targets; inspect JSON, Markdown, JUnit, timeline, observability, and GitHub summary artifacts |
| Acceptance criteria | Exact pass/fail wording, stable id, mapped scenario assertion, evidence artifact, timing/tolerance, owner | `acceptance.criteria`, optional `acceptance.mappings[].id`, `criterion`, `assertions`, `artifacts` | Give referenced assertions stable `name` values; cross-check with `roomci adapter validate CONTRACT --scenario SCENARIO` |

## Protocol Detail Checklist

### MQTT

- [ ] Command and state topic examples are redacted but structurally exact.
- [ ] `{device_id}` placement and extraction behavior are explicit.
- [ ] Required, optional, unknown, wrong-type, out-of-range, and malformed
  payload behavior is agreed.
- [ ] QoS, retained state, reconnect, duplicate delivery, and session
  expectations are stated separately from roomci's supported subset.

### Modbus

- [ ] Transport and unit id are explicit.
- [ ] Every used address has a type, access mode, scale, unit, and byte/word
  interpretation.
- [ ] Read-only and writable boundaries are approved.
- [ ] Unsupported function codes and out-of-range values have expected failure
  behavior.

### BMS / operations

- [ ] Alert source and contact mapping are explicit.
- [ ] Schema version, content type, severity enum, required fields, and
  accepted/rejected samples are supplied.
- [ ] Signature header/algorithm and replay window use non-production secret
  references only.
- [ ] Notification channels, ticket lifecycle, acknowledgment, and escalation
  expectations are defined.

### Auth, TLS, and safety

- [ ] Authentication assumptions contain no real credential material.
- [ ] TLS/trust/client-auth requirements are recorded even when the current
  roomci serve subset cannot enforce them.
- [ ] Allowed and prohibited operations are approved by the customer owner.
- [ ] The PoC is isolated from production and safety-critical actuators.

## Build the PoC Contract

1. Copy the public template:

   ```bash
   cp adapter-contracts/templates/company_adapter_contract.yaml path/to/customer-poc.yaml
   ```

2. Replace only values supported by supplied, non-production evidence.
3. Validate the adapter contract:

   ```bash
   cargo run -p roomci-cli -- adapter validate path/to/customer-poc.yaml
   ```

4. Add stable acceptance-to-evidence mappings and named scenario assertions,
   then cross-check them:

   ```bash
   cargo run -p roomci-cli -- adapter validate \
     path/to/customer-poc.yaml \
     --scenario path/to/customer-scenario.yaml
   ```

5. Run the relevant public baseline before the customer-owned scenario:

   ```bash
   make poc-generic-mqtt
   make poc-building-automation
   make poc-bms-ops
   ```

6. Run the customer-owned scenario and collect only the agreed artifacts:

   ```bash
   cargo run -p roomci-cli -- run path/to/customer-scenario.yaml \
     --report-json reports/customer-poc.json \
     --report-md reports/customer-poc.md \
     --junit reports/customer-poc.xml
   ```

## Exit Criteria

The intake is ready only when:

- every exercised private fact has a customer-owned source or is marked
  blocked;
- no production secret or endpoint is committed;
- each acceptance criterion has an owner and deterministic pass/fail wording;
- every claimed result maps to a named assertion and/or declared artifact;
- unsupported TLS, protocol, timing, or safety behavior is disclosed rather
  than inferred; and
- the final report states that evidence applies only to the supplied
  non-production specification.

See [Adapter Contract Kit](ADAPTER_CONTRACT_KIT.md) for field syntax,
[Integration Onboarding](INTEGRATION_ONBOARDING.md) for the evaluator path,
and [Protocol Support Matrix](PROTOCOL_SUPPORT_MATRIX.md) for implemented
subsets and non-goals.
