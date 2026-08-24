//! Final-path handoff from prepared renderer GPU work to RunenGPU submission.
//!
//! This module deliberately contains no second work model. It consumes the existing
//! `PreparedRenderWorkPlan`, discards its temporary raw-executor sidecar, and returns the one
//! authoritative prepared GPU work graph consumed by `GpuContext::prepare_submission`.

use super::gpu_work::PreparedRenderWorkPlan;
use crate::plugins::gpu::GpuPreparedWorkGraph;

pub(crate) fn into_submission_graph(plan: PreparedRenderWorkPlan) -> GpuPreparedWorkGraph {
    plan.graph().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuBufferDescriptor, GpuBufferInitialization, GpuBufferRange, GpuBufferUsage,
        GpuBufferUsages, GpuInitialCoverage, GpuMemoryIntent, GpuReconstruction,
        GpuResourceCommon, GpuResourceLabel, GpuResourceLifetime, GpuResourceProvenance,
        GpuUploadOperation, GpuWorkResourceIdAllocator, PreparedGpuData, TransferData,
    };
    use crate::plugins::render::adapters::gpu_work::{
        RenderGpuWorkOccurrenceId, ResolvedRenderGpuWorkNode, prepare_render_gpu_frame_work,
    };
    use std::num::NonZeroU64;

    #[test]
    fn submission_handoff_preserves_the_prepared_graph() {
        let label = GpuResourceLabel::new("frame submission handoff").unwrap();
        let buffer_label = GpuResourceLabel::new("frame submission buffer").unwrap();
        let common = GpuResourceCommon::owned(
            buffer_label.clone(),
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Device,
            GpuReconstruction::SourceBacked,
            GpuResourceProvenance::new(buffer_label.clone(), None, None),
        )
        .unwrap();
        let mut allocator =
            GpuWorkResourceIdAllocator::for_owner_scope(NonZeroU64::new(991).unwrap());
        let buffer = allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::new(
                    common,
                    16,
                    GpuBufferUsages::new(&buffer_label, [GpuBufferUsage::CopyDestination]).unwrap(),
                    GpuBufferInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();
        let payload = PreparedGpuData::<TransferData>::from_pod_transfer(
            "frame submission payload",
            &[0_u8; 16],
            GpuResourceProvenance::new(buffer_label.clone(), None, None),
        )
        .unwrap();
        let operation = GpuUploadOperation::new(
            crate::plugins::gpu::GpuBufferRegion::new(
                &buffer,
                GpuBufferRange::new(&buffer, 0, 16).unwrap(),
            )
            .unwrap()
            .into(),
            payload,
        )
        .unwrap();
        let plan = prepare_render_gpu_frame_work(
            label.clone(),
            [ResolvedRenderGpuWorkNode::upload(
                RenderGpuWorkOccurrenceId::new(1),
                GpuResourceLabel::new("frame submission upload").unwrap(),
                operation,
                [],
            )],
        )
        .unwrap();
        let node_count = plan.graph().nodes().len();
        let initialization_count = plan.graph().initialization().len();
        assert!(matches!(
            plan.graph()
                .initialization()
                .iter()
                .find(|entry| entry.resource().diagnostic_identity() == buffer.diagnostic_identity())
                .and_then(GpuInitialCoverage::final_coverage),
            _
        ));

        let graph = into_submission_graph(plan);
        assert_eq!(graph.label(), &label);
        assert_eq!(graph.nodes().len(), node_count);
        assert_eq!(graph.initialization().len(), initialization_count);
    }
}
