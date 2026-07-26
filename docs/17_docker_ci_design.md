# 17. Docker and CI Design

## Why Docker-first

The point of `roomci` is to make smart-home dependencies reproducible without real devices or field environments.

Docker enables:

- local reproduction
- CI execution
- service-emulation style local reproduction
- isolation from host machines
- deterministic startup
- easy integration with backend/mobile tests

## Images

### MVP: single image

```txt
ghcr.io/albert-einshutoin/roomci:0.1.0
```

The GHCR namespace is the public roomci repository owner. Images are published
only by a verified version tag; before the first release, build the local
`Dockerfile` instead of assuming this image already exists.

The single image includes:

- scenario runner
- service-mode config checker
- MQTT behavior simulator
- edge emulator
- device mocks
- BMS ops mocks

### Future: split images

```txt
roomci-core
roomci-edge
roomci-mqtt
roomci-modbus
roomci-dali
roomci-ops
roomci-network
```

Start with one image to avoid overengineering.

## Docker run examples

Scenario mode:

```bash
docker run --rm \
  -v "$PWD/examples:/scenarios" \
  -v "$PWD/reports:/reports" \
  ghcr.io/albert-einshutoin/roomci:0.1.0 \
  run /scenarios/local_first_cloud_outage.yaml \
  --report-md /reports/roomci.md \
  --report-json /reports/roomci.json \
  --junit /reports/roomci.xml \
  --timeline-json /reports/roomci.timeline.json \
  --timeline-ndjson /reports/roomci.timeline.ndjson \
  --observability-json /reports/roomci.observability.json
```

Service-mode config check:

```bash
docker run --rm \
  -v "$PWD/examples:/scenarios" \
  ghcr.io/albert-einshutoin/roomci:0.1.0 \
  serve --config /scenarios/local_first_cloud_outage.yaml --check
```

Serve-mode black-box PoC:

```bash
make compose-poc
```

This starts `roomci serve` as one Compose service and runs `examples/controllers/http_poc_controller.sh` as a separate external controller service. The controller talks to `roomci` through HTTP only and writes JSON, Markdown, and JUnit reports under `reports/`.

## Docker Compose pattern

```yaml
services:
  roomci-serve:
    image: ghcr.io/albert-einshutoin/roomci:0.1.0
    command: serve --config /scenarios/generic_mqtt_retained_state.yaml --host 0.0.0.0 --port 8080 --mqtt-port 1883 --allow-non-loopback
    volumes:
      - ../examples:/scenarios:ro

  external-controller:
    image: ghcr.io/albert-einshutoin/roomci:0.1.0
    entrypoint: ["/bin/sh"]
    command: /controllers/http_poc_controller.sh
    environment:
      ROOMCI_URL: http://roomci-serve:8080
      REPORT_DIR: /reports
    depends_on:
      - roomci-serve
    volumes:
      - ../examples/controllers:/controllers:ro
      - ../reports:/reports

  scenario-smoke:
    image: ghcr.io/albert-einshutoin/roomci:0.1.0
    command: run /scenarios/local_first_cloud_outage.yaml --junit /reports/roomci.xml
    volumes:
      - ../examples:/scenarios
      - ../reports:/reports
```

## GitHub Actions pattern

When a job executes more than one scenario, use `--report-dir` so every
scenario keeps a complete evidence set. The numbered directory is based on the
input file stem rather than `scenario.name`, which prevents collisions between
independently authored files with the same scenario name.

```bash
roomci run /scenarios/local_first_cloud_outage.yaml /scenarios/edge_server_failover.yaml \
  --report-dir /reports/ci
```

The resulting `/reports/ci/summary.json` is the stable `roomci.summary.v1`
aggregate contract. Each `01_<stem>/` directory holds the JSON, Markdown,
JUnit, timeline JSON/NDJSON, and observability artifacts for that scenario.
Keep the existing single-report flags when deliberately exporting only the
last scenario.

```yaml
name: smart-home-ci

on:
  pull_request:

jobs:
  roomci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run roomci scenarios
        run: |
          docker run --rm \
            -v $PWD/examples:/scenarios \
            -v $PWD/reports:/reports \
            ghcr.io/albert-einshutoin/roomci:0.1.0 \
            run /scenarios/local_first_cloud_outage.yaml \
            --junit /reports/roomci.xml \
            --report-md /reports/roomci.md \
            --report-json /reports/roomci.json \
            --timeline-json /reports/roomci.timeline.json \
            --timeline-ndjson /reports/roomci.timeline.ndjson \
            --observability-json /reports/roomci.observability.json

      - name: Verify release evidence
        run: |
          make protocol-evidence
          make adapter-samples-smoke
          make s-tier-evidence-smoke
```

## CI outputs

Required:

- exit code 0/1
- Markdown summary
- JSON report
- JUnit XML
- timeline JSON
- timeline NDJSON
- observability JSON

Useful:

- GitHub Step Summary
- uploaded artifact
- per-scenario timeline
- trace/run correlation id
- artifact-level observability counters

## Report content

A report should include:

- scenario name
- pass/fail
- timeline
- failed assertions
- guest impact
- operations impact
- possible field causes
- suggested checks
- links to runbooks
