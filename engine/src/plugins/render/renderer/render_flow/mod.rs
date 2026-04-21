use super::*;
use crate::plugins::render::RenderResourceDescriptor;
use crate::plugins::render::api::{SURFACE_COLOR_RESOURCE_LABEL, SURFACE_DEPTH_RESOURCE_LABEL};
use crate::plugins::render::backend::ensure_compiled_pass_is_supported;
use crate::plugins::render::frame::{PreparedFlowInputs, PreparedRenderFrame};
use crate::plugins::render::graph::{
    CompiledBindingEntry, CompiledBuiltinImport, CompiledComputeExecutionPlan,
    CompiledCopyExecutionPlan, CompiledPassBindings, CompiledPassExecutionPlan,
    CompiledPresentExecutionPlan, CompiledRasterExecutionPlan, CompiledRenderFlowPlan,
    CompiledResourceRef, CompiledStorageAccess, CompiledTargetPlan, RenderShaderReference,
};
use crate::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, PassTimingSample, RenderCaptureIdentity,
    RenderCapturePointIdentity, RenderCaptureSelector, RenderCaptureSelectorResult,
    RenderCaptureTerminal, RenderCaptureTerminalCode, RenderDebugConfigResource,
    RenderDebugControlResource, RenderPassProvenanceRecord, RenderSelectorResolution,
    ResolvedRenderCapturePlan, RuntimeResourceInspectionEntry, RuntimeResourceReuse,
    resource_kind_name,
};
use crate::plugins::render::pipelines::{
    FlowPassBindGroupKey, FlowPassKind, FlowPassPipelineKey, FlowPrimitiveTopologyClass,
};
use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::sync::mpsc::channel;

mod bindings;
mod capture;
mod execute;
mod execute_passes;
mod provenance;
mod runtime_resources;

pub(super) use capture::{
    FrameCaptureRuntime, PendingCaptureReadback, enqueue_texture_capture_copy, read_capture_back,
    texture_readback_format,
};
#[cfg(test)]
pub(super) use execute::FeaturePassAction;
pub(super) use provenance::{
    EncodedPassEvidence, EncodedPipelinePass, collect_pass_resource_truth,
    compiled_storage_access_to_storage_texture_access, execution_flow_pass_kind,
    execution_pass_feature_id, execution_pass_id, execution_pass_kind_name,
    execution_pass_order_index, feature_runtime_version, hash_bind_group_layout_entries,
    hash_view_signature, material_specialization_fragment_hash, resolve_shader_material,
};
pub(crate) use runtime_resources::{
    FlowRuntimeResources, ResolvedBufferRef, ResolvedTextureRef, RuntimeResourceKey,
    RuntimeResourceKind,
};
