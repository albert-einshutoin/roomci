# Task 02: Repository and Container Namespace Cleanup

## Status

`todo`

## Problem

Public metadata and Docker examples point at the stale
`albert-einshutoin/roomci` namespace. That makes the OSS surface look
accidental or copied from a private/personal scratch repo.

## Scope

- Align `Cargo.toml` repository metadata with the current Git remote.
- Replace container examples with an explicit placeholder namespace that
  evaluators can adapt before publishing.
- Avoid pretending that a public package or container has already been
  published.

## Acceptance Criteria

- No public doc or metadata file references `albert-einshutoin/roomci`.
- Docker docs clearly mark `ghcr.io/OWNER/roomci:latest` as a placeholder.

