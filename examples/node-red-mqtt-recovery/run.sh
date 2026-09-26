#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
out="$root/reports/node-red-external-sut"
mkdir -p "$out"
chmod 777 "$out"
export RUN_ID=bootstrap OMIT_LATEST=false DEVIATION_VALUE= DEVIATION_DELAY_MS=0
compose=(docker compose -f "$here/compose.yaml" -p roomci-node-red)
active_case=

collect() {
  if [[ -n "$active_case" ]]; then
    for service in node-red broker toxiproxy adapter; do
      "${compose[@]}" logs --timestamps --no-color "$service" > "$out/$active_case/$service.log" 2>&1 || true
    done
  fi
  "${compose[@]}" down --volumes --remove-orphans >/dev/null 2>&1 || true
  active_case=
}
trap collect EXIT

"${compose[@]}" build runner adapter

run_case() {
  active_case=$1
  export RUN_ID OMIT_LATEST DEVIATION_VALUE DEVIATION_DELAY_MS
  RUN_ID=$(python3 -c 'import uuid; print(uuid.uuid4().hex)')
  OMIT_LATEST=$2
  DEVIATION_VALUE=${3:-}
  DEVIATION_DELAY_MS=${4:-0}
  mkdir -p "$out/$active_case"
  chmod 777 "$out/$active_case"
  rm -f "$out/$active_case/recovery.json" "$out/$active_case/recovery.xml" \
    "$out/$active_case/roomci.log"
  cp "$here/flows.json" "$here/mapping.yaml" "$here/recovery.yaml" \
    "$here/compose.yaml" "$out/$active_case/"
  printf 'run_id=%s\nnode_red_version=4.1.15\nnode_red_image=%s\nomit_latest=%s\ndeviation_value=%s\ndeviation_delay_ms=%s\n' \
    "$RUN_ID" 'nodered/node-red@sha256:7aa04e1c7be16aec5b4b4d6e64ae863c4720b0ab18e2c1f46905f7b0c71e3a19' \
    "$OMIT_LATEST" "$DEVIATION_VALUE" "$DEVIATION_DELAY_MS" > "$out/$active_case/conditions.txt"
  "${compose[@]}" up -d broker toxiproxy
  local broker_ready=false
  for attempt in {1..10}; do
    if "${compose[@]}" exec -T broker timeout 1 mosquitto_pub -h localhost -t \
      "byos/$RUN_ID/thermostat-1/readiness" -q 1 -m ready >/dev/null 2>&1; then
      broker_ready=true
      printf 'broker_ready_attempt=%s\n' "$attempt" >> "$out/$active_case/conditions.txt"
      break
    fi
    sleep 0.25
  done
  [[ "$broker_ready" == true ]] || { echo 'broker MQTT not ready after bounded retries' >&2; exit 1; }
  if [[ "$OMIT_LATEST" == true ]]; then
    : > "$out/$active_case/injection.log"
    "${compose[@]}" exec -T broker mosquitto_pub -h localhost -t \
      "byos/$RUN_ID/thermostat-1/reported" -q 1 -r -m '{"temperature":24}'
    echo 'published retained temperature=24 before adapter subscription' >> "$out/$active_case/injection.log"
  fi
  "${compose[@]}" up -d adapter
  for _ in {1..120}; do
    if "${compose[@]}" logs adapter 2>&1 | grep -q 'adapter subscribed'; then break; fi
    sleep 0.25
  done
  "${compose[@]}" logs adapter 2>&1 | grep -q 'adapter subscribed'
  if [[ "$OMIT_LATEST" == true ]]; then
    "${compose[@]}" logs adapter 2>&1 | grep -q 'ignored retained report'
    "${compose[@]}" exec -T broker mosquitto_pub -h localhost -t \
      "byos/foreign-$RUN_ID/thermostat-1/reported" -q 1 -m '{"temperature":24}'
    echo 'published foreign-run temperature=24' >> "$out/$active_case/injection.log"
  fi
  "${compose[@]}" run --rm --no-deps --entrypoint /bin/sh runner -c \
    'curl --fail --silent --show-error --retry 10 --retry-delay 1 --retry-connrefused -X POST -H "Content-Type: application/json" -d '\''{"name":"sut","listen":"0.0.0.0:1884","upstream":"broker:1883","enabled":true}'\'' http://toxiproxy:8474/proxies'
  "${compose[@]}" up -d node-red
  python3 - "$("${compose[@]}" ps -q broker)" \
    "$("${compose[@]}" ps -q toxiproxy)" \
    "$("${compose[@]}" ps -q node-red)" <<'PY' | tee "$out/$active_case/isolation.log"
import json, subprocess, sys
containers = json.loads(subprocess.check_output(['docker', 'inspect', *sys.argv[1:]]))
broker, proxy, sut = (set(container['NetworkSettings']['Networks']) for container in containers)
assert broker and sut and proxy, 'network attachment missing'
assert not broker & sut, 'Node-RED has a direct broker network'
assert broker & proxy and sut & proxy, 'proxy does not bridge both networks'
print('broker ready; Node-RED and broker have no shared network; proxy bridges both')
PY

  local injection_pid=
  if [[ "$OMIT_LATEST" == true ]]; then
    (
      for _ in {1..160}; do
        if "${compose[@]}" logs adapter 2>&1 | awk '
          /desired queued revision=2/ { latest=1 }
          latest && /sut status=online/ { found=1 }
          END { exit !found }
        '; then
          "${compose[@]}" exec -T broker mosquitto_pub -h localhost -t \
            "byos/$RUN_ID/thermostat-1/reported" -q 1 -m '{"temperature":21}'
          echo 'published non-retained stale temperature=21 after reconnect'
          exit 0
        fi
        sleep 0.25
      done
      exit 1
    ) >> "$out/$active_case/injection.log" 2>&1 &
    injection_pid=$!
  fi

  local exit_code=0
  "${compose[@]}" run --rm --no-deps runner external-mqtt /contract/recovery.yaml \
    --run-id "$RUN_ID" --sut-version 'Node-RED 4.1.15' \
    --json "/reports/$active_case/recovery.json" \
    --junit "/reports/$active_case/recovery.xml" \
    > "$out/$active_case/roomci.log" 2>&1 || exit_code=$?
  cat "$out/$active_case/roomci.log"
  test -s "$out/$active_case/recovery.xml"
  if [[ -n "$injection_pid" ]]; then wait "$injection_pid"; fi
  if [[ "$active_case" == late-deviation ]]; then sleep 1; fi
  "${compose[@]}" logs --timestamps --no-color adapter > "$out/$active_case/adapter.log"
  "${compose[@]}" logs --timestamps --no-color node-red > "$out/$active_case/node-red.log"
  grep -q 'app desired broker_ack revision=1' "$out/$active_case/adapter.log"
  grep -q 'app desired broker_ack revision=2' "$out/$active_case/adapter.log"
  if [[ "$OMIT_LATEST" == true ]]; then
    grep -q 'ignored retained report' "$out/$active_case/adapter.log"
    ! grep -q 'reported revision=2' "$out/$active_case/adapter.log"
  fi
  if [[ -n "$DEVIATION_VALUE" ]]; then
    grep -q "negative control reported temperature=24" "$out/$active_case/node-red.log"
    grep -q "negative control reported temperature=$DEVIATION_VALUE" "$out/$active_case/node-red.log"
    grep -q "reported revision=2 value=24" "$out/$active_case/adapter.log"
    grep -q "reported revision=.* value=$DEVIATION_VALUE" "$out/$active_case/adapter.log"
  fi
  python3 - "$out/$active_case/recovery.json" "$out/$active_case/recovery.xml" \
    "$exit_code" "$active_case" "$RUN_ID" "$DEVIATION_VALUE" <<'PY' \
    | tee "$out/$active_case/assertions.log"
import json, sys, xml.etree.ElementTree as ET
from datetime import datetime
with open(sys.argv[1]) as source:
    report = json.load(source)
assert report['run_id'] == sys.argv[5], report
case = sys.argv[4]
expected = {
    'stale-retained-foreign': ('failed', 'missing_latest_report'),
    'rollback': ('failed', 'rollback_observed'),
    'unexpected-value': ('failed', 'rollback_observed'),
}.get(case, ('passed', 'latest_report_stable'))
assert int(sys.argv[3]) == (1 if expected[0] == 'failed' else 0), report
assert (report['verdict'], report['reason']) == expected, report
events = [event['event_type'] for event in report['timeline']]
for event in ('initial_ready', 'fault_applied', 'fault_affected_sut', 'desired_accepted',
              'fault_released'):
    assert event in events, (event, events)
if case == 'stale-retained-foreign':
    assert 'latest_reached' not in events, events
    assert 'stale_report' in events, events
else:
    assert 'latest_reached' in events, events
failure = ET.parse(sys.argv[2]).find('.//failure')
assert (failure is None) == (expected[0] == 'passed')
if failure is not None:
    assert failure.attrib['message'] == expected[1]
if case in ('rollback', 'unexpected-value', 'late-deviation'):
    reports = [(event, json.loads(event['message'])) for event in report['timeline']
               if event['event_type'] == 'sut_reported']
    latest = next(event for event, data in reports
                  if data['revision'] == 2 and data['value'] == '24')
    latest_at = datetime.fromisoformat(latest['at'])
    deviation = [(event, data) for event, data in reports
                 if data['value'] == sys.argv[6]
                 and datetime.fromisoformat(event['at']) > latest_at]
    if case == 'late-deviation':
        assert not deviation, deviation
    else:
        assert len(deviation) == 1, deviation
        event, data = deviation[0]
        assert data['revision'] == (1 if case == 'rollback' else 2), data
        delta = (datetime.fromisoformat(event['at']) - latest_at).total_seconds()
        assert 0 < delta < report['contract']['stability_ms'] / 1000, delta
        print(f"observed 24 at {latest['at']}; {data['value']} at {event['at']}; "
              f"delta={delta:.3f}s; verdict={report['verdict']}/{report['reason']}")
print(f"Node-RED: {report['verdict']} ({report['reason']}) run={report['run_id']}")
PY
  collect
}

run_case unexpected-value false 25 200
run_case recovery false
run_case stale-retained-foreign true
run_case rollback false 21 200
run_case late-deviation false 25 1500
