# 09. Security and License Notes

## Documentation Usage

Do not bundle proprietary vendor documentation into the repository.

Instead:
- Link to official docs.
- Implement behavior from public API contracts.
- Keep source references in `docs/00_source_map.md`.
- Avoid copying large verbatim text from vendor docs.

## Emulation Scope

Use compatibility language carefully.

Prefer:
- `AWS IoT Shadow-like adapter`
- `Azure Device Twin-like adapter`
- `Home Assistant MQTT Discovery-like adapter`
- `Matter-like profile model`

Avoid claiming:
- Certified Matter compatibility.
- Full AWS IoT Core compatibility.
- Full Azure IoT Hub compatibility.
- Full Home Assistant replacement.

## Security Defaults

Local testing defaults:
- No public bind unless explicitly requested.
- Bind to `127.0.0.1` by default in CLI mode.
- Docker examples can bind to `0.0.0.0` only for container networking.

Authentication:
- v0.1 can be unauthenticated for local CI.
- Add optional static token for shared development environments.

TLS:
- Not required for v0.1 local CI.
- Optional TLS later for integration environments.

## Dangerous Scenarios

Do not include examples that teach bypassing real smart locks or physical security systems.

Allowed:
- Emulated lock command failure.
- Fallback access event simulation.
- Staff notification simulation.

Avoid:
- Real lock exploit instructions.
- Vendor credential extraction.
- Real device attack payloads.

## Names and Trademarks

Do not use vendor names in a way that implies endorsement.

Use adapter names like:
- `shadow-like`
- `twin-like`
- `ha-discovery-like`
- `matter-profile`

## NOT A HOTEL Positioning Safety

Do not claim to reproduce NOT A HOTEL's internal systems.

Recommended language:

> This project is a public-information-based smart-room CI emulator inspired by hospitality IoT failure scenarios. It does not implement or reproduce any private NOT A HOTEL system.
