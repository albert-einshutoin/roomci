"""Separate reference gateway. Only MQTT topics and payloads are shared with roomci."""

import json
import os
import sys
import threading

import paho.mqtt.client as mqtt


run_id = os.environ["RUN_ID"]
device_id = os.environ.get("DEVICE_ID", "gateway-1")
mode = os.environ.get("SUT_MODE", "fixed")
prefix = f"roomci/{run_id}/{device_id}"
desired = f"{prefix}/desired"
reported = f"{prefix}/reported"
status = f"{prefix}/status"
connections = 0


def envelope(**values):
    return json.dumps({"run_id": run_id, "device_id": device_id, **values})


client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2, client_id=f"sut-{run_id}", clean_session=True, protocol=mqtt.MQTTv311)
client.will_set(status, envelope(status="offline"), qos=1, retain=False)
client.reconnect_delay_set(min_delay=1, max_delay=1)


def on_connect(client, userdata, flags, reason_code, properties):
    global connections
    if reason_code.is_failure:
        print(f"connect failed: {reason_code}", flush=True)
        return
    connections += 1
    client.publish(status, envelope(status="online"), qos=1)
    # With clean_session=True Mosquitto discards subscriptions on each disconnect.
    # The broken variant deliberately skips re-subscription on later connections.
    if mode != "broken" or connections == 1:
        client.subscribe(desired, qos=1)
    print(f"connected {connections} mode={mode}", flush=True)


def on_disconnect(client, userdata, disconnect_flags, reason_code, properties):
    if mode == "stop-after-initial" and connections:
        os._exit(0)


def on_message(client, userdata, message):
    try:
        command = json.loads(message.payload)
        if message.topic != desired or command["run_id"] != run_id or command["device_id"] != device_id:
            return
        revision = command["revision"]
        value = command["value"]
    except (ValueError, KeyError, TypeError):
        return
    if mode == "old-only" and revision == 2:
        revision, value = 1, "old"
    if mode == "foreign-only" and revision == 2:
        client.publish(reported, json.dumps({"run_id": "previous-run", "device_id": device_id, "revision": 2, "value": value}), qos=1)
        client.publish(reported, json.dumps({"run_id": run_id, "device_id": "other-device", "revision": 2, "value": value}), qos=1)
        return
    client.publish(reported, envelope(revision=revision, value=value), qos=1, retain=False)
    print(f"reported revision={revision} value={value}", flush=True)
    if mode == "rollback" and revision == 2:
        threading.Timer(0.2, lambda: client.publish(reported, envelope(revision=1, value="old"), qos=1)).start()


client.on_connect = on_connect
client.on_disconnect = on_disconnect
client.on_message = on_message
try:
    client.connect_async(os.environ.get("PROXY_HOST", "toxiproxy"), int(os.environ.get("PROXY_PORT", "1884")), keepalive=2)
    client.loop_forever(retry_first_connection=True)
except KeyboardInterrupt:
    sys.exit(0)
