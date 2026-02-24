# Execution Plan

## Scope

Active plan for State C:

1. fully ECS-driven render pipeline state,
2. plugin-owned render declarations/executors,
3. breaking changes allowed.

Date: 2026-02-24

## Current Baseline

Implemented:

1. Typed render ids and typed graph builder API are in place (`RenderFeatureGraphSpec::builder(...)`).
2. Feature graph registration supports register/replace/remove by feature id.
3. Runtime executor registry supports custom trait-object executors.
4. Renderer dispatch can execute custom executors.
5. `sdf_renderer` is file-driven from:
   - `engine/examples/sdf_renderer/assets/sdf_params.ron`
   - `engine/examples/sdf_renderer/assets/input_bindings.ron`
   - `engine/examples/sdf_renderer/assets/render_graph.ron`
6. `sdf.compute`, `sdf.compose`, and `ui_composite` run through custom executor paths in the example.

## Target Architecture

1. ECS resources are the sole authority for render graph specs, runtime state, compiled plans, diagnostics, and per-frame data.
2. Render identity is fully id-based (`resource/pipeline/pass/executor` ids).
3. Frame flow is deterministic and staged:
   - extract/update frame data,
   - validate + compile graph,
   - prepare pass data,
   - encode + submit.
4. Plugins own renderer feature definitions and executors.
5. Core engine owns device/swapchain lifecycle, orchestration, and diagnostics surfacing.
6. Graph compiler logic is isolated for backend-agnostic testing (`render_graph_core` target).

## API Contract

Primary authoring surface:

1. Typed builder API for feature graph declarations.
2. Optional RON import that converts into the same typed model.

Executor contract:

```rust
pub trait RenderPassExecutor: Send + Sync {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> anyhow::Result<()>;
    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> anyhow::Result<()>;
}
```

Direct runtime APIs:

1. `render_graph_specs.register_feature_graph(spec)`
2. `render_graph_specs.replace_feature_graph(feature_id, spec)`
3. `render_executor_registry.register_custom(executor_id, custom_executor)`
4. `render_graph_specs.remove_feature_graph(feature_id)`

## Remaining Execution Phases

### Phase 1: ECS Runtime Authority

1. Move active render graph/runtime state fully into ECS render resources.
2. Route frame submit to consume ECS resources as source of truth.

Exit criteria:

1. No renderer-local graph authority remains.

### Phase 2: Dynamic Pipeline Registry

1. Replace enum/slot-based pipeline selection with id-keyed pipeline registry.
2. Remove `PipelineKey` as the primary feature authoring surface.

Exit criteria:

1. Feature pipelines can be added without central enum edits.

### Phase 3: Graph Core Extraction

1. Extract schema/compiler/validation into `render_graph_core`.
2. Wire engine renderer to compiled outputs from that crate.

Exit criteria:

1. Compiler behavior is covered by crate-level unit tests.

### Phase 4: Descriptor and Scene Migration

1. Migrate scene/example descriptors fully to id-based DTOs.
2. Keep SDF direct config apply flow (no event queue requirement).

Exit criteria:

1. Scene and example render descriptors are fully id-based.

### Phase 5: Legacy Removal

1. Remove enum/slot compatibility authoring path.
2. Remove old bridge adapters and compatibility routing.

Exit criteria:

1. Production render authoring path is State C only.

## Validation Plan

1. Unit tests for id validation, ordering, hazard checks, and missing refs.
2. Integration tests for plugin-defined compute+compose flows without core enum edits.
3. Regression tests for startup, resize, diagnostics, and render correctness.
4. Performance checks for compile cache effectiveness and frame encode/submit overhead.

## Definition of Done

1. Render pipeline authority is ECS-owned end-to-end.
2. Plugins can author independent renderer paths with id-based declarations.
3. Core renderer executes compiled plans generically.
4. Legacy enum/slot authoring path is removed.
5. SDF example demonstrates the final custom-renderer authoring model.
