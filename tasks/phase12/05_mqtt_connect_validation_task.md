# Task 05 — Validate MQTT CONNECT Protocol Name and Level

## Goal

Reject MQTT CONNECT packets whose protocol name is not `MQTT` or whose protocol level is not `4` (MQTT 3.1.1). The Phase 10 docs explicitly claim a "MQTT 3.1.1 QoS0 subset" surface; the current implementation accepts any CONNECT and responds CONNACK `0x00` regardless of contents, which silently misrepresents what the server supports.

## Why This Matters

External evaluators who try `roomci serve --mqtt-port N` with a standard MQTT client expect:

- A `MQTT` protocol name → accepted (`CONNACK` return code `0x00`).
- An MQTT 3.1 client sending `MQIsdp` → rejected with `CONNACK` return code `0x01` (unacceptable protocol version).
- A MQTT 5.0 client sending protocol level `5` → rejected with `CONNACK` return code `0x01`.

Today every client gets `CONNACK 0x00`, then sends a PUBLISH the server tries to parse as 3.1.1 QoS0, and the user has no idea why behavior diverges from their expectation. The published docs claim a specific subset; the code must enforce it.

## Implementation Scope

- In the MQTT wire decoder (relocated to `roomci-serve` by Task 01), extend the CONNECT parser to read:
  - Protocol name length (2 bytes, big-endian).
  - Protocol name bytes.
  - Protocol level (1 byte).
- Reject the connection if:
  - Protocol name is not exactly `MQTT`.
  - Protocol level is not exactly `4`.
- On rejection, write a `CONNACK` packet with:
  - Fixed header: `0x20 0x02`.
  - Variable header byte 1: `0x00` (no session present).
  - Variable header byte 2: `0x01` (unacceptable protocol version).
- After writing CONNACK, close the TCP connection cleanly. Do not attempt to read any further packets.
- Add named constants (e.g. `MQTT_PROTOCOL_NAME = "MQTT"`, `MQTT_PROTOCOL_LEVEL_3_1_1 = 4`, `MQTT_CONNACK_UNACCEPTABLE_PROTOCOL = 0x01`) instead of inline magic bytes.
- Add regression tests in `crates/roomci-serve/tests/` (or a `roomci-serve` unit test module):
  - `mqtt_connect_with_mqtt_3_1_1_is_accepted`: existing behavior keeps working.
  - `mqtt_connect_with_legacy_protocol_name_is_rejected`: send `MQIsdp` and assert `CONNACK 0x01`.
  - `mqtt_connect_with_unsupported_level_is_rejected`: send protocol level `5` and assert `CONNACK 0x01`.
  - `mqtt_connect_with_truncated_header_closes_connection`: send a partial CONNECT and assert the server closes within the configured timeout without panicking.
- Update `docs/MQTT_SERVE_SUBSET.md` to document the rejection codes and reference the named constants.

## Acceptance Criteria

- Four new MQTT regression tests pass.
- Existing `external_mqtt_publish_updates_retained_state_through_serve` integration test still passes.
- `docs/MQTT_SERVE_SUBSET.md` lists protocol-name and protocol-level enforcement explicitly.
- `grep -n '0x01' crates/roomci-serve/src/` shows the named constant, not bare hex.

## Out of Scope

- Supporting MQTT 3.1 (`MQIsdp`) or MQTT 5.0 protocol levels.
- Honoring or validating the `Will` flags, `User Name` / `Password` flags, or `Keep Alive`. The current Phase 10 contract does not promise these; this task does not add them.
- TLS or auth on the MQTT port.

## Evidence

- `cargo test -p roomci-serve mqtt_connect_with_legacy_protocol_name_is_rejected` passes.
- `cargo test -p roomci-serve mqtt_connect_with_unsupported_level_is_rejected` passes.
- `cargo test -p roomci-serve mqtt_connect_with_truncated_header_closes_connection` passes.
