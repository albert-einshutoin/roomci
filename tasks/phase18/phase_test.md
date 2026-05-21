# Phase 18 Test Plan

Phase 18 is complete when release-candidate evidence can be regenerated without relying on manual trust.

- `make verify`
- `make adapter-samples-smoke` or equivalent sample verification target
- Protocol/domain evidence checker covers Phase 17 examples or a dedicated Phase 17 evidence file
- Public scorecards mention Phase 17 coverage and remaining non-goals honestly
- Sample verification should be skipped or clearly marked only when required toolchains are unavailable
