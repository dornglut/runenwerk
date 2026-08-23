use super::super::dynamic_targets::RendererDynamicTextureTargetCache;
use super::logical_copy::{
    ProjectedCopyOperation, project_copy_operation, project_present_copy_operation,
};
use super::logical_operations::{
    project_compute_operation, project_render_operation, project_timing_tail, timestamp_writes,
};
use super::logical_timing::LogicalGpuPassTiming;
use super::*;
use crate::plugins::gpu::{
    GpuAttachmentStore, GpuColorAttachmentLoad, GpuExecutionPreference, GpuRealizedBuffer,
    GpuRenderColorAttachment, GpuRenderDraw, GpuRenderOperation, GpuResourceLabel,
    GpuTextureViewHandle, GpuUploadOperation, GpuWorkOperation,
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
/// from invocation scope to frame/surface scope. `surface_color_view` is the exact logical view of
/// the currently acquired G7A image when that authority has already moved to the frame caller.
/// `builtin_ui_draws` is only an execution-complete generic lowering of the current transitional
/// Runenwerk UI batches; it is not RunenUI or future RunenRender semantic authority.
#[derive(Clone, Copy)]
pub(super) struct CanonicalInvocationProjection<'a, 'pass> {
    pub(super) projected_uploads: &'a [RealizedLogicalBufferUpload],
    pub(super) passes: &'a [CanonicalPassProjection<'pass>],
    pub(super) surface_color_view: Option<&'a GpuTextureViewHandle>,
    pub(super) builtin_ui_draws: Option<&'a [GpuRenderDraw]>,
    pub(super) timing: Option<&'a LogicalGpuPassTiming>,
}

/// One invocation plus the render-owned state required to resolve it into canonical GPU work.
///
/// The frame resolver collects these projections before allocating any auxiliary occurrence IDs.
/// This prevents per-invocation timing tails from consuming an identity already assigned to a
/// later invocation in the same physical submission.
#[derive(Clone, Copy)]
pub(super) struct CanonicalFrameInvocationProjection<'a, 'pass> {
    pub(super) flow: &'a CompiledRenderFlowPlan,
    pub(super) flow_inputs: &'a PreparedFlowInputs,
    pub(super) runtime_resources: &'a FlowRuntimeResources,
    pub(super) invocation: CanonicalInvocationProjection<'a, 'pass>,
}

pub(super) struct CanonicalResolvedInvocation {
    nodes: Vec<ResolvedRenderGpuWorkNode>,
    /// One entry per terminal compiled Present omitted because its source was already
    /// `SurfaceColor`. Each inner vector is the render-owned non-data predecessor set that the
    /// eventual frame-terminal `GpuPresentOperation` must preserve.
    terminal_present_controls: Vec<Vec<RenderGpuWorkOccurrenceId>>,
}

pub(super) struct CanonicalResolvedFrame {
    pub(super) nodes: Vec<ResolvedRenderGpuWorkNode>,
    /// Aggregated terminal-Present predecessor sets. An empty inner vector still records a real
    /// omitted Present, allowing the transitional per-invocation path to remain residual.
    pub(super) terminal_present_controls: Vec<Vec<RenderGpuWorkOccurrenceId>>,
}

pub(super) enum CanonicalInvocationResolution {
    Resolved(CanonicalResolvedInvocation),
    /// The invocation contains at least one operation whose durable logical identity/semantics is
    /// intentionally deferred to G7A/G5C. No partial canonical node set is retained in this case.
    PreG7Residual,
}

pub(super) enum CanonicalFrameResolution {
    Resolved(CanonicalResolvedFrame),
    /// At least one invocation is not fully canonical yet. G5C1 requires the entire physical
    /// frame/surface submission to remain on the residual path rather than mixing authorities.
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
/// The durable G5C1 path is `resolve_canonical_frame` followed by one frame-level
/// `prepare_render_gpu_frame_work` call. The current single-invocation caller is deliberately routed
/// through the same frame resolver so all-or-nothing residual handling and auxiliary occurrence
/// allocation remain exercised until the caller itself moves to frame scope.
pub(super) fn prepare_canonical_invocation(
    context: &GpuContext,
    flow: &CompiledRenderFlowPlan,
    flow_inputs: &PreparedFlowInputs,
    runtime_resources: &FlowRuntimeResources,
    projected_uploads: &[RealizedLogicalBufferUpload],
    passes: &[CanonicalPassProjection<'_>],
    timing: Option<&LogicalGpuPassTiming>,
) -> Result<CanonicalInvocationPreparation> {
    let invocation = CanonicalInvocationProjection {
        projected_uploads,
        passes,
        surface_color_view: None,
        builtin_ui_draws: None,
        timing,
    };
    let frame_invocation = CanonicalFrameInvocationProjection {
        flow,
        flow_inputs,
        runtime_resources,
        invocation,
    };
    // The transitional per-invocation physical bridge cannot resolve dynamic-target sidecars or
    // consume terminal-Present control metadata. Only the eventual frame-level G5 path may supply
    // and consume those authorities.
    match resolve_canonical_frame(context, None, [frame_invocation])? {
        CanonicalFrameResolution::Resolved(frame) => {
            if !frame.terminal_present_controls.is_empty() {
                return Ok(CanonicalInvocationPreparation::PreG7Residual);
            }
            Ok(CanonicalInvocationPreparation::Prepared(Box::new(
                prepare_render_gpu_work(flow, frame.nodes)?,
            )))
        }
        CanonicalFrameResolution::PreG7Residual => {
            Ok(CanonicalInvocationPreparation::PreG7Residual)
        }
    }
}

/// Resolves every renderer invocation participating in one physical frame/surface submission.
///
/// Existing occurrence IDs are observed across the complete frame before any timing/capture-tail
/// auxiliary identity can be allocated. This is required even when ordinary pass occurrences were
/// already expanded with one frame-scoped allocator: resolving invocation A must not allocate the
/// ID already assigned to invocation B. The returned frame is all-or-nothing; one residual
/// invocation discards every canonical node and terminal-Present control record accumulated for the
/// frame so the caller cannot mix legacy and G5 execution authority inside one physical submission.
pub(super) fn resolve_canonical_frame<'a, 'pass: 'a>(
    context: &GpuContext,
    dynamic_texture_targets: Option<&'a RendererDynamicTextureTargetCache>,
    invocations: impl IntoIterator<Item = CanonicalFrameInvocationProjection<'a, 'pass>>,
) -> Result<CanonicalFrameResolution> {
    let invocations = invocations.into_iter().collect::<Vec<_>>();
    if invocations.is_empty() {
        return Ok(CanonicalFrameResolution::PreG7Residual);
    }

    let mut maximum_occurrence = 0_u64;
    for invocation in &invocations {
        observe_existing_occurrences(invocation.invocation, &mut maximum_occurrence);
    }

    let mut nodes = Vec::<ResolvedRenderGpuWorkNode>::new();
    let mut terminal_present_controls = Vec::<Vec<RenderGpuWorkOccurrenceId>>::new();
    for invocation in invocations {
        match resolve_canonical_invocation(
            context,
            invocation.flow,
            invocation.flow_inputs,
            invocation.runtime_resources,
            dynamic_texture_targets,
            invocation.invocation,
            &mut maximum_occurrence,
        )? {
            CanonicalInvocationResolution::Resolved(mut resolved) => {
                nodes.append(&mut resolved.nodes);
                terminal_present_controls.append(&mut resolved.terminal_present_controls);
            }
            CanonicalInvocationResolution::PreG7Residual => {
                return Ok(CanonicalFrameResolution::PreG7Residual);
            }
        }
    }

    if nodes.is_empty() && terminal_present_controls.is_empty() {
        Ok(CanonicalFrameResolution::PreG7Residual)
    } else {
        Ok(CanonicalFrameResolution::Resolved(CanonicalResolvedFrame {
            nodes,
            terminal_present_controls,
        }))
    }
}

/// Resolves one renderer invocation into execution-complete canonical GPU occurrences without
/// preparing a G3 graph.
///
/// G5C1 normally reaches this through `resolve_canonical_frame`, which first observes every
/// existing occurrence in the physical frame before auxiliary IDs are allocated. The direct entry
/// point remains for the frame resolver and its bounded transitional wrapper. If an operation is
/// residual, no partial canonical node set or terminal-Present control metadata from this invocation
/// is retained.
pub(super) fn resolve_canonical_invocation(
    context: &GpuContext,
    flow: &CompiledRenderFlowPlan,
    flow_inputs: &PreparedFlowInputs,
    runtime_resources: &FlowRuntimeResources,
    dynamic_texture_targets: Option<&RendererDynamicTextureTargetCache>,
    projection: CanonicalInvocationProjection<'_, '_>,
    maximum_occurrence: &mut u64,
) -> Result<CanonicalInvocationResolution> {
    let CanonicalInvocationProjection {
        projected_uploads,
        passes,
        surface_color_view,
        builtin_ui_draws,
        timing,
    } = projection;

    if passes.iter().any(|pass| pass.has_capture_work) {
        return Ok(CanonicalInvocationResolution::PreG7Residual);
    }

    let mut nodes = Vec::<ResolvedRenderGpuWorkNode>::new();
    let mut terminal_present_controls = Vec::<Vec<RenderGpuWorkOccurrenceId>>::new();
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
                    surface_color_view,
                    timing,
                )?
                else {
                    return Ok(CanonicalInvocationResolution::PreG7Residual);
                };
                operation
            }
            CompiledPassExecutionPlan::Copy(pass) => {
                match project_copy_operation(runtime_resources, dynamic_texture_targets, pass)? {
                    ProjectedCopyOperation::Canonical(operation) => *operation,
                    ProjectedCopyOperation::NoWork | ProjectedCopyOperation::PreG7Residual => {
                        return Ok(CanonicalInvocationResolution::PreG7Residual);
                    }
                }
            }
            CompiledPassExecutionPlan::BuiltinUiComposite(_) => {
                let Some(surface_color_view) = surface_color_view else {
                    return Ok(CanonicalInvocationResolution::PreG7Residual);
                };
                let Some(draws) = builtin_ui_draws else {
                    return Ok(CanonicalInvocationResolution::PreG7Residual);
                };
                let timing = timestamp_projection(timing, projected.timestamp_indices)?;
                if draws.is_empty() && timing.is_none() {
                    // A drawless, untimed UI pass is semantically no GPU work. The current
                    // occurrence remains residual until the live frame caller omits it upstream;
                    // fabricating a clear or zero-instance draw here would change legacy output.
                    return Ok(CanonicalInvocationResolution::PreG7Residual);
                }
                let color_attachment = GpuRenderColorAttachment::new(
                    surface_color_view.clone(),
                    GpuColorAttachmentLoad::Load,
                    GpuAttachmentStore::Store,
                    None,
                )?;
                let timestamp_writes = timing
                    .map(|(timing, indices)| timestamp_writes(timing, indices))
                    .transpose()?;
                GpuWorkOperation::Render(GpuRenderOperation::new(
                    [color_attachment],
                    None,
                    draws.iter().cloned(),
                    timestamp_writes,
                )?)
            }
            CompiledPassExecutionPlan::Present(pass) => {
                match project_present_copy_operation(
                    runtime_resources,
                    dynamic_texture_targets,
                    pass,
                    surface_color_view,
                )? {
                    ProjectedCopyOperation::Canonical(operation) => *operation,
                    ProjectedCopyOperation::NoWork => {
                        terminal_present_controls.push(pass_control);
                        continue;
                    }
                    ProjectedCopyOperation::PreG7Residual => {
                        return Ok(CanonicalInvocationResolution::PreG7Residual);
                    }
                }
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

    if nodes.is_empty() && terminal_present_controls.is_empty() {
        return Ok(CanonicalInvocationResolution::PreG7Residual);
    }

    Ok(CanonicalInvocationResolution::Resolved(
        CanonicalResolvedInvocation {
            nodes,
            terminal_present_controls,
        },
    ))
}

fn observe_existing_occurrences(
    projection: CanonicalInvocationProjection<'_, '_>,
    maximum_occurrence: &mut u64,
) {
    for upload in projection.projected_uploads {
        *maximum_occurrence = (*maximum_occurrence).max(upload.occurrence.raw());
    }
    for pass in projection.passes {
        *maximum_occurrence = (*maximum_occurrence).max(pass.occurrence.raw());
        if let Some(upload) = pass.fixed_step_upload {
            *maximum_occurrence = (*maximum_occurrence).max(upload.occurrence.raw());
        }
    }
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
        | CompiledPassExecutionPlan::BuiltinUiComposite(_) => {
            GpuExecutionPreference::GraphicsRequired
        }
        CompiledPassExecutionPlan::Copy(_) | CompiledPassExecutionPlan::Present(_) => {
            GpuExecutionPreference::TransferPreferred
        }
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
