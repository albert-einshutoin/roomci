# roomci Report — local_first_cloud_outage

Result: `passed`

## Failed Assertions

None.

## Assertions

- [pass] `mqtt_retained:house/minakami/room/living/device/living_light/state` — retained MQTT state matched
- [pass] `guest_experience` — guest experience remained unaffected by upstream outage

## Timeline

- `T+10s` `fault_activated` `mqtt.cloud`: offline fault activated
- `T+15s` `mqtt_publish` `ipad_controller`: published house/minakami/room/living/device/living_light/command
- `T+15s` `edge_command_routed` `edge_primary`: mqtt_command_route routed command from ipad_controller to living_light
- `T+15s` `mqtt_retained_state_updated` `living_light`: retained state updated at house/minakami/room/living/device/living_light/state after 1 delivery

## Suggested Recovery

None.

