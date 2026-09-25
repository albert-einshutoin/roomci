"""Verify broker readiness on broker_net, then direct-route isolation on sut_net."""

import argparse
import socket
import time


def connect(deadline):
    addresses = socket.getaddrinfo("broker", 1883, type=socket.SOCK_STREAM)
    for index, (family, kind, proto, _, address) in enumerate(addresses):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            break
        with socket.socket(family, kind, proto) as connection:
            connection.settimeout(remaining / (len(addresses) - index))
            try:
                connection.connect(address)
            except OSError:
                continue
            return connection.detach()
    return None


def receive_connack(connection):
    packet = b""
    while len(packet) < 4:
        chunk = connection.recv(4 - len(packet))
        if not chunk:
            break
        packet += chunk
    return packet == b"\x20\x02\x00\x00"


def broker_ready(deadline):
    while time.monotonic() < deadline:
        try:
            descriptor = connect(deadline)
            if descriptor is not None:
                with socket.socket(fileno=descriptor) as connection:
                    connection.settimeout(max(0.001, deadline - time.monotonic()))
                    # MQTT 3.1.1 CONNECT with a short, clean-session probe identity.
                    connection.sendall(b"\x10\x15\x00\x04MQTT\x04\x02\x00\x05\x00\x09isolation")
                    if receive_connack(connection):
                        print("broker MQTT CONNACK confirmed from broker network")
                        return
        except OSError:
            pass
        time.sleep(min(0.1, max(0, deadline - time.monotonic())))
    raise SystemExit("broker MQTT not ready before isolation deadline")


def sut_isolated(deadline):
    try:
        descriptor = connect(deadline)
    except OSError:
        descriptor = None
    if descriptor is not None:
        socket.close(descriptor)
        raise SystemExit("SUT has a direct broker route")
    print("SUT direct broker route unavailable")


parser = argparse.ArgumentParser()
parser.add_argument("check", choices=("broker-ready", "sut-isolated"))
parser.add_argument("--timeout", type=float, default=10)
args = parser.parse_args()
if args.timeout <= 0:
    parser.error("--timeout must be positive")
deadline = time.monotonic() + args.timeout
if args.check == "broker-ready":
    broker_ready(deadline)
else:
    sut_isolated(deadline)
