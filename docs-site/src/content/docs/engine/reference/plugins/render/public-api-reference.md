---
title: "Render Public API Reference"
description: "Documentation for Render Public API Reference."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-22
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
  - `RenderDrawSource`
  - `DrawIndirectArgs`
  - `DrawIndexedIndirectArgs`
  - `IndirectDrawArgsBuffer`
  - `RenderRasterState`
  - `RenderPrimitiveTopology`
  - `RenderBlendMode`
  - `RenderDepthPolicy`
  - `RenderCullMode`
- procedural authoring:
  - `ProceduralPassBuilder`
  - `ProceduralPassDescriptor`
  - `ProceduralVisualDescriptor`
  - `ProceduralBufferBinding`
  - `ProceduralIndexBuffer`
  - `ProceduralRenderPolicy`
  - `ProceduralTargetDescriptor`
  - `ProceduralSdf2dImpostorDescriptor`
  - `ProceduralValidationError`
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

## Procedural Population APIs

These APIs are renderer-owned infrastructure for large bounded procedural
populations. They describe derived GPU execution data, not gameplay truth or
product ownership.

- procedural pass authoring:
  - `RenderFlow::procedural_pass`
  - `RenderFlow::procedural_pass_builder`
  - `ProceduralPassBuilder::uniform_from_state`
  - `ProceduralPassBuilder::uniform_from_state_with_surface`
  - `ProceduralPassBuilder::draw_indirect`
  - `ProceduralPassBuilder::draw_indirect_with_offset`
- direct and indirect draw sources:
  - `GraphicsPassBuilder::draw`
  - `GraphicsPassBuilder::draw_with_offsets`
  - `GraphicsPassBuilder::draw_indirect`
  - `GraphicsPassBuilder::draw_indirect_with_offsets`
  - `RenderDrawSource`
  - `DrawIndirectArgs`
  - `DrawIndexedIndirectArgs`
  - `IndirectDrawArgsBuffer`
- GPU primitive contracts:
  - `U32Counter`
  - `U32ScanElement`
  - `PrefixScanMode`
  - `CounterResetDescriptor`
  - `U32PrefixScanDescriptor`
  - `U32ScatterDescriptor`
  - `IndirectDrawArgsGenerationDescriptor`
  - `GeneratedIndirectDrawArgs`
  - `GpuPrimitiveExecutionPlan`
  - `GpuPrimitiveStep`
  - `GpuPrimitiveResourceAccess`
- bounded population support:
  - `BoundedUniformGrid2dConfig`
  - `BoundedUniformGrid2dBuildPlan`
  - `BoundedUniformGrid2dResources`
  - `BoundedUniformGrid2dStage`
  - `BoundedUniformGrid2dStagePlan`

Contract:

- `RenderFlow::procedural_pass(...)` remains the simple direct draw path for
  procedural visuals with a fixed instance count.
- `RenderFlow::procedural_pass_builder(...)` is the advanced path for
  procedural-owned uniform and indirect draw authoring. It lowers internally and
  does not expose `GraphicsPassBuilder` as the procedural extension surface.
- `GraphicsPassBuilder::draw(...)` and `draw_with_offsets(...)` author direct
  draw sources. `draw_indirect(...)` and `draw_indirect_with_offsets(...)`
  author typed indirect draw sources with renderer-owned argument buffer
  expectations.
- GPU primitive descriptors validate labels, real storage-array lengths,
  capacity, and aliasing. Invalid capacity is a diagnostic, not silent drift.
- `BoundedUniformGrid2dBuildPlan` records the canonical clear counts, count
  cells, scan counts, reset cursors, scatter sorted indices, neighbor
  simulation, and publish/draw stage order. Cell resources are total-count-sized
  and sorted-index resources are agent-count-sized.
- Spatial hash and chunked unbounded population support are not part of this
  bounded-grid API. They require a later accepted milestone.

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
- `PreparedParticleVfxFeatureResource`
- `PreparedParticleVfxFeatureContribution`
- `PreparedParticleVfxBatch`
- `PreparedParticleVfxVisualKind`
- `PreparedParticleVfxSortingMode`
- `PreparedParticleVfxTransparencyMode`
- `PreparedParticleVfxTemporalInput`
- `PreparedParticleVfxBatchState`
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
- `PreparedParticleVfxFeatureResource`
- `PreparedDeformationFeatureResource`
- `FeatureContributionStatus` (`Ready | Stale | Disabled | Missing`)
- `FeatureFallbackPolicy` (`ReuseLastGood | EmptyContribution | SkipFeaturePasses | FailFrame`)

Built-in feature IDs:

- `SCENE_ROUTE_RENDER_FEATURE_ID`
- `UI_RENDER_FEATURE_ID`
- `WORLD_DRAW_RENDER_FEATURE_ID`
- `MATERIAL_RENDER_FEATURE_ID`
- `PARTICLE_VFX_RENDER_FEATURE_ID`
- `DEFORMATION_RENDER_FEATURE_ID`

Contribution collector contract:

- feature contribution collectors run during `RenderPrepare`, never during submit.
- collectors declare prepared resources before reading them.
- registered payloads use typed payload kinds, validation, runtime signatures, and inspection hooks instead of feature-specific central enum variants.
- the registered payload bridge coexists with current `PreparedFeaturePayload` variants during migration.
- scene route contribution now flows through the collector registry as the low-risk compatibility migration path.
- particle/VFX/trail/decal contributions use `PreparedParticleVfxFeatureResource`
  plus the registered `particle.vfx.prepared` payload kind. Product domains
  prepare batches, sorting/transparency intent, temporal input declarations,
  residency requests, and fallback/unsupported/over-budget batch states; the
  renderer only orders, inspects, and diagnoses the prepared contribution.
- `PreparedFrameContributions::diagnostics()` exposes typed collector diagnostics for missing resources, duplicate collector registration, and invalid registered payloads.

## Graph and Execution Compilation APIs

These APIs expose graph validation and execution-ready compilation metadata.

- graph compile and validation:
  - `RenderFlowGraph`
  - `RenderPassNode`
  - `RenderPassKind`
  - `RenderPassShapeIntent`
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
  - `CompiledRasterState`
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
  - `diagnose_compiled_pass_shapes`

Contract:

- `compile_flow_plan_checked(...)` wraps static `RenderFlow` validation, resource lifetime window derivation, pass-shape guard diagnostics, and backend-neutral capability validation in typed diagnostics.
- `CompiledRenderFlowPlan::resource_lifetime_windows` exposes first/last read/write/use windows derived from compiled pass order.
- `validate_prepared_render_frame(...)` checks a prepared frame against compiled flows before backend encoding: target alias bindings, dynamic target descriptors, sampleability, dispatch preparation, uniform presence, feature gates, history signatures, and capability mismatches.
- Pass-shape guards reject fullscreen-style generated graphics multiplied by instance count unless `GraphicsPassBuilder::allow_instanced_fullscreen(...)` records explicit bounded author intent. Diagnostics use `FullscreenInstancedWork`, `AmbiguousProceduralShape`, and `InvalidPassShapeIntent`.
- `RenderFlow::procedural_pass(...)` builds normal graphics passes from renderer-owned procedural descriptors. Mesh sprites, quad sprites, and local 2D SDF impostors use typed storage-backed instance buffers and explicit render policy; the API derives renderer execution resources only and does not own product truth or residency policy.
- `RenderFlow::procedural_pass_builder(...)` is the advanced procedural authoring path for per-pass uniforms, surface-aware uniforms, and typed indirect draw arguments. It lowers internally to graphics and does not expose `GraphicsPassBuilder` as the public procedural extension surface.
- `GraphicsPassBuilder::draw(...)` and `draw_with_offsets(...)` remain the direct draw paths. `draw_indirect(...)` and `draw_indirect_with_offsets(...)` author explicit indirect draw sources using typed renderer-owned argument buffers.
- `engine/examples/boids_render_flow` is the canonical procedural-consumer example: compute updates storage-backed boids through a bounded wrapping uniform grid, a publish pass makes the current buffer available as instance data, `boids.draw` is built with `ProceduralPassDescriptor::local_sdf_2d_impostors(...)` through the procedural builder, and the compose shader shades one aspect-correct local impostor without a fullscreen fragment loop over the whole boid set.
- `cargo run -p engine --example boids_render_flow -- --evidence` prints the canonical boids production-evidence report, including pass order, local instance geometry, fixed-step evidence, typed GPU-timing diagnostic evidence, CPU timing fields, and the renderer benchmark command. `cargo bench -p engine --bench render_flow_planning` includes procedural-boids planning and preflight cases.
- Runtime submit uses cached strict prepared-frame preflight by default. Full structural preflight runs on cold cache, structural key changes, failures, or strict mode; cheap runtime guards still run each frame for flow/view/invocation existence, pass-shape hazards, dispatch validity, uniform presence, and history conflicts.
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
  - `RenderGpuResidencyResource`
  - `RenderGpuResidencyBudgetResource`
  - `RenderGpuResidencyBudgetStatus`
  - `RenderGpuResidencyInspection`
  - `RenderGpuResidencyBudgetInspection`
  - `RenderGpuResidencyInspectionEntry`
  - `RenderGpuResidencyJournalInspectionEntry`
  - `inspect_render_gpu_residency`
  - `RenderScaleVisibilityCandidate`
  - `RenderScaleVisibilityCapabilities`
  - `RenderScaleVisibilityCapabilityStatus`
  - `RenderScaleVisibilityConfig`
  - `RenderScaleVisibilityInspection`
  - `RenderScaleVisibilityRecord`
  - `RenderScaleVisibilityDiagnostic`
  - `inspect_render_scale_visibility`
  - `RenderScaleProductionHardwareProfile`
  - `RenderScaleProductionHardwareProfileInspection`
  - `RenderScaleProductionEvidenceRequest`
  - `RenderScaleProductionEvidenceReport`
  - `RenderScaleProductionEvidenceCounts`
  - `RenderScaleProductionTimingEvidence`
  - `RenderScaleProductionEvidenceDiagnostic`
  - `RenderScaleProductionEvidenceSeverity`
  - `inspect_render_scale_production_evidence`
  - `RenderMeshMaterialHandoffInspectionRequest`
  - `RenderMeshMaterialHandoffInspection`
  - `RenderMeshMaterialHandoffCounts`
  - `RenderMeshMaterialHandoffDiagnostic`
  - `RenderMeshMaterialHandoffDiagnosticSeverity`
  - `inspect_render_mesh_material_handoff`
  - `RenderMeshMaterialProductionHardwareProfile`
  - `RenderMeshMaterialProductionEvidenceRequest`
  - `RenderMeshMaterialProductionEvidenceReport`
  - `RenderMeshMaterialProductionEvidenceCounts`
  - `RenderMeshMaterialProductionTimingEvidence`
  - `RenderMeshMaterialRuntimeVisualEvidence`
  - `RenderMeshMaterialProductionEvidenceDiagnostic`
  - `RenderMeshMaterialProductionEvidenceSeverity`
  - `inspect_render_mesh_material_production_evidence`
  - `RenderTemporalInspectionRequest`
  - `RenderTemporalInspection`
  - `RenderTemporalResolutionEvidence`
  - `RenderTemporalResolutionInspection`
  - `RenderTemporalJitterEvidence`
  - `RenderTemporalHistoryEvidence`
  - `RenderTemporalInputEvidence`
  - `RenderTemporalInputKind`
  - `RenderTemporalInputCounts`
  - `RenderTemporalReconstructionMode`
  - `RenderTemporalDiagnostic`
  - `RenderTemporalDiagnosticSeverity`
  - `inspect_render_temporal_inputs`
  - `RenderTemporalUpscalingInspectionRequest`
  - `RenderTemporalUpscalingInspection`
  - `RenderTemporalUpscalingAdapterEvidence`
  - `RenderTemporalUpscalingAdapterKind`
  - `RenderTemporalUpscalingCapabilityState`
  - `RenderRayReconstructionInputEvidence`
  - `RenderRayReconstructionInputKind`
  - `RenderRayReconstructionInputCounts`
  - `inspect_render_temporal_upscaling`
  - `RenderTemporalProductionEvidenceRequest`
  - `RenderTemporalProductionEvidenceReport`
  - `RenderTemporalProductionHardwareProfile`
  - `RenderTemporalProductionHardwareProfileInspection`
  - `RenderTemporalProductionEvidenceCounts`
  - `RenderTemporalProductionTimingEvidence`
  - `RenderTemporalRuntimeVisualEvidence`
  - `RenderTemporalProductionEvidenceDiagnostic`
  - `RenderTemporalProductionEvidenceSeverity`
  - `inspect_render_temporal_production_evidence`
  - `RenderRayQueryCapabilityProfile`
  - `RenderRayQueryCapabilityState`
  - `RenderRayQueryAccelerationResourceEvidence`
  - `RenderRayQueryAccelerationResourceKind`
  - `RenderRayQueryAccelerationResourceStatus`
  - `RenderRayQueryAccelerationSourceLineage`
  - `RenderRayQueryAccelerationResourceCounts`
  - `RenderRayQueryInspectionRequest`
  - `RenderRayQueryInspection`
  - `RenderRayQueryDiagnostic`
  - `RenderRayQueryDiagnosticSeverity`
  - `inspect_render_ray_query_capability`
  - `RenderPipelineFallbackInspectionRequest`
  - `RenderPipelineFallbackInspection`
  - `RenderPipelineFallbackCounts`
  - `RenderPipelineFallbackPassEvidence`
  - `RenderShaderPriorValidFailureEvidence`
  - `RenderPipelineFallbackDiagnostic`
  - `RenderPipelineFallbackDiagnosticSeverity`
  - `inspect_render_pipeline_fallback`
  - `RenderSdfResidencySourceResource`
  - `RenderSdfResidencySourceProduct`
  - `RenderSdfResidencyResource`
  - `RenderSdfResidencyBudgetResource`
  - `RenderSdfResidencyBudgetStatus`
  - `RenderSdfResidencySummary`
  - `RenderSdfChunkResidencyEntry`
  - `RenderSdfPageResidencyRecord`
  - `RenderSdfBrickAtlasRecord`
  - `RenderSdfClipmapWindowRecord`
  - `RenderSdfResidencyInspection`
  - `RenderSdfResidencyBudgetInspection`
  - `RenderSdfChunkResidencyInspectionEntry`
  - `RenderSdfPageResidencyInspectionEntry`
  - `RenderSdfBrickAtlasInspectionEntry`
  - `RenderSdfClipmapWindowInspectionEntry`
  - `inspect_render_sdf_residency`
  - `RenderSdfRaymarchAccelerationConfig`
  - `RenderSdfRaymarchAccelerationResource`
  - `RenderSdfRaymarchAccelerationReport`
  - `RenderSdfRaymarchDiagnostic`
  - `RenderSdfRaymarchDiagnosticKind`
  - `RenderSdfRaymarchDiagnosticSeverity`
  - `RenderSdfDistanceMipLevel`
  - `RenderSdfRaymarchCandidate`
  - `RenderSdfRaymarchCandidateList`
  - `inspect_sdf_raymarch_acceleration`
  - `inspect_render_sdf_raymarch_acceleration`
  - `inspect_last_render_sdf_raymarch_acceleration`
  - `RenderSdfProductionHardwareProfile`
  - `RenderSdfProductionHardwareProfileInspection`
  - `RenderSdfProductionEvidenceRequest`
  - `RenderSdfProductionEvidenceReport`
  - `RenderSdfProductionEvidenceCounts`
  - `RenderSdfProductionTimingEvidence`
  - `RenderSdfProductionEvidenceDiagnostic`
  - `RenderSdfProductionEvidenceSeverity`
  - `RenderSdfRuntimeVisualEvidence`
  - `inspect_render_sdf_production_evidence`
  - `RenderReplayManifest`
  - `RenderReplayArtifactReference`
  - `RenderReplayManifestValidation`
  - `RenderReplayManifestStatus`
  - `validate_render_replay_manifest`
- registries and descriptors:
  - `RenderFlowRegistryResource`
  - `ShaderRegistryResource`
  - `ShaderHandle`
  - `ShaderRegistryEvent`
  - `ShaderRegistryEventKind`
  - `ShaderReloadPollReport`
  - `ShaderReloadPollStatus`
- runtime diagnostics policy:
  - `RenderFrameDiagnosticsPolicyResource`
  - `RenderFrameDiagnosticsMode`
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
- `RenderPassTimingEvidence`, `RenderTimingSource`, `RenderGpuTimingCapability`, and `RenderGpuTimingDiagnostic` keep CPU encode/submit samples separate from GPU timestamp-query evidence. Unsupported, unavailable-this-frame, and readback-pending GPU timing states are explicit DTOs, not missing data.
- `WgpuCtx` records backend timestamp-query capability after device creation. The renderer requests `Features::TIMESTAMP_QUERY` when the adapter supports it, writes pass timestamps for supported pass types, resolves/readbacks timestamp queries after submit, and reports unsupported or unavailable diagnostics when measured GPU data is unavailable.
- `ShaderRegistryResource` performs the first live-reload poll immediately, then throttles normal watch polling to 500 ms by default. `request_reload()` bypasses the throttle.
- `RenderFrameDiagnosticsPolicyResource` defaults to tiered diagnostics: lightweight timing/cache/pacing state every frame, full `RenderDebugFrameReport` only for debug capture/provenance/readback/probes/diffs/export, slow frames, explicit request, or full-every-frame mode.
- `RenderDebugTimingsState` includes shader poll timing/status, diagnostics report timing/mode, preflight cache mode/status/source, GPU timing capability/diagnostics, and frame pacing mode/cap evidence without exposing backend handles.

Production readiness inspection contract:

- `inspect_render_readiness(...)` aggregates existing prepared-frame, product-surface, graph/preflight, fragment, capture, timing, and budget inspection DTOs. It is a read-only tooling surface, not a renderer-owned product policy layer.
- `evaluate_render_readiness_budgets(...)` reports renderer execution evidence budgets for CPU timing, GPU pass timing, GPU timing diagnostics, capture failures, preflight errors, fragment errors, dynamic targets/uploads, and product-surface diagnostics. Budget overruns produce diagnostics; they do not choose product rebuilds or fallback.
- `RenderGpuResidencyResource::derive_from_selections(...)` derives the finite renderer GPU working set from prepared product selections and residency requests. Its summary distinguishes addressable, selected, requested, accepted, resident, allocated, preserved, evicted, rejected, resident-byte, upload-byte, and budget-pressure evidence.
- `inspect_render_gpu_residency(...)` exposes that working-set evidence through backend-neutral DTOs with string cache identities, source-state lineage, byte estimates, budget statuses, and diagnostics. It does not expose WGPU handles, backend allocator internals, product source objects, fallback policy, streaming policy, or product truth.
- `RenderGpuResidencyBudgetResource` configures renderer execution limits for resident entries, resident bytes, upload bytes per frame, and per-entry byte estimates. Over-budget states produce explicit diagnostics and inspection statuses; they never silently downgrade product quality or choose product fallback.
- `inspect_render_scale_visibility(...)` consumes renderer-resident candidates and reports visible, culled, compacted, submitted draw, and indirect command counts. Unsupported storage compaction or indirect submission is explicit and produces zero submitted work rather than an unbounded CPU fallback.
- `RenderScaleVisibilityConfig` owns renderer execution thresholds only: frustum extent, minimum screen size, LOD screen-size bands, and maximum compacted visible candidates. It does not own product semantic LOD, streaming, fallback, freshness, authority, or visibility truth.
- `inspect_render_scale_production_evidence(...)` aggregates residency, visibility, timing, hardware profile, benchmark command, and artifact-path evidence for WR-063 production readiness. Missing hardware profiles, benchmark commands, artifact paths, timing evidence, or broken count invariants are fail-closed diagnostics.
- `RenderScaleProductionEvidenceReport` keeps addressable, selected, resident, visible, compacted, submitted, indirect, CPU timing, GPU timing, and capability-profile evidence separate. It may report unsupported GPU timing or readback as explicit degraded diagnostics, but it must not collapse those states into success-shaped data.
- `inspect_render_mesh_material_handoff(...)` aggregates prepared material instances, scene material shader bundle identity, model/mesh material selections, portable-limit checks, and pass material-binding evidence for WR-067 handoff readiness. Missing source-backed material instances, missing scene shader bundle identity, transient model/mesh region keys, broken pass-count invariants, and absent material-consuming pass evidence are fail-closed diagnostics. Material, asset, model, scene, and product truth remain outside renderer inspection.
- `inspect_render_pipeline_fallback(...)` aggregates pass provenance, pipeline cache statistics, shader reload poll status, and shader failure events for WR-068 pipeline/fallback readiness. Missing pass provenance, missing or empty pipeline cache stats, missing pipeline stats keys, forbidden material shader fallback, missing generated shader revisions, missing material specialization fragments, and shader failures without prior-valid revision evidence are fail-closed diagnostics. Prior-valid shader reuse is reported as renderer execution evidence only; product freshness, fallback legality, material truth, scene truth, and rebuild policy remain outside renderer inspection.
- `inspect_render_mesh_material_production_evidence(...)` aggregates WR-067 material handoff inspection, WR-068 pipeline/fallback inspection, runtime visual artifact references, timing evidence, benchmark commands, raw artifact paths, and human report paths for WR-069 runtime readiness. Missing visual artifacts, missing rendered pixels, missing benchmark or artifact paths, unconsumed material/pipeline inspections, material fallback passes, missing timing diagnostics, and source inspection errors are fail-closed diagnostics. Runtime proof remains renderer execution evidence; material, asset, model, scene, product, shader source, fallback legality, and rebuild policy remain source-owned.
- `inspect_render_temporal_inputs(...)` reports WR-070 temporal input availability, dynamic internal/output resolution, jitter sequence and phase, history resource/signature validity, reconstruction mode, native fallback state, and fail-closed diagnostics. Missing required inputs, hidden dynamic resolution, invalid history signatures, invalid jitter evidence, TAAU without dynamic-resolution evidence, and temporal reconstruction with invalid history and no native fallback are explicit diagnostics. The report is renderer execution evidence only; camera, scene, product, exposure, material reactivity, SDF, ray-query, freshness, fallback legality, and authority truth remain producer-owned.
- `inspect_render_temporal_upscaling(...)` reports WR-071 optional upscaling adapter capability, invocation eligibility, ray reconstruction input availability, native fallback visibility, and fail-closed diagnostics. Unsupported adapters and missing required ray reconstruction inputs are valid only when native fallback is visible; adapter-required rendering, hidden fallback, missing unsupported reasons, stale temporal inputs, and missing ray input product/generation evidence are diagnostics. The report is renderer execution evidence only; vendor SDKs, camera, scene, SDF, ray-query, material, exposure, product freshness, fallback legality, and authority truth remain outside renderer ownership.
- `inspect_render_temporal_production_evidence(...)` aggregates WR-070 temporal input inspection, WR-071 adapter/ray input inspection, runtime visual evidence references, CPU/GPU timing evidence, hardware profile identity, benchmark commands, raw artifact paths, and human report paths for WR-072 runtime readiness. Missing visual evidence, missing benchmark or artifact paths, unconsumed temporal/upscaling inspections, fallback-only visual claims, invalid history visuals, missing timing diagnostics, and broken temporal/ray input invariants are fail-closed diagnostics. Runtime proof remains renderer execution evidence; docs, artifacts, benchmark summaries, camera, scene, product, SDF, ray-query, material, exposure, fallback legality, and authority truth remain outside renderer ownership.
- `inspect_render_ray_query_capability(...)` reports WR-073 optional ray-query capability state, required capability labels, unsupported reasons, visible non-RT fallback, derived acceleration-resource lineage, build/update status, memory evidence, and backend-handle privacy diagnostics. Unsupported or disabled hardware can be valid when the reason and fallback are visible; hidden fallback, missing unsupported reasons, missing source lineage, stale resources without invalidation reasons, over-budget resources, and public backend-handle exposure fail closed. The report is renderer execution evidence only; scene, mesh, material, product, SDF, temporal, camera, exposure, fallback legality, and source truth remain producer-owned.
- `cargo run -p engine --example render_hybrid_ray_sdf_raster_runtime_proof` builds the WR-074 portable hybrid proof that composes raster pass labels, SDF production evidence, temporal production evidence, supported and unsupported ray-query inspections, visible non-RT fallback evidence, and separated raster/SDF/temporal/ray-query/fallback timing labels. The example consumes public renderer inspection DTOs only; hardware RT execution, source truth, fallback legality, and the WR-075 hardware matrix remain outside this bounded contract.
- [Renderer Ray Query Production Evidence](ray-query-production-evidence.md) records the WR-075 optional RT production evidence packet: capability matrix, fallback behavior, acceleration-resource lineage expectations, diagnostics, validation commands, and remaining non-goals.
- `RenderSdfResidencyResource::derive_from_sources(...)` derives renderer-owned sparse SDF brick, page, and clipmap residency from prepared product selections plus domain-owned `SdfChunkPayload` sources. Missing payloads, stale products, generation mismatches, nonresident products, unsupported query policy, and absent residency requests fail closed with diagnostics.
- `inspect_render_sdf_residency(...)` exposes SDF product, page-table, brick-atlas, clipmap-window, generation, byte, upload, invalidation, and budget-pressure evidence without exposing backend handles or moving SDF product truth into the renderer.
- `RenderSdfResidencyBudgetResource` configures renderer execution limits for resident pages, resident bricks, resident bytes, upload bytes, and clipmap pages per window. Over-budget states are visible diagnostics; product fallback, query authority, collision truth, and rebuild policy remain product-owned.
- `RenderSdfRaymarchAccelerationResource::derive_from_residency(...)` consumes the derived `RenderSdfResidencyResource` evidence and builds conservative renderer-owned SDF distance-mip and screen-tile/depth-slice candidate-list evidence. It does not read product sources directly or become SDF product truth.
- `inspect_sdf_raymarch_acceleration(...)` and `inspect_render_sdf_raymarch_acceleration(...)` expose bounded raymarch acceleration evidence: resident product/page/brick counts, distance mip safe-step limits, per-tile candidate lists, rejected-candidate counts, step budgets, and diagnostics for missing residency, invalid budgets, unsafe overstep risk, candidate explosion, fullscreen-per-entity multiplication, and residency pressure.
- `RenderSdfRaymarchAccelerationConfig` owns renderer execution limits only: screen-tile count, depth-slice count, per-list candidate budget, per-ray step budget, conservative empty-space step bound, and fullscreen entity multiplier. Query authority, collision truth, product fallback, source generation, and runtime visual proof remain outside this bounded renderer acceleration contract.
- `inspect_render_sdf_production_evidence(...)` aggregates SDF residency, SDF raymarch acceleration, runtime visual proof references, CPU/GPU timing evidence, hardware profile identity, benchmark commands, and artifact paths for WR-066 SDF runtime readiness. Missing visual evidence, missing benchmark commands, broken residency/raymarch count invariants, missing timing diagnostics, and unsafe overstep evidence are fail-closed diagnostics.
- `RenderSdfProductionEvidenceReport` keeps resident product/page/brick/clipmap counts, distance mips, candidate-list counts, rejected candidates, visual evidence bands, CPU timing, GPU timing, and capability diagnostics separate. It can report unsupported timestamp queries as explicit diagnostics, but it must not collapse missing GPU timing into success-shaped data.
- `RenderSdfRuntimeVisualEvidence` records near, mid, far, and summary view evidence references with step counts and missed-surface or overstep risk flags. These are runtime evidence references, not SDF product truth or authoring data.
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
