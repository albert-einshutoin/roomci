# 00. Source Map — IoT / Smart Home Documentation to Study

## Goal

Build `roomci`: a Docker-first smart-room / IoT device emulator that can run locally and in CI.

The goal is not to fully clone every vendor protocol. The goal is to extract the common control model from major IoT ecosystems and expose a compatible enough emulator surface for application/backend testing.

## Priority Rule

1. **Canonical model first**: device, capability, state, command, event, fault.
2. **Protocol adapters second**: HTTP, MQTT, cloud-shadow-like APIs, Home Assistant discovery-like metadata.
3. **Vendor compatibility third**: add vendor-shaped endpoints/topics only where they improve developer adoption.

## Tier 1 — Must Study / Implement First

### MQTT 3.1.1 / 5.0

Why:
- Common lightweight pub/sub protocol in IoT.
- Good fit for local Docker testing.
- Easy to inspect in CI.

Use in roomci:
- Device telemetry publish.
- Command subscribe.
- Availability topic.
- State topic.
- Fault events.

Required docs:
- MQTT 3.1.1 specification
- MQTT 5.0 specification
- MQTT topic naming best practices
- MQTT retained messages / QoS / Last Will and Testament

Implementation target:
- `roomci` can run an embedded MQTT-compatible broker or connect to Mosquitto.
- v0.1 can also expose MQTT-like topic handling without implementing a full broker.

### HTTP / REST Control API

Why:
- Simplest path for backend integration tests.
- Works with Docker Compose and GitHub Actions without special SDKs.

Use in roomci:
- Command endpoint.
- State endpoint.
- Scenario control endpoint.
- Fault injection endpoint.
- Timeline/report endpoint.

Implementation target:
- `POST /rooms/{room_id}/devices/{device_id}/commands/{command}`
- `GET /rooms/{room_id}/devices/{device_id}/state`
- `POST /scenarios/{id}/run`
- `POST /faults`

### Home Assistant MQTT Discovery-like model

Why:
- Home Assistant is a popular open-source smart home controller.
- Its MQTT discovery approach is practical for describing devices through topics and JSON payloads.
- It gives roomci a recognizable ecosystem integration without needing proprietary vendor APIs.

Use in roomci:
- Emit discovery payloads for lock, light, climate, cover, sensor, binary_sensor.
- Expose state/command topics compatible with a Home Assistant-style mental model.

Implementation target:
- `homeassistant/{component}/{node_id}/{object_id}/config`
- Component mappings: `lock`, `light`, `climate`, `cover`, `sensor`, `binary_sensor`.

## Tier 2 — High Value Cloud-IoT Compatibility

### AWS IoT Core Device Shadow-like API

Why:
- Device Shadow is a common cloud-IoT abstraction: desired state, reported state, delta.
- Useful for backend tests even without real AWS.
- Fits the Floci/LocalStack style: emulate cloud-facing IoT state services locally.

Use in roomci:
- Shadow documents per device.
- Desired/reported state transitions.
- Delta generation.
- Accepted/rejected events.

Implementation target:
- Do not implement full AWS auth or IoT policy engine in v0.1.
- Implement an AWS IoT Shadow-inspired local API:
  - `GET /things/{thingName}/shadow`
  - `POST /things/{thingName}/shadow`
  - MQTT-style topics:
    - `$aws/things/{thingName}/shadow/update`
    - `$aws/things/{thingName}/shadow/update/accepted`
    - `$aws/things/{thingName}/shadow/update/rejected`
    - `$aws/things/{thingName}/shadow/update/delta`

### Azure IoT Hub Device Twin-like API

Why:
- Device Twin is Azure's equivalent desired/reported property abstraction.
- Useful for teams with Azure IoT architecture.

Use in roomci:
- Desired properties.
- Reported properties.
- Cloud-to-device messages.
- Device-to-cloud telemetry.

Implementation target:
- Azure-like semantics, not full Azure auth.
- Expose local routes/topics that preserve the conceptual model.

## Tier 3 — Smart Home Ecosystem Adapters

### Matter / CSA model

Why:
- Matter is the modern cross-vendor smart home standard.
- It defines common device classes such as lights, locks, thermostats, sensors, and covers.

Use in roomci:
- Use Matter as a vocabulary reference for device categories and capabilities.
- Do not implement full Matter commissioning, certificates, or CHIP stack in v0.1.

Implementation target:
- `matter-like` device profile mapping:
  - OnOff
  - LevelControl
  - DoorLock
  - Thermostat
  - TemperatureMeasurement
  - OccupancySensing
  - WindowCovering

### SmartThings capabilities/profile model

Why:
- SmartThings represents devices through capabilities and profiles.
- Useful as a reference for a capability-based abstraction.

Use in roomci:
- Capability naming style and device profile composition.

Implementation target:
- Optional adapter or importer later.

### Google Home / Cloud-to-cloud traits

Why:
- Google Home has a trait-based model for smart home devices.
- Useful for mapping device intent to actions.

Use in roomci:
- Reference only for trait naming and state/action modeling.

Implementation target:
- Not v0.1.

### Philips Hue API v2

Why:
- Good reference for lighting, rooms, zones, scenes, grouped lights.
- Strongly relevant to hotel/room ambiance.

Use in roomci:
- Scene consistency tests.
- Light/group/room model.

Implementation target:
- Optional `hue-like` adapter later.

## Tier 4 — Building Automation / Hotel-grade Systems

### BACnet

Use case:
- HVAC, building automation, facilities systems.

Do not implement in v0.1.

### Modbus

Use case:
- Industrial/energy/HVAC equipment.

Do not implement in v0.1.

### KNX

Use case:
- Lighting, HVAC, building automation in homes/hotels.

Do not implement in v0.1.

### DALI

Use case:
- Lighting control.

Do not implement in v0.1.

## Recommended First Compatibility Matrix

| Adapter | v0.1 | v0.2 | v0.3+ | Notes |
|---|---:|---:|---:|---|
| HTTP REST | Yes | Yes | Yes | Main integration path |
| MQTT generic | Yes | Yes | Yes | Most important IoT path |
| Home Assistant MQTT Discovery-like | Partial | Yes | Yes | Good local validation |
| AWS IoT Shadow-like | Partial | Yes | Yes | Strong Floci-like angle |
| Azure Device Twin-like | No | Partial | Yes | Good cloud-IoT coverage |
| Matter-like model | Profile only | Partial | Maybe | Avoid full protocol early |
| SmartThings-like capabilities | No | Partial | Maybe | Useful model reference |
| Hue-like lighting/scene | No | Partial | Yes | Good NOT A HOTEL demo |
| BACnet / Modbus / KNX / DALI | No | No | Maybe | Too large for early MVP |
