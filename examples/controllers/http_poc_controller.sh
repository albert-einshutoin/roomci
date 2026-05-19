#!/bin/sh
set -eu

ROOMCI_URL="${ROOMCI_URL:-http://127.0.0.1:8080}"
REPORT_DIR="${REPORT_DIR:-reports}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-30}"

mkdir -p "$REPORT_DIR"

deadline=$(( $(date +%s) + TIMEOUT_SECONDS ))
until curl -fsS "$ROOMCI_URL/health" > "$REPORT_DIR/health.json"; do
  if [ "$(date +%s)" -ge "$deadline" ]; then
    echo "roomci did not become healthy at $ROOMCI_URL" >&2
    exit 1
  fi
  sleep 1
done

curl -fsS \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"kind":"controller_probe","source":"external-http-poc-controller"}' \
  "$ROOMCI_URL/fault" > "$REPORT_DIR/fault.json"

curl -fsS \
  -X POST \
  "$ROOMCI_URL/finish" > "$REPORT_DIR/finish.json"

curl -fsS "$ROOMCI_URL/reports/latest.json" > "$REPORT_DIR/external_controller_latest.json"
curl -fsS "$ROOMCI_URL/reports/latest.md" > "$REPORT_DIR/external_controller_latest.md"
curl -fsS "$ROOMCI_URL/reports/latest.junit.xml" > "$REPORT_DIR/external_controller_latest.xml"

grep -q '"finished":true' "$REPORT_DIR/finish.json"
grep -q '"result":"passed"' "$REPORT_DIR/finish.json"
grep -q 'external-http-poc-controller' "$REPORT_DIR/external_controller_latest.json"
grep -q 'failures="0"' "$REPORT_DIR/external_controller_latest.xml"
