# Task 02 — Shadow API and Topics

## Objective

Expose shadow-like behavior through local routes and optional MQTT-like topics.

## Implementation Scope

- Implement REST-like routes:
  - `GET /aws/things/{thingName}/shadow`
  - `POST /aws/things/{thingName}/shadow`
  - `DELETE /aws/things/{thingName}/shadow`
- Implement accepted/rejected response shapes.
- Implement MQTT-like topic names if Phase 2 is present.

## Acceptance Criteria

- Update accepted response includes the resulting shadow document or summary.
- Update rejected response includes stable error code and message.
- MQTT-like accepted/rejected/delta topics match docs.
- Security docs do not imply AWS compatibility certification.

## References

- `docs/09_security_and_license_notes.md`
