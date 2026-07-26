# This Dockerfile is intentionally separate from the root runtime image.
# GitHub Actions bind-mounts the workspace and must write reports into it, so
# this action runs as root; the published root image remains non-root.
FROM rust:1.91-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p roomci-cli

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/roomci /usr/local/bin/roomci
COPY .github/actions/roomci/entrypoint.sh /usr/local/bin/roomci-action
RUN chmod 0755 /usr/local/bin/roomci-action

WORKDIR /github/workspace
ENTRYPOINT ["/usr/local/bin/roomci-action"]
