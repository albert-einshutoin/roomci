# 01. Product Requirements Document — roomci

## Product Name

`roomci`

## Tagline

Test smart-room failures before guests experience them.

## Problem

Smart-room products combine backend services, mobile apps, device gateways, smart locks, climate control, lighting, curtains, sensors, and operations workflows. Physical devices are often unavailable in local development and CI. This makes failure scenarios hard to test before deployment.

Examples:
- Smart lock offline during check-in.
- AC preheat command delayed before guest arrival.
- Welcome lighting scene partially fails.
- Occupancy sensor sends stale values.
- Gateway reconnect causes out-of-order events.
- Backend fails to trigger fallback access or staff notification.

## Target Users

- Backend engineers building IoT-integrated services.
- Mobile app engineers testing smart-room controls.
- QA engineers validating check-in/stay/checkout flows.
- Platform engineers building CI pipelines.
- Smart building / hospitality engineers.

## Non-goals

- Not a full Matter implementation.
- Not a full AWS IoT Core clone.
- Not a full Azure IoT Hub clone.
- Not a Home Assistant replacement.
- Not firmware emulation.
- Not a production device gateway.
- Not a physical device bridge in v0.1.

## v0.1 Goals

1. Run as a Docker image.
2. Provide HTTP control API.
3. Provide MQTT-compatible topic interface or Mosquitto-compatible integration.
4. Load room/device definitions from YAML.
5. Load failure scenarios from YAML.
6. Simulate device state transitions.
7. Inject faults: offline, latency, timeout, stale sensor, command drop, duplicate event, out-of-order event.
8. Emit JSON, Markdown, and JUnit reports.
9. Work in Docker Compose and GitHub Actions.
10. Include hospitality/smart-room examples.

## v0.1 Device Types

- Smart lock
- Light
- Climate / AC / HVAC
- Curtain / blind / cover
- Temperature sensor
- Humidity sensor
- Motion / occupancy sensor
- Network gateway
- Room controller

## Success Metrics

- A developer can start `roomci` with one Docker command.
- A backend service can call a local smart-lock API without real hardware.
- A CI job can fail when fallback access is not triggered after a lock outage.
- A Markdown report explains guest impact, failed assertion, and timeline.
- New device profiles can be added through YAML without Rust changes.

## MVP Demo Scenarios

1. `checkin_lock_offline`
2. `ac_preheat_failed`
3. `welcome_scene_partial_failure`
4. `gateway_reconnect_out_of_order`
5. `checkout_cleaning_mode`

## Core User Stories

### Backend engineer

As a backend engineer, I want to run a local IoT emulator in Docker so that my service can test smart-room commands without physical devices.

### QA engineer

As a QA engineer, I want to define a failure scenario in YAML so that I can reproduce guest-impacting device failures in CI.

### Platform engineer

As a platform engineer, I want JUnit reports so that smart-room scenario failures appear in normal CI dashboards.

### Product/operations stakeholder

As an operations stakeholder, I want a human-readable timeline so that I can understand whether a guest would have been blocked or recovered by fallback operations.
