"""Preload a tempting report; the runner must ignore SUBSCRIBE retained replay."""

import json
import os
import paho.mqtt.client as mqtt

run_id = os.environ["RUN_ID"]
client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2, client_id=f"inject-{run_id}", protocol=mqtt.MQTTv311)
client.connect("broker", 1883)
client.loop_start()
payload = json.dumps({"run_id": run_id, "device_id": "gateway-1", "revision": 2, "value": "new"})
info = client.publish(f"roomci/{run_id}/gateway-1/reported", payload, qos=1, retain=True)
info.wait_for_publish(timeout=5)
assert info.is_published()
client.disconnect()
client.loop_stop()
