FROM golang:1.23-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends nodejs npm \
    && npm install -g tsx@4.20.6 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /work
