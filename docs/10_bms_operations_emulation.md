# 10. BMS and Operations Emulation

## Why BMS belongs in roomci

A smart home does not end at device control. Operations teams need to know when something is wrong, who is responding, where to go, and what runbook to follow.

`roomci` should include BMS-like operations simulation because real field quality depends on the full chain:

```txt
Device alarm -> Notification -> Human awareness -> Assignment -> Runbook -> Resolution -> Recovery notification
```

## MVP features

- alert source model
- Slack-like notification
- phone-call escalation mock
- runbook URL
- ticket state
- assignee
- comments
- recovery notification
- Markdown report
- optional Grafana-like metrics endpoint

## Alert model

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

## Notification model

Slack-like message:

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

Phone-like escalation:

```json
{
  "target": "zoom_phone_mock",
  "message": "Sauna emergency button was pressed at AOSHIMA MASTERPIECE. Please check the site immediately."
}
```

## Ticket state model

```yaml
ticket:
  id: alert-20260101-001
  status: open
  assignee: null
  comments: []
```

States:

```txt
open -> acknowledged -> assigned -> investigating -> resolved
```

## Recovery notification

When a device returns to normal, `roomci` should emit a recovery event and attach it to the original alert thread.

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

## Time-series export

For Grafana-like analysis, export metrics as:

- JSON lines
- Prometheus text format
- Influx line protocol, future

Example:

```txt
roomci_contact_state{house="aoshima",device="sauna_emergency_button"} 1
roomci_alert_active{severity="critical",alert="sauna_emergency_button"} 1
```

## Demo: sauna emergency button

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

## Report example

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
