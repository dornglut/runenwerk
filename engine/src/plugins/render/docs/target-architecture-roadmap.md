# Render Architecture Roadmap

## Scope

This roadmap covers the target architecture defined in:

- `engine/src/plugins/render/docs/target-architecture.md`

It turns that target into an implementation sequence that minimizes churn while preserving a clean long-term shape.

## Roadmap Philosophy

### Main rule

Build the architecture in this order:

1. type and layout foundations
2. resource model
3. simple pass/flow API
4. ECS-first param projection
5. backend execution bridge
6. plugin flow composition
7. advanced pass/resource kinds
8. inspection/tooling
9. performance and lifetime optimization
10. editor/data-driven extensions

### Why this order

The biggest failure mode is doing too much ergonomic API work before:

- GPU type conversion is stable
- resource identity is stable
- pass/resource planning is stable

This roadmap locks low-level contracts first, then builds the clean API on top.

## Phase 0 - Lock Target Contracts

### Goal

Freeze naming and architecture direction before implementation starts spreading.

### Domain

Documentation.

### Target files

- `engine/src/plugins/render/docs/target-architecture.md`
- `engine/docs/reference/plugins/render/render-target-architecture.md`
- `engine/docs/reference/plugins/render/render-flow-usage-guide.md`
- `engine/docs/reference/plugins/render/gpu-params-guide.md`
- `engine/src/plugins/render/README.md`

### Implement

- Lock terminology:
  - `RenderFlow`
  - `RenderFlowContribution`
  - `GpuUniform`
  - `GpuStorage`
  - `ToGpuValue`
  - `GpuParams`
  - `ecs_resource`
  - `compute_pass`
  - `graphics_pass`
  - `fullscreen_pass`
- Document that input bindings belong to input/app API, not render flow.
- Document namespaced pass/resource IDs as the target convention.

### Exit criteria

- no more naming churn on the public target surface
- team/agents know the endgame shape

### Risk

Low.

## Phase 1 - GPU Param Foundations

### Goal

Create the type conversion and layout foundation that everything else depends on.

### Domain

Render params.

### Target modules

- `engine/src/plugins/render/params/mod.rs`
- `engine/src/plugins/render/params/gpu_value.rs`
- `engine/src/plugins/render/params/gpu_params.rs`
- `engine_render_macros/Cargo.toml` (new macro crate)
- `engine_render_macros/src/lib.rs`

### Implement

1. `ToGpuValue`
   - Add trait.
   - Add built-in impls for:
     - `bool`
     - `u32`
     - `i32`
     - `f32`
     - `[u32; N]`
     - `[i32; N]`
     - `[f32; N]`
2. `GpuParams`
   - Add trait.
   - Add internal raw type contract.
3. `GpuUniform` derive
   - Implement `#[derive(GpuUniform)]`.
   - Generate raw type.
   - Generate `GpuParams` impl.
   - Require every field type to implement `ToGpuValue`.
4. Optional `GpuStorage` derive
   - If feasible in same phase: `#[derive(GpuStorage)]`.
   - Otherwise defer to Phase 3.

### File targets

- `engine/src/plugins/render/params/gpu_value.rs`
- `engine/src/plugins/render/params/gpu_params.rs`
- `engine_render_macros/src/lib.rs`

### Verification

Add tests in `engine/tests/render_gpu_params.rs`:

- validate generated raw conversion
- validate bool conversion
- validate array conversion
- validate compile-fail for unsupported field types

### Exit criteria

You can write:

```rust
#[derive(Debug, Clone, Copy, GpuUniform)]
struct Params {
    a: [u32; 2],
    b: bool,
}
```

and get valid generated GPU conversion.

### Risk

Medium.

### Notes

Do not add complex field-level macro DSL here. Keep this phase narrow.

## Phase 2 - Resource ID and Descriptor Model

### Goal

Create a stable internal model for render resources before building public flow composition.

### Domain

Render resources.

### Target modules

- `engine/src/plugins/render/resource/mod.rs`
- `engine/src/plugins/render/resource/descriptors.rs`
- `engine/src/plugins/render/resource/usages.rs`
- `engine/src/plugins/render/resource/import.rs`
- `engine/src/plugins/render/api/ids.rs`

### Implement

1. Resource ID model
   - Start with namespaced string-backed IDs wrapped in small internal newtypes.
   - Examples: `RenderResourceId`, `RenderPassId`, `RenderFlowId`.
2. Resource descriptor model
   - Add descriptors for:
     - uniform buffer
     - storage buffer
     - sampled texture
     - storage texture
     - color target
     - depth target
     - imported texture
     - imported buffer
3. Resource usage model
   - Add internal usage flags/categories:
     - `read`, `write`, `read_write`
     - `sampled`, `storage`
     - `color_target`, `depth_target`
     - `vertex`, `index`, `instance`, `indirect`
   - Keep internal initially.

### File targets

- `engine/src/plugins/render/resource/descriptors.rs`
- `engine/src/plugins/render/resource/usages.rs`
- `engine/src/plugins/render/api/ids.rs`

### Verification

- descriptor construction tests
- ID namespace validation tests
- duplicate ID detection tests

### Exit criteria

The engine can represent intended resource categories internally without public API churn.

### Risk

Medium.

## Phase 3 - Minimal RenderFlow API

### Goal

Ship the first public flow API for simple compute/fullscreen/UI cases.

### Domain

API + graph.

### Target modules

- `engine/src/plugins/render/api/mod.rs`
- `engine/src/plugins/render/api/flow.rs`
- `engine/src/plugins/render/api/resources.rs`
- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/graph/flow_graph.rs`
- `engine/src/plugins/render/graph/pass_graph.rs`
- `engine/src/plugins/render/graph/resource_graph.rs`
- `engine/src/plugins/render/graph/validation.rs`

### Implement

1. `RenderFlow`
   - `RenderFlow::new(...)`
   - `.ecs_resource::<T>()`
   - `.uniform_buffer::<T>(...)`
   - `.storage_buffer::<T>(...)`
   - `.sampled_texture(...)`
   - `.color_target(...)`
   - `.import_texture(...)`
2. Pass builders
   - `.compute_pass(...)`
   - `.fullscreen_pass(...)`
   - `.builtin_ui_composite_pass(...)`
3. Per-pass declarations
   - `.shader(...)`
   - `.reads(...)`
   - `.writes(...)`
   - `.depends_on(...)`
   - `.workgroup_size(...)`
   - `.clear_color(...)`
4. Graph validation
   - missing resources
   - missing pass references
   - cycles
   - duplicate IDs

Suggested first supported example:

- Game of Life SDF example, or
- a simple fullscreen postprocess example

### Verification

Add tests in `engine/tests/render_flow_graph.rs`:

- pass order validation
- resource reference validation
- cycle detection
- duplicate namespace conflict detection

### Exit criteria

You can author a simple compute -> fullscreen -> UI flow declaratively.

### Risk

Medium.

### Notes

Do not implement plugin contribution yet. Do not expose advanced buffer kinds yet.

## Phase 4 - ECS-First Param Projection Helpers

### Goal

Make the public API pleasant by wiring ECS state into pass params.

### Domain

API ergonomics.

### Target modules

- `engine/src/plugins/render/api/bindings.rs`
- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/api/flow.rs`

### Implement

1. `uniform_state`
   - For compute/fullscreen/graphics passes:
   - `.uniform_state(StateType::method_name)`
2. `uniform_state_with_surface`
   - For passes that need surface size:
   - `.uniform_state_with_surface(StateType::method_name)`
3. ECS resource lookup bridge
   - resolve declared ECS resource
   - call supplied state method
   - convert via `GpuParams`
   - route to declared uniform buffer

Example target support:

```rust
.compute_pass("gol.compute")
.uniform_state(GameOfLifeSdfState::compute_params)

.fullscreen_pass("gol.compose")
.uniform_state_with_surface(GameOfLifeSdfState::compose_params)
```

### Verification

Integration tests with:

- missing ECS resource
- missing matching uniform buffer
- successful param upload path

### Exit criteria

No inline context closures are needed for common uniform param upload.

### Risk

Medium.

### Notes

This is where the public API starts feeling good.

## Phase 5 - Backend Execution Bridge

### Goal

Connect declarative flow definitions to actual GPU execution.

### Domain

Backend + execution.

### Target modules

- `engine/src/plugins/render/backend/mod.rs`
- `engine/src/plugins/render/backend/device.rs`
- `engine/src/plugins/render/backend/surface.rs`
- `engine/src/plugins/render/backend/formats.rs`
- `engine/src/plugins/render/backend/pipeline_cache.rs`
- `engine/src/plugins/render/backend/resource_allocator.rs`
- `engine/src/plugins/render/backend/execution.rs`

### Implement

1. Resource allocation bridge
   - map flow resource descriptors to actual GPU resources
2. Pipeline lookup/build
   - compute pipelines
   - fullscreen pipelines
   - builtin UI composite pipeline
3. Pass execution order
   - compute passes
   - fullscreen passes
   - builtin UI composite
4. Upload plumbing for `.uniform_state(...)`
   - fetch ECS resource
   - build params
   - convert to GPU raw
   - upload before pass execution

### Verification

- smoke test with simple compute flow
- smoke test with fullscreen pass
- smoke test with UI composite
- snapshot/log validation of pass order

### Exit criteria

A declarative flow actually runs on the backend.

### Risk

High.

### Notes

One of the biggest phases. Keep supported pass/resource types narrow at first.

## Phase 6 - Current Example Migration Phase

### Goal

Prove the architecture by migrating real examples.

### Domain

Examples and docs.

### Target examples

- `engine/examples/game_of_life_sdf/`
- `engine/examples/sdf_renderer/`
- one minimal postprocess example
- one minimal fullscreen compute example

### Implement

Migrate each example to:

- `RenderFlow`
- `GpuUniform`
- state-owned param methods
- input bindings moved to app/input API

### File targets

- `engine/examples/game_of_life_sdf/runtime/app.rs`
- `engine/examples/game_of_life_sdf/runtime/state.rs`
- `engine/examples/game_of_life_sdf/rendering/params.rs`
- `engine/examples/sdf_renderer/...`

### Verification

- examples compile
- examples run
- previous low-level boilerplate is removed
- docs updated to reflect new API

### Exit criteria

At least two real examples use the new architecture end-to-end.

### Risk

Medium.

## Phase 7 - Graphics Pass and Geometry Pipeline Support

### Goal

Expand beyond fullscreen and basic compute into mesh/geometry workflows.

### Domain

Pass model expansion.

### Target modules

- `engine/src/plugins/render/passes/graphics.rs`
- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/resource/usages.rs`

### Implement

1. `graphics_pass`
   - add generic raster/graphics pass support
2. Buffer binding methods
   - `.vertex_buffer(...)`
   - `.index_buffer(...)`
   - `.instance_buffer(...)`
   - `.indirect_buffer(...)`
3. Depth support
   - `.depth_target(...)`
   - depth usage validation
4. Geometry-generated workflows
   - support compute-generated buffers consumed by graphics passes

Example targets:

- boids draw pass
- simple mesh pass
- generated instance pass
- debug line/shape rendering later

### Verification

Integration tests:

- compute writes buffer -> graphics reads buffer
- graphics pass with depth target
- instance buffer consumption

### Exit criteria

Supports geometry/mesh-style GPU workflows, not just fullscreen composition.

### Risk

High.

## Phase 8 - Flow Contribution and Multi-Plugin Composition

### Goal

Allow multiple plugins/domains to contribute to a single flow.

### Domain

Composition.

### Target modules

- `engine/src/plugins/render/composition/contribution.rs`
- `engine/src/plugins/render/composition/namespaces.rs`
- `engine/src/plugins/render/composition/integration.rs`
- `engine/src/plugins/render/graph/merge.rs`

### Implement

1. `RenderFlowContribution`
   - allow plugins to author namespaced subflows
2. Merge/integration planner
   - merge contributions into the main flow
3. Namespace validation
   - duplicate pass IDs
   - duplicate resource IDs
   - invalid cross-plugin dependencies
4. App integration
   - `app.add_render_flow_contribution(...)`

Example targets:

- one boids contribution
- one postprocess contribution
- one UI contribution
- merged into one main flow

### Verification

Tests:

- merge order
- namespace collision detection
- cross-contribution dependency resolution

### Exit criteria

Multiple plugins can add passes/resources without central graph monoliths.

### Risk

High.

### Notes

Most important endgame flexibility feature.

## Phase 9 - Resource Imports, Persistence, and Transient Modeling

### Goal

Support real engine-grade resource lifetime models.

### Domain

Resource lifetime and import model.

### Target modules

- `engine/src/plugins/render/resource/import.rs`
- `engine/src/plugins/render/resource/lifetime.rs`
- `engine/src/plugins/render/resource/transient.rs`
- `engine/src/plugins/render/backend/resource_allocator.rs`

### Implement

1. Imported resources
   - surface color
   - scene depth
   - external shared textures
   - external buffers
2. Persistent flow-owned resources
   - history textures
   - boid instance buffers
   - clipmap caches
3. Transient resources
   - one-frame intermediate targets
   - temporary blur textures
   - temporary postprocess targets
4. Internal aliasing model (at least internal)
   - transient lifetime windows
   - aliasable resources

### Verification

- imported surface/depth integration
- persistent history test
- transient blur ping-pong flow test

### Exit criteria

Supports real long-lived and short-lived GPU resources correctly.

### Risk

High.

## Phase 10 - Advanced Pass/Resource Kinds

### Goal

Cover more realistic render workloads.

### Domain

API and backend expansion.

### Implement

New pass kinds:

- `copy_pass`
- `present_pass`

New resource kinds:

- `storage_texture(...)`
- `history_texture(...)`

New bindings:

- `.sample_texture(...)`
- `.write_texture(...)`
- `.storage_state(...)`

Support workloads like:

- boids
- particles
- influence maps
- SDF field generation
- ping-pong fluid steps
- temporal AA history
- clipmap update passes

### Verification

Dedicated examples/tests for:

- storage texture write/read
- copy pass
- history texture read/write chain

### Exit criteria

Expresses more than basic compute/fullscreen/graphics flows.

### Risk

Medium to high.

## Phase 11 - Inspection and Debugging Architecture

### Goal

Make large flows debuggable and operable.

### Domain

Inspection.

### Target modules

- `engine/src/plugins/render/inspect/graph_dump.rs`
- `engine/src/plugins/render/inspect/resource_inspector.rs`
- `engine/src/plugins/render/inspect/texture_view.rs`
- `engine/src/plugins/render/inspect/timings.rs`

### Implement

1. Graph dump
   - passes
   - resources
   - dependencies
   - execution order
2. Resource inspector
   - declared resources
   - imported resources
   - live resource metadata
3. Texture inspection
   - storage textures
   - color targets
   - history textures
   - debug resources
4. Timings
   - per-pass timing
   - total frame timing
   - optional debug overlay

### Verification

- debug output tests
- example with graph dump
- example with texture inspection

### Exit criteria

Render graph is inspectable enough to scale.

### Risk

Medium.

## Phase 12 - Documentation, Examples, and Stabilization

### Goal

Turn the architecture into a stable user-facing system.

### Domain

Docs and developer experience.

### Target docs

- `engine/docs/reference/plugins/render/render-target-architecture.md`
- `engine/docs/reference/plugins/render/render-flow-usage-guide.md`
- `engine/docs/reference/plugins/render/gpu-params-guide.md`
- `engine/docs/reference/plugins/render/render-flow-contributions.md`
- `engine/src/plugins/render/README.md`

### Example set

- minimal fullscreen flow
- game of life compute/fullscreen flow
- SDF renderer flow
- boids contribution flow
- postprocess compositor flow
- debug texture inspection example

### Stabilization tasks

- error messages
- naming cleanups
- validation polish
- internal API cleanup
- remove deprecated low-level paths from examples

### Exit criteria

Users can learn and use the architecture without reading internal engine code.

### Risk

Medium.

## Phase 13 - Performance and Optimization Phase

### Goal

Make the architecture fast enough for real workloads.

### Domain

Backend and planning.

### Implement

- pipeline cache improvements
- transient aliasing optimization
- pass scheduling optimization
- reduced redundant uploads
- resource reuse
- backend allocation tuning

### Measurements

Benchmark:

- simple fullscreen flow
- boids sim + draw
- multi-pass compositor
- SDF compute + compose
- mixed contribution flow

### Exit criteria

Architecture is not just clean, but performant under realistic loads.

### Risk

Medium.

## Phase 14 - Editor/Data-Driven Future Phase

### Goal

Open the architecture to future tooling and data-driven workflows.

### Domain

Future extension.

### Candidate work

- asset-authored graph fragments
- editor graph inspection
- graph hot reload
- plugin-contributed visual graph views
- viewport-specific flow variants
- multi-view rendering

### Exit criteria

Optional long-term expansion, not required for core success.

### Risk

Variable.

## Recommended Order of Attack

### Best immediate sequence

First wave:

- Phase 0
- Phase 1
- Phase 2
- Phase 3

Second wave:

- Phase 4
- Phase 5
- Phase 6

Third wave:

- Phase 7
- Phase 8
- Phase 9

Fourth wave:

- Phase 10
- Phase 11
- Phase 12

Last wave:

- Phase 13
- Phase 14

This order minimizes churn.

## What to Implement First, Concretely

### Very first concrete implementation target

Files:

- `engine/src/plugins/render/params/gpu_value.rs`
- `engine/src/plugins/render/params/gpu_params.rs`
- `engine_render_macros/src/lib.rs`
- `engine/src/plugins/render/api/flow.rs`
- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/graph/validation.rs`

### Very first example target

- `engine/examples/game_of_life_sdf/`

Because it gives you:

- compute pass
- fullscreen pass
- UI composite
- ECS-driven params
- a good test of ergonomics

## Major Design Checkpoints

### Checkpoint A

After Phase 3:

- Is the flow API pleasant enough for simple cases?
- Are IDs/resource declarations stable?

### Checkpoint B

After Phase 5:

- Does the backend bridge preserve the intended API?
- Did execution force bad public API compromises?

### Checkpoint C

After Phase 8:

- Does multi-plugin contribution feel clean?
- Are namespaces and merge rules good enough?

### Checkpoint D

After Phase 11:

- Can you debug a complex mixed flow without pain?

These checkpoints should be explicit.

## Risks to Watch

### Biggest architectural risks

1. Over-magical macros
   - Avoid giant derive DSLs.
   - Keep state extraction explicit.
2. One giant global flow file
   - If plugin contribution is delayed too long, architecture rot begins.
3. Weak resource typing
   - If resource descriptors remain too vague, mixed compute/graphics workflows get brittle.
4. Leaking backend details too early
   - Do not expose executor/plumbing internals in the main API.
5. Delaying inspection tooling too long
   - Large graphs become painful without visibility.
