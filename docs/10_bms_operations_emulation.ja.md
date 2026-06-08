# 10. BMS および運用エミュレーション

## roomci に BMS が含まれる理由

スマートホームはデバイス制御で完結しません。運用チームは、何か異常が起きたとき、誰が対応するか、どこへ行くか、どの runbook に従うかを把握する必要があります。

`roomci` には BMS 的な運用シミュレーションを含めるべきです。現場品質は次の一連の流れ全体に依存するからです。

```txt
Device alarm -> Notification -> Human awareness -> Assignment -> Runbook -> Resolution -> Recovery notification
```

## MVP 機能

- アラートソースモデル
- Slack 風通知
- 電話エスカレーションのモック
- runbook URL
- チケット状態
- 担当者（assignee）
- コメント
- 復旧通知
- Markdown レポート
- オプションの Grafana 風メトリクスエンドポイント

## アラートモデル

```yaml
alerts:
  - id: sauna_emergency_button
    source: contact.sauna_emergency_button
    severity: critical
    location: sauna_room
    notify:
      slack: true
      phone: true
    runbook_url: https://example.com/runbooks/sauna-emergency
```

## アダプターコントラクトの強化

企業固有の BMS webhook 前提条件はアダプターコントラクトに属します。コアランタイムはベンダー中立のままですが、PoC の配線ミスを早期に検出できる十分な形状を検証します。

```yaml
bms:
  alerts:
    - id: sauna_emergency
      source: contact.sauna_emergency_button
      severity: critical
      schema_version: bms.alert.v1
      content_type: application/json
      severity_enum: [info, warning, critical, emergency]
      hmac:
        header: X-RoomCI-Signature
        algorithm: hmac-sha256
        secret_ref: env:ROOMCI_BMS_WEBHOOK_SECRET
      replay_window_seconds: 300
```

`roomci serve` は、`POST /external/bms/contact` 上でサポート外の外部 BMS severity および重複する `replay_id` も拒否します。HMAC メタデータはアダプターコントラクトで宣言されます。ローカル PoC ランタイムは本番シークレットを検証しません。

## 通知モデル

Slack 風メッセージ:

```json
{
  "channel": "#ops-alerts",
  "title": "Sauna emergency button pressed",
  "severity": "critical",
  "location": "AOSHIMA MASTERPIECE / Sauna",
  "runbook_url": "https://example.com/runbooks/sauna-emergency",
  "actions": ["acknowledge", "assign", "resolve"]
}
```

電話風エスカレーション:

```json
{
  "target": "zoom_phone_mock",
  "message": "Sauna emergency button was pressed at AOSHIMA MASTERPIECE. Please check the site immediately."
}
```

## チケット状態モデル

```yaml
ticket:
  id: alert-20260101-001
  status: open
  assignee: null
  comments: []
```

状態:

```txt
open -> acknowledged -> assigned -> investigating -> resolved
```

## 復旧通知

デバイスが正常に戻ったとき、`roomci` は復旧イベントを発行し、元のアラートスレッドに添付する必要があります。

```yaml
steps:
  - at: T
    contact:
      id: sauna_emergency_button
      state: on
  - at: T+60s
    contact:
      id: sauna_emergency_button
      state: off
  - at: T+61s
    assert:
      ops:
        recovery_notification_sent: true
```

## 時系列エクスポート

Grafana 風の分析のために、メトリクスを次の形式でエクスポートします。

- JSON lines
- Prometheus text format
- Influx line protocol（将来）

例:

```txt
roomci_contact_state{house="aoshima",device="sauna_emergency_button"} 1
roomci_alert_active{severity="critical",alert="sauna_emergency_button"} 1
```

## デモ: サウナ非常ボタン

```yaml
scenario:
  name: bms_sauna_emergency_alert

steps:
  - at: T
    contact:
      id: sauna_emergency_button
      state: on
  - at: T+1s
    assert:
      ops:
        slack_notification_sent: true
        phone_call_triggered: true
        runbook_url_included: true
  - at: T+20s
    ops:
      action: acknowledge
      assignee: ops_member_01
  - at: T+30s
    assert:
      ops:
        ticket_status: acknowledged
```

## レポート例

```txt
PASS bms_sauna_emergency_alert

Alert:
- sauna_emergency_button = ON
- severity = critical

Notifications:
- Slack notification sent
- Phone escalation triggered
- Runbook URL included

Ticket:
- status changed to acknowledged
- assignee = ops_member_01
```
