# 05. Docker and CI Design

## Docker Image

Image name:

```text
ghcr.io/albert-einshutoin/roomci:latest
```

## Modes

### Serve mode

```bash
docker run --rm \
  -p 8080:8080 \
  -p 1883:1883 \
  -v $(pwd)/roomci:/roomci \
  ghcr.io/albert-einshutoin/roomci:latest \
  serve --config /roomci/room.yaml
```

### Run mode

```bash
docker run --rm \
  -v $(pwd)/examples:/scenarios \
  -v $(pwd)/reports:/reports \
  ghcr.io/albert-einshutoin/roomci:latest \
  run /scenarios/checkin_lock_offline.yaml \
  --junit /reports/roomci.xml \
  --markdown /reports/roomci.md \
  --json /reports/roomci.json
```

## Dockerfile Requirements

- Small image.
- Single static binary if possible.
- Non-root user.
- Healthcheck endpoint.
- Supports `serve`, `run`, `validate`.

Example:

```dockerfile
FROM rust:1-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN useradd -m -u 10001 roomci
COPY --from=builder /app/target/release/roomci /usr/local/bin/roomci
USER roomci
EXPOSE 8080 1883
ENTRYPOINT ["roomci"]
CMD ["serve", "--http", "0.0.0.0:8080", "--mqtt", "0.0.0.0:1883"]
```

## Docker Compose Design

Use Compose to run:

- `roomci`
- app/backend under test
- test runner
- optional Mosquitto

See `examples/docker-compose.yml`.

## GitHub Actions Design

```yaml
name: smart-room-ci

on:
  pull_request:

jobs:
  roomci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run smart-room scenario tests
        run: |
          mkdir -p reports
          docker run --rm \
            -v $PWD/examples:/scenarios \
            -v $PWD/reports:/reports \
            ghcr.io/albert-einshutoin/roomci:latest \
            run /scenarios/checkin_lock_offline.yaml \
            --junit /reports/roomci.xml \
            --markdown /reports/roomci.md

      - name: Upload roomci report
        uses: actions/upload-artifact@v4
        with:
          name: roomci-report
          path: reports/
```

## CI Report Requirements

### JUnit

Must represent failed assertions as test failures.

### Markdown

Must include:

- Scenario name
- Result
- Guest impact
- Timeline
- Failed assertions
- Suggested recovery/fallback

### JSON

Machine-readable event timeline and assertions.

## Healthcheck

```http
GET /healthz
```

Response:

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```
