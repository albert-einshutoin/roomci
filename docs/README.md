# roomci docs package

Smart-room / IoT device emulator for Docker, local PC, and CI.

This package contains development documents for building a Floci-like IoT emulator image focused on smart-room and hospitality scenarios.

## Documents

- `docs/00_source_map.md` — vendor/platform documentation map and adapter priority
- `docs/01_product_requirements.md` — PRD
- `docs/02_architecture.md` — system architecture
- `docs/03_device_model.md` — canonical device/capability model
- `docs/04_protocol_adapters.md` — HTTP, MQTT, AWS IoT Shadow-like, Azure IoT Hub-like, Home Assistant MQTT Discovery-like adapters
- `docs/05_docker_ci_design.md` — Docker image, Docker Compose, GitHub Actions design
- `docs/06_scenario_spec.md` — YAML scenario specification
- `docs/07_fault_injection.md` — failure model
- `docs/08_adapter_implementation_plan.md` — roadmap by adapter
- `docs/09_security_and_license_notes.md` — security, licensing, and legal constraints
- `docs/10_notahotel_positioning.md` — NOT A HOTEL-oriented positioning and demo plan
- `examples/docker-compose.yml` — sample local setup
- `examples/checkin_lock_offline.yaml` — sample scenario
- `examples/ac_preheat_failed.yaml` — sample scenario
- `schemas/scenario.schema.json` — draft JSON Schema for scenarios
