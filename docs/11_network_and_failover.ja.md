# 11. ネットワークおよびフェイルオーバー

## なぜ network emulation matters

A local-first smart home depends on the local network. Internet failures should not break core guest controls, but they still affect VOD, workcation use, remote monitoring, staff communication, and cloud sync.

`roomci` should model both:

1. local network health for smart-home control
2. internet/WAN health for operations and cloud integration

## Network model

```yaml
network:
  segments:
    guest:
      vlan: 10
    owner:
      vlan: 20
    staff:
      vlan: 30
    device_control:
      vlan: 40
    camera:
      vlan: 50
  policies:
    - from: guest
      to: device_control
      allow: false
    - from: staff
      to: device_control
      allow: true
```

## VLAN isolation assertion

```yaml
assert:
  network:
    from: guest
    to: device_control
    reachable: false
```

## WAN model

```yaml
wan:
  primary:
    id: mcat_cable
    type: cable
    status: up
  backup:
    id: backup_satellite
    type: satellite
    status: standby
  failover:
    enabled: true
    expected_within: 30s
```

## Fault types

```txt
wan_down
isp_degraded
packet_loss
latency
dns_failure
vlan_leak
router_reboot
l3_switch_failure
backup_wan_degraded
power_domain_outage
```

Executable Phase 17 fault profiles use the existing `fault` step and emit
timeline/final-state BMS evidence:

```yaml
steps:
  - at: T
    fault:
      target: network.segment.guest_vlan
      type: isolated
  - at: T+1s
    fault:
      target: firewall.policy.edge_to_bms
      type: drift
  - at: T+2s
    fault:
      target: mqtt.local
      type: unreachable
assertions:
  - at: T+3s
    assert:
      network_control_panel_faults: observed
```

## Demo: backup WAN failover

```yaml
scenario:
  name: starlink_failover

faults:
  - at: T+10s
    target: wan.primary
    type: down

steps:
  - at: T+20s
    assert:
      target: wan.backup
      condition: active
  - at: T+30s
    assert:
      target: mqtt.local
      condition: available
  - at: T+40s
    assert:
      ops:
        slack_notification_sent: true
```

## Report example

```txt
PASS starlink_failover

Primary WAN:
- mcat_cable down

Backup WAN:
- backup WAN active within 20s

Guest impact:
- local smart-home controls unaffected
- internet service degraded but available

Operations:
- Slack failover notification sent
```

## Local-only mode

When WAN is down:

- local MQTT should work
- edge server should work
- DALI/Modbus/contact I/O should work
- cloud sync may be delayed
- ops notification may use local fallback or queue

## Future: network-vendor profiles

Future modules can emulate vendor-style health data:

```yaml
network_profiles:
  wifi_controller:
    aps:
      - id: ap_living
        packet_loss: 0.01
        clients: 12
  edge_router:
    routers:
      - id: rtx_main
        cpu: 20
        vpn_status: down
```
