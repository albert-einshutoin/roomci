# 00. エグゼクティブサマリー

## プロジェクト

`roomci` is a Docker-based local and CI emulator for smart-home commissioning, local-first control, BMS operations, and field-failure testing.

## これが重要な理由

Smart-home systems in hospitality do not fail like ordinary web applications. A failure can mean:

- the iPad controller cannot operate lighting or HVAC
- the room becomes uncomfortable
- an emergency button alert does not reach staff
- a door/intercom/contact relay does not behave as expected
- a network segment misconfiguration exposes a control network
- a cloud outage breaks remote operations
- commissioning knowledge remains trapped in individual experts

The strongest opportunity is not to emulate “every IoT device in the world.” The strongest opportunity is to make **one building / one smart home / one operational flow** reproducible in Docker.

## 最終的な概念

```txt
roomci
Local-first Smart Home QA & Operations Emulator for CI
```

## 設計 thesis

The system should be built around the following ideas:

1. **Local-first operation**  
   Core guest experience must continue even if cloud, internet, VPN, or upstream services fail.

2. **MQTT-centered state synchronization**  
   Local broker, retained messages, QoS1 duplicate/retry behavior, and reconnect synchronization are central.

3. **Edge server as the operational brain**  
   The house-level home control server runs local control logic and routes commands to devices.

4. **Building automation protocol mocks**  
   DALI, KNX-like bus, Modbus, contact I/O, HVAC, lighting, sauna, bath, and sensor models should be represented at a practical QA level.

5. **BMS and operations are first-class**  
   Slack notification, phone escalation, runbook links, ticket states, recovery notifications, Grafana-like metrics, and time-series exports matter as much as device commands.

6. **Commissioning knowledge should become code**  
   Field QA checklists, network checks, device mappings, and maintenance expectations should be executable scenarios.

## MVP scope

The recommended MVP is intentionally narrow:

- `roomci-core`: scenario runner, virtual time, assertions, report generator
- `roomci-mqtt`: local MQTT broker model, retained state, QoS1 failure modes
- `roomci-edge`: edge server emulator
- `roomci-devices`: Modbus TCP mock, DALI-like lighting, contact I/O, HVAC mock
- `roomci-ops`: BMSアラート, Slack/phone/ticket/runbook mock
- `roomci-network`: cloud outage, WAN outage, Starlink failover, VLAN model
- Docker Compose and GitHub Actions examples

## Strongest demos

1. Cloud outage while local iPad control still works
2. Modbus floor-heating register map with 0.1°C control
3. DALI scene partial failure detection
4. Sauna emergency button contact input triggers Slack + phone escalation
5. Edge server primary failure switches to secondary
6. Starlink-style WAN failover preserves operational communication
7. HVAC auto mode maintains target discomfort index without user override

## 非目的

- Full protocol conformance for KNX / DALI / BACnet / Matter
- Production BMS replacement
- Real device control in v0.1
- Firmware emulation
- SIP/PBX completeness
