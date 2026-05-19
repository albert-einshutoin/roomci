# 01. NOT A HOTEL Research Synthesis

This document summarizes design implications extracted from public hiring pages, video summaries, smart-home engineering articles, and user-provided research notes. It should be read as product research and external interpretation, not as a claim about private NOT A HOTEL implementation details.

## Key observed themes

### 1. Smart home is core infrastructure, not a side feature

NOT A HOTEL treats smart home as a core part of the lodging experience. Lighting, HVAC, security, entertainment, blinds, sauna, bath, intercom, BMS alerts, and network operations all contribute to the stay experience.

The smart-home team covers software, hardware, network, electrical systems, equipment, field commissioning, and QA. The team builds and validates cloud, mobile, edge-server, hardware, monitoring, and operations tooling.

### 2. Local-first architecture is central

The smart-home system moved from a cloud-centric design to a hybrid local/cloud design.

Core idea:

```txt
iPad controller + local network + edge server + local MQTT broker
= guest experience can continue even during cloud/internet problems
```

The system uses MQTT for both local and cloud communications. Retained messages simplify state synchronization. QoS 1 shifts retry complexity into the protocol layer while requiring idempotent or duplicate-safe handling.

### 3. MQTT should be a first-class primitive

The earlier generic IoT design treated MQTT as one adapter among many. After the smart-home articles, MQTT must be elevated to the heart of `roomci`.

`roomci` should emulate:

- local broker
- cloud broker mock
- retained state
- duplicate delivery
- reconnect recovery
- cloud outage while local remains operational
- local-to-cloud bridge behavior

### 4. Field commissioning and QA are scaling bottlenecks

The team is scaling from roughly 10 locations toward many more. The articles emphasize that scaling cannot be solved only by adding people. Commissioning knowledge, field QA, construction coordination, device selection, control-panel design, and maintenance practices must become standardized and repeatable.

`roomci` should therefore model commissioning tasks as executable scenarios.

### 5. Building automation protocol strategy is pragmatic

NOT A HOTEL does not use one protocol everywhere. The direction is to pick the appropriate protocol per device class and failure domain.

Publicly discussed or research-derived stack and concepts:

- Go, TypeScript, Swift, Lua
- Protocol Buffers, gRPC, TCP/IP, MQTT, RS485, Modbus, KNX, DALI, SIP
- Google Cloud, AWS, Cloudflare, Twilio, Jamf
- UniFi, YAMAHA, Starlink
- Rebro / CAD
- Grafana / InfluxDB-like time-series analysis
- Slack / Zoom Phone / PagerDuty-like operations flows

### 6. Modbus deserves high priority

The provided Modbus article says NOT A HOTEL moved toward Modbus for floor heating, power meters, bath, and other systems. The benefits include:

- control-panel space saving
- cost optimization
- GUI debugging/monitoring tools
- smoother communication with partners via standard register maps
- detailed control and error-code acquisition beyond contact I/O
- simpler wiring through multi-drop bus behavior

`roomci` should support Modbus register-map validation from MVP.

### 7. BMS and operations are product areas

The BMS article shows that the internal BMS focuses on being useful for operations and LCM teams, not on heavyweight legacy BMS complexity.

Important flows:

- Slack notification for all alerts
- phone escalation for critical alerts
- runbook URL in notifications
- recovery notifications in Slack threads
- ticket-like action modal with assignee/status/comments
- web app list for alert tracking
- time-series storage and Grafana alerts
- future figure: floorplan mapping, unified monitoring and ticket UI, AI-based predictive detection

`roomci` should treat BMS alerting and operations flow as a first-class subsystem.

### 8. Control panel is the heart

The control-panel article describes:

- wired connections to most building systems
- replacement of uniform KNX dependency with a protocol-per-device strategy
- DALI gateway selection for low-level lighting control
- redundant 24V power supplies
- 24V UPS
- circuit protectors
- edge-computer redundancy
- device failure as an expected condition
- delivery stability, inventory, and serviceability

`roomci` should model failure-first thinking: power loss, UPS degradation, short circuit isolation, redundant edge failover, and module-specific failure modes.

### 9. Comfort automation is measurable

The HVAC articles emphasize:

- ceiling sensors and living-area sensors can differ significantly
- temperature/humidity can be translated into discomfort index
- room-level tuning matters
- auto mode should reduce user operations
- user override/cancel timing is a key feedback signal
- Grafana/InfluxDB and custom sensors support iteration
- sensor placement matters

`roomci` can include a comfort automation module to test if a scenario reaches target discomfort index and avoids user override.

### 10. Intercom and access control connect UX and safety

The intercom article includes:

- pre-check-in app registration
- PIN code entry
- DoorBird API
- camera/photo capture
- ONVIF / UniFi Protect recording
- SIP via Twilio to Zoom Phone
- DTMF controlling a relay contact
- PoE speakers triggered by webhook
- future need for local PBX fallback

This is not MVP, but it is a compelling future module.

## Implication for roomci

The most accurate positioning is:

```txt
A Docker-based emulator that turns NOT A HOTEL-like smart-home local-first architecture, field QA, BMS alerting, and building automation operations into repeatable CI scenarios.
```
