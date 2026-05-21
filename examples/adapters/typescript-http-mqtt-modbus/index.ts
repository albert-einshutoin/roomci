import net from "node:net";

const httpBase = process.env.ROOMCI_HTTP ?? "http://127.0.0.1:8080";
const mqttAddr = process.env.ROOMCI_MQTT ?? "127.0.0.1:1883";
const modbusAddr = process.env.ROOMCI_MODBUS ?? "127.0.0.1:1502";

main().catch((error) => {
  console.error(error);
  process.exit(1);
});

async function main() {
  await postBms();
  await publishMqtt();
  await readModbus();
}

async function postBms() {
  const response = await fetch(`${httpBase}/external/bms/contact`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      source: "contact.sauna_emergency_button",
      state: "on",
      severity: "critical",
      schema_version: "bms.alert.v1",
      replay_id: `ts-sample-${Date.now()}`,
    }),
  });
  console.log("BMS POST:", response.status, await response.text());
  if (!response.ok) throw new Error(`BMS POST failed: ${response.status}`);
}

async function publishMqtt() {
  const [host, port] = splitAddress(mqttAddr);
  const socket = await connect(host, port);
  socket.write(mqttConnectPacket("roomci-ts-sample"));
  const connack = await readExactly(socket, 4);
  if (connack[0] !== 0x20 || connack[3] !== 0x00) {
    throw new Error(`unexpected CONNACK: ${connack.toString("hex")}`);
  }
  socket.write(
    mqttPublishPacket(
      "fleet/demo/site/lab/device/env_sensor_01/command",
      Buffer.from(
        JSON.stringify({ online: true, sample_interval_seconds: 60 }),
      ),
    ),
  );
  socket.write(Buffer.from([0xe0, 0x00]));
  socket.end();
  console.log("MQTT PUBLISH: accepted");
}

async function readModbus() {
  const [host, port] = splitAddress(modbusAddr);
  const socket = await connect(host, port);
  socket.write(
    Buffer.from([
      0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00,
      0x02,
    ]),
  );
  const response = await readExactly(socket, 13);
  socket.end();
  if (response[7] !== 0x03) {
    throw new Error(`unexpected Modbus response: ${response.toString("hex")}`);
  }
  const registers: number[] = [];
  for (let index = 9; index + 1 < response.length; index += 2) {
    registers.push(response.readUInt16BE(index));
  }
  console.log("Modbus read:", registers);
}

function mqttConnectPacket(clientId: string): Buffer {
  const body = Buffer.concat([
    mqttString("MQTT"),
    Buffer.from([0x04, 0x02, 0x00, 0x3c]),
    mqttString(clientId),
  ]);
  return Buffer.concat([Buffer.from([0x10, body.length]), body]);
}

function mqttPublishPacket(topic: string, payload: Buffer): Buffer {
  const body = Buffer.concat([mqttString(topic), payload]);
  return Buffer.concat([Buffer.from([0x30, body.length]), body]);
}

function mqttString(value: string): Buffer {
  const payload = Buffer.from(value);
  return Buffer.concat([
    Buffer.from([payload.length >> 8, payload.length & 0xff]),
    payload,
  ]);
}

function splitAddress(value: string): [string, number] {
  const [host, port] = value.split(":");
  return [host, Number(port)];
}

function connect(host: string, port: number): Promise<net.Socket> {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection({ host, port }, () => resolve(socket));
    socket.setTimeout(2_000, () => {
      socket.destroy(new Error("socket timeout"));
    });
    socket.once("error", reject);
  });
}

function readExactly(socket: net.Socket, size: number): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    let total = 0;
    const onData = (chunk: Buffer) => {
      chunks.push(chunk);
      total += chunk.length;
      if (total >= size) {
        socket.off("data", onData);
        resolve(Buffer.concat(chunks, total).subarray(0, size));
      }
    };
    socket.on("data", onData);
    socket.once("error", reject);
  });
}
