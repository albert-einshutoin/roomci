#!/usr/bin/env sh
# Minimal external controller for the /external/bms/contact PoC surface.
#
# Drives the roomci serve HTTP API as a black-box client (POST a BMS contact
# event, observe state and timeline, finish the run, fetch the rendered
# report) and asserts each step actually landed on the server.
#
# Set ROOMCI_URL to the running serve base URL (defaults to localhost:8080)
# and REPORT_DIR to where this script should write artifacts (defaults to
# ./reports).

set -eu

ROOMCI_URL="${ROOMCI_URL:-http://127.0.0.1:8080}"
REPORT_DIR="${REPORT_DIR:-reports}"

mkdir -p "$REPORT_DIR"

# 1. Post a BMS contact observation.
curl -fsS -X POST "$ROOMCI_URL/external/bms/contact" \
  -H 'content-type: application/json' \
  -d '{"source":"contact.sauna_emergency_button","state":"on","severity":"critical"}' \
  > "$REPORT_DIR/bms_contact_response.json"

# Response must echo the sanitized source so the controller can confirm
# its observation was actually accepted (not just absorbed by a 202).
grep -q '"accepted":true' "$REPORT_DIR/bms_contact_response.json"
grep -q '"source":"contact.sauna_emergency_button"' "$REPORT_DIR/bms_contact_response.json"

# 2. Observe live serve state before finishing the run. The observation
#    must be visible under the dedicated external_observations bucket.
curl -fsS "$ROOMCI_URL/state" > "$REPORT_DIR/bms_contact_state.json"
grep -q '"external_observations"' "$REPORT_DIR/bms_contact_state.json"
grep -q 'contact.sauna_emergency_button' "$REPORT_DIR/bms_contact_state.json"

# 3. Observe the timeline. The event_type must match the documented label
#    so downstream consumers can filter by it deterministically.
curl -fsS "$ROOMCI_URL/timeline" > "$REPORT_DIR/bms_contact_timeline.json"
grep -q 'external_bms_contact_observed' "$REPORT_DIR/bms_contact_timeline.json"
grep -q 'contact.sauna_emergency_button' "$REPORT_DIR/bms_contact_timeline.json"

# 4. Finish the run. The response must confirm the run has completed so
#    the controller can move on to result collection instead of polling.
curl -fsS -X POST "$ROOMCI_URL/finish" > "$REPORT_DIR/bms_contact_finish.json"
grep -q '"finished":true' "$REPORT_DIR/bms_contact_finish.json"

# 5. Fetch the rendered Markdown report. The BMS observation surfaces in
#    the timeline section of the report; both the event_type and the
#    sanitized target must be present so a human reviewer can see what
#    the external controller did.
curl -fsS "$ROOMCI_URL/reports/latest.md" > "$REPORT_DIR/bms_contact_latest.md"
grep -q 'external_bms_contact_observed' "$REPORT_DIR/bms_contact_latest.md"
grep -q 'contact.sauna_emergency_button' "$REPORT_DIR/bms_contact_latest.md"

echo "ok: bms_webhook_poc_controller validated all assertions"
