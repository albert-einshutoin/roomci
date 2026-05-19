# Interview Demo

This is the interview path for explaining `roomci` as a NOT A HOTEL-facing portfolio project.

## One-line framing

`roomci` reproduces smart-home field failures before guests experience them.

## Three-minute pitch

NOT A HOTEL's public smart-home materials suggest that the product quality problem is not just device control. The harder problem is protecting the stay experience across local control, edge servers, building-automation protocols, network failover, field commissioning, and operations response.

I built `roomci` as a Docker-friendly external QA emulator. It does not claim to reproduce NOT A HOTEL's private implementation. It models the behavior that matters for QA: local-first control during cloud outage, edge failover, Modbus register checks, DALI-like scene failures, BMS escalation, WAN failover, comfort automation, access-permission drift, and commissioning checks.

The core idea is to turn field failures and commissioning knowledge into repeatable CI scenarios.

## Ten-minute demo path

### 1. Local-first cloud outage

```bash
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --verbose
```

What to say:

- Cloud MQTT is intentionally taken offline.
- The local iPad-style controller still publishes through local MQTT.
- The edge route and retained state preserve guest-facing control.

### 2. Edge failover

```bash
cargo run -p roomci-cli -- run examples/edge_server_failover.yaml --verbose
```

What to say:

- The primary edge loses power.
- The standby edge is promoted inside the expected failover window.
- This models local reliability, not cloud availability.

### 3. BMS emergency alert

```bash
cargo run -p roomci-cli -- run examples/bms_sauna_emergency_alert.yaml --verbose
```

What to say:

- A critical contact input triggers a ticket, Slack notification, phone escalation, and runbook URL.
- This treats operations response as part of product quality.

### 4. Commissioning checklist

```bash
cargo run -p roomci-cli -- run examples/commissioning_checklist.yaml
```

What to say:

- Room and device declarations become executable field checks.
- The target is repeatability as sites scale.

### 5. Access-permission drift

```bash
cargo run -p roomci-cli -- run examples/access_permission_drift.yaml
```

What to say:

- The scenario compares identity-group members with access-system users.
- It detects a stale user who still has access permission.

### 6. Intentional failure report

```bash
cargo run -p roomci-cli -- run examples/dali_scene_partial_failure.yaml \
  --report-md reports/dali_scene_partial_failure.md
```

Expected result: exit code `1`.

What to say:

- This is intentionally failing.
- A missing fixture command produces a guest-impact report.
- The point is not to hide failures; it is to make them reproducible and visible in CI.

## Fast path

```bash
make demo
```

## Full verification

```bash
make verify
```

## Questions to expect

### Why Rust?

Rust is used here as a deterministic external QA engine and Docker binary. It is not a proposal to replace Go, Lua, Swift, TypeScript, or the production edge stack.

### Is this a real protocol implementation?

No. It is a behavioral QA emulator. Full KNX, DALI, BACnet, SIP, or Modbus compatibility is intentionally out of scope for the MVP.

### Does this reproduce NOT A HOTEL's internal system?

No. It is based on public materials and external product reasoning. The goal is to demonstrate understanding of likely failure domains and build a testable tool around them.

### What would make it production-grade?

Long-running HTTP/MQTT adapters, richer scenario typing, real protocol adapters behind the behavioral model, and integration with actual mobile/backend E2E flows.
