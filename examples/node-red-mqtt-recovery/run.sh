#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
out="$root/reports/node-red-external-sut"
mkdir -p "$out"
chmod 777 "$out"
export RUN_ID=bootstrap OMIT_LATEST=false
compose=(docker compose -f "$here/compose.yaml" -p roomci-node-red)
active_case=

collect() {
  if [[ -n "$active_case" ]]; then
    for service in node-red broker toxiproxy adapter; do
      "${compose[@]}" logs --no-color "$service" > "$out/$active_case/$service.log" 2>&1 || true
    done
  fi
  "${compose[@]}" down --volumes --remove-orphans >/dev/null 2>&1 || true
  active_case=
}
trap collect EXIT

"${compose[@]}" build runner adapter

run_case() {
  active_case=$1
  export RUN_ID OMIT_LATEST
  RUN_ID=$(python3 -c 'import uuid; print(uuid.uuid4().hex)')
  OMIT_LATEST=$2
  mkdir -p "$out/$active_case"
  chmod 777 "$out/$active_case"
  rm -f "$out/$active_case/recovery.json" "$out/$active_case/recovery.xml" \
    "$out/$active_case/roomci.log"
  cp "$here/flows.json" "$here/mapping.yaml" "$here/recovery.yaml" \
    "$here/compose.yaml" "$out/$active_case/"
  printf 'run_id=%s\nnode_red_version=4.1.15\nnode_red_image=%s\nomit_latest=%s\n' \
    "$RUN_ID" 'nodered/node-red@sha256:7aa04e1c7be16aec5b4b4d6e64ae863c4720b0ab18e2c1f46905f7b0c71e3a19' \
    "$OMIT_LATEST" > "$out/$active_case/conditions.txt"
  "${compose[@]}" up -d broker toxiproxy
  "${compose[@]}" exec -T broker mosquitto_pub -h localhost -t \
    "byos/$RUN_ID/thermostat-1/readiness" -q 1 -m ready
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
  "${compose[@]}" logs --no-color adapter > "$out/$active_case/adapter.log"
  grep -q 'app desired broker_ack revision=1' "$out/$active_case/adapter.log"
  grep -q 'app desired broker_ack revision=2' "$out/$active_case/adapter.log"
  if [[ "$OMIT_LATEST" == true ]]; then
    grep -q 'ignored retained report' "$out/$active_case/adapter.log"
    ! grep -q 'reported revision=2' "$out/$active_case/adapter.log"
  fi
  python3 - "$out/$active_case/recovery.json" "$exit_code" "$OMIT_LATEST" "$RUN_ID" <<'PY'
import json, sys
with open(sys.argv[1]) as source:
    report = json.load(source)
assert report['run_id'] == sys.argv[4], report
negative = sys.argv[3] == 'true'
assert int(sys.argv[2]) == (1 if negative else 0), report
expected = ('failed', 'missing_latest_report') if negative else ('passed', 'latest_report_stable')
assert (report['verdict'], report['reason']) == expected, report
events = [event['event_type'] for event in report['timeline']]
for event in ('initial_ready', 'fault_applied', 'fault_affected_sut', 'desired_accepted',
              'fault_released'):
    assert event in events, (event, events)
if negative:
    assert 'latest_reached' not in events, events
    assert 'stale_report' in events, events
else:
    assert 'latest_reached' in events, events
print(f"Node-RED: {report['verdict']} ({report['reason']}) run={report['run_id']}")
PY
  collect
}

run_case recovery false
run_case stale-retained-foreign true
