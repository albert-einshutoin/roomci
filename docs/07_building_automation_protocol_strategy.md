# 07. Building Automation Protocol Strategy

## Strategy

`roomci` should not attempt to fully implement every protocol. It should emulate protocol behavior at the level needed for local QA, commissioning, and failure-mode testing.

## Protocol priority

| Protocol / system | Priority | Rationale |
|---|---:|---|
| MQTT | v0.1 | Core local/cloud communication model |
| Modbus TCP | v0.1 | Register-map validation and equipment control |
| DALI-like | v0.1 | Lighting scene quality and partial failure testing |
| Contact I/O | v0.1 | Alarms, relays, safety, intercom/gate control |
| KNX-like | v0.1.5 | Legacy/standard bus model and throughput simulation |
| BACnet-like | v0.2 | HVAC/BMS-style object model |
| SIP / DTMF | v0.3 | Intercom and staff call workflows |
| ONVIF-like | v0.3 | Door camera / NVR event model |
| Matter / CSA Aliro | Future | Watch-mode and future standardization |
| Home Assistant discovery | Appendix | General IoT integration, lower priority |

## Mocking level definitions

### Level 0: Conceptual model

Only device type and state are modeled.

### Level 1: Command/state model

Commands and state transitions are modeled, but protocol frames are not.

### Level 2: Protocol-like interface

The emulator exposes an interface resembling real protocol concepts.

Examples:

- Modbus holding registers
- DALI fixture addresses
- KNX group addresses

### Level 3: Wire protocol subset

The emulator speaks a subset of the real wire protocol.

Use sparingly.

### Level 4: Full compatibility

Not a goal for v0.x.

## Recommended MVP levels

| Area | Level |
|---|---:|
| MQTT | Level 3-ish, via broker behavior |
| Modbus TCP | Level 2 or Level 3 subset |
| DALI | Level 2 |
| Contact I/O | Level 1 |
| KNX | Level 2 |
| BMS alerts | Level 1 |
| SIP/DTMF | Level 1 initially |

## Why avoid full compatibility early?

Full compatibility requires deep protocol edge cases, certification concerns, vendor-specific differences, and extensive testing. The goal of `roomci` is to demonstrate smart-home QA insight, not to compete with industrial protocol stacks.

## Adapter interface

A protocol adapter should implement:

```rust
trait DeviceAdapter {
    fn apply_command(&mut self, command: Command) -> Result<DeviceEvent, DeviceError>;
    fn read_state(&self) -> DeviceState;
    fn inject_fault(&mut self, fault: Fault) -> Result<(), FaultError>;
    fn recover(&mut self, target: RecoveryTarget) -> Result<(), RecoveryError>;
}
```

## Field-debug output

Every adapter should provide field-oriented hints.

Example:

```txt
DALI fixture failed to reach expected level.
Possible field causes:
- wrong fixture address
- fixture not assigned to group
- gateway timeout
- wiring issue
```
