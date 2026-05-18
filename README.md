# roomci

**Local-first Smart Home QA & Operations Emulator for CI**

`roomci` is a Docker-based emulator for reproducing smart-home commissioning, local-first control, field QA, BMS alerting, and operations scenarios without real devices or an on-site environment.

It is designed as a **Floci / LocalStack-like emulator**, but for smart-home and building-automation systems:

- Floci / LocalStack: emulate cloud services for local and CI testing.
- roomci: emulate a smart-home control stack for local and CI testing.

## Core concept

Modern hospitality smart homes are not just IoT devices. They combine:

- iPad / mobile controllers
- local MQTT brokers
- edge servers / home control servers
- cloud MQTT brokers and cloud event pipelines
- DALI / KNX / Modbus / contact I/O / HVAC / lighting devices
- BMS alerts, Slack notifications, phone escalation, and runbooks
- network segmentation, WAN failover, Starlink-style backup paths
- commissioning, field QA, maintenance, and continuous operation

`roomci` makes these dependencies reproducible in Docker so that teams can test failure scenarios before guests experience them.

## Tagline

> Reproduce smart-home field failures before guests experience them.

## Primary use cases

1. **Local-first control QA**  
   Verify that local iPad → local MQTT broker → edge server → device control still works when the cloud is unavailable.

2. **Commissioning QA**  
   Turn field commissioning checks into reusable YAML scenarios.

3. **Building automation protocol simulation**  
   Test DALI-like lighting, Modbus device register maps, contact I/O alerts, HVAC devices, and KNX-like legacy bus behavior.

4. **BMS / operations alert simulation**  
   Verify Slack / phone-call / ticket / runbook flows for emergency alerts and recoveries.

5. **Network failure simulation**  
   Reproduce ISP outage, Starlink-style failover, VLAN isolation issues, packet loss, and local-only operation.

6. **Comfort automation simulation**  
   Test temperature/humidity sensors, discomfort index targets, HVAC auto mode, user override, and room-specific tuning.

## What this is not

`roomci` is not intended to be:

- a full KNX implementation
- a full DALI implementation
- a full BACnet implementation
- a full Matter / CSA Aliro implementation
- a replacement for production smart-home controllers
- a real BMS product
- a production-grade SIP / PBX implementation

The goal is to model enough behavior to make smart-home QA, field failures, commissioning, and operations flows testable in local and CI environments.

## Directory structure

```txt
roomci-docs-latest/
  README.md
  docs/
    00_executive_summary.md
    01_notahotel_research_synthesis.md
    02_product_requirements.md
    03_architecture.md
    04_local_first_mqtt_architecture.md
    05_edge_server_emulator.md
    06_device_model.md
    07_building_automation_protocol_strategy.md
    08_modbus_strategy.md
    09_dali_lighting_strategy.md
    10_bms_operations_emulation.md
    11_network_and_failover.md
    12_control_panel_fault_model.md
    13_comfort_automation.md
    14_intercom_and_access_control.md
    15_scenario_spec.md
    16_fault_injection.md
    17_docker_ci_design.md
    18_mvp_roadmap.md
    19_interview_positioning.md
    20_appendix_future_integrations.md
  examples/
    local_first_cloud_outage.yaml
    modbus_floor_heating.yaml
    dali_scene_partial_failure.yaml
    bms_sauna_emergency_alert.yaml
    edge_server_failover.yaml
    comfort_auto_mode.yaml
    starlink_failover.yaml
  compose/
    docker-compose.yml
  github/
    smart-home-ci.yml
  schemas/
    scenario.schema.json
  diagrams/
    architecture.mmd
```

## Recommended MVP

The first public version should focus on:

- Docker Compose stack
- local MQTT broker
- edge server emulator
- iPad controller simulator
- Modbus TCP device mock
- DALI-like lighting mock
- contact I/O mock
- BMS alert mock
- network/cloud outage scenario
- Markdown / JSON / JUnit reports

## Suggested positioning

> I analyzed NOT A HOTEL's smart-home hiring pages, videos, and engineering articles. Their smart-home system is not just IoT device control; it is a local-first, MQTT-driven, edge-server-based, building-automation and operations platform. I designed `roomci` as a Docker-based emulator to turn field QA, commissioning knowledge, and operational failure modes into repeatable CI scenarios.
