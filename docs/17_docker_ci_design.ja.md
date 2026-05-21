# 17. DockerおよびCI設計

## なぜ Docker-first

The point of `roomci` is to make smart-home dependencies reproducible without real devices or field environments.

Docker enables:

- local reproduction
- CI execution
- Floci-like service emulation
- isolation from host machines
- deterministic startup
- easy integration with backend/mobile tests

## Images

### MVP: single image

```txt
ghcr.io/albert-einshutoin/roomci:latest
```

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
  ghcr.io/albert-einshutoin/roomci:latest \
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
  ghcr.io/albert-einshutoin/roomci:latest \
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
    image: ghcr.io/albert-einshutoin/roomci:latest
    command: serve --config /scenarios/generic_mqtt_retained_state.yaml --host 0.0.0.0 --port 8080 --mqtt-port 1883 --allow-non-loopback
    volumes:
      - ../examples:/scenarios:ro

  external-controller:
    image: ghcr.io/albert-einshutoin/roomci:latest
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
    image: ghcr.io/albert-einshutoin/roomci:latest
    command: run /scenarios/local_first_cloud_outage.yaml --junit /reports/roomci.xml
    volumes:
      - ../examples:/scenarios
      - ../reports:/reports
```

## GitHub Actions pattern

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
            ghcr.io/albert-einshutoin/roomci:latest \
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
