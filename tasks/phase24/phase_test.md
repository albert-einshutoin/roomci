# Phase 24 Test Plan

Phase 24 is complete when:

- existing scenario fixtures still validate and run;
- assertion migration has compatibility coverage for current YAML scenarios;
- runtime sub-state extraction does not change reports or timeline exports;
- parser/property tests cover malformed MQTT/Modbus/scenario inputs;
- concurrency tests cover `/run`, `/state`, MQTT publish, and report access
  overlap;
- the HTTP/MQTT/Modbus server implementation strategy is captured in an ADR.

