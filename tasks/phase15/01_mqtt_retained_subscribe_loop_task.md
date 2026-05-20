# Task 01: MQTT Retained Subscribe Loop

## Why

The current MQTT runtime accepts CONNECT, QoS0 PUBLISH, PINGREQ, and DISCONNECT. That is enough for command-ingress PoC testing, but many real MQTT evaluators expect a client to subscribe to state topics and receive retained state.

## Acceptance Criteria

- Implement the narrowest useful SUBSCRIBE/SUBACK path for configured state topics.
- Replay retained state to subscribers for topics matched by configured MQTT contracts.
- Reject or ignore unsupported subscription shapes honestly.
- Add a standard MQTT client test covering publish, subscribe, retained replay, and disconnect.
- Update `docs/MQTT_SERVE_SUBSET.md` and the protocol support matrix.

## Out of Scope

- QoS 1/2 delivery guarantees.
- Wildcard subscription generality beyond the documented subset.
- Full broker session persistence.
