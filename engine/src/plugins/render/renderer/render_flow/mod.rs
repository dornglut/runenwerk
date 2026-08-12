use super::pipeline_cache::FlowPipelineArtifactCache;
use super::*;
use crate::plugins::render::api::{
    RenderFixedStepIterationUniform, SURFACE_COLOR_RESOURCE_LABEL, SURFACE_DEPTH_RESOURCE_LABEL,
};
use crate::plugins::render::backend::ensure_compiled_pass_is_supported;
use crate::plugins::render::frame::{PreparedFlowInputs, PreparedRenderFrame};
use crate::plugins::render::graph::{
    CompiledBindingEntry, CompiledBuiltinImport, CompiledComputeExecutionPlan,
    CompiledCopyExecutionPlan, CompiledFixedStepRegion, CompiledPassBindings,
    CompiledPassExecutionPlan, CompiledPresentExecutionPlan, CompiledRasterExecutionPlan,
    CompiledRenderFlowPlan, CompiledResourceRef, CompiledStorageAccess, CompiledTargetPlan,
    RenderShaderReference, preflight_prepared_render_frame,
};
use crate::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, PassTimingSample, RenderCaptureIdentity,
    RenderCapturePointIdentity, RenderCaptureSelector, RenderCaptureSelectorResult,
    RenderCaptureTerminal, RenderCaptureTerminalCode, RenderDebugConfigResource,
    RenderDebugControlResource, RenderGpuTimingCapability, RenderGpuTimingDiagnostic,
    RenderPassMaterialBindingEvidence, RenderPassModelMeshMaterialSelectionEvidence,
    RenderPassProvenanceRecord, RenderPassTimingEvidence, RenderSelectorResolution,
    ResolvedRenderCapturePlan, RuntimeResourceInspectionEntry, RuntimeResourceReuse,
    resource_kind_name,
};
use crate::plugins::render::pipelines::{FlowPassKind, FlowPassPipelineKey};
use crate::plugins::render::{RenderResourceDeclaration, current_runtime_gpu_capabilities};
use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::sync::mpsc::channel;

mod bindings;
mod capture;
mod execute;
mod execute_passes;
mod gpu_timing;
mod preflight_cache;
mod program_sources;
mod provenance;
mod runtime_resources;

/// A G4C2 realization result carried from the render batch's first phase into the temporary
/// G4C3/G5 operation phase. It is deliberately renderer-local and does not create a pipeline.
pub(super) struct PreparedPipelinePass {
    pub(super) bindings: bindings::RealizedFlowProgramBindings,
    pub(super) shader_id: String,
    pub(super) shader_revision: u64,
    pub(super) fallback_used: bool,
}

pub(super) use capture::{
    CaptureTextureSource, FrameCaptureRuntime, PendingCaptureReadback, PreparedCaptureReadback,
    encode_prepared_texture_capture_copy, prepare_texture_capture_copy, read_capture_back,
    texture_readback_format,
};
#[cfg(test)]
pub(super) use execute::FeaturePassAction;
pub(super) use gpu_timing::{
    GpuPassTimestampIndices, GpuPassTimestampWrites, GpuPassTimingFrame,
    PendingGpuPassTimingReadback, read_gpu_pass_timing_evidence,
};
pub(crate) use preflight_cache::RendererPreparedFramePreflightCacheEntry;
pub(crate) use program_sources::RendererProgramSourceAuthority;
pub(super) use provenance::{
    EncodedPassEvidence, EncodedPipelinePass, collect_pass_material_binding_evidence,
    collect_pass_resource_truth, execution_flow_pass_kind, execution_pass_authoring_index,
    execution_pass_feature_id, execution_pass_id, execution_pass_kind_name,
    execution_pass_shader_reference, feature_runtime_version, hash_view_signature,
    material_specialization_fragment_hash, pass_consumes_material_resources,
    resolve_shader_material, resolve_shader_material_for_packet,
};
pub(crate) use runtime_resources::{
    FlowRuntimeResources, ResolvedBufferRef, ResolvedColorTargetView, ResolvedDepthTargetView,
    ResolvedTextureRef, RuntimeResourceKey, RuntimeResourceKind, RuntimeTextureRef,
    RuntimeTextureView,
};
