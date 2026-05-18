# syntax=docker/dockerfile:1

FROM rust:1.91-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p roomci-cli

FROM debian:bookworm-slim AS runtime

RUN useradd --uid 10001 --user-group --create-home --home-dir /home/roomci --shell /usr/sbin/nologin roomci
COPY --from=builder /app/target/release/roomci /usr/local/bin/roomci

USER roomci
WORKDIR /work
ENTRYPOINT ["roomci"]
CMD ["--help"]
