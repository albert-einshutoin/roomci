# 04. Local-first MQTT Architecture

## Why MQTT is central

`roomci` should model MQTT as the main communication substrate, not as a secondary adapter.

A local-first smart-home architecture uses MQTT because:

- local devices and controllers can operate without cloud availability
- retained messages simplify state synchronization
- QoS 1 supports at-least-once delivery and shifts retry behavior to the protocol layer
- reconnecting clients can recover the latest known state
- local and cloud bridge patterns are easier to test

## Target topology

```txt
+-------------------+        MQTT         +---------------------+
| iPad Controller   | <-----------------> | Local MQTT Broker   |
+-------------------+                     +----------+----------+
                                                     |
                                                     | MQTT subscribe/publish
                                                     v
                                          +---------------------+
                                          | Edge Server         |
                                          | Home Controller     |
                                          +----------+----------+
                                                     |
                                                     | Device protocols
                                                     v
                                     +-------------------------------+
                                     | DALI / KNX / Modbus / Contact |
                                     +-------------------------------+

Local MQTT Broker
   |
   | bridge / outbound client
   v
Cloud MQTT Broker Mock / AWS IoT Core-like endpoint
```

## MQTT topic conventions

Recommended default topic shape:

```txt
house/{house_id}/room/{room_id}/device/{device_id}/command
house/{house_id}/room/{room_id}/device/{device_id}/state
house/{house_id}/room/{room_id}/scene/{scene_id}/command
house/{house_id}/room/{room_id}/scene/{scene_id}/state
house/{house_id}/bms/alert/{alert_id}
house/{house_id}/ops/ticket/{ticket_id}
```

## Retained state model

Every device should expose a retained state topic.

Example:

```json
{
  "device_id": "living_light_01",
  "power": true,
  "brightness": 60,
  "updated_at": "2026-01-01T00:00:00Z"
}
```

Scenario assertion:

```yaml
assert:
  mqtt:
    topic: house/minakami/room/living/device/living_light_01/state
    retained:
      power: true
      brightness: 60
```

## QoS 1 duplicate simulation

MQTT QoS 1 means a message may be delivered more than once. Device logic and edge routing should be duplicate-safe.

Scenario example:

```yaml
faults:
  - at: T+1s
    target: mqtt.local
    type: duplicate_delivery
    topic: house/minakami/room/sauna/device/sauna_01/command
    count: 2
```

Expected behavior:

- device command is semantically idempotent
- no double-counted side effects
- final retained state is correct

## Reconnect recovery

Scenario:

```yaml
steps:
  - at: T
    client: ipad_controller
    disconnect: true
  - at: T+10s
    mqtt_publish:
      topic: house/minakami/room/living/device/light/state
      retain: true
      payload:
        power: true
  - at: T+20s
    client: ipad_controller
    reconnect: true
  - at: T+21s
    assert:
      client: ipad_controller
      received_retained_state:
        topic: house/minakami/room/living/device/light/state
        payload:
          power: true
```

## Cloud outage scenario

Core guest experience should continue if cloud MQTT is offline.

```yaml
scenario:
  name: local_first_cloud_outage

faults:
  - at: T+10s
    target: mqtt.cloud
    type: offline

steps:
  - at: T+15s
    mqtt_publish:
      client: ipad_controller
      topic: house/minakami/room/living/device/light/command
      payload:
        power: true
  - at: T+16s
    assert:
      mqtt:
        topic: house/minakami/room/living/device/light/state
        retained:
          power: true
  - at: T+20s
    assert:
      guest_experience: unaffected
```

## Cloud bridge model

The cloud bridge should model:

- local to cloud state forwarding
- cloud to local command forwarding
- durable queue for feedback
- loss/retry behavior
- delayed synchronization after cloud recovery

This can be AWS IoT Core-like without being fully AWS IoT compatible.
