# Bring Your Own SUT: Node-RED external MQTT validation

This is an external-validity PoC, not a supported-product claim. Node-RED 4.1.15
(`nodered/node-red@sha256:7aa04e1c7be16aec5b4b4d6e64ae863c4720b0ab18e2c1f46905f7b0c71e3a19`)
runs an ordinary MQTT In → Function → MQTT Out thermostat flow. No Node-RED
runtime or roomci core code is changed. Run `bash examples/node-red-mqtt-recovery/run.sh`
with Docker Compose. The separate `node-red-external-sut` CI job saves two
cases under `reports/node-red-external-sut/`; the reference SUT's ten cases
remain in `external-mqtt-recovery`.

## Mapping and boundaries

| Surface | PoC mapping |
|---|---|
| Device identity | `thermostat-1` in both topic namespaces; no identity field in the Node-RED payload |
| roomci topics | `roomci/<run-id>/thermostat-1/{desired,reported,status}` |
| Node-RED topics | `byos/<run-id>/thermostat-1/{desired,reported,status}` from container environment variables |
| Desired payload | roomci revision 1/2 values `"21"`/`"24"` → retained `{"targetTemperature":21}`/`{"targetTemperature":24}` |
| Reported payload | non-retained `{"temperature":21}`/`{"temperature":24}` → roomci report with the corresponding value and revision |
| Status and disconnect | Node-RED broker config publishes non-retained `online` birth and `offline` last will; adapter maps these to roomci status JSON |
| Readiness | adapter subscribes first; broker MQTT publish succeeds; roomci independently waits for observer SUBACKs and the initial report within 15 s |
| Recovery and stability | 30 s monotonic recovery deadline; 1 s stability window |

`flows.json` uses only built-in MQTT and Function nodes. The function reads
`targetTemperature` and emits `temperature`; it does not read or emit run ID,
device ID, revision, or roomci's `value`. The `NR_OMIT_LATEST` switch is used
only by the negative control to simulate an application that stops reporting
the new value. `adapter.py` is an external broker-side process, not code added
to Node-RED or roomci. It verifies the roomci desired identity and the two
contract values, forwards retained desired state, and converts observed state
and birth/will messages. Its MQTT connection remains on `broker_net`; Node-RED
has only `sut_net` and reaches the broker through `toxiproxy:1884`.

## Correlation decision

The current `external-mqtt` command fixes its topic and report JSON schema.
A bare Node-RED `{"temperature":24}` report cannot satisfy that schema; merely
changing YAML cannot map its fields. The first solution therefore uses an
external bridge and leaves roomci core unchanged. The bridge creates an
adapter-owned revision **only** after seeing roomci's desired on a fresh run
topic, and accepts a non-retained matching Node-RED report only while a
non-retained online birth has been observed. The initial `21` response is
observed as a baseline before the fault. The new `24` value has not been used
earlier in the run. After the offline will, the retained latest desired is
delivered to Node-RED's clean MQTT session on reconnect; a matching report
from that connection is required for success. The runner still requires the
broker's desired PUBACK, and the harness checks the bridge's separate QoS 1
PUBACK for **both** application desired publishes. It also requires the
SUT-only cut and offline will, path restoration,
latest report before deadline, and a stable window.

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

The script builds the existing roomci runner and a Python/Paho adapter, starts
Mosquitto and Toxiproxy, retries a broker MQTT publish within ten bounded
attempts, waits for the adapter subscription, and checks Docker network membership:
Node-RED and broker share
no network, while the proxy shares one with each. The roomci fault check then
requires the Node-RED offline will after the proxy TCP cut.

For each case it records the run ID, image digest, contract, mapping, flow,
roomci JSON and JUnit, and separate Node-RED, broker, proxy, and adapter logs.

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

Before increasing the contract's recovery deadline, the same ordinary
Node-RED image and 12 s deadline reached `initial_ready` and
`fault_affected_sut`, then failed with `missing_latest_report`. The local
Node-RED log showed its reconnect about 15 s after the cut. With a 30 s
deadline the same path reached `latest_reached`. This is a SUT timing fact,
not a change to roomci's deadline behavior. The committed contract uses 30 s
to include Node-RED's observed reconnect cadence.

## Integration cost and reusable boundary

On this PoC: roomci core changed **0 lines**. The Node-RED application is one
seven-line exported flow file with a three-statement Function body plus one
negative-control guard. The external bridge is 99 lines of Python; the
explicit mapping is 12 YAML lines. Compose, contract, and adapter Dockerfile
are 45, 14, and 4 lines respectively; the 146-line shell harness supplies
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
