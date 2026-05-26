# Phase 25 Test: Public Surface Evidence Freshness

Phase 25 is done when:

- `rg "100 tests|100 テスト" README.md README.ja.md` returns no matches;
- `rg "INTERVIEW_DEMO" README.md README.ja.md docs/INTERVIEW_DEMO.md` shows the
  README links and the committed target document;
- `test -f docs/INTERVIEW_DEMO.md` passes;
- `cargo test --workspace --all-targets` passes and confirms the README test
  count;
- `cargo run -p roomci-cli -- validate examples/local_first_cloud_outage.yaml examples/generic_mqtt_retained_state.yaml examples/dali_scene_partial_failure.yaml examples/matter_gateway_profile.yaml`
  passes;
- `cargo run -p roomci-cli -- serve --config examples/local_first_cloud_outage.yaml --check`
  passes.

Full `make verify` remains the release checklist gate and can be run before a
formal release build.
