# Render Public API Reference

This page maps the `engine::plugins::render` public API to its intended usage level.

## Common Path APIs

These are the APIs most users should start with.

- `RenderPlugin`
- `RenderFlow`
- pass builders:
  - `ComputePassBuilder`
  - `FullscreenPassBuilder`
  - `BuiltinUiCompositePassBuilder`
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
- `PreparedSurfaceInfo`
- `PreparedSceneInfo`
- `PreparedUiInput`
- `PreparedFlowInputs`
- `PreparedStateTypeInfo`
- `PreparedShaderSnapshot`

Contract:

- `frame_render_prepare_system` publishes one owned `PreparedRenderFrame` per renderable frame.
- `ui_render_submit_system` consumes the prepared frame and does not perform live ECS extraction for flow data.

Current UI note:

- `PreparedUiInput::RawDrawList` is a phase-1 transport format.
- The target direction is backend-neutral extracted UI prepared input.

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
  - `ResourceLifetime`
  - `TransientAliasAssignment`
  - `TransientAliasCandidate`
  - `TransientAliasSlot`
  - `TransientResourceWindow`

## Compatibility Surface

`RenderFrameDataRegistry` remains public for projection helper compatibility and tests.

It is not part of the active runtime submission path and should not be used as a substitute for `PreparedRenderFrame`.
