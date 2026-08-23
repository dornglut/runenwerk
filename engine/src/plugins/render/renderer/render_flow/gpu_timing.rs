use super::logical_operations::{ProjectedTimingTail, project_timing_tail};
use super::*;
use crate::plugins::gpu::{
    CurrentRenderBufferCopyTerminal, CurrentRenderReadbackBufferTerminal,
    CurrentRenderTimestampResourcesTerminal, GpuBufferHandle, GpuBufferUsage, GpuContext,
    GpuMemoryIntent, GpuQueryRange, GpuQueryResolveOperation, GpuQuerySetHandle, GpuReadbackId,
    GpuReadbackOperation, GpuRealizedBuffer, GpuRealizedQuerySet, GpuResourceLifetime,
    GpuTransferRegion, GpuWorkResourceIdAllocator,
};
use crate::plugins::render::renderer::resource_descriptors::buffer_descriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::plugins::render::renderer) struct GpuPassTimestampIndices {
    pub begin: u32,
    pub end: u32,
}

#[derive(Debug, Clone)]
pub(in crate::plugins::render::renderer) struct GpuPassTimestampWrites {
    pub query_set: GpuRealizedQuerySet,
    pub indices: GpuPassTimestampIndices,
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

#[derive(Debug)]
pub(in crate::plugins::render::renderer) struct GpuPassTimingFrame {
    timing_tail: ProjectedTimingTail,
    query_set: GpuRealizedQuerySet,
    resolve_buffer: GpuRealizedBuffer,
    /// Temporary physical staging identity for the raw renderer executor only. Canonical timing
    /// readback is `GpuReadbackOperation` from the resolve buffer and does not name this resource.
    _legacy_readback_buffer_handle: GpuBufferHandle,
    legacy_readback_buffer: GpuRealizedBuffer,
    query_capacity: u32,
    query_count: u32,
    timestamp_period_ns: f32,
    entries: Vec<GpuPassTimingEntry>,
    resolve_encoded: bool,
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
                "physical GPU timing frame requires nonzero query capacity after logical timing admission"
            );
        }
        let readback_size = u64::from(query_capacity) * u64::from(QUERY_SIZE);
        if query_set_handle.descriptor().count() != query_capacity
            || resolve_buffer_handle.descriptor().size_bytes() < readback_size
        {
            anyhow::bail!(
                "prepared timing handles do not cover the declared query capacity {query_capacity}"
            );
        }
        let timing_tail = project_timing_tail(
            query_set_handle,
            GpuQueryRange::new(query_set_handle, 0, query_capacity)?,
            resolve_buffer_handle,
            readback_id,
        )?;
        let query_set = context.realize_query_set(query_set_handle)?;
        let resolve_buffer = context.realize_buffer(resolve_buffer_handle)?;

        // Removal condition: delete this resource with the raw timing encoder/map bridge when the
        // frame-wide GPU submission path consumes `timing_tail.readback()` directly.
        let mut legacy_allocator = GpuWorkResourceIdAllocator::new();
        let legacy_readback_buffer_handle =
            legacy_allocator.allocate_buffer_handle(buffer_descriptor(
                "render.flow.timestamp_legacy_readback",
                readback_size,
                [GpuBufferUsage::CopyDestination],
                GpuResourceLifetime::Transient,
                GpuMemoryIntent::Readback,
            )?)?;
        let legacy_readback_buffer = context.realize_buffer(&legacy_readback_buffer_handle)?;
        Ok(Self {
            timing_tail,
            query_set,
            resolve_buffer,
            _legacy_readback_buffer_handle: legacy_readback_buffer_handle,
            legacy_readback_buffer,
            query_capacity,
            query_count: 0,
            timestamp_period_ns: context.timestamp_period_ns().unwrap_or(0.0),
            entries: Vec::new(),
            resolve_encoded: false,
        })
    }

    pub(super) fn resolve_operation(&self) -> &GpuQueryResolveOperation {
        self.timing_tail.resolve()
    }

    pub(super) fn readback_operation(&self) -> &GpuReadbackOperation {
        self.timing_tail.readback()
    }

    /// Renderer timing interpretation is backend-neutral. The temporary raw executor only needs
    /// to know whether timestamp values can be interpreted for this frame; no Queue observation is
    /// part of that decision.
    pub fn timestamp_scale_available(&self) -> bool {
        self.timestamp_period_ns > 0.0
    }

    /// Registers renderer-owned evidence identity for one already-admitted timestamp range.
    ///
    /// This is realization metadata, not physical encoder state. The temporary raw bridge consumes
    /// it later only to decode bytes; the eventual frame-wide submission path can consume the same
    /// metadata with normalized readback results.
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
        self.query_count = self.query_count.max(indices.end.saturating_add(1));
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

    pub fn timestamp_writes(&self, indices: GpuPassTimestampIndices) -> GpuPassTimestampWrites {
        GpuPassTimestampWrites {
            query_set: self.query_set.clone(),
            indices,
        }
    }

    pub fn encode_resolve(
        &mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        operation: &GpuQueryResolveOperation,
    ) -> Result<bool> {
        if self.query_count == 0 {
            return Ok(false);
        }
        if operation != self.timing_tail.resolve() {
            anyhow::bail!(
                "scheduled canonical timing resolve disagrees with the admitted timing tail"
            );
        }
        if operation.source_range().count() != self.query_count {
            anyhow::bail!(
                "scheduled canonical timing resolve covers {} queries but {} timestamp queries were registered",
                operation.source_range().count(),
                self.query_count
            );
        }
        context
            .current_render_execution_bridge()
            .for_timestamp_resources(
                &self.query_set,
                &self.resolve_buffer,
                &self.legacy_readback_buffer,
                ResolveTimingQueries {
                    encoder,
                    query_range: operation.source_range(),
                    destination_offset: operation.destination_offset(),
                },
            )?;
        self.resolve_encoded = true;
        Ok(true)
    }

    /// Temporary raw execution of the canonical readback operation.
    ///
    /// Source range and readback identity come exclusively from `operation`; the renderer-owned
    /// destination below is private staging required only because the frame still submits through
    /// the legacy encoder. It is not part of canonical GPU work and is deleted with that bridge.
    pub fn encode_legacy_readback(
        self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        operation: &GpuReadbackOperation,
    ) -> Result<Option<PendingGpuPassTimingReadback>> {
        if !self.resolve_encoded || self.query_count == 0 {
            return Ok(None);
        }
        if operation != self.timing_tail.readback() {
            anyhow::bail!(
                "scheduled canonical timing readback disagrees with the admitted timing tail"
            );
        }
        let GpuTransferRegion::Buffer(source) = operation.source() else {
            anyhow::bail!("renderer timing readback requires a canonical buffer source");
        };
        if source.buffer().diagnostic_identity() != self.resolve_buffer.logical_identity() {
            anyhow::bail!(
                "scheduled canonical timing readback source disagrees with its admitted resolve-buffer realization"
            );
        }
        let readback_size = source.range().size();
        context.current_render_execution_bridge().for_buffer_copy(
            &self.resolve_buffer,
            &self.legacy_readback_buffer,
            CopyTimingReadback {
                encoder,
                source_offset: source.range().offset(),
                destination_offset: 0,
                readback_size,
            },
        )?;
        Ok(Some(PendingGpuPassTimingReadback {
            readback_buffer: self.legacy_readback_buffer,
            readback_size,
            timestamp_period_ns: self.timestamp_period_ns,
            entries: self.entries,
        }))
    }
}

struct ResolveTimingQueries<'a> {
    encoder: &'a mut CommandEncoder,
    query_range: GpuQueryRange,
    destination_offset: BufferAddress,
}

impl CurrentRenderTimestampResourcesTerminal for ResolveTimingQueries<'_> {
    fn use_timestamp_resources(
        self,
        query_set: &QuerySet,
        resolve_buffer: &Buffer,
        _readback_buffer: &Buffer,
    ) {
        let first = self.query_range.first();
        let end = first + self.query_range.count();
        self.encoder.resolve_query_set(
            query_set,
            first..end,
            resolve_buffer,
            self.destination_offset,
        );
    }
}

struct CopyTimingReadback<'a> {
    encoder: &'a mut CommandEncoder,
    source_offset: BufferAddress,
    destination_offset: BufferAddress,
    readback_size: BufferAddress,
}

impl CurrentRenderBufferCopyTerminal for CopyTimingReadback<'_> {
    fn copy_buffers(self, source: &Buffer, destination: &Buffer) {
        self.encoder.copy_buffer_to_buffer(
            source,
            self.source_offset,
            destination,
            self.destination_offset,
            self.readback_size,
        );
    }
}

#[derive(Debug)]
pub(in crate::plugins::render::renderer) struct PendingGpuPassTimingReadback {
    readback_buffer: GpuRealizedBuffer,
    readback_size: BufferAddress,
    timestamp_period_ns: f32,
    entries: Vec<GpuPassTimingEntry>,
}

struct ReadGpuTimingBuffer<'a> {
    device: &'a Device,
    readback_size: BufferAddress,
    timestamp_period_ns: f32,
    entries: Vec<GpuPassTimingEntry>,
    output: &'a mut Option<Vec<RenderPassTimingEvidence>>,
}

impl CurrentRenderReadbackBufferTerminal for ReadGpuTimingBuffer<'_> {
    fn read_buffer(self, buffer: &Buffer) {
        *self.output = Some(read_gpu_timing_buffer(
            self.device,
            buffer,
            self.readback_size,
            self.timestamp_period_ns,
            self.entries,
        ));
    }
}

pub(in crate::plugins::render::renderer) fn read_gpu_pass_timing_evidence(
    context: &GpuContext,
    device: &Device,
    pending: PendingGpuPassTimingReadback,
) -> Vec<RenderPassTimingEvidence> {
    let PendingGpuPassTimingReadback {
        readback_buffer,
        readback_size,
        timestamp_period_ns,
        entries,
    } = pending;
    let fallback_entries = entries.clone();
    let mut output = None;
    let bridge_result = context
        .current_render_execution_bridge()
        .for_buffer_readback(
            &readback_buffer,
            ReadGpuTimingBuffer {
                device,
                readback_size,
                timestamp_period_ns,
                entries,
                output: &mut output,
            },
        );
    if let Err(error) = bridge_result {
        return fallback_entries
            .into_iter()
            .map(|entry| {
                gpu_timing_unavailable_evidence(
                    entry,
                    format!("GPU timestamp resource bridge rejected readback: {error}"),
                )
            })
            .collect();
    }
    output.unwrap_or_else(|| {
        fallback_entries
            .into_iter()
            .map(|entry| {
                gpu_timing_unavailable_evidence(
                    entry,
                    "GPU timestamp resource bridge produced no readback evidence",
                )
            })
            .collect()
    })
}

fn read_gpu_timing_buffer(
    device: &Device,
    readback_buffer: &Buffer,
    readback_size: BufferAddress,
    timestamp_period_ns: f32,
    entries: Vec<GpuPassTimingEntry>,
) -> Vec<RenderPassTimingEvidence> {
    let slice = readback_buffer.slice(0..readback_size);
    let (sender, receiver) = channel();
    slice.map_async(MapMode::Read, move |result| {
        let _ = sender.send(result);
    });

    if let Err(err) = device.poll(PollType::wait_indefinitely()) {
        return entries
            .into_iter()
            .map(|entry| {
                gpu_timing_unavailable_evidence(
                    entry,
                    format!("device.poll failed for GPU timestamp readback: {err}"),
                )
            })
            .collect();
    }

    match receiver.recv() {
        Ok(Ok(())) => {}
        Ok(Err(err)) => {
            return entries
                .into_iter()
                .map(|entry| {
                    gpu_timing_unavailable_evidence(
                        entry,
                        format!("GPU timestamp map_async failed: {err}"),
                    )
                })
                .collect();
        }
        Err(err) => {
            return entries
                .into_iter()
                .map(|entry| {
                    gpu_timing_unavailable_evidence(
                        entry,
                        format!("GPU timestamp map_async channel failed: {err}"),
                    )
                })
                .collect();
        }
    }

    let data = match slice.get_mapped_range() {
        Ok(data) => data,
        Err(err) => {
            readback_buffer.unmap();
            return entries
                .into_iter()
                .map(|entry| {
                    gpu_timing_unavailable_evidence(
                        entry,
                        format!("failed to access mapped GPU timestamp bytes: {err}"),
                    )
                })
                .collect();
        }
    };
    let evidence = entries
        .into_iter()
        .map(|entry| gpu_timing_evidence_from_bytes(&data, timestamp_period_ns, entry))
        .collect::<Vec<_>>();
    drop(data);
    readback_buffer.unmap();
    evidence
}

fn gpu_timing_evidence_from_bytes(
    data: &[u8],
    timestamp_period_ns: f32,
    entry: GpuPassTimingEntry,
) -> RenderPassTimingEvidence {
    let begin = query_timestamp(data, entry.indices.begin);
    let end = query_timestamp(data, entry.indices.end);
    let Some((begin, end)) = begin.zip(end) else {
        return gpu_timing_unavailable_evidence(
            entry,
            "GPU timestamp readback bytes did not contain both pass queries",
        );
    };
    if end < begin {
        return gpu_timing_unavailable_evidence(
            entry,
            "GPU timestamp readback ended before it began",
        );
    }
    let millis = ((end - begin) as f64 * f64::from(timestamp_period_ns) / 1_000_000.0) as f32;
    RenderPassTimingEvidence::gpu_sample(
        Some(entry.frame_index),
        Some(entry.render_surface_id),
        entry.flow_id,
        entry.pass_id,
        entry.pass_kind,
        millis,
    )
}

fn query_timestamp(data: &[u8], query_index: u32) -> Option<u64> {
    let start = query_index as usize * QUERY_SIZE as usize;
    let end = start.checked_add(QUERY_SIZE as usize)?;
    let bytes: [u8; QUERY_SIZE as usize] = data.get(start..end)?.try_into().ok()?;
    Some(u64::from_le_bytes(bytes))
}

fn gpu_timing_unavailable_evidence(
    entry: GpuPassTimingEntry,
    message: impl Into<String>,
) -> RenderPassTimingEvidence {
    RenderPassTimingEvidence::gpu_diagnostic(
        Some(entry.frame_index),
        Some(entry.render_surface_id),
        entry.flow_id,
        entry.pass_id,
        entry.pass_kind,
        RenderGpuTimingDiagnostic::unavailable_this_frame(message),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        CurrentRenderTimestampWritesTerminal, GpuCapabilityFeature, GpuCapabilityProfile,
        GpuCapabilityRequirement, GpuContext, GpuContextDescriptor, GpuMemoryIntent,
        GpuPreferredFallback, GpuQueryKind, GpuQuerySetDescriptor, GpuResourceLifetime,
        GpuWorkResourceIdAllocator,
    };
    use crate::plugins::render::inspect::RenderTimingSource;
    use crate::plugins::render::renderer::resource_descriptors::owned_common;
    use pollster::block_on;

    #[test]
    #[ignore = "runtime evidence test: requires a local WGPU adapter and may depend on driver timestamp-query support"]
    fn render_gpu_timing_runtime_query_readback_reports_measured_or_unsupported() {
        let mut requirements = GpuCapabilityProfile::ComputeBaseline.requirements();
        requirements
            .insert(GpuCapabilityRequirement::Preferred {
                feature: GpuCapabilityFeature::TimestampQuery,
                fallback: GpuPreferredFallback::DisableInstrumentation,
            })
            .expect("timestamp preference should merge");
        let context = match block_on(GpuContext::request(GpuContextDescriptor::new(requirements))) {
            Ok(context) => context,
            Err(error) => {
                println!("runtime GPU timing evidence: {error}");
                return;
            }
        };
        if !context
            .device_facts()
            .is_enabled(GpuCapabilityFeature::TimestampQuery)
        {
            println!("runtime GPU timing evidence: timestamp queries unsupported by adapter");
            return;
        }
        let mut allocator = GpuWorkResourceIdAllocator::new();
        let query_set = allocator
            .allocate_query_set_handle(
                GpuQuerySetDescriptor::new(
                    owned_common(
                        "engine_test_gpu_timestamps",
                        GpuResourceLifetime::Transient,
                        GpuMemoryIntent::Device,
                    )
                    .expect("query common descriptor"),
                    GpuQueryKind::Timestamp,
                    2,
                )
                .expect("query descriptor"),
            )
            .expect("query handle");
        let resolve_buffer = allocator
            .allocate_buffer_handle(
                buffer_descriptor(
                    "engine_test_gpu_timestamp_resolve",
                    16,
                    [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
                    GpuResourceLifetime::Transient,
                    GpuMemoryIntent::Device,
                )
                .expect("resolve descriptor"),
            )
            .expect("resolve handle");
        let readback_id = GpuReadbackId::allocate().expect("readback identity should allocate");
        let mut frame =
            GpuPassTimingFrame::new(&context, &query_set, &resolve_buffer, readback_id, 2)
                .expect("timestamp resources should realize");
        let resolve_operation = frame.resolve_operation().clone();
        let readback_operation = frame.readback_operation().clone();
        let indices = GpuPassTimestampIndices { begin: 0, end: 1 };
        assert!(frame.register_pass_metadata(
            indices,
            1,
            1,
            "runtime.gpu",
            "timestamp.empty_compute",
            "compute",
        ));
        let evidence = {
            let loan = context.current_render_device_queue();
            assert!(frame.timestamp_scale_available());
            let writes = frame.timestamp_writes(indices);
            let mut encoder = loan
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("engine_test_gpu_timestamp_encoder"),
                });
            context
                .current_render_execution_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTestTimestampPass {
                        encoder: &mut encoder,
                        indices: writes.indices,
                    },
                )
                .expect("timestamp query should bridge");
            assert!(
                frame
                    .encode_resolve(&context, &mut encoder, &resolve_operation)
                    .expect("timestamp resolve should encode")
            );
            let pending = frame
                .encode_legacy_readback(&context, &mut encoder, &readback_operation)
                .expect("timestamp readback should encode")
                .expect("timestamp queries should resolve");
            loan.queue.submit(std::iter::once(encoder.finish()));
            read_gpu_pass_timing_evidence(&context, loan.device, pending)
        };

        println!("runtime GPU timing evidence: {evidence:?}");
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].source, RenderTimingSource::GpuTimestampQuery);
        assert_eq!(
            evidence[0].gpu_capability,
            RenderGpuTimingCapability::Supported
        );
        assert!(evidence[0].millis.is_some());
    }

    struct EncodeTestTimestampPass<'a> {
        encoder: &'a mut CommandEncoder,
        indices: GpuPassTimestampIndices,
    }

    impl CurrentRenderTimestampWritesTerminal for EncodeTestTimestampPass<'_> {
        fn write_timestamps(self, query_set: &QuerySet) {
            let _pass = self.encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("engine_test_gpu_timestamp_compute_pass"),
                timestamp_writes: Some(ComputePassTimestampWrites {
                    query_set,
                    beginning_of_pass_write_index: Some(self.indices.begin),
                    end_of_pass_write_index: Some(self.indices.end),
                }),
            });
        }
    }
}
