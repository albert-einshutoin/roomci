# 17. Docker and CI Design

## Why Docker-first

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
  --junit /reports/roomci.xml
```

Future service mode:

```bash
# planned, not implemented in v0.1
docker run --rm \
  -p 8080:8080 \
  -p 1883:1883 \
  -v "$PWD/examples:/scenarios" \
  ghcr.io/albert-einshutoin/roomci:latest \
  serve --config /scenarios/local_first_cloud_outage.yaml
```

## Docker Compose pattern

```yaml
services:
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
            --report-md /reports/roomci.md
```

## CI outputs

Required:

- exit code 0/1
- Markdown summary
- JSON report
- JUnit XML

Useful:

- GitHub Step Summary
- uploaded artifact
- per-scenario timeline

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
