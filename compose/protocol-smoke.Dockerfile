FROM python:3.12-slim

ENV PIP_ROOT_USER_ACTION=ignore

RUN pip install --no-cache-dir paho-mqtt==1.6.1 pymodbus==3.6.9

WORKDIR /work
CMD ["python", "/controllers/protocol_smoke_controller.py"]
