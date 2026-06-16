# roomci Report — welcome_scene_partial_failure

Result: `failed`

## Failed Assertions

- `scene_consistency:welcome`: DALI-like scene consistency violation: D411S10 expected level 60, actual 0
  Guest impact: Lighting scene did not match intended guest ambience.

## Assertions

- [fail] `scene_consistency:welcome` — DALI-like scene consistency violation: D411S10 expected level 60, actual 0

## Timeline

- `T` `fault_activated` `dali.fixture.D411S10`: command_drop fault activated
- `T` `command_received` `scene.welcome`: activate command received
- `T` `dali_command_dropped` `D411S10`: fixture command dropped
- `T` `dali_level_changed` `D411S11`: fixture level changed to 40
- `T` `scene_activation_requested` `welcome`: scene activation requested

## Suggested Recovery

- Lighting scene did not match intended guest ambience.

