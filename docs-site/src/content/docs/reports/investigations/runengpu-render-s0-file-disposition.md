---
title: GPU and Render S0 File Disposition
description: Exact tracked-file ownership disposition for the current combined GPU and render implementation.
status: active
owner: render
layer: investigation
canonical: true
last_reviewed: 2026-07-22
related_docs:
  - ./runengpu-render-s0-inventory.md
  - ./runengpu-render-s0-identity-consumer-lifecycle.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../workspace/planning/roadmap.md
---

# GPU and Render S0 File Disposition

## Decision rule

This matrix accounts for every tracked file currently under
`engine/src/plugins/render/**` and `engine_render_macros/**` at the S0
baseline. The disposition names semantic ownership; it does not authorize
direct source movement.

## Summary

| Disposition | Files |
|---|---:|
| move to RunenGPU | 25 |
| move to RunenRender | 10 |
| stay in Runenwerk | 60 |
| stay with another domain | 28 |
| redesign before movement | 50 |
| delete | 1 |

**Total: 174 files.**

## Exact matrix

| Current file | Disposition | Reason |
|---|---|---|
| `engine/src/plugins/render/api/bindings.rs` | redesign before movement | current public API mixes GPU handles, render-flow semantics, producer/product identities, and engine paths |
| `engine/src/plugins/render/api/dispatch.rs` | redesign before movement | current public API mixes GPU handles, render-flow semantics, producer/product identities, and engine paths |
| `engine/src/plugins/render/api/flow.rs` | redesign before movement | current public API mixes GPU handles, render-flow semantics, producer/product identities, and engine paths |
| `engine/src/plugins/render/api/handles.rs` | redesign before movement | current public API mixes GPU handles, render-flow semantics, producer/product identities, and engine paths |
| `engine/src/plugins/render/api/ids.rs` | redesign before movement | current public API mixes GPU handles, render-flow semantics, producer/product identities, and engine paths |
| `engine/src/plugins/render/api/mod.rs` | redesign before movement | current public API mixes GPU handles, render-flow semantics, producer/product identities, and engine paths |
| `engine/src/plugins/render/api/passes.rs` | redesign before movement | current public API mixes GPU handles, render-flow semantics, producer/product identities, and engine paths |
| `engine/src/plugins/render/api/resources.rs` | redesign before movement | current public API mixes GPU handles, render-flow semantics, producer/product identities, and engine paths |
| `engine/src/plugins/render/backend/device.rs` | move to RunenGPU | low-level backend, device, resource, pipeline, and execution responsibility belongs to GPU execution |
| `engine/src/plugins/render/backend/execution.rs` | move to RunenGPU | low-level backend, device, resource, pipeline, and execution responsibility belongs to GPU execution |
| `engine/src/plugins/render/backend/formats.rs` | move to RunenGPU | low-level backend, device, resource, pipeline, and execution responsibility belongs to GPU execution |
| `engine/src/plugins/render/backend/mod.rs` | move to RunenGPU | low-level backend, device, resource, pipeline, and execution responsibility belongs to GPU execution |
| `engine/src/plugins/render/backend/pipeline_cache.rs` | move to RunenGPU | low-level backend, device, resource, pipeline, and execution responsibility belongs to GPU execution |
| `engine/src/plugins/render/backend/resource_allocator.rs` | move to RunenGPU | low-level backend, device, resource, pipeline, and execution responsibility belongs to GPU execution |
| `engine/src/plugins/render/backend/surface.rs` | redesign before movement | GPU surface mechanics are mixed with native-window lifetime, ECS storage, and current RenderSurfaceId semantics |
| `engine/src/plugins/render/backend/wgpu_ctx.rs` | move to RunenGPU | low-level backend, device, resource, pipeline, and execution responsibility belongs to GPU execution |
| `engine/src/plugins/render/composition/fragment_registry.rs` | move to RunenRender | render contribution and fragment composition are image-formation inputs |
| `engine/src/plugins/render/composition/fragment_validation.rs` | move to RunenRender | render contribution and fragment composition are image-formation inputs |
| `engine/src/plugins/render/composition/fragments.rs` | move to RunenRender | render contribution and fragment composition are image-formation inputs |
| `engine/src/plugins/render/composition/hot_reload.rs` | stay in Runenwerk | source reload and cross-domain integration remain product and host policy |
| `engine/src/plugins/render/composition/integration.rs` | stay in Runenwerk | source reload and cross-domain integration remain product and host policy |
| `engine/src/plugins/render/composition/mod.rs` | redesign before movement | module exports combine renderer composition with Runenwerk integration |
| `engine/src/plugins/render/features/caves/mod.rs` | stay with another domain | world, cave, and VFX semantics remain source-domain owned and feed adapters |
| `engine/src/plugins/render/features/detail/mod.rs` | move to RunenRender | renderer detail and quality semantics belong to image formation |
| `engine/src/plugins/render/features/editor_picking/mod.rs` | stay in Runenwerk | feature registry, editor picking, and UI bridge behavior are product/integration policy |
| `engine/src/plugins/render/features/editor_picking/resource.rs` | stay in Runenwerk | feature registry, editor picking, and UI bridge behavior are product/integration policy |
| `engine/src/plugins/render/features/mod.rs` | stay in Runenwerk | feature registry, editor picking, and UI bridge behavior are product/integration policy |
| `engine/src/plugins/render/features/particle_vfx/mod.rs` | stay with another domain | world, cave, and VFX semantics remain source-domain owned and feed adapters |
| `engine/src/plugins/render/features/ui/descriptor.rs` | stay in Runenwerk | feature registry, editor picking, and UI bridge behavior are product/integration policy |
| `engine/src/plugins/render/features/ui/mod.rs` | stay in Runenwerk | feature registry, editor picking, and UI bridge behavior are product/integration policy |
| `engine/src/plugins/render/features/ui/prepared.rs` | stay in Runenwerk | feature registry, editor picking, and UI bridge behavior are product/integration policy |
| `engine/src/plugins/render/features/ui/render_output_proof.rs` | stay in Runenwerk | feature registry, editor picking, and UI bridge behavior are product/integration policy |
| `engine/src/plugins/render/features/ui/resource.rs` | stay in Runenwerk | feature registry, editor picking, and UI bridge behavior are product/integration policy |
| `engine/src/plugins/render/features/ui/submission.rs` | stay in Runenwerk | feature registry, editor picking, and UI bridge behavior are product/integration policy |
| `engine/src/plugins/render/features/world/lod.rs` | stay with another domain | world, cave, and VFX semantics remain source-domain owned and feed adapters |
| `engine/src/plugins/render/features/world/mod.rs` | stay with another domain | world, cave, and VFX semantics remain source-domain owned and feed adapters |
| `engine/src/plugins/render/features/world/runtime_cache.rs` | stay with another domain | world, cave, and VFX semantics remain source-domain owned and feed adapters |
| `engine/src/plugins/render/features/world/sdf_raymarch.rs` | stay with another domain | world, cave, and VFX semantics remain source-domain owned and feed adapters |
| `engine/src/plugins/render/features/world/sdf_residency.rs` | stay with another domain | world, cave, and VFX semantics remain source-domain owned and feed adapters |
| `engine/src/plugins/render/features/world/visuals/mod.rs` | stay with another domain | world, cave, and VFX semantics remain source-domain owned and feed adapters |
| `engine/src/plugins/render/frame/context.rs` | move to RunenRender | prepared scene, view, and contribution semantics belong to image formation |
| `engine/src/plugins/render/frame/contribution_diagnostics.rs` | move to RunenRender | prepared scene, view, and contribution semantics belong to image formation |
| `engine/src/plugins/render/frame/contribution_registry.rs` | move to RunenRender | prepared scene, view, and contribution semantics belong to image formation |
| `engine/src/plugins/render/frame/contributions.rs` | move to RunenRender | prepared scene, view, and contribution semantics belong to image formation |
| `engine/src/plugins/render/frame/mod.rs` | move to RunenRender | prepared scene, view, and contribution semantics belong to image formation |
| `engine/src/plugins/render/frame/packet.rs` | redesign before movement | prepared packets mix renderer identities, GPU resources, and Runenwerk producer/product state |
| `engine/src/plugins/render/frame/product_selection.rs` | stay in Runenwerk | product selection and host surface policy remain Runenwerk-owned |
| `engine/src/plugins/render/frame/product_surface.rs` | stay in Runenwerk | product selection and host surface policy remain Runenwerk-owned |
| `engine/src/plugins/render/frame/view.rs` | move to RunenRender | prepared scene, view, and contribution semantics belong to image formation |
| `engine/src/plugins/render/gpu_primitives/compaction.rs` | move to RunenGPU | generic GPU scan, compaction, counter, and indirect-draw primitives are execution building blocks |
| `engine/src/plugins/render/gpu_primitives/counters.rs` | move to RunenGPU | generic GPU scan, compaction, counter, and indirect-draw primitives are execution building blocks |
| `engine/src/plugins/render/gpu_primitives/draw_args.rs` | move to RunenGPU | generic GPU scan, compaction, counter, and indirect-draw primitives are execution building blocks |
| `engine/src/plugins/render/gpu_primitives/mod.rs` | move to RunenGPU | generic GPU scan, compaction, counter, and indirect-draw primitives are execution building blocks |
| `engine/src/plugins/render/gpu_primitives/plan.rs` | move to RunenGPU | generic GPU scan, compaction, counter, and indirect-draw primitives are execution building blocks |
| `engine/src/plugins/render/gpu_primitives/scan.rs` | move to RunenGPU | generic GPU scan, compaction, counter, and indirect-draw primitives are execution building blocks |
| `engine/src/plugins/render/graph/capabilities.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/diagnostics.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/execution_plan.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/flow_graph.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/merge.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/mod.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/pass_graph.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/pass_shape.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/planning.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/prepared_validation.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/resource_graph.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/resource_lifetimes.rs` | move to RunenGPU | resource lifetime validation belongs to GPU execution |
| `engine/src/plugins/render/graph/validation.rs` | redesign before movement | current graph combines GPU hazards/resources with render passes, fixed-step regions, features, and product validation |
| `engine/src/plugins/render/graph/validation_builtin_ui.rs` | delete | UI-specific graph validation is replaced by neutral overlay contribution validation |
| `engine/src/plugins/render/inspect/artifacts.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/budgets.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/capture.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/config.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/gpu_residency.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/graph_dump.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/material_handoff.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/material_production.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/mod.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/pass_provenance.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/pipeline_fallback.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/plan.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/prepared_frame.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/producer.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/product_visual_evidence.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/query_snapshot.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/ray_query.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/readiness.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/report.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/resource_inspector.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/scale_production.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/scale_visibility.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/sdf_production.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/sdf_raymarch.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/sdf_residency.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/temporal.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/temporal_production.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/temporal_upscaling.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/texture_preview.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/texture_view.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/timings.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/inspect/world_runtime.rs` | stay in Runenwerk | capture, artifacts, readiness, product evidence, and diagnostics presentation remain host/product responsibilities |
| `engine/src/plugins/render/material_compiler/bindings.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/diagnostics.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/identity.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/mod.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/tests.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/types.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/validation.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/wgsl/literals.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/wgsl/mod.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/wgsl/preview.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/wgsl/program.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/material_compiler/wgsl/scene.rs` | stay with another domain | material source and IR compilation remain material-authoring integration, not GPU/render core |
| `engine/src/plugins/render/mod.rs` | stay in Runenwerk | plugin assembly, product integration, and asset/upload orchestration remain host-owned |
| `engine/src/plugins/render/params/gpu_params.rs` | move to RunenGPU | GPU value and parameter layout/upload representation are execution ABI responsibilities |
| `engine/src/plugins/render/params/gpu_value.rs` | move to RunenGPU | GPU value and parameter layout/upload representation are execution ABI responsibilities |
| `engine/src/plugins/render/params/mod.rs` | move to RunenGPU | GPU value and parameter layout/upload representation are execution ABI responsibilities |
| `engine/src/plugins/render/pipelines/cache.rs` | move to RunenGPU | backend pipeline identity, cache, and specialization mechanics belong to GPU execution |
| `engine/src/plugins/render/pipelines/flow_keys.rs` | redesign before movement | flow-pass keys and module exports mix render semantics with backend realization |
| `engine/src/plugins/render/pipelines/keys.rs` | move to RunenGPU | backend pipeline identity, cache, and specialization mechanics belong to GPU execution |
| `engine/src/plugins/render/pipelines/mod.rs` | redesign before movement | flow-pass keys and module exports mix render semantics with backend realization |
| `engine/src/plugins/render/pipelines/specialization.rs` | move to RunenGPU | backend pipeline identity, cache, and specialization mechanics belong to GPU execution |
| `engine/src/plugins/render/plugin.rs` | stay in Runenwerk | plugin assembly, product integration, and asset/upload orchestration remain host-owned |
| `engine/src/plugins/render/procedural/authoring.rs` | stay with another domain | procedural authoring, population, camera, lowering, and validation remain procgen/source-domain responsibility |
| `engine/src/plugins/render/procedural/camera.rs` | stay with another domain | procedural authoring, population, camera, lowering, and validation remain procgen/source-domain responsibility |
| `engine/src/plugins/render/procedural/descriptors.rs` | stay with another domain | procedural authoring, population, camera, lowering, and validation remain procgen/source-domain responsibility |
| `engine/src/plugins/render/procedural/lowering.rs` | stay with another domain | procedural authoring, population, camera, lowering, and validation remain procgen/source-domain responsibility |
| `engine/src/plugins/render/procedural/mod.rs` | stay with another domain | procedural authoring, population, camera, lowering, and validation remain procgen/source-domain responsibility |
| `engine/src/plugins/render/procedural/population/mod.rs` | stay with another domain | procedural authoring, population, camera, lowering, and validation remain procgen/source-domain responsibility |
| `engine/src/plugins/render/procedural/population/uniform_grid.rs` | stay with another domain | procedural authoring, population, camera, lowering, and validation remain procgen/source-domain responsibility |
| `engine/src/plugins/render/procedural/validation.rs` | stay with another domain | procedural authoring, population, camera, lowering, and validation remain procgen/source-domain responsibility |
| `engine/src/plugins/render/renderer/dynamic_targets.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/extract.rs` | stay in Runenwerk | ECS/host extraction, native setup, and capture/artifact policy remain Runenwerk-owned |
| `engine/src/plugins/render/renderer/frame_bindings.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/mod.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/pipeline_cache.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/prepare.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/bindings.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/capture.rs` | stay in Runenwerk | ECS/host extraction, native setup, and capture/artifact policy remain Runenwerk-owned |
| `engine/src/plugins/render/renderer/render_flow/execute.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/execute_passes.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/gpu_timing.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/mod.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/preflight_cache.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/provenance.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources/inspect.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources/realize.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources/resolve.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources.rs` | redesign before movement | current renderer execution mixes image formation, WGPU realization, runtime resources, product targets, and diagnostics |
| `engine/src/plugins/render/renderer/setup.rs` | stay in Runenwerk | ECS/host extraction, native setup, and capture/artifact policy remain Runenwerk-owned |
| `engine/src/plugins/render/residency/handle.rs` | redesign before movement | current residency combines GPU handles, renderer caches, and world/SDF lifecycle ownership |
| `engine/src/plugins/render/residency/mod.rs` | redesign before movement | current residency combines GPU handles, renderer caches, and world/SDF lifecycle ownership |
| `engine/src/plugins/render/residency/resource.rs` | redesign before movement | current residency combines GPU handles, renderer caches, and world/SDF lifecycle ownership |
| `engine/src/plugins/render/resource/descriptors.rs` | move to RunenGPU | GPU resource descriptors, imports, usages, and lifetimes belong to execution |
| `engine/src/plugins/render/resource/dynamic_target.rs` | redesign before movement | dynamic and transient targets mix render-product semantics and host/runtime allocation policy |
| `engine/src/plugins/render/resource/import.rs` | move to RunenGPU | GPU resource descriptors, imports, usages, and lifetimes belong to execution |
| `engine/src/plugins/render/resource/lifetime.rs` | move to RunenGPU | GPU resource descriptors, imports, usages, and lifetimes belong to execution |
| `engine/src/plugins/render/resource/mod.rs` | move to RunenGPU | GPU resource descriptors, imports, usages, and lifetimes belong to execution |
| `engine/src/plugins/render/resource/transient.rs` | redesign before movement | dynamic and transient targets mix render-product semantics and host/runtime allocation policy |
| `engine/src/plugins/render/resource/usages.rs` | move to RunenGPU | GPU resource descriptors, imports, usages, and lifetimes belong to execution |
| `engine/src/plugins/render/runtime/debug_eval.rs` | stay in Runenwerk | frame preparation/submission, dynamic uploads, debug evaluation, and ECS scheduling are engine integration |
| `engine/src/plugins/render/runtime/dynamic_targets.rs` | stay in Runenwerk | frame preparation/submission, dynamic uploads, debug evaluation, and ECS scheduling are engine integration |
| `engine/src/plugins/render/runtime/dynamic_texture_uploads.rs` | stay in Runenwerk | frame preparation/submission, dynamic uploads, debug evaluation, and ECS scheduling are engine integration |
| `engine/src/plugins/render/runtime/frame_prepare.rs` | stay in Runenwerk | frame preparation/submission, dynamic uploads, debug evaluation, and ECS scheduling are engine integration |
| `engine/src/plugins/render/runtime/frame_submit.rs` | stay in Runenwerk | frame preparation/submission, dynamic uploads, debug evaluation, and ECS scheduling are engine integration |
| `engine/src/plugins/render/runtime/mod.rs` | stay in Runenwerk | frame preparation/submission, dynamic uploads, debug evaluation, and ECS scheduling are engine integration |
| `engine/src/plugins/render/shader/helpers.rs` | stay in Runenwerk | filesystem discovery, registry/revision, and hot-reload product policy remain host-owned |
| `engine/src/plugins/render/shader/hot_reload.rs` | stay in Runenwerk | filesystem discovery, registry/revision, and hot-reload product policy remain host-owned |
| `engine/src/plugins/render/shader/mod.rs` | redesign before movement | current shader module/types mix source identity/revision with renderer use and future GPU admission |
| `engine/src/plugins/render/shader/registry.rs` | stay in Runenwerk | filesystem discovery, registry/revision, and hot-reload product policy remain host-owned |
| `engine/src/plugins/render/shader/types.rs` | redesign before movement | current shader module/types mix source identity/revision with renderer use and future GPU admission |
| `engine/src/plugins/render/texture_upload.rs` | stay in Runenwerk | plugin assembly, product integration, and asset/upload orchestration remain host-owned |
| `engine_render_macros/Cargo.toml` | redesign before movement | macro ABI hard-codes engine paths and requires independent consumer proof |
| `engine_render_macros/src/lib.rs` | redesign before movement | macro ABI hard-codes engine paths and requires independent consumer proof |

## Interpretation

- Files marked `move` still move only after internal public-boundary proof.
- Files marked `redesign` are split or replaced rather than copied.
- Runenwerk retains lifecycle, adapters, product policy, source reload, capture, and recovery.
- Source-domain semantics remain with their owning domains and feed prepared contributions.
- `graph/validation_builtin_ui.rs` is deleted only after neutral overlay validation replaces it.

A completed cutover migrates every active consumer and deletes the original
implementation. No alias, compatibility module, source mirror, or parallel
execution path remains.
