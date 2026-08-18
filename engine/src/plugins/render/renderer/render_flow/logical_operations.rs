use super::*;
use super::logical_timing::LogicalGpuPassTiming;
use crate::plugins::gpu::{
    GpuComputeOperation, GpuDispatchIntent, GpuDispatchSize, GpuQueryAccess, GpuQueryAccessKind,
    GpuQueryRange, GpuWorkOperation,
};

/// Projects one already-realized compute pass into its execution-complete RunenGPU operation.
///
/// This function does not inspect compiled binding declarations or reconstruct accesses. The
/// exact pipeline descriptor and complete logical runtime binding set are consumed from G4C2/C3
/// realization, and all access/capability meaning is subsequently derived by `GpuWorkOperation`.
pub(super) fn project_compute_operation(
    context: &GpuContext,
    pass: &CompiledComputeExecutionPlan,
    flow_inputs: &PreparedFlowInputs,
    pipeline: &PreparedPipelinePass,
    timing: Option<(&LogicalGpuPassTiming, usize)>,
) -> Result<GpuWorkOperation> {
    let PreparedFlowPipeline::Compute(realized_pipeline) = &pipeline.pipeline else {
        bail!(
            "compute pass '{}' realized a non-compute GPU pipeline before G5A operation projection",
            pass.pass_id
        );
    };
    let dispatch = flow_inputs
        .projected_dispatch_workgroups
        .get(&pass.pass_id)
        .copied()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "compute pass '{}' is missing its prepared dispatch projection",
                pass.pass_id
            )
        })?;
    let dispatch = GpuDispatchIntent::direct(
        GpuDispatchSize::new(dispatch[0], dispatch[1], dispatch[2])?,
        context.device_facts().device_limits().values(),
    )?;
    let mut operation = GpuComputeOperation::new(
        realized_pipeline.descriptor().clone(),
        pipeline.bindings.runtime_bindings.clone(),
        dispatch,
    )?;
    if let Some((timing, ordinal)) = timing {
        operation = operation.with_timestamp_writes([timestamp_access(timing, ordinal)?])?;
    }
    Ok(GpuWorkOperation::Compute(operation))
}

pub(super) fn timestamp_access(
    timing: &LogicalGpuPassTiming,
    ordinal: usize,
) -> Result<GpuQueryAccess> {
    let indices = timing.range_for_occurrence(ordinal)?;
    let range = GpuQueryRange::new(timing.query_set(), indices.begin, 2)?;
    Ok(GpuQueryAccess::new(
        timing.query_set(),
        range,
        GpuQueryAccessKind::WriteTimestamp,
    )?)
}
