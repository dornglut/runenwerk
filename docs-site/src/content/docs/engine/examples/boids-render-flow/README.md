---
title: Boids Render Flow Example
description: Production procedural boids render-flow example with fixed-step evidence, procedural pass authoring, and aspect-correct impostors.
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-25
---

# Boids Render Flow Example

Windowed boids simulation sample built on the public `RenderFlow` and
procedural population authoring APIs.

What it demonstrates:

- ECS simulation state (`BoidsRenderState`) projected into flow uniforms.
- ping-pong storage simulation (`boids.instances`) on a compute pass.
- procedural-owned draw authoring through `RenderFlow::procedural_pass_builder(...)`.
- bounded uniform-grid neighbor lookup instead of a production O(n^2) all-boids loop.
- reusable procedural camera projection through `ProceduralCamera2d`,
  `ProceduralCamera2dUniform`, and `ProceduralSpriteSizing`.
- smoothed visual heading stored separately from simulation velocity.
- graph-level fixed-step catch-up evidence; multi-step catch-up is owned by the
  render-flow graph, not hidden in this example.
- flow-owned color target and explicit terminal present pass.
- pure builtin compiled runtime path (no custom executor hooks).
- production evidence for no silent grid overflow, bounded submitted work,
  unsupported timing diagnostics, and pixel-space resize/aspect checks.

Flow chain:

- `boids.seed_or_hold` (compute, `assets/shaders/boids_compute.wgsl`)
- `boids.grid.clear_counts`
- `boids.grid.count_cells`
- `boids.grid.scan_counts`
- `boids.grid.reset_cursors`
- `boids.grid.scatter_sorted_indices`
- `boids.grid.simulate_neighbors` (compute over adjacent grid cells)
- `boids.grid.publish_draw` (compute publish to the render-facing instance buffer)
- `boids.draw` (graphics, `assets/shaders/boids_compose.wgsl`)
- `boids.present` (present, `boids.color` -> surface)

The grid path is bounded and total-count-sized. Cell counts, scan offsets,
scatter cursors, and sorted indices are explicit flow resources; dense cells
must not silently overflow or drop neighbors. Spatial hash and chunked
unbounded populations are intentionally out of scope for this example.

The simulation region is declared as `boids.fixed_step` and derives submitted
substeps from runtime fixed-time resources. The frame delta is kept as evidence
input, not a hidden example-local scheduler.

The draw pass uses a producer-owned `ProceduralCamera2d` with
`FillViewport { fixed_axis: Vertical }`. Renderer procedural code derives the
projection uniform from camera intent and target size; `PreparedViewFrame` does
not own camera truth. The boids shader projects world positions through
`ProceduralCamera2dUniform`, so landscape, portrait, square, and extreme aspect
surfaces keep equal projected world x/y scale.

Run:

```bash
cargo run -p engine --example boids_render_flow
```

Evidence:

```bash
cargo run -p engine --example boids_render_flow -- --evidence
```

The evidence output includes the canonical pass order, fixed-step contract,
grid capacity counters, unsupported GPU timing diagnostics, CPU preflight
timing, benchmark command, camera projection evidence for landscape, portrait,
square, and extreme aspect surfaces, and resize pixel evidence proving sprite
aspect reconstruction.

Benchmarks:

```bash
cargo bench -p engine --bench render_flow_planning
```

The benchmark suite includes procedural population cases for prefix scan
planning, scan/scatter/indirect-args planning, bounded-grid build planning,
boids production flow planning/preflight, and boids production evidence
reporting.

Tests:

```bash
cargo test -p engine --example boids_render_flow
```
