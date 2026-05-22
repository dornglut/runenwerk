---
title: WR-057 Render Flow Pass Shape And Instance Contract Guards Closeout
description: Closeout evidence for renderer-owned pass-shape and instance-count guard diagnostics.
status: completed
owner: engine
layer: engine-runtime / render graph validation
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-execution-graph-compiler-maturity-design.md
related_roadmaps:
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-057 Render Flow Pass Shape And Instance Contract Guards Closeout

## Outcome

`WR-057` is complete. Render-flow compilation and prepared-frame preflight now
diagnose fullscreen-style generated graphics multiplied by instance count unless
the pass declares explicit bounded advanced intent. The guard is renderer-owned,
typed, pre-submit, and independent of boids-specific policy or product-domain
truth.

## Implementation Evidence

Changed modules:

- `engine/src/plugins/render/graph/pass_graph.rs`:
  `RenderPassShapeIntent` and `RenderPassNode::shape_intent`.
- `engine/src/plugins/render/api/passes.rs`:
  `GraphicsPassBuilder::allow_instanced_fullscreen(...)` for explicit bounded
  author intent.
- `engine/src/plugins/render/graph/diagnostics.rs`:
  `FullscreenInstancedWork`, `AmbiguousProceduralShape`, and
  `InvalidPassShapeIntent` diagnostic kinds.
- `engine/src/plugins/render/graph/pass_shape.rs`:
  focused pass-shape classification and diagnostics.
- `engine/src/plugins/render/graph/planning.rs`:
  compiler integration through `compile_flow_plan_checked(...)`.
- `engine/src/plugins/render/graph/prepared_validation.rs`:
  full preflight and cached runtime-guard integration.
- `engine/tests/render_flow_v2.rs`:
  default rejection, explicit opt-in, opt-in limit enforcement, local geometry
  preservation, and cached runtime guard coverage.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`,
  `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`,
  and `docs-site/src/content/docs/engine/reference/plugins/render/usage-guide.md`:
  public contract documentation.

## Validation

Focused validation:

```text
cargo test -p engine render_flow
cargo test -p engine render_cutoff_guard
cargo test -p engine render_runtime_inspect
task docs:validate
```

Workflow validation after metadata updates:

```text
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

- The guard classifies pass shape from declared draw, buffer, storage-binding,
  and explicit intent metadata. It does not inspect shader source to detect
  fragment loops over storage arrays.
- The runtime evidence is a prepared-frame preflight guard test, not a windowed
  swapchain smoke run.
- Later procedural API and boids proof rows must show the preferred local
  geometry path in a production example.

These gaps are intentional for `WR-057`; they stay visible until the procedural
instance API and canonical boids proof consume the guard contract.
