# syntax=docker/dockerfile:1

FROM rust:1.91-slim-bookworm@sha256:8514999d4786ef12efe89239e86b3d0a021b94b9d35108c8efe6c79ca7dc1a65 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --locked --release -p roomci-cli

FROM debian:bookworm-slim@sha256:7b140f374b289a7c2befc338f42ebe6441b7ea838a042bbd5acbfca6ec875818 AS runtime

ARG ROOMCI_VERSION=0.1.0
ARG ROOMCI_REVISION=unknown
LABEL org.opencontainers.image.source="https://github.com/albert-einshutoin/roomci" \
      org.opencontainers.image.version="${ROOMCI_VERSION}" \
      org.opencontainers.image.revision="${ROOMCI_REVISION}" \
      org.opencontainers.image.licenses="Apache-2.0"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --uid 10001 --user-group --create-home --home-dir /home/roomci --shell /usr/sbin/nologin roomci
COPY --from=builder /app/target/release/roomci /usr/local/bin/roomci

USER roomci
WORKDIR /work
ENTRYPOINT ["roomci"]
CMD ["--help"]
