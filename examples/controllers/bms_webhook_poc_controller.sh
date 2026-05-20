#!/usr/bin/env sh
set -eu

ROOMCI_URL="${ROOMCI_URL:-http://127.0.0.1:8080}"
REPORT_DIR="${REPORT_DIR:-reports}"

mkdir -p "$REPORT_DIR"

curl -fsS -X POST "$ROOMCI_URL/external/bms/contact" \
  -H 'content-type: application/json' \
  -d '{"source":"contact.sauna_emergency_button","state":"on","severity":"critical"}' \
  > "$REPORT_DIR/bms_contact_response.json"

curl -fsS "$ROOMCI_URL/state" > "$REPORT_DIR/bms_contact_state.json"
curl -fsS "$ROOMCI_URL/timeline" > "$REPORT_DIR/bms_contact_timeline.json"
curl -fsS -X POST "$ROOMCI_URL/finish" > "$REPORT_DIR/bms_contact_finish.json"
curl -fsS "$ROOMCI_URL/reports/latest.md" > "$REPORT_DIR/bms_contact_latest.md"

grep -q 'external.bms.contact.sauna_emergency_button' "$REPORT_DIR/bms_contact_state.json"
grep -q 'external_bms_contact_observed' "$REPORT_DIR/bms_contact_timeline.json"
