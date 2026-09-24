use super::super::resource_descriptors::{buffer_descriptor, owned_common};
use super::*;
use crate::plugins::render::adapters::RenderGpuFrameTimingBracket;
use runen_gpu::{
    GpuBufferHandle, GpuBufferRange, GpuBufferRegion, GpuBufferUsage, GpuContext, GpuMemoryIntent,
    GpuQueryKind, GpuQueryRange, GpuQueryResolveOperation, GpuQuerySetDescriptor,
    GpuQuerySetHandle, GpuReadbackBytes, GpuReadbackId, GpuReadbackOperation, GpuResourceLifetime,
    GpuSubmissionFailure, GpuTimestampMarkerOperation, GpuWorkFragment, GpuWorkResourceIdAllocator,
};

const TIMESTAMP_SIZE_BYTES: usize = 8;

pub(in crate::plugins::render::renderer) struct PreparedComposedGpuTiming {
    bracket: RenderGpuFrameTimingBracket,
    frame: GpuComposedFrameTimingFrame,
}

impl PreparedComposedGpuTiming {
    pub fn bracket(&self) -> &RenderGpuFrameTimingBracket {
        &self.bracket
    }

    pub fn into_frame(self) -> GpuComposedFrameTimingFrame {
        self.frame
    }
}

#[derive(Debug)]
pub(in crate::plugins::render::renderer) struct GpuComposedFrameTimingFrame {
    readback_id: GpuReadbackId,
    timestamp_period_ns: f32,
    frame_index: u64,
    render_surface_id: u64,
}

impl GpuComposedFrameTimingFrame {
    pub const fn readback_id(&self) -> GpuReadbackId {
        self.readback_id
    }

    pub fn pending_evidence(&self) -> RenderComposedFrameGpuTimingEvidence {
        RenderComposedFrameGpuTimingEvidence::gpu_diagnostic(
            self.frame_index,
            self.render_surface_id,
            RenderGpuTimingDiagnostic::readback_pending(
                "composed renderer GPU timestamp submission was accepted; readback is pending",
            ),
        )
    }

    pub fn ready_evidence(
        &self,
        readback: &GpuReadbackBytes,
    ) -> RenderComposedFrameGpuTimingEvidence {
        if readback.texture_format().is_some() {
            return self.diagnostic_evidence(
                "composed renderer GPU timestamp readback unexpectedly carried texture metadata",
            );
        }
        let bytes = readback.as_bytes();
        if bytes.len() != 2 * TIMESTAMP_SIZE_BYTES {
            return self.diagnostic_evidence(format!(
                "composed renderer GPU timestamp readback byte length mismatch: expected {}, got {}",
                2 * TIMESTAMP_SIZE_BYTES,
                bytes.len()
            ));
        }
        let Some(begin) = decode_timestamp(bytes, 0) else {
            return self.diagnostic_evidence("composed renderer GPU start timestamp is missing");
        };
        let Some(end) = decode_timestamp(bytes, 1) else {
            return self.diagnostic_evidence("composed renderer GPU end timestamp is missing");
        };
        let Some(delta_ticks) = end.checked_sub(begin) else {
            return self.diagnostic_evidence(format!(
                "composed renderer GPU end timestamp {end} precedes start timestamp {begin}"
            ));
        };
        let millis = (delta_ticks as f64) * f64::from(self.timestamp_period_ns) / 1_000_000.0;
        if !millis.is_finite() || millis > f64::from(f32::MAX) {
            return self.diagnostic_evidence(
                "composed renderer GPU timestamp conversion exceeded the timing domain",
            );
        }
        RenderComposedFrameGpuTimingEvidence::gpu_sample(
            self.frame_index,
            self.render_surface_id,
            millis as f32,
        )
    }

    pub fn failed_evidence(
        &self,
        failure: &GpuSubmissionFailure,
    ) -> RenderComposedFrameGpuTimingEvidence {
        self.diagnostic_evidence(format!(
            "composed renderer GPU timestamp readback failed ({:?}): {}",
            failure.kind(),
            failure.detail()
        ))
    }

    pub fn diagnostic_evidence(
        &self,
        message: impl Into<String>,
    ) -> RenderComposedFrameGpuTimingEvidence {
        RenderComposedFrameGpuTimingEvidence::gpu_diagnostic(
            self.frame_index,
            self.render_surface_id,
            RenderGpuTimingDiagnostic::unavailable_this_frame(message),
        )
    }
}

pub(in crate::plugins::render::renderer) fn prepare_composed_gpu_timing(
    context: &GpuContext,
    frame_index: u64,
    render_surface_id: u64,
) -> Result<PreparedComposedGpuTiming> {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let query_set = allocator.allocate_query_set_handle(GpuQuerySetDescriptor::new(
        owned_common(
            "render.frame.composed.timestamps",
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Device,
        )?,
        GpuQueryKind::Timestamp,
        2,
    )?)?;
    let resolve_buffer = allocator.allocate_buffer_handle(buffer_descriptor(
        "render.frame.composed.timestamp-resolve",
        (2 * TIMESTAMP_SIZE_BYTES) as u64,
        [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
        GpuResourceLifetime::Transient,
        GpuMemoryIntent::Device,
    )?)?;
    let readback_id = GpuReadbackId::allocate()?;
    let resolve = GpuQueryResolveOperation::new(
        &query_set,
        GpuQueryRange::new(&query_set, 0, 2)?,
        &resolve_buffer,
        0,
    )?;
    let readback = GpuReadbackOperation::new(
        GpuBufferRegion::new(
            &resolve_buffer,
            GpuBufferRange::new(&resolve_buffer, 0, (2 * TIMESTAMP_SIZE_BYTES) as u64)?,
        )?
        .into(),
        readback_id,
    )?;
    let start_marker = GpuTimestampMarkerOperation::new(&query_set, 0)?;
    let end_marker = GpuTimestampMarkerOperation::new(&query_set, 1)?;
    let mut start_node = None;
    let mut end_node = None;
    let fragment = GpuWorkFragment::build("render.frame.composed-timing", |work| {
        start_node = Some(work.operation("composed renderer GPU timing start", start_marker)?);
        end_node = Some(work.operation("composed renderer GPU timing end", end_marker)?);
        work.operation("resolve composed renderer GPU timestamps", resolve)?;
        work.operation("read back composed renderer GPU timestamps", readback)?;
        Ok(())
    })?;
    let timestamp_period_ns = context.timestamp_period_ns().ok_or_else(|| {
        anyhow::anyhow!("composed renderer GPU timing was admitted without a timestamp period")
    })?;
    Ok(PreparedComposedGpuTiming {
        bracket: RenderGpuFrameTimingBracket::new(
            fragment,
            start_node.expect("composed timing start marker must be authored"),
            end_node.expect("composed timing end marker must be authored"),
        ),
        frame: GpuComposedFrameTimingFrame {
            readback_id,
            timestamp_period_ns,
            frame_index,
            render_surface_id,
        },
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::plugins::render::renderer) struct GpuPassTimestampIndices {
    pub begin: u32,
    pub end: u32,
}

#[derive(Debug, Clone)]
struct GpuPassTimingEntry {
    frame_index: u64,
    render_surface_id: u64,
    flow_id: String,
    pass_id: String,
    pass_kind: String,
    indices: GpuPassTimestampIndices,
}

/// Renderer-owned observation metadata for timestamp operations submitted by G5.
///
/// Query/resolve/readback resources are materialized only by RunenGPU preparation. The renderer
/// retains semantic identities so it can publish truthful pending evidence at acceptance.
#[derive(Debug)]
pub(in crate::plugins::render::renderer) struct GpuPassTimingFrame {
    readback_id: GpuReadbackId,
    timestamp_period_ns: f32,
    query_capacity: u32,
    entries: Vec<GpuPassTimingEntry>,
}

impl GpuPassTimingFrame {
    pub fn new(
        context: &GpuContext,
        query_set_handle: &GpuQuerySetHandle,
        resolve_buffer_handle: &GpuBufferHandle,
        readback_id: GpuReadbackId,
        query_capacity: u32,
    ) -> Result<Self> {
        if query_capacity == 0 {
            anyhow::bail!(
                "GPU timing frame requires nonzero query capacity after logical timing admission"
            );
        }
        let readback_size = u64::from(query_capacity) * TIMESTAMP_SIZE_BYTES as u64;
        if query_set_handle.descriptor().count() != query_capacity
            || resolve_buffer_handle.descriptor().size_bytes() < readback_size
        {
            anyhow::bail!(
                "prepared timing handles do not cover the declared query capacity {query_capacity}"
            );
        }
        let timestamp_period_ns = context.timestamp_period_ns().ok_or_else(|| {
            anyhow::anyhow!(
                "GPU timing was admitted without a finite backend-neutral timestamp period"
            )
        })?;
        Ok(Self {
            readback_id,
            timestamp_period_ns,
            query_capacity,
            entries: Vec::new(),
        })
    }

    pub fn register_pass_metadata(
        &mut self,
        indices: GpuPassTimestampIndices,
        frame_index: u64,
        render_surface_id: u64,
        flow_id: impl Into<String>,
        pass_id: impl Into<String>,
        pass_kind: impl Into<String>,
    ) -> bool {
        if indices.begin >= indices.end
            || indices.end >= self.query_capacity
            || self.entries.iter().any(|entry| entry.indices == indices)
        {
            return false;
        }
        self.entries.push(GpuPassTimingEntry {
            frame_index,
            render_surface_id,
            flow_id: flow_id.into(),
            pass_id: pass_id.into(),
            pass_kind: pass_kind.into(),
            indices,
        });
        true
    }

    pub const fn readback_id(&self) -> GpuReadbackId {
        self.readback_id
    }

    pub fn pending_evidence(&self) -> Vec<RenderPassTimingEvidence> {
        self.entries
            .iter()
            .map(|entry| {
                RenderPassTimingEvidence::gpu_diagnostic(
                    Some(entry.frame_index),
                    Some(entry.render_surface_id),
                    entry.flow_id.clone(),
                    entry.pass_id.clone(),
                    entry.pass_kind.clone(),
                    RenderGpuTimingDiagnostic::readback_pending(
                        "GPU timestamp submission was accepted; readback is pending",
                    ),
                )
            })
            .collect()
    }

    pub fn ready_evidence(&self, readback: &GpuReadbackBytes) -> Vec<RenderPassTimingEvidence> {
        if readback.texture_format().is_some() {
            return self.diagnostic_evidence(
                "GPU timestamp readback unexpectedly carried texture-format metadata",
            );
        }
        let expected_len =
            usize::try_from(u64::from(self.query_capacity) * TIMESTAMP_SIZE_BYTES as u64)
                .expect("u32 timestamp query capacity times eight fits usize on supported targets");
        let bytes = readback.as_bytes();
        if bytes.len() != expected_len {
            return self.diagnostic_evidence(format!(
                "GPU timestamp readback byte length mismatch: expected {expected_len}, got {}",
                bytes.len()
            ));
        }

        self.entries
            .iter()
            .map(|entry| {
                let begin = decode_timestamp(bytes, entry.indices.begin);
                let end = decode_timestamp(bytes, entry.indices.end);
                let Some((begin, end)) = begin.zip(end) else {
                    return timing_diagnostic(
                        entry,
                        "GPU timestamp readback did not contain the admitted begin/end pair",
                    );
                };
                let Some(delta_ticks) = end.checked_sub(begin) else {
                    return timing_diagnostic(
                        entry,
                        format!(
                            "GPU timestamp end value {end} precedes begin value {begin}; no sample was published"
                        ),
                    );
                };
                let millis = (delta_ticks as f64) * f64::from(self.timestamp_period_ns) / 1_000_000.0;
                if !millis.is_finite() || millis > f64::from(f32::MAX) {
                    return timing_diagnostic(
                        entry,
                        "GPU timestamp conversion exceeded the renderer timing domain",
                    );
                }
                RenderPassTimingEvidence::gpu_sample(
                    Some(entry.frame_index),
                    Some(entry.render_surface_id),
                    entry.flow_id.clone(),
                    entry.pass_id.clone(),
                    entry.pass_kind.clone(),
                    millis as f32,
                )
            })
            .collect()
    }

    pub fn failed_evidence(&self, failure: &GpuSubmissionFailure) -> Vec<RenderPassTimingEvidence> {
        self.diagnostic_evidence(format!(
            "GPU timestamp readback failed ({:?}): {}",
            failure.kind(),
            failure.detail()
        ))
    }

    pub fn diagnostic_evidence(&self, message: impl Into<String>) -> Vec<RenderPassTimingEvidence> {
        let message = message.into();
        self.entries
            .iter()
            .map(|entry| timing_diagnostic(entry, message.clone()))
            .collect()
    }
}

fn decode_timestamp(bytes: &[u8], index: u32) -> Option<u64> {
    let start = usize::try_from(index)
        .ok()?
        .checked_mul(TIMESTAMP_SIZE_BYTES)?;
    let end = start.checked_add(TIMESTAMP_SIZE_BYTES)?;
    let encoded: [u8; TIMESTAMP_SIZE_BYTES] = bytes.get(start..end)?.try_into().ok()?;
    Some(u64::from_le_bytes(encoded))
}

fn timing_diagnostic(
    entry: &GpuPassTimingEntry,
    message: impl Into<String>,
) -> RenderPassTimingEvidence {
    RenderPassTimingEvidence::gpu_diagnostic(
        Some(entry.frame_index),
        Some(entry.render_surface_id),
        entry.flow_id.clone(),
        entry.pass_id.clone(),
        entry.pass_kind.clone(),
        RenderGpuTimingDiagnostic::unavailable_this_frame(message),
    )
}

#[cfg(test)]
mod composed_native_proof {
    use super::*;
    use crate::plugins::render::adapters::{
        RenderGpuWorkOccurrenceId, ResolvedRenderGpuWorkNode, prepare_render_gpu_frame_work,
    };
    use runen_gpu::{
        GpuBufferDescriptor, GpuBufferInitialization, GpuBufferRegion, GpuBufferUsage,
        GpuCapabilityFeature, GpuCapabilityProfile, GpuCapabilityRequirement, GpuClearOperation,
        GpuContextDescriptor, GpuContextRequestErrorCategory, GpuExecutionPreference,
        GpuReadbackStatus, GpuReconstruction, GpuSubmissionStatus, GpuWorkFragment,
        GpuWorkOperation, GpuWorkResourceIdAllocator,
    };
    use std::time::{Duration, Instant};

    fn timestamp_context() -> Option<GpuContext> {
        let mut requirements = GpuCapabilityProfile::ComputeBaseline.requirements();
        requirements
            .insert(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::TimestampQuery,
            ))
            .expect("timestamp requirement should be compatible with compute baseline");
        let descriptor =
            GpuContextDescriptor::new(requirements).with_label("RunenRender composed timing proof");
        match pollster::block_on(GpuContext::request(descriptor)) {
            Ok(context) => Some(context),
            Err(error)
                if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable =>
            {
                assert_ne!(
                    std::env::var("RUNENRENDER_R7_REQUIRE_GPU").ok().as_deref(),
                    Some("1"),
                    "permanent R7 composition CI requires a timestamp-capable RunenGPU adapter"
                );
                None
            }
            Err(error) => panic!("unexpected composed timing RunenGPU context failure: {error}"),
        }
    }

    fn clear_buffer(
        allocator: &mut GpuWorkResourceIdAllocator,
        label: &str,
    ) -> runen_gpu::GpuBufferHandle {
        allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::ordinary_owned(
                    label,
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    4,
                    [GpuBufferUsage::CopyDestination],
                    GpuBufferInitialization::Uninitialized,
                )
                .expect("timing proof buffer descriptor"),
            )
            .expect("timing proof buffer handle")
    }

    #[test]
    fn deterministic_composition_r7_proof_composed_timing_progresses_pending_to_measured() {
        let Some(context) = timestamp_context() else {
            return;
        };

        let mut allocator = GpuWorkResourceIdAllocator::new();
        let producer_buffer = clear_buffer(&mut allocator, "timed immutable producer buffer");
        let producer_clear = GpuClearOperation::buffer_zero(
            GpuBufferRegion::whole(&producer_buffer).expect("producer clear region"),
        )
        .expect("producer clear operation");
        let producer = GpuWorkFragment::build("timed immutable producer fragment", |work| {
            work.operation("timed immutable producer clear", producer_clear)?;
            Ok(())
        })
        .expect("producer fragment");

        let renderer_buffer = clear_buffer(&mut allocator, "timed renderer buffer");
        let renderer_clear = GpuClearOperation::buffer_zero(
            GpuBufferRegion::whole(&renderer_buffer).expect("renderer clear region"),
        )
        .expect("renderer clear operation");
        let renderer_node = ResolvedRenderGpuWorkNode::pass(
            RenderGpuWorkOccurrenceId::new(1),
            GpuResourceLabel::new("timed renderer clear").expect("renderer node label"),
            GpuWorkOperation::Clear(renderer_clear),
            GpuExecutionPreference::TransferPreferred,
            [],
        );

        let timing = prepare_composed_gpu_timing(&context, 77, 5)
            .expect("composed timing resources should prepare");
        let graph = prepare_render_gpu_frame_work(
            &context,
            GpuResourceLabel::new("native composed timing proof").expect("graph label"),
            [renderer_node],
            &[producer],
            &[],
            Some(timing.bracket()),
        )
        .expect("composed timing graph should prepare");
        let timing_frame = timing.into_frame();

        let prepared = pollster::block_on(context.prepare_submission(graph))
            .expect("composed timing graph should realize");
        let submission = context
            .submit_prepared(prepared)
            .expect("composed timing submission should be accepted");
        let readback = submission
            .readback(timing_frame.readback_id())
            .expect("accepted composed timing submission must retain its readback")
            .clone();

        let pending = timing_frame.pending_evidence();
        assert_eq!(pending.frame_index, 77);
        assert_eq!(pending.render_surface_id, 5);
        assert_eq!(
            pending.gpu_capability,
            RenderGpuTimingCapability::ReadbackPending
        );
        assert_eq!(pending.millis, None);

        let deadline = Instant::now() + Duration::from_secs(15);
        let measured = loop {
            context.progress();
            match readback.status() {
                GpuReadbackStatus::Pending => {
                    assert!(
                        Instant::now() < deadline,
                        "composed timing readback did not complete before timeout"
                    );
                    std::thread::yield_now();
                }
                GpuReadbackStatus::Ready(bytes) => break timing_frame.ready_evidence(&bytes),
                GpuReadbackStatus::Failed(failure) => {
                    panic!("composed timing readback failed: {failure:?}")
                }
            }
        };

        assert!(matches!(
            submission.status(),
            GpuSubmissionStatus::Completed | GpuSubmissionStatus::Accepted
        ));
        assert_eq!(measured.frame_index, 77);
        assert_eq!(measured.render_surface_id, 5);
        assert_eq!(
            measured.gpu_capability,
            RenderGpuTimingCapability::Supported
        );
        assert!(
            measured
                .millis
                .is_some_and(|millis| millis.is_finite() && millis >= 0.0),
            "completed composed timing must publish one finite GPU duration"
        );
        assert!(measured.diagnostics.is_empty());
    }
}
