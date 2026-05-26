# Interview Demo

Use this walkthrough when presenting `roomci` as a product evaluation build.
It is intentionally command-first: every claim should map to a local command,
scenario, or generated artifact.

## 1. Frame The Product

`roomci` is a CI-first QA contract emulator for MQTT-based edge-device,
smart-home, and building-automation systems.

The shortest accurate framing is:

```txt
Reproduce edge-device and smart-home field failures before users experience them.
```

Do not frame it as a production MQTT broker, production smart-home controller,
protocol certification suite, or private vendor-stack emulator.

## 2. Run The Strongest Passing Scenario

```bash
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --verbose
```

What to show:

- local-first control survives a cloud MQTT outage;
- retained MQTT state remains visible;
- the timeline explains the failure mode without real devices.

## 3. Run The Intentional Failure Report

```bash
cargo run -p roomci-cli -- run examples/dali_scene_partial_failure.yaml \
  --report-md reports/dali_scene_partial_failure.md \
  --report-json reports/dali_scene_partial_failure.json \
  --junit reports/dali_scene_partial_failure.xml
```

Expected result: the command exits with code `1`. That is the product point:
`roomci` catches a field-impacting contract failure and produces evidence for
humans and CI systems.

Open the Markdown report:

```bash
sed -n '1,120p' reports/dali_scene_partial_failure.md
```

What to show:

- failed assertion;
- guest or user impact;
- JSON/JUnit outputs for CI.

## 4. Show The Black-Box PoC Path

```bash
make compose-poc
```

What to show:

- `roomci serve` runs as a localhost service;
- a separate controller container drives the HTTP API;
- reports are written under `reports/`;
- the evaluator does not need private hardware or a live site.

## 5. Show Protocol Boundaries Honestly

Use these docs when the evaluator asks how deep the protocol support is:

- [`PROTOCOL_SUPPORT_MATRIX.md`](PROTOCOL_SUPPORT_MATRIX.md)
- [`PROTOCOL_CONFORMANCE_REGISTRY.md`](PROTOCOL_CONFORMANCE_REGISTRY.md)
- [`MQTT_SERVE_SUBSET.md`](MQTT_SERVE_SUBSET.md)
- [`MODBUS_TCP_SUBSET.md`](MODBUS_TCP_SUBSET.md)
- [`B_TIER_PROTOCOL_PROFILES.md`](B_TIER_PROTOCOL_PROFILES.md)

The correct claim is that MQTT and Modbus have practical verified subsets.
Matter, BACnet, KNX, and OPC UA are contract-profile surfaces, not wire-level
implementations.

## 6. Close With The Evaluator Path

Point evaluators to:

- [`INTEGRATION_ONBOARDING.md`](INTEGRATION_ONBOARDING.md)
- [`EVALUATION_EVIDENCE_PACK.md`](EVALUATION_EVIDENCE_PACK.md)
- [`PRE_ADOPTION_POC_CHECKLIST.md`](PRE_ADOPTION_POC_CHECKLIST.md)
- [`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md)

The next useful step is to map real non-production topics, payload fields,
Modbus registers, BMS alert contracts, auth assumptions, and acceptance
criteria into an adapter contract.
