---
title: "Render Public API Reference"
description: "Documentation for Render Public API Reference."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-07
---

# Render Public API Reference

This page maps the `engine::plugins::render` public API to its intended usage level.

## Common Path APIs

These are the APIs most users should start with.

- `RenderPlugin`
- `RenderFlow`
- pass builders:
  - `ComputePassBuilder`
  - `FullscreenPassBuilder`
  - `GraphicsPassBuilder`
  - `CopyPassBuilder`
  - `PresentPassBuilder`
  - `BuiltinUiCompositePassBuilder`
  - advanced pass-feature hook: `ComputePassBuilder::for_feature`, `FullscreenPassBuilder::for_feature`, `GraphicsPassBuilder::for_feature`
- graphics authoring:
  - `RenderVertexBufferLayout`
  - `RenderVertexAttribute`
  - `RenderVertexFormat`
  - `RenderVertexStepMode`
  - `RenderDrawDescriptor`
- handles and IDs:
  - `PassHandle`
  - `RenderFlowId`
  - `RenderPassId`
  - `RenderResourceId`
  - `UniformHandle`
  - `StorageArrayHandle`
  - `DoubleBufferHandle`
- bindings and projection helpers:
  - `PassParamBinding`
  - `ComputeDispatchBinding`
  - `ComputeDispatchDescriptor`
  - `PassUniformProjection`
  - `ProjectedUniformSet`
  - `ProjectedUniformBuffer`
  - `ParamProjectionError`
  - `ParamProjectionErrorKind`
- params derives and traits:
  - `GpuUniform`
  - `GpuStorage`
  - `GpuParams`
  - `GpuUniformField`
  - `GpuBoolU32`
  - `ToGpuValue`
  - `write_uniform_field`
  - `align_up_const`

Use these guides for the common path:

- `render-flow-usage-guide.md`
- `gpu-params-guide.md`
- `usage-guide.md`

## Frame Boundary APIs

These are advanced runtime boundary types produced by `RenderPrepare` and consumed by `RenderSubmit`.

- `PreparedRenderFrame`
- `PreparedRenderFrameResource`
- `PreparedFrameContext`
- `PreparedSurfaceInfo`
- `PreparedViewFrame`
- `PreparedFlowInputs`
- `PreparedFlowInvocation`
- `PreparedFlowInvocationId`
- `PreparedFlowInvocationRequest`
- `PreparedRenderFrameRequestResource`
- `PreparedTargetBinding`
- `PreparedStateTypeInfo`
- `PreparedShaderSnapshot`
- `PreparedFrameContributions`
- `PreparedFeatureContribution`
- `PreparedFeaturePayload`
- `PreparedFeatureGate`
- `PreparedUiFeatureContribution`
- `PreparedSceneRouteContribution`
- `PreparedDrawFeatureContribution`
- `PreparedDrawBatch`
- `PreparedMaterialFeatureContribution`
- `PreparedMaterialInstanceInput`
- `PreparedDeformationFeatureContribution`
- `PreparedDeformationStream`

Contract:

- `frame_render_prepare_system` publishes one owned `PreparedRenderFrame` per renderable frame.
- `frame_render_submit_system` consumes the prepared frame and does not perform live ECS extraction for flow data.
- `PreparedRenderFrame::views` carries main-surface and offscreen product views.
- `PreparedRenderFrame::flow_invocations` carries per-view/per-product flow invocation requests and target alias bindings.
- `PreparedRenderFrame::dynamic_texture_targets` carries the frame-stable dynamic target descriptor snapshot.

Current UI note:

- UI uses `PreparedFeaturePayload::Ui` carrying `PreparedUiFrameContribution`.
- Submissions carry owned `UiFrame` payloads plus ordering metadata.

## Feature Contribution APIs

Feature ordering and fallback policies are explicit and live in ECS metadata:

- `RenderFeatureId`
- `RenderFeatureDescriptor`
- `RenderFeatureRegistryResource`
- `PreparedDrawFeatureResource`
- `PreparedMaterialFeatureResource`
- `PreparedDeformationFeatureResource`
- `FeatureContributionStatus` (`Ready | Stale | Disabled | Missing`)
- `FeatureFallbackPolicy` (`ReuseLastGood | EmptyContribution | SkipFeaturePasses | FailFrame`)

Built-in feature IDs:

- `SCENE_ROUTE_RENDER_FEATURE_ID`
- `UI_RENDER_FEATURE_ID`
- `WORLD_DRAW_RENDER_FEATURE_ID`
- `MATERIAL_RENDER_FEATURE_ID`
- `DEFORMATION_RENDER_FEATURE_ID`

## Graph and Execution Compilation APIs

These APIs expose graph validation and execution-ready compilation metadata.

- graph compile and validation:
  - `RenderFlowGraph`
  - `RenderPassNode`
  - `RenderPassKind`
  - `RenderShaderReference`
  - `FlowValidationReport`
  - `RenderFlowValidationError`
  - `validate_flow_graph`
  - `compile_flow_plan`
  - `CompiledRenderFlowPlan`
  - `CompiledPassDescriptor`
  - `CompiledComputePass`
  - `CompiledFullscreenPass`
  - `CompiledGraphicsPass`
  - `CompiledCopyPass`
  - `CompiledPresentPass`
  - `CompiledUiCompositePass`
- execution compile types:
  - `CompiledFlowExecutionPlan`
  - `CompiledPassExecutionPlan`
  - `CompiledComputeExecutionPlan`
  - `CompiledRasterExecutionPlan`
  - `CompiledCopyExecutionPlan`
  - `CompiledPresentExecutionPlan`
  - `CompiledUiCompositeExecutionPlan`
  - `CompiledPassBindings`
  - `CompiledBindGroupPlan`
  - `CompiledBindingEntry`
  - `CompiledDispatchPlan`
  - `CompiledStorageAccess`
  - `CompiledStorageBinding`
  - `CompiledTargetPlan`
  - `CompiledDrawBufferPlan`
  - `CompiledVertexBufferBinding`
  - `CompiledVertexBufferLayout`
  - `CompiledDrawPlan`
  - `CompiledResourceRef`
  - `CompiledBuiltinImport`
  - `CompiledStateRequirement`

## Runtime and Debug Surfaces

These APIs are for advanced runtime embedding, diagnostics, and inspection.

- renderer/runtime handles:
  - `Renderer`
  - `Gfx`
  - `RendererFrameTimings`
  - `GfxFrameTimings`
- registries and descriptors:
  - `RenderFlowRegistryResource`
  - `ShaderRegistryResource`
  - `ShaderHandle`
- `RenderResourceDescriptor`
- `RenderDynamicTextureTargetDescriptor`
- `RenderDynamicTextureTargetKey`
- `RenderDynamicTextureRetention`
- `RenderTextureTargetFormat`
- `RenderTextureTargetUsage`
- `RenderTextureSampleMode`
- `ImportedTextureSemantic`
  - `ImportedBufferSemantic`
  - `ResourceLifetime`
  - `TransientAliasAssignment`
  - `TransientAliasCandidate`
  - `TransientAliasSlot`
  - `TransientResourceWindow`

Typed imported-resource contract:

- Prefer typed imports:
  - `RenderResourceDescriptor::imported_surface_color`
  - `RenderResourceDescriptor::imported_history_texture`
  - `RenderResourceDescriptor::imported_history_buffer`
- `RenderResourceDescriptor::imported_surface_depth` remains a typed declaration compatibility API, but it is not accepted as a runtime graphics depth attachment. Use flow-owned `RenderFlow::with_depth_target(...)` and `GraphicsPassBuilder::depth_target(...)` for executable graphics depth.
- `imported_texture` / `imported_buffer` remain compatibility constructors and compile to `External` semantics.
- Active runtime flow validation rejects `External` imports.

Pipeline-key specialization/runtime contract:

- `FlowPassPipelineKey` is core-render-owned and includes shader/layout/target/view/runtime signatures.
- material features contribute specialization fragment hashes folded into the core key.

Current multi-view contract:

- prepared frame packets can carry main-surface and offscreen product views plus per-flow invocations.
- active renderer execution for dynamic target aliases remains part of the render product surface bundle; avoid cloning flows or suffixing static labels as a substitute.

## Product Surface APIs

These APIs are for dynamic product surfaces, viewport products, material/asset previews, and debug texture viewers:

- `RenderFlow::with_color_target_alias`
- `RenderFlow::with_depth_target_alias`
- `RenderFlow::with_target_alias`
- pass view scoping:
  - `ComputePassBuilder::main_surface_only`
  - `ComputePassBuilder::offscreen_products_only`
  - `FullscreenPassBuilder::main_surface_only`
  - `FullscreenPassBuilder::offscreen_products_only`
  - `GraphicsPassBuilder::main_surface_only`
  - `GraphicsPassBuilder::offscreen_products_only`
  - `CopyPassBuilder::main_surface_only`
  - `PresentPassBuilder::main_surface_only`
- texture binding helpers:
  - `RenderFlow::with_sampled_texture`
  - `RenderFlow::with_storage_texture`
  - `ComputePassBuilder::write_texture`
  - `FullscreenPassBuilder::sample_texture`
  - `FullscreenPassBuilder::write_texture`
  - `GraphicsPassBuilder::sample_texture`
  - `GraphicsPassBuilder::write_texture`

Current status:

- dynamic target descriptors, producer-scoped request registry snapshots, prepared views, and prepared invocations are public inspection/prepare packet contracts;
- renderer-owned dynamic target cache allocation and target-alias pass execution are implemented foundation behavior and should not be faked with editor-specific flow ids.

## Compatibility Surface

`RenderFrameDataRegistry` remains public for projection helper compatibility and tests.

It is not part of the active runtime submission path and should not be used as a substitute for `PreparedRenderFrame`.
