# Adoption Maximization Review

## Review Result

`reviewed`

`roomci` is credible as a strong OSS/portfolio MVP and a pre-adoption PoC foundation, but it should not yet be positioned as a ready default choice for real company evaluations.

Current adoption score:

```txt
Overall evaluator readiness: 67 / 100
hospitality-focused relevance: 78 / 100
Generic IoT/SmartHome relevance: 70 / 100
External PoC readiness: 64 / 100
Runtime confidence: 52 / 100
Evidence quality: 68 / 100
```

The product is directionally strong because it combines a generic MQTT/edge/device QA contract emulator with a high-signal hospitality smart-home domain pack. The remaining blockers are not about the concept. They are about evaluator trust, adapter readiness, protocol depth, and serve-runtime hardening.

## Evidence Reviewed

- `README.md`
- `docs/PRODUCT_POSITIONING.md`
- `docs/DOMAIN_PACKS.md`
- `docs/PRE_ADOPTION_POC_CHECKLIST.md`
- `docs/MQTT_SERVE_SUBSET.md`
- `tasks/phase11/*`
- `tasks/phase12/*`
- `examples/*.yaml`
- `reports/external_controller_latest.md`
- `reports/dali_scene_partial_failure.md`
- `.github/workflows/smart-home-ci.yml`

Commands/evidence:

- `make compose-poc` passed.
- README badge endpoint `https://github.com/albert-einshutoin/roomci/actions/workflows/smart-home-ci.yml/badge.svg` returns `HTTP/2 404`.
- `wc -l crates/roomci-cli/src/main.rs` reports `1093`, confirming the Phase 12 crate-extraction task is justified.
- `rg 'expect\("serve state mutex poisoned"\)' crates/roomci-cli/src/main.rs` confirms serve-state poison handling is still panic-based.
- `/health` currently returns `"status":"ok"` regardless of run state.

## Scorecard

| Dimension | Score | Assessment |
|---|---:|---|
| Category clarity | 76 | The product reads as a contract-first local/CI emulator, not only a Not A Hotel demo. |
| hospitality relevance | 78 | Hospitality/local-first scenarios map well to local MQTT, edge, Modbus-like equipment, BMS/ops, WAN failover, and commissioning concerns. |
| Generic IoT/SmartHome relevance | 70 | Generic MQTT examples exist, but adapter contracts and protocol support matrix are still missing. |
| Protocol credibility | 58 | MQTT serve mode is useful but minimal; Modbus/BMS are still scenario models, not externally drivable endpoints. |
| First-run experience | 72 | README, Make targets, Docker, Compose, and reports are clear; public badge 404 and missing adapter guide reduce trust. |
| Runtime confidence | 52 | Phase 10 surface works, but concurrency, timeout, mutex poison, `/run` lock scope, `/health`, and MQTT CONNECT validation need Phase 12. |
| PoC handoff readiness | 62 | The checklist exists, but there is no complete evaluator pack or company-fillable adapter contract kit yet. |
| Evidence quality | 68 | Reports are useful and `make compose-poc` passes, but there is no consolidated evidence pack or scorecard for evaluators. |

## Findings

### P0 — Public trust is weakened by badge/repo mismatch

The workflow file exists locally, but the README GitHub Actions badge URL returns 404. For an external evaluator, this is a visible credibility hit. Phase 12 Task 06 should be treated as adoption-critical, not polish.

### P0 — Runtime hardening blocks serious external PoC confidence

The serve runtime is functionally useful, but it is not yet robust enough to leave running during a multi-step external evaluation. The biggest risks are single-process HTTP behavior, no read timeout, panic-based mutex poison handling, `/run` lock scope, fixed `/health`, and loose MQTT CONNECT validation. Phase 12 correctly captures these.

### P1 — Not A Hotel relevance is strong but needs a dedicated evaluator path

The hospitality domain pack demonstrates useful understanding without claiming private compatibility. To maximize evaluation, the repo still needs a hospitality-focused guide that maps unknown private specs to generic contract inputs and explains exactly what they would need to provide.

### P1 — Generic industry positioning needs adapter proof

The README and positioning docs say the product is generic, and the generic MQTT examples support that claim. But the claim becomes much stronger only after the adapter contract kit, protocol support matrix, and customer PoC packs exist.

### P1 — External endpoint depth is still too narrow

Only HTTP and minimal MQTT are externally drivable. Modbus and BMS are still valuable scenario models, but a company evaluating emulator fit will expect at least one second externally driven protocol surface.

### P2 — Evidence is present but scattered

Reports, examples, CI workflow, Compose PoC, and docs exist, but an evaluator has to assemble the story manually. Phase 11 Task 10 should consolidate this into an evidence pack with separate scorecards.

## Adoption Decision

Do not mark Phase 11 complete yet.

The product can credibly ask for feedback as:

```txt
an early integration-ready PoC foundation for IoT, SmartHome, and hospitality smart-home QA
```

It should not yet claim:

```txt
the default emulator choice for real company adoption
```

That stronger claim becomes defensible after Phase 11 Tasks 01-10 and Phase 12 Tasks 01-06 close the evaluator-facing gaps.

## Top Blockers Before Asking for Serious Company Evaluation

1. Fix or remove broken public badges and confirm public CI visibility.
2. Add protocol support matrix.
3. Add adapter contract kit.
4. Add hospitality evaluator guide.
5. Add evidence pack and evaluator scorecards.
6. Harden `roomci serve` runtime through Phase 12.
7. Add one non-MQTT externally drivable protocol endpoint.

## Recommended Execution Order

1. Phase 12 Task 06 or equivalent badge/release truthfulness fix.
2. Phase 11 Task 02 protocol support matrix.
3. Phase 11 Task 08 hospitality evaluation path.
4. Phase 11 Task 09 dual-track positioning.
5. Phase 11 Task 03 adapter contract kit.
6. Phase 12 Task 01-05 runtime hardening.
7. Phase 11 Task 04 external protocol depth.
8. Phase 11 Task 05 and Task 10 customer PoC packs and evidence pack.

## Task 11 Status

Task 11 is complete as an initial adoption review. It intentionally keeps Phase 11 open because the review found concrete blockers that must be closed before the product can credibly ask for a real company evaluation.
