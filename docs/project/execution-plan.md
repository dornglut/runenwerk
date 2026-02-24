# Execution Plan

## Purpose

Define the active implementation plan for engine/runtime architecture.

This document now commits to **State C**:

1. fully ECS-driven render pipeline state,
2. breaking changes allowed,
3. no required dependence on legacy world-renderer naming.

## Decision

Date: 2026-02-24

Decision:

1. Move render pipeline state to ECS resources end-to-end.
2. Keep GPU lifecycle in engine core, but move render declarations/execution mapping to data + plugin systems.
3. Treat legacy enum/slot render APIs as removable compatibility code.

Superseded assumptions:

1. Incremental bridge behavior parity with old world path is no longer the primary constraint.
2. `world_compute` / `world_compose` naming is not a required architectural dependency.

## Current State Review

What exists today:

1. Engine has plugin scheduling and runtime plugin setup.
2. Legacy render graph owner registration exists (`RenderGraphRegistryResource`) and supports owner upsert/replace.
3. Graph compilation/order + diagnostics exist in renderer.

Current blockers for State C:

1. Pipeline identity is still enum/slot based (`PipelineKey`, `PassSlot`, `PipelineRegistry`).
2. Executor dispatch is still bound to a fixed built-in executor list.
3. Render registry does not yet include first-class resource declarations.
4. Render graph state is held in runtime structs, not a dedicated ECS render resource model.
5. Scene manifest render descriptors are enum constrained.

## Target Architecture (State C)

### 1. ECS-Owned Render Runtime

Create a render runtime ECS state model owned by resources, including:

1. render graph declarations,
2. pipeline declarations,
3. executor bindings,
4. compile cache and compiled frame plan,
5. compile/validation diagnostics,
6. per-frame extracted render data.

### 2. String/Newtype IDs for Render Identity

Use feature-defined ids (not central enums) for:

1. resources,
2. pipelines,
3. passes,
4. executors.

### 3. Compile Then Execute Flow

Render path should be stage-driven:

1. extract/update ECS render resources,
2. validate + compile graph into executable frame plan,
3. prepare pass data via bound executors,
4. encode/submit command buffers.

### 4. Plugin Ownership

Plugins own:

1. declarations (resources/pipelines/passes),
2. executor behavior registration,
3. per-frame data updates.

Core engine owns:

1. swapchain/device lifecycle,
2. command encoder lifecycle and submit,
3. deterministic compile/execute orchestration and diagnostics surfacing.

## Boundary Decision: Core vs Plugin vs Crate

Chosen split:

1. **Core engine**: GPU lifecycle, frame submit orchestration, render scheduler wiring.
2. **Plugins**: render declarations + executors + feature frame data.
3. **Separate crate (`render_graph_core`)**: pure graph schema, compiler, validation, dependency/hazard logic.
4. **ECS integration**: all active render state in ECS resources; no hidden renderer-local authority for graph declarations.

Rationale:

1. Keeps hardware/backend control centralized.
2. Keeps feature ownership decentralized.
3. Keeps graph compiler testable and backend-independent.

## Planned Breaking Changes

1. Remove/replace `PipelineKey`-centric feature pipeline authoring.
2. Remove/replace `PassSlot` compatibility routing as the primary authoring surface.
3. Remove fixed static executor table in renderer.
4. Replace scene render append enums with id-based descriptors.
5. Move render graph state authority from ad-hoc runtime fields to ECS resources.
6. Update `Gfx`/renderer APIs to consume compiled plans and resource-driven executor bindings.
7. Update examples and docs to self-owned feature ids (`sdf.*`, etc.).

## API Shape (Target Contract)

Design goal:

1. Minimize user boilerplate for custom renderers.
2. Keep authoring data-driven and ECS-first.
3. Keep engine internals responsible for compile/prepare/encode orchestration.
4. Use typed builder API as the primary authoring surface.

### Terminology (Renamed)

Use feature-centric naming instead of owner-centric naming in new APIs:

1. `RenderOwnerRegistration` -> `RenderFeatureGraphSpec`
2. `RenderOwnerId` -> `RenderFeatureId`
3. `upsert_owner(...)` -> `register_feature_graph(...)` / `replace_feature_graph(...)`

### ID Newtypes

Use strongly-typed ids (all `Clone + Eq + Hash + serde`):

1. `RenderFeatureId`
2. `RenderResourceId`
3. `RenderPipelineId`
4. `RenderPassId`
5. `RenderPassExecutorId`

### ECS Resources (Authoritative State)

1. `RenderGraphSpecsResource`
   - feature graph specs (resources/pipelines/passes)
   - revision tracking
2. `RenderExecutorRegistryResource`
   - `RenderPassExecutorId -> Box<dyn RenderPassExecutor>`
3. `RenderCompiledPlanResource`
   - compiled execution order + resolved bindings
   - compile cache key/revision
4. `RenderDiagnosticsResource`
   - structured compile/validation diagnostics
5. `RenderFrameDataResource`
   - per-frame extracted data consumed by executors

### Declaration DTOs

1. `RenderFeatureGraphSpecDto`
2. `RenderResourceDeclDto`
3. `RenderPipelineDeclDto`
4. `RenderPassDeclDto`
5. `RenderPipelineRefDto` (`named`, `inline`, etc. as needed)

All DTOs must support:

1. schema `version`,
2. deterministic validation errors,
3. conversion to typed runtime declarations.

### Executor Trait

```rust
pub trait RenderPassExecutor: Send + Sync {
    fn id(&self) -> RenderPassExecutorId;
    fn prepare(&mut self, ctx: &mut RenderPrepareContext) -> anyhow::Result<()>;
    fn encode(&mut self, ctx: &mut RenderEncodeContext) -> anyhow::Result<()>;
}
```

Constraints:

1. `prepare` may read ECS frame data and stage GPU resources.
2. `encode` only records commands using prepared state + resolved targets.
3. No pass-specific hardcoded dispatch in core renderer.

### Low-Boilerplate Plugin API

Primary API: typed builder.

```rust
let spec = RenderFeatureGraphSpec::builder("sdf_renderer")
    .resource("sdf.params")
    .resource("sdf.color")
    .resource("surface.color")
    .pipeline_compute("sdf.compute.raymarch", "assets/shaders/sdf_compute_3d_example.wgsl")
    .pipeline_render_builtin("sdf.compose.fullscreen", "compose.fullscreen")
    .compute_pass("sdf.compute")
        .pipeline("sdf.compute.raymarch")
        .executor("sdf.compute")
        .reads(["sdf.params"])
        .writes(["sdf.color"])
        .finish()
    .render_pass("sdf.compose")
        .pipeline("sdf.compose.fullscreen")
        .executor("sdf.compose")
        .reads(["sdf.color"])
        .writes(["surface.color"])
        .depends_on(["sdf.compute"])
        .finish()
    .build()?;

render_graph_specs.register_feature_graph(spec);
render_executor_registry.register("sdf.compute", SdfComputeExecutor);
render_executor_registry.register("sdf.compose", SdfComposeExecutor);
```

Optional helper API (example convenience only):

1. load + validate config files,
2. apply input mappings,
3. register specs into `RenderGraphSpecsResource`,
4. register executors into `RenderExecutorRegistryResource`,
5. emit concise diagnostics summary.

### Minimal Direct API (without helper)

1. `render_graph_specs.register_feature_graph(spec)`
2. `render_graph_specs.replace_feature_graph(feature_id, spec)`
3. `render_executor_registry.register(executor_id, executor)`
4. `render_graph_specs.remove_feature_graph(feature_id)` for teardown/hot-reload.

### Config Apply Model (SDF Rule)

1. SDF setup stays direct: `load -> validate -> apply`.
2. No event queue is required for initial SDF configuration path.
3. Event system remains optional for hot-reload orchestration later.

### `sdf.params` Source of Truth

`sdf.params` is a logical render resource id. The data source should be:

1. `sdf_params.ron` (authoring/tuning source), and/or
2. typed Rust defaults struct for fallback/validation.

Required runtime flow:

1. parse into typed `SdfParams` struct,
2. write/update `sdf.params` frame data resource,
3. compute pass reads `sdf.params` by id.

### Alternatives Considered (API)

1. Raw `wgpu` descriptors as primary plugin authoring API.
   - Rejected for primary surface: too much backend leakage and high boilerplate for feature authors.
2. Pure RON-only declarative API.
   - Rejected for primary surface: weaker compile-time guarantees and poorer IDE discoverability.
3. Chosen: typed builder API as primary, with optional RON import.
   - Keeps authoring ergonomic and type-safe while supporting config-driven workflows.

## Full Execution Plan

## Phase 0: Contract Freeze and Migration Baseline

Goals:

1. Freeze the target State C contracts.
2. Prevent accidental expansion of legacy enum APIs.

Deliverables:

1. Architectural contract section in docs for id model + resource ownership.
2. Migration inventory of legacy APIs to remove.
3. Baseline regression list for render correctness and startup behavior.

Exit criteria:

1. State C contracts are explicit and approved.
2. Legacy API removal list is complete.

## Phase 1: ECS Render Runtime Skeleton

Goals:

1. Introduce ECS resources for render runtime state ownership.
2. Route render submit path through ECS-owned render resources.

Deliverables:

1. Render runtime resource set (graph declarations, compile cache, diagnostics, frame plan).
2. Scheduler nodes for extract/compile/prepare/encode flow.
3. Transitional bridge from old runtime fields into new resources.

Exit criteria:

1. Frame render submit consumes ECS render resources as source of truth.
2. Existing smoke tests run with resource-backed state enabled.

## Phase 2: `render_graph_core` Crate Extraction

Goals:

1. Move graph schema and compile/validation logic to a backend-agnostic crate.

Deliverables:

1. New crate with id-based descriptors (`ResourceId`, `PipelineId`, `PassId`, `ExecutorId`).
2. Compiler output type for executable pass plan.
3. Validation diagnostics types with stable error categories.

Exit criteria:

1. Engine renderer uses crate output instead of local ad-hoc compile structures.
2. Unit tests in crate cover ordering, hazards, missing refs, cycles.

## Phase 3: Dynamic Pipeline + Executor Registries

Goals:

1. Replace enum/slot-centric and static-executor behavior with registry resources.

Deliverables:

1. ECS resource for pipeline declarations/factories by `PipelineId`.
2. ECS resource for executor implementations by `ExecutorId`.
3. Generic runtime lookup path for prepare/encode by id.

Exit criteria:

1. Renderer no longer depends on static executor arrays for pass dispatch.
2. Plugin-defined executor ids are sufficient for pass execution.

## Phase 4: Scene and Config Descriptor Migration

Goals:

1. Move scene/example render declarations to id-based config DTOs.

Deliverables:

1. Scene manifest render descriptors updated to id model (breaking format change allowed).
2. SDF example config files:
   - `sdf_params.ron`
   - `input_bindings.ron`
   - `render_graph.ron`
3. Direct (non-event) config load -> validate -> apply flow for SDF.

Exit criteria:

1. SDF render path is self-owned (`sdf.*` ids) with no required `world_*` naming.
2. Invalid configs emit concise startup diagnostics.

## Phase 5: Built-in Feature Migration

Goals:

1. Re-register built-in world/ui/mesh features through the same id-based ECS resources.

Deliverables:

1. Built-in feature plugins publish declarations and executors via ECS resources.
2. Core renderer executes compiled plans only.

Exit criteria:

1. No built-in feature requires hardcoded pass branches in core render loop.
2. Frame output remains acceptable for gameplay and UI paths.

## Phase 6: Legacy API Removal (Hard Break)

Goals:

1. Delete compatibility surfaces and enforce State C as the only path.

Deliverables:

1. Remove deprecated enum/slot authoring APIs.
2. Remove legacy scene render append enum parsing.
3. Remove old bridging adapters.
4. Update docs/examples/tests to only show State C usage.

Exit criteria:

1. No production code path depends on removed legacy render authoring types.
2. Build/test suite passes with legacy paths deleted.

## Validation and Test Plan

1. Unit tests:
   - id descriptor validation,
   - compile ordering,
   - hazard/dependency checks,
   - missing pipeline/executor diagnostics.
2. Integration tests:
   - plugin-defined compute+compose path with no core enum edits,
   - SDF example config apply path,
   - scene manifest descriptor load and compile.
3. Regression tests:
   - startup render smoke test,
   - resize path correctness,
   - diagnostics stability and readability.
4. Performance checks:
   - frame compile cache effectiveness,
   - no significant regression in frame encode/submit timing.

## Intended Usage (State C)

As a renderer author:

1. I define resources/pipelines/passes/executors with feature-owned ids.
2. I register declarations and executors via plugin systems writing ECS resources.
3. Engine compiles and runs passes from declared graph state.
4. I can tune/replace feature graph config without core engine edits.

SDF example concrete expectation:

1. SDF setup is direct config apply (not event queue required).
2. `render_graph.ron` owns SDF pass/pipeline/executor ids.
3. Renaming an SDF pass id only requires matching executor binding updates in example/plugin scope.

## Definition of Done

1. Render pipeline state authority is ECS resources end-to-end.
2. Plugins can author independent renderer paths using id-based declarations.
3. Core renderer executes compiled plans generically, without feature hardcoding.
4. Legacy enum/slot authoring path is removed.
5. SDF example demonstrates the final custom-renderer authoring model.
