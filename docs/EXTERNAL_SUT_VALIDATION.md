# Bring Your Own SUT: Node-RED external MQTT validation

This is an external-validity PoC, not a supported-product claim. Node-RED 4.1.15
(`nodered/node-red@sha256:7aa04e1c7be16aec5b4b4d6e64ae863c4720b0ab18e2c1f46905f7b0c71e3a19`)
runs an ordinary MQTT In → Function → MQTT Out thermostat flow. No Node-RED
runtime or roomci core code is changed. Run `bash examples/node-red-mqtt-recovery/run.sh`
with Docker Compose. The separate `node-red-external-sut` CI job saves five
cases under `reports/node-red-external-sut/`; the reference SUT's ten cases
remain in `external-mqtt-recovery`.

Use source commit `954b316e175ffb76f9f73155c0f89d5a2235fd3c` or a
reviewed descendant. Published `v0.1.1` binaries and the versioned Action do
not contain this external example or the `external-mqtt` command.

## Mapping and boundaries

| Surface | PoC mapping |
|---|---|
| Device identity | `thermostat-1` in both topic namespaces; no identity field in the Node-RED payload |
| roomci topics | `roomci/<run-id>/thermostat-1/{desired,reported,status}` |
| Node-RED topics | `byos/<run-id>/thermostat-1/{desired,reported,status}` from container environment variables |
| Desired payload | roomci revision 1/2 values `"21"`/`"24"` → retained `{"targetTemperature":21}`/`{"targetTemperature":24}` |
| Reported payload | non-retained integer `temperature` → unchanged observed value plus bridge-assigned revision; unexpected `25` remains `"value":"25"` |
| Status and disconnect | Node-RED broker config publishes non-retained `online` birth and `offline` last will; adapter maps these to roomci status JSON |
| Readiness | adapter subscribes first; broker MQTT publish succeeds; roomci independently waits for observer SUBACKs and the initial report within 15 s |
| Recovery and stability | 30 s monotonic recovery deadline; 1 s stability window |

`flows.json` uses only built-in MQTT and Function nodes. The function reads
`targetTemperature` and emits `temperature`; it does not read or emit run ID,
device ID, revision, or roomci's `value`. The `NR_OMIT_LATEST` switch is used
only by the missing-report negative control. The deviation controls make the
same Function node publish `21` or `25` after its correct `24` report.
`adapter.py` is an external broker-side process, not code added
to Node-RED or roomci. It verifies the roomci desired identity and the two
contract values, forwards retained desired state, and converts observed state
and birth/will messages. Its MQTT connection remains on `broker_net`; Node-RED
has only `sut_net` and reaches the broker through `toxiproxy:1884`.

### What the example actually reads

| File | Role and change boundary |
|---|---|
| `recovery.yaml` | Runtime contract mounted into the roomci runner. Deadlines and stability window are YAML values; initial/latest values, device, broker, and proxy must remain consistent with the other runtime files. Editing this YAML alone does not remap an application. |
| `compose.yaml` and `flows.json` | Runtime service wiring and Node-RED flow. Compose supplies run-specific topic variables; the flow fixes `targetTemperature` → `temperature`, MQTT endpoint, clean session, and `online` birth/`offline` will. A different topic scheme, payload, or connection signal needs corresponding flow/Compose changes. |
| `adapter.py` | Executed Python bridge. It fixes the two desired pairs `(1, "21")`/`(2, "24")`, topic prefixes, payload extraction, status conversion, and bridge-owned revision assignment. Customer mappings require code changes here, with matching flow/contract/harness changes where applicable. |
| `run.sh` | Executed case harness. It controls the fault, negative injections, expected verdicts, and artifact checks for this PoC; changing the application or assertions may require a harness edit. |
| `mapping.yaml` | Explanation copied into the artifact only. The CLI, adapter, and Node-RED never load it; editing it alone changes no test behavior. It is separate from the internal-model `adapter validate` contract. |

Leaving roomci core unchanged does not mean a customer can integrate by
editing configuration alone. This example assumes changeable run topics and
a trustworthy publisher on its reported/status topics.

## Correlation decision

The current `external-mqtt` command fixes its topic and report JSON schema.
A bare Node-RED `{"temperature":24}` report cannot satisfy that schema; merely
changing YAML cannot map its fields. The first solution therefore uses an
external bridge and leaves roomci core unchanged. The bridge creates an
adapter-owned revision **only** after seeing roomci's desired on a fresh run
topic. It accepts each non-retained integer report on that run's exact topic
while a non-retained online birth has been observed. This is the correlation
gate; matching the expected value is a separate roomci verdict check.
Previously requested `21`/`24` map to their respective revision. A valid
unrequested value such as `25` keeps its actual value and takes the current
desired phase's adapter-assigned revision (2 after desired `24`). The SUT
never returns run ID, device ID, request ID, or revision. Those fields in
roomci's report are **bridge-generated correlation metadata**, not SUT-origin
proof. If a report arrives before any desired phase can be assigned, the
adapter raises an explicit correlation error instead of silently discarding it.

The initial `21` response is observed as a baseline before the fault. The new
`24` value has not been used
earlier in the run. After the offline will, the retained latest desired is
delivered to Node-RED's clean MQTT session on reconnect; a matching report
from that connection is required for success. The runner still requires the
broker's desired PUBACK, and the harness checks the bridge's separate QoS 1
PUBACK for **both** application desired publishes. It also requires the
SUT-only cut and offline will, path restoration,
latest report before deadline, and a stable window.
An unexpected `25` after the latest `24` reaches roomci as revision 2,
value `25`; during stability roomci returns `failed/rollback_observed`.
The bridge does not normalize `25` into `24` or hide it in a log-only warning.

| Candidate | Decision |
|---|---|
| Device ID plus value alone | Insufficient: an old retained `24` could look current. |
| Fresh topic per run | Used: previous runs' reports are outside the adapter's exact subscriptions. Requires a SUT whose topics can be configured for a run. |
| Retained clear at start | Not required for this PoC: fresh namespace and retained-report rejection give an explicit negative check. Retained **desired** is intentional. |
| Initial baseline | Used: initial `21` report precedes the cut; later `21` remains revision 1 and cannot satisfy revision 2. |
| Broker timestamp/order | MQTT 3.1.1 does not provide a trusted publish timestamp or publisher identity to this subscriber. We use observed event order and the runner's monotonic deadline, not broker wall time. |
| Connection evidence | The broker emits the Node-RED offline will on the TCP cut; a later online birth precedes acceptance of a latest report. |

This proof assumes the isolated test broker and exclusive control of the
current-run reported and status topics. A different publisher deliberately
writing a fresh, non-retained `24` to the **current** run's reported topic
after online is indistinguishable from Node-RED under anonymous MQTT.
Customer use requires broker credentials/ACLs or an equivalent authenticated
provenance boundary. A fixed-topic SUT that cannot use per-run namespaces also
needs a separately specified reset/provenance strategy; this PoC does not
claim to validate that case.

## Reproduction and evidence

Install Git, Bash, Python 3, and a running Docker Engine with the
`docker compose` CLI (v2 or later). From a new checkout, run the following at
the repository root:

```bash
git clone https://github.com/albert-einshutoin/roomci.git
cd roomci
git checkout 954b316e175ffb76f9f73155c0f89d5a2235fd3c
docker compose version
bash examples/node-red-mqtt-recovery/run.sh
```

The script builds the existing roomci runner and a Python/Paho adapter, starts
Mosquitto and Toxiproxy, retries a broker MQTT publish within ten bounded
attempts, waits for the adapter subscription, and checks Docker network membership:
Node-RED and broker share
no network, while the proxy shares one with each. The roomci fault check then
requires the Node-RED offline will after the proxy TCP cut.

For each case it records the run ID, image digest, contract, mapping, flow,
roomci JSON and JUnit, assertion evidence, and timestamped separate Node-RED,
broker, proxy, and adapter logs.
Inspect `reports/node-red-external-sut/<case>/recovery.json` for `verdict`,
`reason`, and timestamped `timeline` events; `recovery.xml` for JUnit;
`assertions.log` and `conditions.txt` for the case checks and inputs; and
`node-red.log`, `adapter.log`, `broker.log`, and `toxiproxy.log` for the
observed path. The script itself succeeds only when all five case-specific
results below hold; a negative case's `failed` verdict is expected evidence.

1. `recovery`: Node-RED reports `21`, receives an actual proxy cut, and the
   broker accepts desired `24` while it is offline. Node-RED's normal MQTT
   reconnect and retained subscription yield `24` within the 30 s deadline.
   The roomci verdict is `passed/latest_report_stable` after 1 s of stability.
2. `stale-retained-foreign`: the harness preloads a retained `24` on this run's
   reported topic and publishes a non-retained `24` on a different run topic.
   The Function node deliberately omits the latest report; after reconnect the
   harness publishes a non-retained old `21`. The adapter logs rejection of
   the retained report and broker ACK of the latest application desired,
   the roomci timeline records `stale_report`, and the
   verdict is `failed/missing_latest_report`. The foreign run topic is outside
   the adapter's exact subscription. The negative case must reach
   `initial_ready`, `fault_applied`, `fault_affected_sut`, and `fault_released`;
   an environment failure does not count as the negative result.
3. `rollback`: the Node-RED Function publishes `24`, then `21` 200 ms later.
   The bridge preserves revision 1 for this previously requested value; roomci
   observes the report within the 1 s stability window and returns
   `failed/rollback_observed`.
4. `unexpected-value`: the same Function publishes `24`, then an unrequested
   `25` 200 ms later. The bridge forwards actual value `25` with its
   adapter-assigned current revision 2; roomci records a timestamped
   `sut_reported` event and returns `failed/rollback_observed`. The harness
   checks the required fault and recovery milestones, report order and timing,
   exit code, JSON verdict, JUnit failure reason, and both SUT and bridge logs.
   No third-party publisher injects this deviation.
5. `late-deviation`: the Function publishes `25` 1500 ms after `24`, beyond
   the 1 s stability window. The adapter log still records the late value;
   the already finalized JSON/JUnit result remains
   `passed/latest_report_stable`.

Before increasing the contract's recovery deadline, the same ordinary
Node-RED image and 12 s deadline reached `initial_ready` and
`fault_affected_sut`, then failed with `missing_latest_report`. The local
Node-RED log showed its reconnect about 15 s after the cut. With a 30 s
deadline the same path reached `latest_reached`. This is a SUT timing fact,
not a change to roomci's deadline behavior. The committed contract uses 30 s
to include Node-RED's observed reconnect cadence.

## Integration cost and reusable boundary

On this PoC: roomci core changed **0 lines**. The Node-RED application is one
seven-line exported flow file; its Function body handles the target temperature
and negative-control deviations. The external bridge is 101 lines of Python; the
explicit mapping is 12 YAML lines. Compose, contract, and adapter Dockerfile
are 47, 14, and 4 lines respectively; the 189-line shell harness supplies
repeatable setup, fault controls, assertions, and artifact collection. These
counts describe the PoC files, not an estimated customer integration effort.

First connection required: choose the existing application's desired/report
topics and payload fields; reserve a fresh test run namespace; configure its
MQTT broker as `toxiproxy:1884`, clean session, QoS 1, and non-retained
birth/will; place it only on `sut_net`; start the bridge before Node-RED;
choose an initial and distinct latest value; measure its reconnect timing;
and set the recovery deadline accordingly. The reference-SUT documentation
identifies the JSON mapping and offline evidence boundary but does not select
a generic payload-to-revision rule, determine whether a third-party SUT can
vary topic prefixes, define a publisher-provenance control, or establish that
SUT's reconnect cadence. These are new intake decisions, not implicit roomci
defaults.

The shared future adapter candidates are topic/payload mapping, strict
desired/report value conversion, status evidence conversion, and a declared
correlation/provenance strategy. This PoC does not introduce an adapter
framework: its fixed two-value mapping and timing belong to this example.
See [Evaluator Intake Kit](EVALUATOR_INTAKE_KIT.md) for customer-owned topic,
schema, device, authentication, TLS, timing, safety, and report requirements.
No customer specification has been obtained for those fields.
The steps and file counts above describe a team-run reproduction, not a
first-time user's measured setup effort. Record that effort and any work
saved or added in the Intake Kit before claiming ease of adoption.
