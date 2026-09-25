"""Fail if the SUT container can bypass its dedicated proxy network."""

import socket

try:
    with socket.create_connection(("broker", 1883), timeout=1):
        raise SystemExit("SUT has a direct broker route")
except (OSError, TimeoutError):
    print("SUT direct broker route unavailable")
