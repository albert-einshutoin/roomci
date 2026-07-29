# Changelog

All notable changes to `roomci` are documented in this file.

The format follows Keep a Changelog, and this project currently uses pre-1.0 phase-based releases.

Entries from `0.0.0` through the early pre-1.0 phase checkpoints were created
during rapid product development. Treat them as design/release-log checkpoints,
not as separately published calendar releases.

## [Unreleased]

### Added

- Evaluator intake kits in English and Japanese that map required MQTT,
  payload, identity, Modbus, BMS, auth/TLS, safety, report, and acceptance
  inputs to adapter contract fields and PoC commands while explicitly
  preventing private/vendor compatibility overclaims (#9).
- Backward-compatible acceptance-to-evidence mappings with stable criterion
  ids, scoped named-assertion validation, artifact capability checks, and
  report reference ids (#7). Adapter YAML remains wire-compatible; the new
  fields on pre-1.0 public Rust structs require downstream struct literals to
  be updated.
- Actionable MQTT topic identity diagnostics with contract-specific field paths,
  exact `{device_id}` placeholder checks, command/state placeholder consistency,
  and an explicit `placeholder:{device_id}` strategy boundary (#5).
- Typed MQTT adapter payload constraints for required/optional fields, JSON
  value types, enums, and numeric ranges, enforced consistently by scenario and
  serve-mode publish validation without introducing a full JSON Schema engine
  (#2).
- `roomci init [PATH] [--ci github] [--force]` for clone-free evaluator
  onboarding. It generates a runnable MQTT retained-state smoke scenario,
  VS Code YAML schema settings, and an optional versioned GitHub Actions
  workflow with atomic overwrite preflight and symlink protection (#47).
- Dependency security policy with weekly grouped Dependabot updates, a
  fail-closed RustSec audit gate, and `serde_yaml` migration parity
  requirements (#8).
- Repository-root Docker GitHub Action (`uses: albert-einshutoin/roomci@main`),
  copy-paste workflow example, action self-test, and GitHub Step Summary output
  for scenario batches. `--github-summary` explicitly selects a summary file;
  otherwise `GITHUB_STEP_SUMMARY` is detected and appended when available (#46).
- `roomci run --report-dir <dir>` for complete per-scenario JSON, Markdown,
  JUnit, timeline, and observability artifacts in one invocation, plus the
  stable `roomci.summary.v1` aggregate contract. Dry runs write only a summary
  with explicit `dry_run` entries (#45).
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

- README quality measurements now match the current full workspace gate: 260
  tests and 88.63% line coverage.
- Public release metadata and README badges are being aligned with verifiable local repository state.
- `roomci-cli` is now a thin argument-parsing and scenario-runner binary; serve runtime ownership moved to `roomci-serve`.
- `roomci serve` now handles HTTP clients independently with read/write timeouts and an in-flight connection cap.
- `POST /run` no longer holds the serve-state lock while executing a scenario, and poisoned serve state now returns HTTP 500 instead of panicking route handlers.
- `/health` now reports serve lifecycle status (`idle`, `running`, `passed`, `failed`) and returns HTTP 503 for failed health.
- MQTT `CONNECT` now validates protocol name `MQTT` and protocol level `4`, rejecting unsupported protocol versions with `CONNACK` `0x01`.
- `roomci-report` Markdown "Suggested Recovery" now lists each failed assertion's guest-impact message, replacing three hard-coded recovery hints whose triggering assertion types were never produced by the runtime (#28).

### Removed

- Removed public constructors from `roomci-device-model`:
  - `ModbusModel::from_config`
  - `LightingModel::from_config`
  - `ContactModel::from_config`
  - `roomci-device-model` API users should migrate to `try_from_config` and handle constructor-level parse failures explicitly.

  These APIs are intentionally replaced by `try_from_config` for clearer failure signaling.

- Unused `roomci-fault` crate (a phantom dependency of `roomci-core`, imported nowhere). The runtime semantics of `faults[].duration` remain unimplemented; whether to implement fault recovery is tracked as a separate decision.
- Dead code: `yaml_state_to_json` and the unused `RoomDefinition` / `DeviceDefinition` types in `roomci-device-model` (and its now-unused `serde` dependency), plus `ValidatedScheduledEvent::order()` and `MqttTopicTemplate::as_str()` in `roomci-scenario` (#28).

### Fixed

- Updated transitive `anyhow` from `1.0.102` to `1.0.104`, resolving
  `RUSTSEC-2026-0190` without an audit ignore (#8).
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
