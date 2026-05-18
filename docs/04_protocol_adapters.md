# 04. Protocol Adapters

## Adapter Philosophy

Adapters translate external protocol shapes into the canonical roomci model.

```text
Vendor/API shape -> Adapter -> Canonical Command/Event/State
```

## HTTP Adapter

### Commands

```http
POST /rooms/{room_id}/devices/{device_id}/commands/{command}
Content-Type: application/json

{
  "value": 24,
  "request_id": "req_001"
}
```

### State

```http
GET /rooms/{room_id}/devices/{device_id}/state
```

### Faults

```http
POST /faults
Content-Type: application/json

{
  "target": "lock_entrance",
  "type": "offline",
  "from": "T-5m",
  "to": "T+3m"
}
```

### Timeline

```http
GET /timeline
```

## MQTT Adapter

### Topic Convention

```text
roomci/{room_id}/{device_id}/command
roomci/{room_id}/{device_id}/state
roomci/{room_id}/{device_id}/telemetry
roomci/{room_id}/{device_id}/availability
roomci/{room_id}/{device_id}/event
roomci/{room_id}/{device_id}/fault
```

### Command Payload

```json
{
  "action": "unlock",
  "request_id": "req_001"
}
```

### State Payload

```json
{
  "lock_state": "unlocked",
  "connectivity": "online",
  "updated_at": "2026-05-18T10:00:00+09:00"
}
```

## Home Assistant MQTT Discovery-like Adapter

### Discovery Topic

```text
homeassistant/lock/room_nasu_001/lock_entrance/config
```

### Discovery Payload

```json
{
  "name": "Entrance Smart Lock",
  "unique_id": "room_nasu_001_lock_entrance",
  "state_topic": "roomci/room_nasu_001/lock_entrance/state",
  "command_topic": "roomci/room_nasu_001/lock_entrance/command",
  "availability_topic": "roomci/room_nasu_001/lock_entrance/availability",
  "payload_lock": "LOCK",
  "payload_unlock": "UNLOCK"
}
```

## AWS IoT Shadow-like Adapter

### Design

Implement the shadow abstraction, not AWS itself.

Core concepts:
- desired state
- reported state
- delta
- accepted/rejected response

### REST-like Endpoints

```http
GET /aws/things/{thingName}/shadow
POST /aws/things/{thingName}/shadow
DELETE /aws/things/{thingName}/shadow
```

### Update Payload

```json
{
  "state": {
    "desired": {
      "lock_state": "unlocked"
    }
  }
}
```

### MQTT-like Topics

```text
$aws/things/{thingName}/shadow/get
$aws/things/{thingName}/shadow/get/accepted
$aws/things/{thingName}/shadow/get/rejected
$aws/things/{thingName}/shadow/update
$aws/things/{thingName}/shadow/update/accepted
$aws/things/{thingName}/shadow/update/rejected
$aws/things/{thingName}/shadow/update/delta
$aws/things/{thingName}/shadow/delete
$aws/things/{thingName}/shadow/delete/accepted
$aws/things/{thingName}/shadow/delete/rejected
```

### Mapping

```text
shadow.desired.lock_state=unlocked
  -> Canonical Command: unlock(lock_entrance)
  -> State transition
  -> shadow.reported.lock_state=unlocked
```

## Azure IoT Hub Device Twin-like Adapter

### Design

Implement twin-like desired/reported properties.

```json
{
  "desired": {
    "ac_living": {
      "setpoint_celsius": 24
    }
  },
  "reported": {
    "ac_living": {
      "temperature_celsius": 26.5,
      "mode": "cool"
    }
  }
}
```

### Mapping

```text
desired property patch -> command
reported property patch -> state/telemetry
cloud-to-device message -> command
```

## Matter-like Profile Adapter

Do not implement full Matter.

Use a Matter-like vocabulary:

| Matter-like cluster | roomci capability |
|---|---|
| OnOff | `on_off` |
| LevelControl | `brightness` |
| DoorLock | `lock` |
| Thermostat | `thermostat` |
| TemperatureMeasurement | `temperature_measurement` |
| OccupancySensing | `occupancy` |
| WindowCovering | `cover_position` |

## Adapter Development Order

1. HTTP
2. Generic MQTT
3. Home Assistant MQTT Discovery-like
4. AWS Shadow-like
5. Azure Twin-like
6. Hue-like lighting/scene
7. Matter-like profiles
