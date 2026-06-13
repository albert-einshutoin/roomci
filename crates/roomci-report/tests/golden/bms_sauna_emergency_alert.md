# roomci Report — bms_sauna_emergency_alert

Result: `passed`

## Failed Assertions

None.

## Assertions

- [pass] `ops` — ops escalation matched expected state
- [pass] `ops` — ops escalation matched expected state

## Timeline

- `T` `contact_changed` `sauna_emergency_button`: contact state changed to on
- `T` `ops_ticket_opened` `sauna_emergency_button`: Ops ticket opened with status open
- `T` `ops_slack_notification_sent` `sauna_emergency_button`: Slack notification sent with runbook https://example.com/runbooks/sauna-emergency
- `T` `ops_phone_call_triggered` `sauna_emergency_button`: Phone escalation triggered
- `T` `ops_runbook_url_included` `sauna_emergency_button`: Runbook URL included: https://example.com/runbooks/sauna-emergency
- `T+20s` `ops_ticket_acknowledged` `sauna_emergency_button`: Ops ticket acknowledged by ops_member_01
- `T+20s` `ops_action`: ops action applied

## Suggested Recovery

None.

