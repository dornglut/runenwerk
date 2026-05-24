---
title: WR-068 Renderer Lighting Pipeline Cache And Last Good Fallback Closeout
description: Closeout evidence for renderer pipeline cache diagnostics, shader fallback inspection, and prior-valid shader failure evidence.
status: completed
owner: engine
layer: engine-runtime / renderer lighting pipeline cache
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-068-renderer-lighting-pipeline-cache-and-last-good-fallback/plan.md
---

# WR-068 Renderer Lighting Pipeline Cache And Last Good Fallback Closeout

## Result

`WR-068` is complete at `bounded_contract` quality. Renderer inspection now
has a fail-closed pipeline/fallback report that aggregates pass provenance,
pipeline cache statistics, shader reload poll state, shader failure events, and
prior-valid shader revision evidence.

The implementation stays inside renderer-owned execution diagnostics. It does
not move material graph truth, scene truth, asset truth, product freshness,
shader source authority, rebuild policy, or fallback legality into the
renderer.

## Changed Modules

- `engine/src/plugins/render/inspect/pipeline_fallback.rs`: new
  `inspect_render_pipeline_fallback(...)` API, inspection request/report DTOs,
  count summary, prior-valid shader failure evidence, pipeline pass evidence,
  severity-coded diagnostics, and fail-closed checks for missing pass
  provenance, missing pipeline cache stats, missing pipeline stats keys,
  forbidden material shader fallback, missing generated shader revisions,
  missing material specialization fragments, and shader failures without
  prior-valid revision evidence.
- `engine/src/plugins/render/inspect/mod.rs`: exports the pipeline/fallback
  inspection API from the renderer inspection surface.
- `engine/src/plugins/render/pipelines/cache.rs`,
  `engine/src/plugins/render/renderer/pipeline_cache.rs`, and
  `engine/src/plugins/render/runtime/frame_submit.rs`: keep pipeline cache
  state stats-only while carrying hit, miss, and failure count diagnostics into
  ECS inspection resources.
- `engine/src/plugins/render/mod.rs`: re-exports shader registry event types
  alongside the existing shader registry public entry points.
- `engine/tests/render_pipeline_fallback.rs`: focused tests for ready
  pipeline/fallback evidence, forbidden material shader fallback, missing
  pipeline stats keys, missing pipeline cache stats, shader failures without
  prior-valid revision evidence, and non-material fallback warning behavior.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documents the new inspection API and its renderer/product ownership
  boundary.

## Governance Evidence

Accepted gates:

- `docs-site/src/content/docs/design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md`
- `docs-site/src/content/docs/reports/closeouts/pm-render-mesh-material-001-mesh-material-lighting-handoff-doctrine/closeout.md`
- `docs-site/src/content/docs/reports/closeouts/wr-067-renderer-mesh-material-shader-asset-handoff/closeout.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-068-renderer-lighting-pipeline-cache-and-last-good-fallback/plan.md`

ADR decision: no ADR required. The implementation hardens existing
renderer-owned shader, pipeline cache, pass provenance, and inspection
contracts. It does not persist a new cross-domain ABI, change dependency
direction, move source truth, or change fallback authority.

## Validation

Focused validation passed:

```text
cargo fmt
cargo test -p engine render_pipeline
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow_fragments
cargo test -p engine render_cutoff_guard
```

Planning validation after closeout metadata:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- Runtime mesh/material production examples, benchmarks, visible runtime proof,
  and production evidence remain WR-069 scope.
- WR-068 proves renderer pipeline/fallback inspection and diagnostics; it does
  not claim `runtime_proven` or `perfectionist_verified`.
- Final perfectionist verification remains blocked until
  `PT-RENDER-PERFECTION` audits the completed renderer production stack.
