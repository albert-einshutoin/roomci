"""External bridge from a generic thermostat MQTT application to roomci's fixed wire contract."""

import json
import os
import time

import paho.mqtt.client as mqtt


RUN_ID = os.environ["RUN_ID"]
DEVICE_ID = os.environ["DEVICE_ID"]
ROOT = f"roomci/{RUN_ID}/{DEVICE_ID}"
APP = f"byos/{RUN_ID}/{DEVICE_ID}"


class Bridge:
    def __init__(self):
        self.desired = {}
        self.current_revision = None
        self.pending_desired = {}
        self.online = False
        self.client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2, client_id=f"roomci-byos-{RUN_ID}")
        self.client.on_connect = self.on_connect
        self.client.on_message = self.on_message
        self.client.on_publish = self.on_publish

    def publish(self, topic, payload, retain=False):
        result = self.client.publish(topic, json.dumps(payload, separators=(",", ":")), qos=1, retain=retain)
        if result.rc != mqtt.MQTT_ERR_SUCCESS:
            raise RuntimeError(f"publish {topic}: {result.rc}")
        return result.mid

    def on_publish(self, _client, _userdata, mid, reason, _properties):
        if reason.is_failure:
            raise RuntimeError(f"broker rejected publish mid={mid}: {reason}")
        revision = self.pending_desired.pop(mid, None)
        if revision is not None:
            print(f"app desired broker_ack revision={revision}", flush=True)

    def on_connect(self, client, _userdata, _flags, reason, _properties):
        if reason.is_failure:
            raise RuntimeError(f"broker connection rejected: {reason}")
        for topic in (f"{ROOT}/desired", f"{APP}/reported", f"{APP}/status"):
            client.subscribe(topic, qos=1)
        print("adapter subscribed", flush=True)

    def on_message(self, _client, _userdata, message):
        try:
            if message.topic == f"{ROOT}/desired":
                data = json.loads(message.payload)
                if data["run_id"] != RUN_ID or data["device_id"] != DEVICE_ID:
                    raise ValueError("desired run or device mismatch")
                revision, value = data["revision"], data["value"]
                if (revision, value) not in ((1, "21"), (2, "24")):
                    raise ValueError("desired revision or value mismatch")
                self.desired[value] = revision
                mid = self.publish(f"{APP}/desired", {"targetTemperature": int(value)}, retain=True)
                self.current_revision = revision
                self.pending_desired[mid] = revision
                print(f"desired queued revision={revision} value={value}", flush=True)
            elif message.topic == f"{APP}/status":
                if message.retain or message.payload not in (b"online", b"offline"):
                    print("ignored status", flush=True)
                    return
                self.online = message.payload == b"online"
                self.publish(f"{ROOT}/status", {"run_id": RUN_ID, "device_id": DEVICE_ID,
                                                "status": message.payload.decode()})
                print(f"sut status={message.payload.decode()}", flush=True)
            elif message.topic == f"{APP}/reported":
                if message.retain:
                    print("ignored retained report", flush=True)
                    return
                if not self.online:
                    print("ignored offline report", flush=True)
                    return
                data = json.loads(message.payload)
                if not isinstance(data, dict) or type(data.get("temperature")) is not int:
                    raise ValueError("report temperature must be an integer")
                value = str(data["temperature"])
                # The SUT supplies only temperature; revision is bridge-owned correlation.
                revision = self.desired.get(value, self.current_revision)
                if revision is None:
                    raise RuntimeError(f"cannot correlate report before desired: value={value}")
                self.publish(f"{ROOT}/reported", {"run_id": RUN_ID, "device_id": DEVICE_ID,
                                                  "revision": revision, "value": value})
                print(f"reported revision={revision} value={value}", flush=True)
        except (AssertionError, KeyError, ValueError, TypeError, UnicodeDecodeError) as error:
            print(f"ignored invalid {message.topic}: {error}", flush=True)

    def run(self):
        while True:
            try:
                self.client.connect(os.environ.get("BROKER_HOST", "broker"), 1883, keepalive=5)
                self.client.loop_forever(retry_first_connection=True)
            except OSError as error:
                print(f"broker unavailable: {error}", flush=True)
                time.sleep(1)


if __name__ == "__main__":
    Bridge().run()
