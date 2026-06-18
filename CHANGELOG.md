# Changelog

All notable changes to `roomci` are documented in this file.

The format follows Keep a Changelog, and this project currently uses pre-1.0 phase-based releases.

Entries from `0.0.0` through the early pre-1.0 phase checkpoints were created
during rapid product development. Treat them as design/release-log checkpoints,
not as separately published calendar releases.

## [Unreleased]

### Added

- Phase 11 integration-readiness tasks for adapter contracts, protocol matrix, customer PoC packs, and evaluator evidence.
- Phase 12 runtime-hardening tasks for `roomci serve`, release metadata, and public-surface truthfulness.
- `roomci-serve` crate for the long-running HTTP/MQTT PoC runtime.
- Adapter contract templates, examples, docs, and `roomci adapter validate` for company-specific integration specs.
- Hospitality evaluator guide and dual-track positioning docs for industry-wide adoption without private-compatibility overclaims.
- Customer PoC pack targets and evaluation evidence pack with scorecards, report artifacts, and unsupported-feature disclosure.
- Integration onboarding and category readiness docs for first-run evaluator experience and competitive positioning.
- BMS/contact external serve endpoint and controller script for non-MQTT external PoC interactions.
- `examples/device_command_value.yaml` demonstrating value-bearing device commands (`set_brightness`).
- Golden snapshot tests pinning the `RunReport` JSON contract for every `examples/*.yaml` (`roomci-core`) and the Markdown/JUnit/NDJSON/observability renders for representative scenarios (`roomci-report`), with `UPDATE_GOLDEN=1` regeneration documented in `CONTRIBUTING.md`. This is the Phase 0 safety net for behavior-preserving refactors.

### Changed

- Public release metadata and README badges are being aligned with verifiable local repository state.
- `roomci-cli` is now a thin argument-parsing and scenario-runner binary; serve runtime ownership moved to `roomci-serve`.
- `roomci serve` now handles HTTP clients independently with read/write timeouts and an in-flight connection cap.
- `POST /run` no longer holds the serve-state lock while executing a scenario, and poisoned serve state now returns HTTP 500 instead of panicking route handlers.
- `/health` now reports serve lifecycle status (`idle`, `running`, `passed`, `failed`) and returns HTTP 503 for failed health.
- MQTT `CONNECT` now validates protocol name `MQTT` and protocol level `4`, rejecting unsupported protocol versions with `CONNACK` `0x01`.

### Removed

- Unused `roomci-fault` crate (a phantom dependency of `roomci-core`, imported nowhere). The runtime semantics of `faults[].duration` remain unimplemented; whether to implement fault recovery is tracked as a separate decision.
- Public `ModbusModel::from_config`, `LightingModel::from_config`, and `ContactModel::from_config` constructors were removed from `roomci-device-model`.
  Consumers should migrate to the `try_from_config` constructors and explicit conversion helpers at the model layer.

### Fixed

- `command` steps now wire their `value` payload through to device state, so `set_brightness` / `set_temperature` / `set_mode` / `set_position` apply the requested value instead of silently no-op'ing. A value-requiring command with no value is now rejected (`command_rejected`), and `command_state_updated` is emitted only when device state actually changes (#30).

## [0.12.0] - 2026-05-19

### Added

- Serve-runtime hardening plan covering crate extraction, HTTP concurrency, read timeouts, mutex poison handling, `/health` semantics, MQTT CONNECT validation, release metadata, and changelog work.

## [0.11.0] - 2026-05-19

### Added

- Integration-ready emulator platform plan for company-specific IoT, SmartHome, building-automation, and hospitality-focused PoCs.
- Adoption maximization review with evaluator-readiness scores and concrete blockers.

## [0.10.0] - 2026-05-19

### Added

- `roomci serve` HTTP control/report API.
- Minimal MQTT 3.1.1 QoS0 publish ingress for external PoC clients.
- Configurable MQTT connection contracts for topic mapping and required payload fields.
- Docker Compose black-box controller PoC that writes JSON, Markdown, and JUnit reports.
- MQTT subset and pre-adoption PoC checklist docs.

## [0.9.0] - 2026-05-19

### Added

- Generic MQTT contract positioning and examples.
- Product positioning and domain-pack docs.
- Demo targets split between hospitality and generic MQTT paths.

## [0.8.0] - 2026-05-19

### Added

- Public release and interview polish tasks.
- Apache 2.0 license file and release-readiness gates.

### Changed

- README and hospitality-focused docs were reframed around public research and industry-wide patterns.

## [0.7.0] - 2026-05-19

### Added

- Production-readiness quality gates for docs, coverage, CLI ergonomics, and CI.
- Error-path tests and public API documentation.

### Changed

- README gained quick start, demo scenarios, report formats, CLI reference, and quality-gate sections.

## [0.6.0] - 2026-05-19

### Added

- WAN failover, comfort automation, access-permission drift, and commissioning checklist scenarios.
- Future milestone split for deeper integrations.

## [0.5.0] - 2026-05-19

### Added

- Docker image flow and Compose smoke-test workflow.
- GitHub Actions workflow for quality gates and report artifacts.

## [0.4.0] - 2026-05-19

### Added

- BMS/operations mock for critical alerts, notification channels, ticket state, and runbook links.

## [0.3.0] - 2026-05-19

### Added

- Modbus register-map model with scaling and writable/read-only validation.
- DALI-like scene and contact I/O models.

## [0.2.0] - 2026-05-19

### Added

- Edge routing and primary/secondary failover model.

## [0.1.0] - 2026-05-19

### Added

- Local MQTT retained-state model with duplicate delivery and reconnect behavior.

## [0.0.0] - 2026-05-19

### Added

- Initial latest scenario contract, local-first scenario runner, CLI validation/run commands, and JSON/Markdown/JUnit report flags.
