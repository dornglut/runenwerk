use super::logical_copy::{ProjectedCopyOperation, project_copy_operation};
use super::logical_operations::{
    project_compute_operation, project_render_operation, project_timing_tail,
};
use super::logical_timing::LogicalGpuPassTiming;
use super::*;
use crate::plugins::gpu::{
    GpuExecutionPreference, GpuRealizedBuffer, GpuResourceLabel, GpuUploadOperation,
};
use crate::plugins::render::{
    PreparedRenderWorkPlan, RenderGpuWorkOccurrenceId, ResolvedRenderGpuWorkNode,
    prepare_render_gpu_work,
};

/// One execution-complete logical upload paired only with its already-realized physical buffer.
///
/// `operation` is the sole source of destination coverage and immutable payload meaning. `realized`
/// is an opaque pre-G5B physical sidecar and may not duplicate or reconstruct those semantics.
pub(super) struct RealizedLogicalBufferUpload {
    pub(super) occurrence: RenderGpuWorkOccurrenceId,
    pub(super) operation: GpuUploadOperation,
    pub(super) realized: GpuRealizedBuffer,
    pub(super) control_order_after: Vec<RenderGpuWorkOccurrenceId>,
}

pub(super) struct CanonicalPassProjection<'a> {
    pub(super) occurrence: RenderGpuWorkOccurrenceId,
    pub(super) control_order_after: &'a [RenderGpuWorkOccurrenceId],
    pub(super) pass: &'a CompiledPassExecutionPlan,
    pub(super) pipeline: Option<&'a PreparedPipelinePass>,
    pub(super) timestamp_indices: Option<GpuPassTimestampIndices>,
    pub(super) fixed_step_upload: Option<&'a RealizedLogicalBufferUpload>,
    pub(super) has_capture_work: bool,
}

/// One invocation's already-realized inputs for canonical GPU-node resolution.
///
/// This groups the execution-complete projection instead of growing the resolver into a bag of
/// unrelated parameters. G5C1 can therefore reuse the same boundary while moving graph preparation
/// from invocation scope to frame/surface scope.
pub(super) struct CanonicalInvocationProjection<'a, 'pass> {
    pub(super) projected_uploads: &'a [RealizedLogicalBufferUpload],
    pub(super) passes: &'a [CanonicalPassProjection<'pass>],
    pub(super) timing: Option<&'a LogicalGpuPassTiming>,
}

pub(super) enum CanonicalInvocationResolution {
    Resolved(Vec<ResolvedRenderGpuWorkNode>),
    /// The invocation contains at least one operation whose durable logical identity/semantics is
    /// intentionally deferred to G7A/G5C. No partial canonical node set is retained in this case.
    PreG7Residual,
}

pub(super) enum CanonicalInvocationPreparation {
    Prepared(Box<PreparedRenderWorkPlan>),
    /// Transitional compatibility for the current caller. G5C1 removes this per-invocation
    /// preparation boundary when the realized frame batch owns one G3 graph.
    PreG7Residual,
}

/// Transitional per-invocation preparation wrapper.
///
/// The durable G5C1 path is `resolve_canonical_invocation` followed by one frame-level
/// `prepare_render_gpu_frame_work` call. This wrapper remains only so each dependency-ordered
/// checkpoint compiles before the large renderer caller cutover lands in the same PR.
pub(super) fn prepare_canonical_invocation(
    context: &GpuContext,
    flow: &CompiledRenderFlowPlan,
    flow_inputs: &PreparedFlowInputs,
    runtime_resources: &FlowRuntimeResources,
    projected_uploads: &[RealizedLogicalBufferUpload],
    passes: &[CanonicalPassProjection<'_>],
    timing: Option<&LogicalGpuPassTiming>,
) -> Result<CanonicalInvocationPreparation> {
    let mut maximum_occurrence = 0_u64;
    let projection = CanonicalInvocationProjection {
        projected_uploads,
        passes,
        timing,
    };
    match resolve_canonical_invocation(
        context,
        flow,
        flow_inputs,
        runtime_resources,
        projection,
        &mut maximum_occurrence,
    )? {
        CanonicalInvocationResolution::Resolved(nodes) => {
            Ok(CanonicalInvocationPreparation::Prepared(Box::new(
                prepare_render_gpu_work(flow, nodes)?,
            )))
        }
        CanonicalInvocationResolution::PreG7Residual => {
            Ok(CanonicalInvocationPreparation::PreG7Residual)
        }
    }
}

/// Resolves one renderer invocation into execution-complete canonical GPU occurrences without
/// preparing a G3 graph.
///
/// G5C1 calls this for every invocation participating in one physical frame/surface submission,
/// using one shared `maximum_occurrence`. Only after every invocation resolves canonically may the
/// caller prepare one bounded frame graph. If any invocation is residual, the caller must discard
/// all resolved canonical nodes and keep the complete frame on the residual path.
pub(super) fn resolve_canonical_invocation(
    context: &GpuContext,
    flow: &CompiledRenderFlowPlan,
    flow_inputs: &PreparedFlowInputs,
    runtime_resources: &FlowRuntimeResources,
    projection: CanonicalInvocationProjection<'_, '_>,
    maximum_occurrence: &mut u64,
) -> Result<CanonicalInvocationResolution> {
    let CanonicalInvocationProjection {
        projected_uploads,
        passes,
        timing,
    } = projection;

    if passes.iter().any(|pass| pass.has_capture_work) {
        return Ok(CanonicalInvocationResolution::PreG7Residual);
    }

    let mut nodes = Vec::<ResolvedRenderGpuWorkNode>::new();
    for upload in projected_uploads {
        *maximum_occurrence = (*maximum_occurrence).max(upload.occurrence.raw());
        nodes.push(ResolvedRenderGpuWorkNode::upload(
            upload.occurrence,
            occurrence_label(flow, "upload", upload.occurrence)?,
            upload.operation.clone(),
            upload.control_order_after.iter().copied(),
        ));
    }

    for projected in passes {
        *maximum_occurrence = (*maximum_occurrence).max(projected.occurrence.raw());
        let mut pass_control = projected.control_order_after.to_vec();
        if let Some(upload) = projected.fixed_step_upload {
            *maximum_occurrence = (*maximum_occurrence).max(upload.occurrence.raw());
            nodes.push(ResolvedRenderGpuWorkNode::upload(
                upload.occurrence,
                occurrence_label(flow, "fixed-step-upload", upload.occurrence)?,
                upload.operation.clone(),
                upload.control_order_after.iter().copied(),
            ));
            pass_control.clear();
            pass_control.push(upload.occurrence);
        }

        let operation = match projected.pass {
            CompiledPassExecutionPlan::Compute(pass) => {
                let pipeline = projected.pipeline.ok_or_else(|| {
                    anyhow::anyhow!(
                        "compute occurrence '{}' has no realized pipeline for canonical G5A work",
                        projected.occurrence
                    )
                })?;
                let timing = timestamp_projection(timing, projected.timestamp_indices)?;
                project_compute_operation(context, pass, flow_inputs, pipeline, timing)?
            }
            CompiledPassExecutionPlan::Fullscreen(_) | CompiledPassExecutionPlan::Graphics(_) => {
                let pipeline = projected.pipeline.ok_or_else(|| {
                    anyhow::anyhow!(
                        "render occurrence '{}' has no realized pipeline for canonical G5A work",
                        projected.occurrence
                    )
                })?;
                let timing = timestamp_projection(timing, projected.timestamp_indices)?;
                let Some(operation) = project_render_operation(
                    context,
                    runtime_resources,
                    projected.pass,
                    pipeline,
                    timing,
                )?
                else {
                    return Ok(CanonicalInvocationResolution::PreG7Residual);
                };
                operation
            }
            CompiledPassExecutionPlan::Copy(pass) => {
                match project_copy_operation(runtime_resources, pass)? {
                    ProjectedCopyOperation::Canonical(operation) => *operation,
                    ProjectedCopyOperation::NoWork | ProjectedCopyOperation::PreG7Residual => {
                        return Ok(CanonicalInvocationResolution::PreG7Residual);
                    }
                }
            }
            CompiledPassExecutionPlan::Present(_)
            | CompiledPassExecutionPlan::BuiltinUiComposite(_) => {
                return Ok(CanonicalInvocationResolution::PreG7Residual);
            }
        };

        nodes.push(ResolvedRenderGpuWorkNode::pass(
            projected.occurrence,
            occurrence_label(flow, "pass", projected.occurrence)?,
            operation,
            execution_preference(projected.pass),
            pass_control,
        ));
    }

    if let Some(timing) = timing {
        let tail = project_timing_tail(
            timing.query_set(),
            timing.query_range()?,
            timing.resolve_buffer(),
            timing.readback_buffer(),
        )?;
        let resolve_occurrence = allocate_aux_occurrence(maximum_occurrence)?;
        nodes.push(ResolvedRenderGpuWorkNode::timing_resolve(
            resolve_occurrence,
            occurrence_label(flow, "timing-resolve", resolve_occurrence)?,
            tail.resolve().clone(),
            [],
        ));
        let readback_occurrence = allocate_aux_occurrence(maximum_occurrence)?;
        nodes.push(ResolvedRenderGpuWorkNode::timing_readback_copy(
            readback_occurrence,
            occurrence_label(flow, "timing-readback-copy", readback_occurrence)?,
            tail.readback_copy().clone(),
            [],
        ));
    }

    if nodes.is_empty() {
        return Ok(CanonicalInvocationResolution::PreG7Residual);
    }

    Ok(CanonicalInvocationResolution::Resolved(nodes))
}

fn timestamp_projection(
    timing: Option<&LogicalGpuPassTiming>,
    indices: Option<GpuPassTimestampIndices>,
) -> Result<Option<(&LogicalGpuPassTiming, GpuPassTimestampIndices)>> {
    match (timing, indices) {
        (Some(timing), Some(indices)) => Ok(Some((timing, indices))),
        (None, None) => Ok(None),
        (Some(_), None) => anyhow::bail!(
            "timestampable canonical pass is missing its occurrence-local timestamp indices"
        ),
        (None, Some(_)) => anyhow::bail!(
            "canonical pass carries timestamp indices without logical timing resources"
        ),
    }
}

const fn execution_preference(pass: &CompiledPassExecutionPlan) -> GpuExecutionPreference {
    match pass {
        CompiledPassExecutionPlan::Compute(_) => GpuExecutionPreference::ComputePreferred,
        CompiledPassExecutionPlan::Fullscreen(_)
        | CompiledPassExecutionPlan::Graphics(_)
        | CompiledPassExecutionPlan::BuiltinUiComposite(_)
        | CompiledPassExecutionPlan::Present(_) => GpuExecutionPreference::GraphicsRequired,
        CompiledPassExecutionPlan::Copy(_) => GpuExecutionPreference::TransferPreferred,
    }
}

fn occurrence_label(
    flow: &CompiledRenderFlowPlan,
    kind: &str,
    occurrence: RenderGpuWorkOccurrenceId,
) -> Result<GpuResourceLabel> {
    Ok(GpuResourceLabel::new(format!(
        "{}.{}.{}",
        flow.flow_label,
        kind,
        occurrence.raw()
    ))?)
}

pub(super) fn allocate_aux_occurrence(
    maximum_occurrence: &mut u64,
) -> Result<RenderGpuWorkOccurrenceId> {
    *maximum_occurrence = maximum_occurrence.checked_add(1).ok_or_else(|| {
        anyhow::anyhow!("render GPU execution occurrence identity space is exhausted")
    })?;
    Ok(RenderGpuWorkOccurrenceId::new(*maximum_occurrence))
}
