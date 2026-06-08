# 11. ネットワークおよびフェイルオーバー

## ネットワークエミュレーションが重要な理由

ローカルファーストのスマートホームはローカルネットワークに依存します。インターネット障害はコアとなるゲスト制御を壊してはいけませんが、VOD、ワーケーション利用、リモート監視、スタッフ通信、クラウド同期には影響します。

`roomci` は次の両方をモデル化する必要があります。

1. スマートホーム制御用のローカルネットワーク健全性
2. 運用およびクラウド連携用のインターネット／WAN 健全性

## ネットワークモデル

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

## VLAN 分離アサーション

```yaml
assert:
  network:
    from: guest
    to: device_control
    reachable: false
```

## WAN モデル

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

## 障害タイプ

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

実行可能な Phase 17 障害プロファイルは既存の `fault` ステップを使用し、タイムライン／最終状態の BMS エビデンスを出力します。

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

## デモ: バックアップ WAN フェイルオーバー

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

## レポート例

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

## ローカルのみモード

WAN がダウンしているとき:

- ローカル MQTT は動作する必要がある
- エッジサーバーは動作する必要がある
- DALI/Modbus/接点 I/O は動作する必要がある
- クラウド同期は遅延する可能性がある
- 運用通知はローカルフォールバックまたはキューイングを使用する可能性がある

## 将来: ネットワークベンダープロファイル

将来のモジュールはベンダー風の健全性データをエミュレートできます。

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
