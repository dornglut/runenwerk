# Render Graph Plugin Refactor Plan

## Date

2026-02-24

## Purpose

Refactor rendering so compute/render pipelines and pass responsibilities are plugin-owned and data-driven, rather than hard-wired into engine core paths such as `world_compute`.

## Status Snapshot (2026-02-24)

Implemented:

1. Typed feature graph declarations and ids are available in engine code:
   - `RenderFeatureGraphSpec`
   - `RenderFeatureId`, `RenderResourceId`, `RenderPipelineId`, `RenderPassId`, `RenderPassExecutorId`
2. Feature-centric registry bridge methods are available:
   - `register_feature_graph`
   - `replace_feature_graph`
   - `remove_feature_graph`
3. `sdf_renderer` example now registers a graph via typed builder with SDF-owned pass ids (`sdf.compute`, `sdf.compose`).
4. Runtime executor registry exists with builtin + custom trait registration:
   - `RenderPassExecutorRegistryResource`
   - `BuiltinRenderPassExecutor`
   - `RenderPassExecutor`
5. `sdf_renderer` maps `executor_bindings` to custom executors; SDF compute/compose are feature-owned implementations.

Still pending:

1. Runtime dynamic pipeline registry by id.
2. Full ECS-owned compile/runtime render state.
3. Broader custom executor context for plugin-owned GPU resources beyond current bridge-level context.

## Problem Statement

Current render flow mixes responsibilities across core render code and individual features:

1. Core renderer contains hardcoded world compute variants and shader wiring.
2. `FramePassSlot` and `PipelineKey` are centrally enumerated, so adding a new feature pipeline requires core edits.
3. Feature responsibilities (example: `sdf_renderer`) cannot be fully isolated in one plugin.
4. Graph structure supports mixed compute/render ordering, but registration is not yet plugin-driven.

## Target Outcome

A plugin should be able to define and own all render responsibilities for a feature, including:

1. resources it reads/writes,
2. pass graph nodes,
3. pipeline declarations,
4. per-frame data preparation,
5. pass encode behavior,
6. diagnostics labels/status.

This includes defining an SDF compute path fully inside an `SdfRenderPlugin`, without hardcoding its pipeline variants into generic engine internals.

## Design Principles

1. ECS/data-driven first: runtime render state is stored in resources/components and consumed by systems.
2. Plugin ownership: feature plugins register graph/pipelines/executors; core renderer only orchestrates execution.
3. Stable boundaries: core owns swapchain, command encoder lifecycle, pass ordering execution, and validation.
4. Deterministic graph execution: explicit dependencies + inferred resource barriers remain deterministic.
5. Incremental migration: keep existing world pipeline working while introducing plugin API.

## Proposed Architecture

### 1. Render Graph Registry (data-driven)

Introduce a `RenderGraphSpecsResource` in ECS/runtime containing declarative feature graph specs:

- `RenderResourceDescriptor { id, kind }`
- `RenderPipelineDescriptor { id, kind, shader refs, layout signature }`
- `RenderPassDescriptor { id, kind, reads, writes, depends_on, executor_id, pipeline_id }`

All feature plugins register/replace their graph specs at setup.

Terminology direction:

1. Prefer feature-centric naming (`RenderFeatureGraphSpec`, `RenderFeatureId`).
2. Keep owner-centric terms only as legacy compatibility aliases during migration.

### 2. Pass Executor Registry (behavior binding)

Add `RenderPassExecutorRegistry` mapping `RenderPassExecutorId -> dyn RenderPassExecutor`.

Executor trait should be plugin-friendly:

- `prepare(&self, ctx)`
- `encode(&self, ctx)`

Core renderer invokes by id; it does not know feature-specific pass logic.

Authoring API direction:

1. Primary: typed builder API (`RenderFeatureGraphSpec::builder(...)`).
2. Secondary: RON DTO import that converts into the same typed spec model.

### 3. Pipeline Registry V2 (string/resource-keyed)

Replace hardcoded enum-only `PipelineKey` path with key-based identifiers:

- `PipelineId` (string/newtype)
- compatibility layer from existing enums during migration

Goal: plugins can register pipelines without touching core enums.

### 4. Frame Compilation Step

Per frame (or on invalidation), compiler builds executable plan:

1. collect active plugin graph descriptors,
2. resolve resource edges and dependencies,
3. validate missing pipeline/executor ids,
4. produce sorted pass list and diagnostics.

Compilation can be cached by graph revision hash.

### 5. Runtime Data Access

Feature data should live in ECS resources (example `SdfRenderStateResource`), not in renderer-specific ad-hoc structs.

Per-frame systems update these resources before render submit.

`sdf.params` note:

1. `sdf.params` is a logical render resource id.
2. Data should originate from `sdf_params.ron` and/or typed defaults struct.
3. Runtime normalizes it into typed frame data before pass prepare/encode.

## Plugin Model (SDF Example)

Target plugin responsibilities for `SdfRenderPlugin`:

1. Register resources: `sdf.color`, `sdf.params`, `surface.color` (or plugin-owned output target name).
2. Register pipeline(s): `sdf.compute.raymarch`, `sdf.compose.fullscreen` (feature ids, not core world ids).
3. Register passes:
   - `sdf.compute` (compute, writes `sdf.color`)
   - `sdf.compose` (render, reads `sdf.color`, writes `surface.color`)
4. Provide executor implementation for both passes.
5. Own input/camera/update systems that produce `SdfRenderStateResource`.

This keeps SDF feature code in plugin scope and out of generic world-compute hardwiring.

`sdf.params` source rule:

1. `sdf.params` is a logical render resource id.
2. Data comes from `sdf_params.ron` and/or typed defaults struct.
3. Runtime normalizes this into typed frame data before pass execution.

Explicit goal for the example:

- Do not require reuse of `world_compute` / `world_compose` pass ids for SDF features.
- Engine should provide generic renderer-building abstractions so users can author their own renderer paths.

Current bridge note:

1. Feature graph uses feature-owned executor ids (for example `sdf.compute`).
2. Example setup registers custom executors via `register_custom`.
3. SDF compute/compose use feature-owned GPU pipelines/resources in the example plugin.
4. UI composite also runs through a custom executor path (using render-context UI dispatch helper).

## Migration Plan

### Phase 0: Foundations (no behavior change)

1. Add key-based ids alongside enum ids.
2. Add registry resources and validation types.
3. Add executor registry abstraction.

### Phase 1: Bridge Existing World Path

1. Register current world compute/compose/ui/mesh passes through new registry.
2. Keep old enums as compatibility aliases.
3. Ensure frame output parity with current renderer.

### Phase 2: First Feature Plugin Ownership

1. Move `sdf_renderer` render path to plugin-owned registrations/executors.
2. Remove SDF-specific branches from core renderer internals.
3. Validate no regressions in existing gameplay world render path.

### Phase 3: Generalize and Clean Up

1. Migrate remaining hardcoded pipelines to plugin registrations.
2. Remove legacy enum-only pipeline selection code.
3. Add tooling/diagnostics dump for active graph and pass timings.

## Data Structures (target shape)

```rust
pub struct RenderGraphSpecsResource {
    pub features: Vec<RenderFeatureGraphSpec>,
    pub revision: u64,
}

pub struct RenderFeatureGraphSpec {
    pub feature: RenderFeatureId,
    pub resources: Vec<RenderResourceDescriptor>,
    pub pipelines: Vec<RenderPipelineDescriptor>,
    pub passes: Vec<RenderPassDescriptor>,
}

pub struct RenderPassDescriptor {
    pub id: RenderPassId,
    pub kind: PassKind,
    pub reads: Vec<RenderResourceId>,
    pub writes: Vec<RenderResourceId>,
    pub depends_on: Vec<RenderPassId>,
    pub pipeline_id: PipelineId,
    pub executor_id: ExecutorId,
}
```

## Validation Rules

1. Every pass references existing `pipeline_id` and `executor_id`.
2. Resource hazards are topologically resolvable.
3. Cycles produce clear compile diagnostics.
4. Missing plugin registrations fail early with actionable errors.

## Testing Plan

1. Unit tests for graph registry merge and validation.
2. Unit tests for pass ordering/barrier inference.
3. Integration test: plugin registers compute+compose path and renders without core enum edits.
4. Integration test: existing world path parity during bridge phase.
5. Regression test: removing plugin removes its passes/resources cleanly.

## Acceptance Criteria

1. New feature pipeline can be added without editing central `PipelineKey` enum.
2. SDF render path is fully plugin-owned (registration + execution + state) with feature-defined ids.
3. Core renderer executes generic compiled graph only.
4. Existing render features remain functional during migration.
5. Graph compile/validation errors are visible and debuggable.

## Risks

1. Dynamic registry complexity can hide ordering bugs if diagnostics are weak.
2. Plugin lifecycle ordering must be explicit to avoid missing registrations.
3. Pipeline caching invalidation must track shader/layout changes correctly.

## Open Decisions

1. Whether graph compilation occurs every frame or only on revision changes.
2. Whether to support runtime scene-driven pass append at same layer as plugin registrations or as overlay on top.

## Recommendation

Historical recommendation (incremental track):

1. Proceed with Phase 0 + Phase 1 first, then migrate `sdf_renderer` as the proving feature in Phase 2.

Active project decision:

1. The canonical plan now lives in `docs/project/execution-plan.md`.
2. Project is explicitly switching to State C (fully ECS-driven render pipeline state, breaking changes allowed).
