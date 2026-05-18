# 10. NOT A HOTEL-oriented Positioning

## Positioning

`roomci` should be presented as:

> A Docker-first smart-room CI emulator that reproduces guest-impacting IoT failures before deployment.

Not as:

> A universal IoT protocol emulator.

## Why This Fits NOT A HOTEL-style Products

A smart-stay product is not only a website or mobile app. It is a connected physical experience.

The software touches:

- Reservation state.
- Guest identity and check-in.
- Smart locks.
- Lighting scenes.
- Air conditioning and comfort.
- Curtains/blinds.
- Sensors.
- Network gateways.
- Staff operations.
- Fallback flows.

A bug can become a physical experience failure.

## Best Demo

### Demo 1 — Lock offline at check-in

Message:

> Even if a smart lock is offline, the system should recover through fallback access and staff notification before the guest is blocked.

### Demo 2 — AC preheat failure

Message:

> Luxury hospitality requires comfort before arrival. CI should catch when a backend change breaks pre-arrival climate control.

### Demo 3 — Welcome scene partial failure

Message:

> Ambience is part of the product. Lighting and curtains should be validated as a scene, not isolated commands.

## Interview Pitch

> I understood NOT A HOTEL as a company where software controls real-world hospitality experiences. In that environment, a backend or app bug can become a physical failure: a door that does not unlock, a room that is too hot, or a welcome scene that fails. So I built a Docker-first Rust emulator that lets developers reproduce smart-room device failures in local development and CI. It does not try to replace Matter, AWS IoT, or real devices. Instead, it focuses on the testing gap between application code and physical room behavior.

## Why Rust

- Single binary distribution.
- Fast deterministic scenario execution.
- Strong typing for state machines.
- Good fit for CLI/CI tools.
- Does not force production stack replacement.

## What to Show in README

1. One-line Docker run.
2. Docker Compose with backend + roomci.
3. Check-in lock failure scenario.
4. Markdown report screenshot.
5. JUnit CI failure.
6. Adapter roadmap.
7. Explicit non-goals.

## Product Differentiation

Existing tools often focus on:

- Real smart-home control.
- Research simulation.
- Protocol compliance.
- Cloud IoT infrastructure.

`roomci` focuses on:

- Smart-room experience testing.
- Failure scenario replay.
- Docker-first CI usage.
- Hospitality-oriented device model.
