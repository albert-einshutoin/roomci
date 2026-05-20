# Core QA Journey

This journey is the product center of gravity for `roomci`.

It is designed to be useful to hospitality smart-home teams while remaining generic enough for smart-home, IoT, and building-automation companies.

## Journey Shape

```txt
1. Local MQTT command/state contract
2. Edge controller routes command under local-first assumptions
3. Device protocol behavior changes state
4. Network or control-panel fault is injected
5. BMS/ops escalation records safety or guest-impact evidence
6. JSON, Markdown, and JUnit reports capture the full timeline
```

## Current Runnable Pack

The current core journey is composed from existing scenarios:

| Step | Scenario | Evidence |
|---|---|---|
| Local MQTT and retained state | `examples/local_first_cloud_outage.yaml` | Local command still works when cloud MQTT is unavailable |
| Edge failover | `examples/edge_server_failover.yaml` | Secondary edge takes over and local control remains available |
| Device protocol behavior | `examples/modbus_floor_heating.yaml` | Modbus register map and 0.1 degree scaling are verified |
| BMS/ops escalation | `examples/bms_sauna_emergency_alert.yaml` | Contact alert triggers Slack-like, phone-like, ticket, and runbook evidence |
| Network failover | `examples/starlink_failover.yaml` | Backup WAN path preserves operational availability |
| Comfort automation | `examples/comfort_auto_mode.yaml` | Discomfort-index target and override behavior are checked |
| Access drift | `examples/access_permission_drift.yaml` | Identity/access mismatch is detected without real lock control |
| Field QA | `examples/commissioning_checklist.yaml` | Commissioning evidence is generated as reusable report output |

Run it with:

```bash
make poc-core-qa
```

The target writes report artifacts under `reports/poc_core_*.{json,md,xml}`.

## Why This Is Not a Full-stack Emulator

The journey intentionally avoids replacing:

- production mobile/web apps,
- cloud backends,
- vendor network systems,
- real BMS products,
- physical access-control systems,
- electrical safety validation,
- CAD/construction tooling.

Those systems appear as contracts, assumptions, mocked evidence, or future profiles. The emulator focuses on the QA behaviors that can be made deterministic in local and CI environments.
