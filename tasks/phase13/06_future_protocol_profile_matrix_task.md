# Task 06 — Future Protocol Profile Matrix

## Goal

Track high-value standards without overclaiming implementation. BACnet, OPC UA, Zigbee, Thread, KNX, and Matter should be visible as future protocol profiles, not implied current support.

## Specification Sources

- BACnet / ANSI/ASHRAE Standard 135: https://www.ashrae.org/technical-resources/bookstore/bacnet
- BACnet 135-2024 notice: https://bacnet.org/news/ansi-ashrae-135-2024-now-published/
- OPC UA: https://opcfoundation.org/developer-tools/specifications-unified-architecture
- OPC UA Online Reference: https://reference.opcfoundation.org/
- Zigbee: https://csa-iot.org/all-solutions/zigbee/
- Zigbee Revision 23: https://csa-iot.org/wp-content/uploads/2023/04/05-3474-23-csg-zigbee-specification-compressed.pdf
- Thread: https://threadgroup.org/ThreadSpec
- KNX: https://support.knx.org/hc/en-us/articles/360000040999-KNX-Specifications
- Matter: https://csa-iot.org/all-solutions/matter/
- Matter developer overview: https://developers.home.google.com/matter/overview?hl=ja

## Implementation Scope

- Add profile rows for each future protocol in the conformance registry.
- For each protocol, record:
  - why it matters to IoT/SmartHome/building automation
  - likely first useful subset
  - expected adapter contract shape
  - external tools likely needed for black-box verification
  - why it is not implemented yet
- Identify which protocol should be next after MQTT and Modbus based on evaluator value and implementation tractability.

## Acceptance Criteria

- Future protocols are visible in docs without being marketed as implemented.
- Each profile has a concrete first-subset candidate or a reason to defer.
- Support matrix and README remain honest about unsupported protocols.
