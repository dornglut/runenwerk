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

pub(super) enum CanonicalInvocationPreparation {
    Prepared(Box<PreparedRenderWorkPlan>),
    /// The invocation contains at least one operation whose durable logical identity/semantics is
    /// intentionally deferred to G7A/G5C. No partial G3 graph is retained in this case.
    PreG7Residual,
}

/// Prepare one renderer invocation as a single canonical G3 graph, or retain the whole invocation
/// on the explicit pre-G7 residual path.
///
/// This function never returns a partial graph. Surface/UI/present/dynamic-target/capture work and
/// genuine copy no-work currently keep the complete invocation residual so RunenRender cannot
/// accidentally become a second scheduler around a partial `GpuPreparedWorkGraph`.
pub(super) fn prepare_canonical_invocation(
    context: &GpuContext,
    flow: &CompiledRenderFlowPlan,
    flow_inputs: &PreparedFlowInputs,
    runtime_resources: &FlowRuntimeResources,
    projected_uploads: &[RealizedLogicalBufferUpload],
    passes: &[CanonicalPassProjection<'_>],
    timing: Option<&LogicalGpuPassTiming>,
) -> Result<CanonicalInvocationPreparation> {
    if passes.iter().any(|pass| pass.has_capture_work) {
        return Ok(CanonicalInvocationPreparation::PreG7Residual);
    }

    let mut nodes = Vec::<ResolvedRenderGpuWorkNode>::new();
    for upload in projected_uploads {
        nodes.push(ResolvedRenderGpuWorkNode::upload(
            upload.occurrence,
            occurrence_label(flow, "upload", upload.occurrence)?,
            upload.operation.clone(),
            upload.control_order_after.iter().copied(),
        ));
    }

    let mut maximum_occurrence = projected_uploads
        .iter()
        .map(|upload| upload.occurrence.raw())
        .max()
        .unwrap_or(0);

    for projected in passes {
        maximum_occurrence = maximum_occurrence.max(projected.occurrence.raw());
        let mut pass_control = projected.control_order_after.to_vec();
        if let Some(upload) = projected.fixed_step_upload {
            maximum_occurrence = maximum_occurrence.max(upload.occurrence.raw());
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
                    return Ok(CanonicalInvocationPreparation::PreG7Residual);
                };
                operation
            }
            CompiledPassExecutionPlan::Copy(pass) => {
                match project_copy_operation(runtime_resources, pass)? {
                    ProjectedCopyOperation::Canonical(operation) => *operation,
                    ProjectedCopyOperation::NoWork | ProjectedCopyOperation::PreG7Residual => {
                        return Ok(CanonicalInvocationPreparation::PreG7Residual);
                    }
                }
            }
            CompiledPassExecutionPlan::Present(_)
            | CompiledPassExecutionPlan::BuiltinUiComposite(_) => {
                return Ok(CanonicalInvocationPreparation::PreG7Residual);
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
        let resolve_occurrence = allocate_aux_occurrence(&mut maximum_occurrence)?;
        nodes.push(ResolvedRenderGpuWorkNode::timing_resolve(
            resolve_occurrence,
            occurrence_label(flow, "timing-resolve", resolve_occurrence)?,
            tail.resolve().clone(),
            [],
        ));
        let readback_occurrence = allocate_aux_occurrence(&mut maximum_occurrence)?;
        nodes.push(ResolvedRenderGpuWorkNode::timing_readback_copy(
            readback_occurrence,
            occurrence_label(flow, "timing-readback-copy", readback_occurrence)?,
            tail.readback_copy().clone(),
            [],
        ));
    }

    if nodes.is_empty() {
        return Ok(CanonicalInvocationPreparation::PreG7Residual);
    }

    Ok(CanonicalInvocationPreparation::Prepared(Box::new(
        prepare_render_gpu_work(flow, nodes)?,
    )))
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
