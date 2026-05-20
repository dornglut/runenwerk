---
title: "Render Plugin Advanced Guide"
description: "Documentation for Render Plugin Advanced Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-21
---

# Render Plugin Advanced Guide

## Advanced Surfaces

- typed explicit resource handles (`storage_array` + `bind_storage`)
- ergonomic ping-pong binding (`double_buffer_storage_array` + `bind_ping_pong_storage`)
- state-projected uniforms and dispatch
- transient/persistent/imported resource lifetime modeling
- inspection surfaces under `engine::plugins::render::inspect`

## Validation and Safety

Use:

- `RenderFlow::validate()` for chainable validation
- `RenderFlow::validation_report()` for inspectable contract checks

Validation catches:

- duplicate and unknown IDs
- pass-shape errors
- dependency cycles
- invalid resource usage for pass bindings

## Contract Inspection

Use:

- `flow.graph()` for pass/resource declarations
- `flow.project_uniforms(...)` for frame-level uniform projection checks
- `dump_flow_graph(...)`, `inspect_resources(...)`, `inspect_texture_resources(...)`, and `summarize_pass_timings(...)` for runtime diagnostics
- `inspect_prepared_render_frame(...)` for prepared views, per-flow invocations, target alias bindings, dynamic target descriptors, and history signatures

These APIs keep the graph explicit and testable while keeping common-path declaration compact.

## Execution-Compile Surfaces

Advanced integrations can inspect compile outputs in two layers:

- graph compile (`CompiledRenderFlowPlan` with `pass_order` and `resources`)
- execution compile (`CompiledFlowExecutionPlan` and `CompiledPassExecutionPlan`)

Execution compile metadata includes:

- bind group layout entries (`CompiledBindGroupPlan`, `CompiledBindingEntry`)
- uniform/storage binding order (`CompiledPassBindings`, `CompiledStorageBinding`)
- storage access mode (`CompiledStorageAccess`)
- target and draw buffer plans (`CompiledTargetPlan`, `CompiledDrawBufferPlan`)
- dispatch shape (`CompiledDispatchPlan`)
- imported/builtin resources (`CompiledResourceRef`, `CompiledBuiltinImport`)

This split keeps validation/graph inspection explicit while enabling renderer execution paths to consume execution-ready metadata.

## Runtime Frame Boundary APIs

Prepare/submit boundary types are public for inspection and integration:

- `PreparedRenderFrame`
- `PreparedRenderFrameResource`
- `PreparedFrameContext`
- `PreparedViewFrame`
- `PreparedFlowInputs`
- `PreparedFlowInvocation`
- `PreparedTargetBinding`
- `PreparedRenderFrameRequestResource`
- `RenderDynamicTextureTargetDescriptor`
- `PreparedSurfaceInfo`
- `PreparedShaderSnapshot`
- `PreparedFrameContributions`
- `PreparedFeatureContribution`
- `PreparedFeaturePayload`
- `PreparedUiFeatureContribution`
- `PreparedSceneRouteContribution`
- `FeatureContributionStatus`
- `FeatureFallbackPolicy`

`RenderFrameDataRegistry` remains available for projection helper compatibility and tests, but it is not part of the active runtime submit/render path.

Prepared product-surface packets:

- publish offscreen product views through producer-scoped `PreparedRenderFrameRequestResource` contributions;
- bind target aliases per `PreparedFlowInvocationRequest`;
- publish dynamic targets through producer-scoped `RenderDynamicTextureTargetRequestRegistryResource` contributions;
- inspect the frozen submit packet with `inspect_prepared_render_frame(...)`.

## Feature Fallback Contract

Prepare resolves contribution health for each feature and packages status into `PreparedFrameContributions`:

- `Ready`
- `Stale`
- `Disabled`
- `Missing`

Fallback policy is explicit:

- `ReuseLastGood`
- `EmptyContribution`
- `SkipFeaturePasses` (default)
- `FailFrame`

Submit/runtime does not re-query ECS for missing feature payloads.

## Material Specialization Contract

- Compile-time specialization is reserved for pipeline-shaping state.
- Runtime parameter values remain bind/update payload data.
- Pipeline key ownership is split by responsibility:
  - core render owns the canonical key type (`FlowPassPipelineKey`)
  - material feature contributes a specialization fragment hash folded into that key

## Multi-view Execution Contract

- Prepare carries view containers and execution plans carry `CompiledViewMask`.
- Active runtime execution supports prepared offscreen product views and per-flow prepared invocations through the product-surface path.
- View-scoped pass subsets are expressed through execution view masks and pass scoping APIs; product-surface prepared views are active, while broader native OS multi-window and multi-swapchain presentation remains future work.
