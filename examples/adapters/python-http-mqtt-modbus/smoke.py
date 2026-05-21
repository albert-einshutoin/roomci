#!/usr/bin/env python3
import json
import os
import time

import paho.mqtt.client as mqtt
from pymodbus.client import ModbusTcpClient

from roomci_sdk import RoomciClient


ROOMCI_HTTP = os.environ.get("ROOMCI_HTTP", "http://127.0.0.1:8080")
ROOMCI_MQTT = os.environ.get("ROOMCI_MQTT", "127.0.0.1:1883")
ROOMCI_MODBUS = os.environ.get("ROOMCI_MODBUS", "127.0.0.1:5020")


def split_host_port(value):
    host, port = value.rsplit(":", 1)
    return host, int(port)


def wait_for_health(client):
    deadline = time.time() + 20
    while time.time() < deadline:
        try:
            health = client.health()
            if health.get("status") in {"idle", "running", "passed"}:
                return
        except Exception:
            time.sleep(0.25)
    raise RuntimeError("roomci serve did not become healthy")


def drive_bms(client):
    response = client.post_bms_contact(
        "contact.sauna_emergency_button",
        "on",
        severity="critical",
        replay_id="python-sdk-smoke-001",
    )
    if not response.get("accepted"):
        raise RuntimeError(f"BMS contact was not accepted: {response!r}")
    timeline = client.timeline()
    if not any(
        event.get("event_type") == "external_bms_contact_observed"
        for event in timeline
    ):
        raise RuntimeError(f"BMS contact was not visible in timeline: {timeline!r}")


def drive_mqtt():
    host, port = split_host_port(ROOMCI_MQTT)
    client = mqtt.Client(client_id="roomci-python-sdk-smoke", protocol=mqtt.MQTTv311)
    client.connect(host, port, keepalive=10)
    result = client.publish(
        "fleet/demo/site/lab/device/env_sensor_01/command",
        json.dumps({"online": True, "sample_interval_seconds": 45}),
        qos=0,
        retain=False,
    )
    result.wait_for_publish(timeout=5)
    client.disconnect()


def drive_modbus():
    host, port = split_host_port(ROOMCI_MODBUS)
    client = ModbusTcpClient(host, port=port)
    if not client.connect():
        raise RuntimeError("failed to connect to Modbus TCP endpoint")
    try:
        write = client.write_register(0, 260, slave=1)
        if write.isError():
            raise RuntimeError(f"Modbus write failed: {write!r}")
        read = client.read_holding_registers(0, 2, slave=1)
        if read.isError() or read.registers != [260, 210]:
            raise RuntimeError(f"unexpected Modbus registers: {read!r}")
    finally:
        client.close()


def assert_state(client):
    deadline = time.time() + 5
    while time.time() < deadline:
        state = client.state()
        retained = state.get("retained_messages", {})
        modbus = state.get("external_modbus_registers", {})
        mqtt_payload = retained.get("fleet/demo/site/lab/device/env_sensor_01/state", {})
        modbus_payload = modbus.get("modbus.floor_heating_01.40001", {})
        if (
            mqtt_payload.get("sample_interval_seconds") == 45
            and modbus_payload.get("readable_value") == 26.0
        ):
            return
        time.sleep(0.25)
    raise RuntimeError(f"roomci state did not include SDK smoke outputs: {state!r}")


def main():
    client = RoomciClient(ROOMCI_HTTP)
    wait_for_health(client)
    drive_bms(client)
    drive_mqtt()
    drive_modbus()
    assert_state(client)
    finish = client.finish()
    if not finish.get("finished"):
        raise RuntimeError(f"finish failed: {finish!r}")
    report = client.latest_report_json()
    if report.get("schema_version") != "roomci.report.v1":
        raise RuntimeError(f"unexpected report schema: {report!r}")
    print("python sdk smoke ok")


if __name__ == "__main__":
    main()
