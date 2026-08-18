use super::logical_timing::LogicalGpuPassTiming;
use super::*;
use crate::plugins::gpu::{
    GpuBufferHandle, GpuBufferRange, GpuBufferRegion, GpuComputeOperation, GpuDispatchIntent,
    GpuDispatchSize, GpuQueryAccess, GpuQueryAccessKind, GpuQueryRange, GpuUploadOperation,
    GpuWorkOperation, PreparedGpuData, TransferData,
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

/// Projects an immutable renderer-prepared byte sequence into exact logical CPU→GPU buffer work.
/// Physical `queue.write_buffer` remains a temporary G5B/G5C realization detail.
pub(super) fn project_buffer_upload(
    buffer: &GpuBufferHandle,
    bytes: &[u8],
) -> Result<GpuUploadOperation> {
    let size = u64::try_from(bytes.len())
        .map_err(|_| anyhow::anyhow!("render GPU upload length exceeds u64"))?;
    let range = GpuBufferRange::new(buffer, 0, size)?;
    let region = GpuBufferRegion::new(buffer, range)?;
    let payload = PreparedGpuData::<TransferData>::from_pod_transfer(
        format!(
            "{}.render-upload",
            buffer.descriptor().common().label().as_str()
        ),
        bytes,
        buffer.descriptor().common().provenance().clone(),
    )?;
    Ok(GpuUploadOperation::new(region.into(), payload)?)
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
