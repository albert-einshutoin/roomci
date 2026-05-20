# 20. Appendix: Future Integrations

## Matter / CSA Aliro

Matter and CSA Aliro are interesting future standards, but they are not MVP priorities.

Use future positioning:

```txt
roomci can add Matter-like or Aliro-like profiles once the local-first QA core is stable.
```

## Home Assistant discovery

Home Assistant MQTT Discovery-like support can make demos more familiar to smart-home developers, but hospitality and building-automation evaluators generally prioritize local MQTT, Modbus, DALI, BMS, and network operations over generic Home Assistant conventions.

## AWS IoT Core-like bridge

This is more relevant than generic Azure Device Twin support because the local/cloud MQTT architecture can use an AWS IoT Core-like broker, rule router, and queue.

Potential components:

- publish endpoint
- rule routing
- queueing
- callback to Cloud Run-like service
- delayed delivery
- replay after outage

## Azure Device Twin-like

Useful as general IoT reference, but lower priority unless a target company uses Azure IoT.

## BACnet-like BMS object model

Useful for future BMS maturity:

- analogInput
- analogValue
- binaryInput
- binaryValue
- device status
- alarm state

## DoorBird / ONVIF / SIP / DTMF

Future module:

- PIN check-in
- photo capture event
- relay contact output
- staff call
- SIP/DTMF simulation
- local PBX fallback
- ONVIF recording event

## UniFi / Okta access-control drift

Future module:

- identity group source
- access-system group target
- drift detection
- deletion job canary test
- staging compatibility report

## Grafana / InfluxDB / Prometheus

MVP can export JSON/Prometheus-like metrics. Later versions can provide:

- Influx line protocol
- Grafana dashboard JSON
- alert rule templates

## Rebro / CAD / floorplan

Future direction:

- fixture/device map from floorplan
- alert location on drawing
- commissioning checklist by room/area
- control panel terminal mapping

## AI-assisted operations

Future AI integrations:

- incident summary generation
- runbook suggestion
- anomaly explanation
- commissioning report draft
- field issue clustering

AI should be assistant-layer, not core logic.
