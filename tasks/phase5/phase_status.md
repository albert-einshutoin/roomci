# Phase 5 Status — Docker and CI Packaging

## Phase Status

`done`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_docker_image_task.md` | `done` | Codex | `docker build -t roomci:phase5 .`; `docker run roomci:phase5 validate ...`; `docker run roomci:phase5 run ...` | Image runs validate/run and writes report files |
| `02_compose_ci_task.md` | `done` | Codex | `docker compose -f compose/docker-compose.yml up --abort-on-container-exit --exit-code-from scenario-smoke`; GitHub Actions command parity review | Compose and CI build local image and use supported CLI flags |

## Blockers

- None.

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Docker build test | `done` | `docker build -t roomci:phase5 .` |
| Container validate smoke test | `done` | `docker run --rm -v "$PWD/examples:/scenarios:ro" roomci:phase5 validate /scenarios/local_first_cloud_outage.yaml /scenarios/bms_sauna_emergency_alert.yaml` |
| Container run/report smoke test | `done` | `docker run --rm -v "$PWD/examples:/scenarios:ro" -v "$PWD/reports:/reports" roomci:phase5 run /scenarios/bms_sauna_emergency_alert.yaml --junit /reports/bms.xml --report-md /reports/bms.md --report-json /reports/bms.json` |
| Compose smoke test | `done` | `docker compose -f compose/docker-compose.yml up --abort-on-container-exit --exit-code-from scenario-smoke` |
| GitHub Actions command parity check | `done` | `.github/workflows/smart-home-ci.yml` builds `roomci:ci`, validates scenarios, runs report-producing CLI commands, uploads `reports` |
