# Developer Workflow

This guide is the practical path for evaluating and extending `roomci` as a QA
contract emulator.

## First Run

```bash
cargo run -p roomci-cli -- validate examples/*.yaml
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml
```

For the full local gate:

```bash
make verify
```

## Adapter Contracts

Validate the template and shipped examples:

```bash
cargo run -p roomci-cli -- adapter validate \
  adapter-contracts/templates/company_adapter_contract.yaml \
  adapter-contracts/examples/*.yaml
```

Use adapter contracts for customer-specific MQTT topics, payload schemas,
register maps, alert severities, auth assumptions, device identity, and
acceptance criteria.

## Python Automation

Use the Python reference client when an evaluator wants to drive `roomci serve`
from normal Python test code:

```bash
make python-sdk-smoke
```

See [`PYTHON_SDK.md`](PYTHON_SDK.md) for the client surface and manual command.

## Scenario Debugging

Use `roomci debug` when a scenario fails and the author needs deterministic
execution evidence:

```bash
cargo run -p roomci-cli -- debug examples/dali_scene_partial_failure.yaml \
  --debug-json reports/dali.debug.json \
  --debug-md reports/dali.debug.md
```

The JSON artifact uses `roomci.debug.v1` and includes execution order, resolved
virtual-time labels, cumulative state diffs, assertion results, failure causes,
and suggested checks.

## Editor Authoring

VSCode assets are intentionally deferred from Phase 20. Until that task is
implemented, use:

```bash
cargo run -p roomci-cli -- validate examples/<scenario>.yaml
cargo run -p roomci-cli -- debug examples/<scenario>.yaml
```

## CI Artifact Review

Use the S Tier evidence path for report, timeline, and observability artifacts:

```bash
make s-tier-evidence-smoke
```

Use the A Tier developer-experience gate for Python and debugger evidence:

```bash
make developer-experience-smoke
```

## Boundaries

These workflows are local evaluator workflows. They do not claim a production
SDK, PyPI package, marketplace extension, hosted observability backend,
production orchestration layer, or protocol certification.
