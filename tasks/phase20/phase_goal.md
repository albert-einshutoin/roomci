# Phase 20 Goal: A-Tier Developer Experience Completion

Phase 20 completes the A Tier surface.

After Phase 19, `roomci` should have strong CI and observability evidence. Phase
20 makes that evidence easy to consume from normal developer workflows:
Python automation, scenario debugging, and editor-assisted authoring.

## A Tier Completion Definition

`roomci` reaches A Tier developer-readiness when:

- Python users can drive `roomci serve` without writing raw socket or curl code
- scenario authors can inspect execution order, resolved virtual time, state
  diffs, assertion inputs, and failure causes
- VSCode users get schema validation, snippets, tasks, and command examples
- SDK/debugger/editor assets are verified in CI or local smoke targets
- docs explain when to use SDK/debugger/editor support versus raw CLI/HTTP

## Explicit Non-Goals

- Replacing the Rust CLI.
- Publishing to PyPI or the VSCode Marketplace in this phase.
- Building a full GUI debugger.
- Building production client SDKs with auth/TLS policy.
- Adding new protocol runtime breadth.
