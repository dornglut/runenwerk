---
title: WR-059 Boids Hybrid SDF/Mesh Procedural Sprite Rewrite Closeout
description: Closeout evidence for rewriting the canonical boids example through the renderer procedural API.
status: completed
owner: engine
layer: engine-runtime / renderer examples
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-059 Boids Hybrid SDF/Mesh Procedural Sprite Rewrite Closeout

## Outcome

`WR-059` is complete at `bounded_contract` quality. The canonical boids example
now keeps simulation in storage-backed compute passes, publishes the simulated
state into the render instance buffer, and renders through the public
`RenderFlow::procedural_pass(...)` API with
`ProceduralPassDescriptor::local_sdf_2d_impostors(...)`. The old history copy
and fullscreen-per-boid fragment-loop render shape are removed.

The row is not marked `runtime_proven` because the local validation could start
the windowed example without shader, pipeline, or runtime errors, but it did not
capture a finite timing artifact before manual interrupt. Numeric runtime
budgets and production smoke evidence remain owned by `WR-060`.

## Implementation Evidence

Changed modules:

- `engine/examples/boids_render_flow/rendering/graph.rs::build_render_flow`:
  declares `boids.simulate`, `boids.publish_instances`, procedural `boids.draw`,
  and `boids.present` passes; removes `boids.history`; and consumes
  `RenderFlow::procedural_pass(...)` with a local 2D SDF impostor descriptor.
- `engine/examples/boids_render_flow/rendering/graph.rs::boid_instance_layout`:
  exposes position and velocity as explicit instance attributes for the public
  procedural API path.
- `engine/examples/boids_render_flow/rendering/state.rs::BoidsRenderState`:
  keeps compute simulation parameters separate from `publish_params`, which
  selects the render-buffer publish mode.
- `assets/shaders/boids_compute.wgsl::cs_main`: preserves storage-backed boids
  simulation and adds publish mode to copy the simulated `boids_b` state into
  `boids_a` for rendering.
- `assets/shaders/boids_compose.wgsl::vs_main` and
  `assets/shaders/boids_compose.wgsl::fs_main`: render one local per-instance
  SDF impostor from instance position and velocity inputs, with no storage
  binding and no loop over all boids.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  documents boids as the canonical storage-compute plus procedural-local-SDF
  consumer path.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  names the boids example as the canonical procedural consumer.

## Validation

Focused validation:

```text
cargo fmt
cargo test -p engine --example boids_render_flow
cargo test -p engine render_runtime_inspect
```

The boids example test command executed nine tests covering pass declaration,
history-copy removal, procedural local instance geometry, pass ordering,
uniform projection, shader-loop removal, state publish parameters, and dispatch
coverage. The render runtime inspection command executed the existing
`render_runtime_inspect` coverage without regressions.

Runtime smoke:

```text
cargo run -p engine --example boids_render_flow
```

The windowed example built and entered `target\debug\examples\boids_render_flow.exe`
without shader, pipeline, or runtime errors before manual `Ctrl-C` interrupt.
The smoke did not emit a finite timing artifact in this environment, so runtime
timing evidence remains a `WR-060` production evidence requirement.

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

## Pass-Shape Proof

`engine/examples/boids_render_flow/rendering/graph.rs::procedural_draw_pass_binds_local_instance_geometry`
asserts that `boids.draw` binds one instance buffer with the expected position
and velocity layout, emits a six-vertex local quad, uses `DEFAULT_BOID_COUNT` as
the instance count, and passes `compile_flow_plan_checked(...)` with runtime
default backend capabilities.

`engine/examples/boids_render_flow/rendering/graph.rs::flow_does_not_keep_history_copy_without_history_consumer`
asserts there are no copy passes, no pass labels containing `history`, and no
`boids.history` resource.

## Shader-Loop Removal Evidence

`engine/examples/boids_render_flow/rendering/graph.rs::compose_shader_uses_instance_inputs_without_storage_loop`
loads `assets/shaders/boids_compose.wgsl` and asserts that the shader consumes
`@location(0) instance_position` and `@location(1) instance_velocity` while not
containing `var<storage` or `for (var i`.

The compute shader still contains storage buffers and the simulation neighbor
loop. That remains intentional engine-example behavior for storage-backed boids
simulation; the removed hazard is the fullscreen fragment shader loop over all
boids.

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- `WR-060` still owns production runtime evidence, numeric timing budgets, and
  stress thresholds for the complete procedural visual chain.
- The local windowed smoke was manually interrupted and did not capture a finite
  compute, draw, and present timing artifact.
- The current procedural API path does not bind compose-pass uniforms for the
  boids impostor style; the shader uses constants for visual scale and color.

These gaps do not invalidate `WR-059`; they preserve the intended boundary
between the canonical example rewrite and the later production evidence row.
