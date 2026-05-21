# Task 01: MQTT Retained Subscribe Loop

## Why

The current MQTT runtime accepts CONNECT, QoS0 PUBLISH, PINGREQ, and DISCONNECT. That is enough for command-ingress PoC testing, but many real MQTT evaluators expect a client to subscribe to state topics and receive retained state.

## Acceptance Criteria

- Implement the narrowest useful SUBSCRIBE/SUBACK path for configured state topics.
- Replay retained state to subscribers for topics matched by configured MQTT contracts.
- Reject or ignore unsupported subscription shapes honestly.
- Add a standard MQTT client test covering subscribe before publish, publish then retained replay, and disconnect.
- Extend the Docker `protocol-smoke` controller so a standard `paho-mqtt` client proves retained subscribe behavior in black-box mode.
- Update `docs/MQTT_SERVE_SUBSET.md` and the protocol support matrix.

## Coordination

Implement this together with Task 04 when practical. The MQTT behavior should land inside the extracted MQTT module rather than deepening the current monolithic serve file.

## Out of Scope

- QoS 1/2 delivery guarantees.
- Wildcard subscription generality beyond the documented subset.
- Full broker session persistence.
