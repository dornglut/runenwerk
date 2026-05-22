---
title: WR-058 Hybrid Mesh/SDF Procedural Instance Rendering API Closeout
description: Closeout evidence for renderer-owned procedural mesh sprite, quad sprite, and local 2D SDF impostor APIs.
status: completed
owner: engine
layer: engine-runtime / renderer public API
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
related_roadmaps:
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-058 Hybrid Mesh/SDF Procedural Instance Rendering API Closeout

## Outcome

`WR-058` is complete. The renderer now exposes a focused procedural API that
builds graphics passes for mesh sprites, generated quad sprites, and local 2D
SDF impostors from typed storage-backed buffers and explicit render policy. The
API emits normal render-flow passes, satisfies the `WR-057` pass-shape guard by
using local instance geometry, and keeps product truth, freshness, fallback,
rebuild, and residency policy outside the renderer.

## Implementation Evidence

Changed modules:

- `engine/src/plugins/render/procedural/mod.rs`:
  public procedural subsystem boundary.
- `engine/src/plugins/render/procedural/descriptors.rs`:
  procedural pass, visual, buffer, target, local 2D SDF impostor, and render
  policy descriptors.
- `engine/src/plugins/render/procedural/validation.rs`:
  typed validation errors for missing shaders/targets, invalid layouts, depth
  policy mismatches, and unsupported generated-sprite topology.
- `engine/src/plugins/render/procedural/builders.rs`:
  translation from validated procedural descriptors into existing
  `RenderFlow::graphics_pass(...)` passes.
- `engine/src/plugins/render/api/flow.rs`:
  `RenderFlow::procedural_pass(...)`.
- `engine/src/plugins/render/api/passes.rs`:
  `GraphicsPassBuilder::raster_state(...)` plus crate-internal raw buffer hooks
  used by the procedural translator after typed handle capture.
- `engine/src/plugins/render/graph/pass_graph.rs` and
  `engine/src/plugins/render/graph/execution_plan.rs`:
  typed raster state and compiled raster-state evidence.
- `engine/src/plugins/render/renderer/render_flow/bindings.rs`,
  `engine/src/plugins/render/pipelines/flow_keys.rs`, and
  `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`:
  raster-state pipeline-key and backend execution support for blend, depth,
  cull, and primitive topology policy.
- `engine/tests/procedural_instance.rs`:
  focused API, validation, pass-shape, and compiled policy coverage.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`
  and
  `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  public usage documentation.

## Validation

Focused validation:

```text
cargo fmt
cargo test -p engine procedural_instance
cargo test -p engine render_flow
cargo test -p engine render_runtime_inspect
```

The `procedural_instance` command executed six focused tests covering quad
sprites, mesh sprites, local 2D SDF impostors, explicit render policy
compilation, invalid instance layouts, and invalid depth policy.

Workflow validation after metadata updates:

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

- The canonical boids example still needs to consume the procedural API in
  `WR-059`.
- Runtime GPU evidence for the complete procedural visual chain remains a
  `WR-060` requirement.
- Local 2D SDF impostors intentionally do not include 3D SDF raymarch, sparse
  residency, or product-owned SDF authority decisions.

These gaps are intentional for `WR-058`; the API row is complete, but the
renderer GPU/procedural track cannot claim `runtime_proven` until the canonical
boids proof and production evidence rows close.
