# Phase 5 Goal — Docker and CI Packaging

## Goal

Make the emulator usable through Docker Compose and GitHub Actions.

## In Scope

- Dockerfile.
- Docker Compose demo.
- GitHub Actions workflow.
- Report artifact output.

## Exit Criteria

- `docker compose -f compose/docker-compose.yml up --abort-on-container-exit` works.
- CI workflow uses supported CLI flags.
