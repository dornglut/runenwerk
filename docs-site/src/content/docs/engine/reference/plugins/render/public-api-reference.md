---
title: "Render Public API Reference"
description: "Documentation for Render Public API Reference."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-21
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
- `PreparedRenderFrameRequestDiagnostic`
- `PreparedRenderFrameRequestError`
- `PreparedRenderFrameRequestKind`
- `PreparedTargetBinding`
- `RenderProductSurfaceRequest`
- `RenderProductSurfaceRequestBatch`
- `RenderProductSurfaceManifest`
- `RenderProductSurfaceProductBinding`
- `RenderProductSurfaceViewportBinding`
- `RenderProductSurfaceStatus`
- `RenderProductSurfaceStatusKind`
- `RenderProductSurfaceDiagnostic`
- `RenderProductSurfaceDiagnosticKind`
- `RenderProductSurfaceDiagnosticSeverity`
- `RenderProductSurfaceRequestKind`
- `PreparedStateTypeInfo`
- `PreparedShaderSnapshot`
- `PreparedFrameContributions`
- `PreparedFeatureContribution`
- `PreparedFeaturePayload`
- `PreparedFeatureGate`
- `PreparedFeatureContributionDiagnostic`
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
- `RenderProductSurfaceRequest`, `RenderProductSurfaceRequestBatch`, and `RenderProductSurfaceManifest` are return-only helpers for assembling dynamic target descriptors, dynamic uploads, prepared views, prepared flow invocation requests, UI binding intents, and product-surface status. Producers still publish render parts explicitly into ECS resources.
- `PreparedRenderFrameRequestResource::diagnostics()` exposes typed producer-scoped duplicate view/invocation diagnostics.

Current UI note:

- UI uses `PreparedFeaturePayload::Ui` carrying `PreparedUiFrameContribution`.
- Submissions carry owned `UiFrame` payloads plus ordering metadata.

## Feature Contribution APIs

Feature ordering and fallback policies are explicit and live in ECS metadata:

- `RenderFeatureId`
- `RenderFeatureDescriptor`
- `RenderFeatureRegistryResource`
- `RenderFeatureContributionCollectorId`
- `RenderFeatureContributionPayloadKind`
- `RenderFeatureContributionCollectorDescriptor`
- `RenderFeatureContributionResourceRequirement`
- `RenderFeatureContributionCollectorRegistryResource`
- `RenderFeatureContributionContext`
- `PreparedRegisteredFeaturePayload`
- `PreparedRegisteredFeaturePayloadInspection`
- `PreparedRegisteredFeaturePayloadValue`
- `StaticRegisteredFeaturePayload`
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

Contribution collector contract:

- feature contribution collectors run during `RenderPrepare`, never during submit.
- collectors declare prepared resources before reading them.
- registered payloads use typed payload kinds, validation, runtime signatures, and inspection hooks instead of feature-specific central enum variants.
- the registered payload bridge coexists with current `PreparedFeaturePayload` variants during migration.
- scene route contribution now flows through the collector registry as the low-risk compatibility migration path.
- `PreparedFrameContributions::diagnostics()` exposes typed collector diagnostics for missing resources, duplicate collector registration, and invalid registered payloads.

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
  - `compile_flow_plan_checked`
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
  - `CompiledResourceLifetimeWindow`
  - `CompiledResourceAccessKind`
- typed compiler/preflight diagnostics:
  - `RenderExecutionGraphDiagnostic`
  - `RenderExecutionGraphDiagnosticKind`
  - `RenderExecutionGraphDiagnosticSeverity`
  - `RenderExecutionGraphDiagnosticReport`
  - `RenderExecutionGraphCompileError`
  - `RenderExecutionGraphPreparedError`
  - `RenderExecutionGraphPreparedReport`
  - `RenderPreflightValidationConfigResource`
  - `RenderPreparedFramePreflightMode`
  - `RenderPreparedFramePreflightCacheState`
  - `validate_prepared_render_frame`
  - `preflight_prepared_render_frame`
  - `preflight_prepared_render_frame_runtime_guards`
  - `prepared_render_frame_preflight_cache_key`
  - `RenderBackendCapabilityProfile`
  - `RenderBackendCapabilityInspection`
  - `validate_compiled_flow_capabilities`

Contract:

- `compile_flow_plan_checked(...)` wraps static `RenderFlow` validation, resource lifetime window derivation, and backend-neutral capability validation in typed diagnostics.
- `CompiledRenderFlowPlan::resource_lifetime_windows` exposes first/last read/write/use windows derived from compiled pass order.
- `validate_prepared_render_frame(...)` checks a prepared frame against compiled flows before backend encoding: target alias bindings, dynamic target descriptors, sampleability, dispatch preparation, uniform presence, feature gates, history signatures, and capability mismatches.
- Runtime submit uses cached strict prepared-frame preflight by default. Full structural preflight runs on cold cache, structural key changes, failures, or strict mode; cheap runtime guards still run each frame for flow/view/invocation existence, dispatch validity, uniform presence, and history conflicts.
- `RenderPreflightValidationConfigResource` can force strict full preflight every frame. `RUNENWERK_RENDER_STRICT_PREFLIGHT=1` is the local/test override.
- The compiler/preflight diagnostics are render execution diagnostics. They do not own product truth, freshness, authority, fallback legality, rebuild policy, product dependency truth, or residency policy.

## Render Fragment APIs

These APIs are for authored render-flow fragments that merge into normal
`RenderFlow` definitions:

- fragment identity and descriptors:
  - `RenderFragmentPackageId`
  - `RenderFragmentId`
  - `RenderFragmentNamespace`
  - `RenderFragmentPackageDescriptor`
  - `RenderFragmentDescriptor`
  - `RenderFragmentResourceDescriptor`
  - `RenderFragmentPassDescriptor`
- diagnostics and reports:
  - `RenderFragmentDiagnostic`
  - `RenderFragmentDiagnosticKind`
  - `RenderFragmentDiagnosticReport`
  - `RenderFragmentMergeReport`
  - `RenderFragmentMergeError`
- registry and reload:
  - `RenderFragmentRegistryResource`
  - `RenderFragmentPackageRecord`
  - `RenderFragmentPackageStatus`
  - `RenderFragmentHotReloadRequest`
  - `apply_render_fragment_hot_reload`
- merge and inspection:
  - `validate_fragment_package`
  - `merge_fragment_package_into_flow`
  - `inspect_render_fragment_merge_report`
  - `inspect_fragment_pass_provenance`

Contract:

- fragment packages validate before they can affect active rendering;
- local resource and pass labels are namespace-qualified during merge;
- valid fragments merge into normal `RenderFlow` and then pass
  `compile_flow_plan_checked(...)`;
- compute fragment passes may dispatch and write storage textures, but sampled
  texture references on compute passes are rejected until the render API adds a
  first-class `ComputePassBuilder` sampling contract;
- failed reloads keep the last-good active package revision available to the
  registry;
- fragment descriptors do not allocate backend resources, publish prepared
  requests, select products, decide freshness, own fallback legality, own
  authority, own rebuild policy, or own residency policy.

## Runtime and Debug Surfaces

These APIs are for advanced runtime embedding, diagnostics, and inspection.

- renderer/runtime handles:
  - `Renderer`
  - `Gfx`
  - `RendererFrameTimings`
  - `GfxFrameTimings`
- production readiness inspection:
  - `RenderReadinessReport`
  - `RenderReadinessReportRequest`
  - `RenderReadinessSourceReportSummary`
  - `RenderReadinessDiagnostic`
  - `RenderReadinessDiagnosticKind`
  - `RenderReadinessDiagnosticSeverity`
  - `inspect_render_readiness`
  - `RenderReadinessBudgetMeasurements`
  - `RenderReadinessBudgetThreshold`
  - `RenderReadinessBudgetKind`
  - `RenderReadinessBudgetStatus`
  - `RenderReadinessBudgetReport`
  - `evaluate_render_readiness_budgets`
  - `RenderReplayManifest`
  - `RenderReplayArtifactReference`
  - `RenderReplayManifestValidation`
  - `RenderReplayManifestStatus`
  - `validate_render_replay_manifest`
- registries and descriptors:
  - `RenderFlowRegistryResource`
  - `ShaderRegistryResource`
  - `ShaderHandle`
- `RenderResourceDescriptor`
- `RenderDynamicTextureTargetDescriptor`
- `RenderDynamicTextureTargetKey`
- `RenderDynamicTextureRetention`
- `RenderTextureTargetFormat`
- `RenderTextureFormatPolicy`
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
- active renderer execution resolves dynamic target aliases through the render product-surface foundation; avoid cloning flows or suffixing static labels as a substitute.
- native OS multi-window and multi-swapchain presentation is a separate future surface-scoped runtime capability.

Flow-owned color target format contract:

- `RenderFlow::with_color_target(label)` declares a surface-sized, flow-owned color target that resolves to the selected surface format.
- `RenderFlow::with_color_target_exact(label, format)` declares a surface-sized, flow-owned color target with an exact texture format.
- Exact-format targets are for byte-truth product/proof/intermediate data. They are not fixed-size targets; fixed-size exact targets require a separate future API.
- `copy_pass(...)` is a raw transfer. It may copy between identical color formats or color formats equal after removing the sRGB suffix, and it must reject unrelated formats and depth/stencil formats.

Compiler/preflight inspection contract:

- `inspect_compiled_render_flow_plan(...)` summarizes compiled pass/resource counts, resource lifetime windows, compiler diagnostics, and the active backend-neutral capability profile.
- `inspect_render_execution_graph_preflight(...)` summarizes prepared-frame preflight diagnostics for tooling.
- `inspect_render_execution_graph_preflight_with_cache(...)` adds cache mode, cache status, and report source without exposing backend handles.
- `Renderer::last_preflight_report()` exposes the last successful submit preflight report, and `Renderer::last_preflight_cache_state()` exposes whether it came from full validation or cache.
- `RendererFrameTimings` separates `preflight_ms`, `flow_encode_ms`, and `encode_submit_ms` so frame budgets can identify validation cost separately from GPU submission.

Production readiness inspection contract:

- `inspect_render_readiness(...)` aggregates existing prepared-frame, product-surface, graph/preflight, fragment, capture, timing, and budget inspection DTOs. It is a read-only tooling surface, not a renderer-owned product policy layer.
- `evaluate_render_readiness_budgets(...)` reports renderer execution evidence budgets for timing, capture failures, preflight errors, fragment errors, dynamic targets/uploads, and product-surface diagnostics. Budget overruns produce diagnostics; they do not choose product rebuilds or fallback.
- `validate_render_replay_manifest(...)` is fail-closed. It rejects missing capability profiles, prepared-frame digests, artifact paths, formats, and empty artifact sets before replay evidence can be treated as valid.
- Readiness, budget, and replay DTOs must not expose WGPU handles, mutable backend caches, product source objects, app workflow state, or domain-owned product truth.

## Product Surface APIs

These APIs are for dynamic product surfaces, viewport products, material/asset previews, and debug texture viewers:

- `RenderFlow::with_color_target_alias`
- `RenderFlow::with_depth_target_alias`
- `RenderFlow::with_target_alias`
- `PreparedViewFrame::with_history_signature`
- `PreparedFlowInvocationRequest::new`
- `PreparedFlowInvocationRequest::bind_dynamic_texture_alias`
- `PreparedFlowInvocationRequest::bind_surface_color_alias`
- `PreparedFlowInvocationRequest::bind_surface_depth_alias`
- `PreparedFlowInvocationRequest::bind_flow_owned_alias`
- `PreparedFlowInvocationRequest::with_history_signature`
- `PreparedFlowInvocationRequest::with_uniform_override`
- `RenderDynamicTextureTargetDescriptor::color_sampled`
- `RenderDynamicTextureTargetDescriptor::color_attachment_only`
- `RenderDynamicTextureTargetDescriptor::storage_sampled`
- `RenderDynamicTextureTargetDescriptor::depth_sampled`
- `RenderProductSurfaceRequest`
- `RenderProductSurfaceRequestBatch`
- `RenderProductSurfaceManifest`
- `RenderProductSurfaceManifest::with_upload_backed_product_surface_binding`
- `RenderProductSurfaceManifest::diagnostics`
- `RenderProductSurfaceManifest::into_render_parts`
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

- dynamic target descriptors, dynamic upload descriptors, producer-scoped request registry snapshots, prepared views, prepared invocations, UI binding intents, product-surface status, and product-surface diagnostics are public inspection/prepare packet contracts;
- product-surface helpers are return-only authoring helpers and do not publish into ECS resources or infer product truth, freshness, fallback legality, authority, rebuild policy, drawing semantics, field semantics, or residency policy;
- upload-backed product surfaces must opt in through `with_upload_backed_product_surface_binding(...)`; the manifest then reports `missing_upload` if a producer declares a sampled product-surface binding without a matching upload descriptor;
- viewport surface bindings and generic product-surface bindings are backend-neutral dynamic texture sources and are validated for declared target traceability and sampleability;
- prepared-frame request validation reports typed duplicate view/invocation diagnostics through `PreparedRenderFrameRequestDiagnostic`;
- renderer-owned dynamic target cache allocation and target-alias pass execution are implemented foundation behavior and should not be faked with editor-specific flow ids.

## Compatibility Surface

`RenderFrameDataRegistry` remains public for projection helper compatibility and tests.

It is not part of the active runtime submission path and should not be used as a substitute for `PreparedRenderFrame`.
