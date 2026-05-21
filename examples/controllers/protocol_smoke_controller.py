#!/usr/bin/env python3
import json
import os
import time
import urllib.request

import paho.mqtt.client as mqtt
from pymodbus.client import ModbusTcpClient


ROOMCI_URL = os.environ.get("ROOMCI_URL", "http://roomci-serve:8080")
MQTT_HOST = os.environ.get("MQTT_HOST", "roomci-serve")
MQTT_PORT = int(os.environ.get("MQTT_PORT", "1883"))
MODBUS_HOST = os.environ.get("MODBUS_HOST", "roomci-serve")
MODBUS_PORT = int(os.environ.get("MODBUS_PORT", "5020"))


def http_json(path):
    with urllib.request.urlopen(f"{ROOMCI_URL}{path}", timeout=5) as response:
        return json.loads(response.read().decode("utf-8"))


def wait_for_health():
    deadline = time.time() + 20
    while time.time() < deadline:
        try:
            health = http_json("/health")
            if health.get("status") in {"idle", "passed", "running"}:
                return
        except Exception:
            time.sleep(0.25)
    raise RuntimeError("roomci did not become healthy")


def drive_mqtt():
    client = mqtt.Client(client_id="roomci-paho-smoke", protocol=mqtt.MQTTv311)
    client.connect(MQTT_HOST, MQTT_PORT, keepalive=10)
    result = client.publish(
        "fleet/demo/site/lab/device/env_sensor_01/command",
        json.dumps({"online": True, "sample_interval_seconds": 30}),
        qos=0,
        retain=False,
    )
    result.wait_for_publish(timeout=5)
    client.disconnect()


def drive_modbus():
    client = ModbusTcpClient(MODBUS_HOST, port=MODBUS_PORT)
    if not client.connect():
        raise RuntimeError("failed to connect to Modbus TCP endpoint")
    try:
        holding = client.read_holding_registers(0, 1, slave=1)
        if holding.isError() or holding.registers[0] != 245:
            raise RuntimeError(f"unexpected holding register response: {holding!r}")
        write = client.write_register(0, 250, slave=1)
        if write.isError():
            raise RuntimeError(f"unexpected write response: {write!r}")
        updated = client.read_holding_registers(0, 1, slave=1)
        if updated.isError() or updated.registers[0] != 250:
            raise RuntimeError(f"unexpected updated register response: {updated!r}")
    finally:
        client.close()


def assert_roomci_state():
    deadline = time.time() + 5
    while time.time() < deadline:
        state = http_json("/state")
        retained = state.get("retained_messages", {})
        modbus = state.get("external_modbus_registers", {})
        mqtt_payload = retained.get("fleet/demo/site/lab/device/env_sensor_01/state", {})
        if (
            mqtt_payload.get("sample_interval_seconds") == 30
            and modbus.get("modbus.floor_heating_01.40001", {}).get("readable_value") == 25.0
        ):
            return
        time.sleep(0.25)
    raise RuntimeError(f"roomci state did not include protocol smoke outputs: {state!r}")


def main():
    wait_for_health()
    drive_mqtt()
    drive_modbus()
    assert_roomci_state()
    print("protocol smoke ok")


if __name__ == "__main__":
    main()
