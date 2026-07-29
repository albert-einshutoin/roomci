# Task 02: Topic Identity Diagnostics

## Status

`done`

## Problem

Topic template and `device_id_from_topic` errors are currently functional but
minimal. Evaluators need clearer feedback when mapping private MQTT topics.

## Scope

- Improve diagnostics for missing `{device_id}` placeholders.
- Detect mismatched placeholders between command and state topics.
- Keep `placeholder:{device_id}` as the only runtime strategy until a concrete
  need is proven.

## Acceptance Criteria

- Invalid topic templates fail validation with the contract name and field.
- Existing examples still validate.
- Runtime matching behavior is unchanged.

## Evidence

- `mqtt_contract_topic_errors_include_contract_name_and_field_path`
- `rejects_mismatched_mqtt_contract_topic_placeholders`
- `unsupported_mqtt_identity_strategy_reports_the_exact_field`
- `adapter_contract_topic_diagnostics_identify_the_exact_mapping`
