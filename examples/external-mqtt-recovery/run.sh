#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
out="$root/reports/external-mqtt"
mkdir -p "$out"
chmod 777 "$out"
export RUN_ID=bootstrap
compose=(docker compose -f "$here/compose.yaml" -p roomci-external-mqtt)
cleanup() { "${compose[@]}" down --volumes --remove-orphans >/dev/null 2>&1 || true; }
trap cleanup EXIT

"${compose[@]}" build runner sut
"${compose[@]}" up -d broker toxiproxy
"${compose[@]}" run --rm --no-deps --entrypoint python sut /verify_isolation.py
"${compose[@]}" run --rm --no-deps --entrypoint /bin/sh runner -c \
  'curl --fail --silent --show-error --retry 10 --retry-delay 1 --retry-connrefused -X POST -H "Content-Type: application/json" -d '\''{"name":"sut","listen":"0.0.0.0:1884","upstream":"broker:1883","enabled":true}'\'' http://toxiproxy:8474/proxies'

run_case() {
  local name=$1 mode=$2 expected_verdict=$3 expected_reason=$4 contract=${5:-/contract/recovery.yaml}
  export RUN_ID
  RUN_ID=$(python3 -c 'import uuid; print(uuid.uuid4().hex)')
  export SUT_MODE="$mode"
  rm -f "$out/$name.json" "$out/$name.xml" "$out/$name.log"
  "${compose[@]}" stop sut >/dev/null 2>&1 || true
  "${compose[@]}" rm -f sut >/dev/null 2>&1 || true
  if [[ "$mode" != "no-sut" ]]; then
    "${compose[@]}" up -d sut
  fi
  if [[ "$name" == "retained" ]]; then
    "${compose[@]}" run --rm --no-deps --entrypoint python injector /inject_retained.py
  fi
  local exit_code=0
  "${compose[@]}" run --rm --no-deps runner external-mqtt "$contract" \
    --run-id "$RUN_ID" --sut-version "$mode" \
    --json "/reports/$name.json" --junit "/reports/$name.xml" || exit_code=$?
  "${compose[@]}" logs --no-color broker toxiproxy sut > "$out/$name.log" 2>&1 || true
  python3 - "$out/$name.json" "$exit_code" "$expected_verdict" "$expected_reason" "$RUN_ID" <<'PY'
import json, sys
report = json.load(open(sys.argv[1]))
code, verdict, reason = int(sys.argv[2]), sys.argv[3], sys.argv[4]
assert report['evaluation_target'] == 'external_sut'
assert report['observation_source'] == 'real_mqtt_broker'
assert report['run_id'] == sys.argv[5]
assert report['verdict'] == verdict, report
if reason == 'fault_not_applied':
    assert report['reason'].startswith('fault_not_applied:'), report
else:
    assert report['reason'] == reason, report
assert code == (0 if verdict == 'passed' else 1), code
assert report['assertions'][0]['passed'] == (verdict == 'passed')
events = {event['event_type'] for event in report['timeline']}
if verdict == 'passed' or reason in ('missing_latest_report', 'rollback_observed'):
    assert {'initial_ready', 'fault_requested', 'fault_applied', 'fault_affected_sut', 'desired_accepted', 'fault_released'} <= events, events
print(f"{report['sut_version']}: {verdict} ({reason}) run={report['run_id']}")
PY
}

run_case fixed fixed passed latest_report_stable
run_case broken broken failed missing_latest_report
run_case no-sut no-sut inconclusive initial_report_missing
run_case stopped stop-after-initial failed missing_latest_report
run_case old-only old-only failed missing_latest_report
run_case foreign foreign-only failed missing_latest_report
run_case rollback rollback failed rollback_observed
run_case retained broken failed missing_latest_report
sed 's/proxy_name: sut/proxy_name: missing/' "$here/recovery.yaml" > "$out/missing-proxy.yaml"
run_case unapplied fixed inconclusive fault_not_applied /reports/missing-proxy.yaml
run_case fixed-again fixed passed latest_report_stable
python3 - "$out/fixed.json" "$out/broken.json" <<'PY'
import json, sys
fixed, broken = (json.load(open(path)) for path in sys.argv[1:])
assert fixed['contract'] == broken['contract'], 'fixed/broken contracts differ'
assert fixed['assertions'][0]['assertion_type'] == broken['assertions'][0]['assertion_type']
print('fixed and broken used the same contract and assertion')
PY
