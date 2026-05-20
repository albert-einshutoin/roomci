# Phase 11 Test Plan

## Quality Gates

- Contract templates validate without custom code changes.
- At least one company-style PoC can be created from templates and run through `roomci serve`.
- MQTT external client interoperability is tested with at least one standard client/library instead of only hand-written packets.
- A second external protocol surface is exercised through a real network endpoint.
- Modbus/BMS behavior is observable through reports generated from external interactions.
- Protocol support matrix is accurate and explicitly marks unsupported production features.
- Clean-checkout verification covers:
  - formatting
  - clippy
  - workspace tests
  - docs build
  - coverage floor
  - Docker image build
  - Compose black-box PoC
  - adapter-template validation
- Public docs include an evaluator checklist for hospitality and for a generic IoT/SmartHome company.
- Dual-track positioning explains why industry-wide scope strengthens, rather than weakens, hospitality relevance.
- Evaluation evidence pack includes separate scorecards for hospitality-focused hospitality, generic MQTT/IoT, building automation/BMS, and CI/platform engineering.
- Adoption maximization review scores Not A Hotel relevance, generic market relevance, protocol credibility, first-run experience, and PoC handoff readiness.
- Release-readiness check confirms README claims match actual files and commands.

## Done Means

Phase 11 is done when a company can evaluate `roomci` by supplying its own protocol contracts and running a repeatable local/CI PoC, with every unsupported protocol feature documented as an explicit boundary rather than an implicit surprise.

It is also done only when the product can credibly serve both paths:

- broad IoT/SmartHome/building-automation emulator evaluation
- hospitality-focused hospitality smart-home evaluation without private compatibility overclaiming
