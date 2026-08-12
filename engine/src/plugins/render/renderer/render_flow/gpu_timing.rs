use super::*;
use crate::plugins::gpu::{
    CurrentRenderReadbackBufferTerminal, CurrentRenderTimestampResourcesTerminal, GpuBufferHandle,
    GpuContext, GpuQuerySetHandle, GpuRealizedBuffer, GpuRealizedQuerySet,
};

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
    _query_set_handle: GpuQuerySetHandle,
    query_set: GpuRealizedQuerySet,
    _resolve_buffer_handle: GpuBufferHandle,
    resolve_buffer: GpuRealizedBuffer,
    readback_buffer_handle: GpuBufferHandle,
    readback_buffer: GpuRealizedBuffer,
    query_capacity: u32,
    query_count: u32,
    readback_size: BufferAddress,
    timestamp_period_ns: f32,
    entries: Vec<GpuPassTimingEntry>,
    resolve_encoded: bool,
}

impl GpuPassTimingFrame {
    pub fn new(
        context: &GpuContext,
        query_set_handle: &GpuQuerySetHandle,
        resolve_buffer_handle: &GpuBufferHandle,
        readback_buffer_handle: &GpuBufferHandle,
        query_capacity: u32,
    ) -> Result<Option<Self>> {
        if query_capacity == 0 {
            return Ok(None);
        }
        let readback_size = u64::from(query_capacity) * u64::from(QUERY_SIZE);
        if query_set_handle.descriptor().count() != query_capacity
            || resolve_buffer_handle.descriptor().size_bytes() < readback_size
            || readback_buffer_handle.descriptor().size_bytes() < readback_size
        {
            anyhow::bail!(
                "prepared timing handles do not cover the declared query capacity {query_capacity}"
            );
        }
        let query_set = context.realize_query_set(query_set_handle)?;
        let resolve_buffer = context.realize_buffer(resolve_buffer_handle)?;
        let readback_buffer = context.realize_buffer(readback_buffer_handle)?;
        Ok(Some(Self {
            _query_set_handle: query_set_handle.clone(),
            query_set,
            _resolve_buffer_handle: resolve_buffer_handle.clone(),
            resolve_buffer,
            readback_buffer_handle: readback_buffer_handle.clone(),
            readback_buffer,
            query_capacity,
            query_count: 0,
            readback_size,
            timestamp_period_ns: 0.0,
            entries: Vec::new(),
            resolve_encoded: false,
        }))
    }

    /// Timestamp-period observation is a G5 queue operation, so it is populated only after the
    /// batch has entered the raw operation interval. G4C1 query/buffer realization above remains
    /// entirely in the first phase.
    pub fn activate(&mut self, queue: &Queue) -> bool {
        self.timestamp_period_ns = queue.get_timestamp_period();
        self.timestamp_period_ns > 0.0
    }

    pub fn register_pass(
        &mut self,
        indices: GpuPassTimestampIndices,
        frame_index: u64,
        render_surface_id: u64,
        flow_id: impl Into<String>,
        pass_id: impl Into<String>,
        pass_kind: impl Into<String>,
    ) -> Option<GpuPassTimestampIndices> {
        if indices.begin >= indices.end
            || indices.end >= self.query_capacity
            || self.entries.iter().any(|entry| entry.indices == indices)
        {
            return None;
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
        Some(indices)
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
    ) -> Result<bool> {
        if self.query_count == 0 {
            return Ok(false);
        }
        context
            .current_render_pipeline_bridge()
            .for_timestamp_resources(
                &self.query_set,
                &self.resolve_buffer,
                &self.readback_buffer,
                ResolveTimingQueries {
                    encoder,
                    query_count: self.query_count,
                },
            )?;
        self.resolve_encoded = true;
        Ok(true)
    }

    pub fn encode_readback_copy(
        mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
    ) -> Result<Option<PendingGpuPassTimingReadback>> {
        if !self.resolve_encoded || self.query_count == 0 {
            return Ok(None);
        }
        let readback_size = u64::from(self.query_count) * u64::from(QUERY_SIZE);
        context
            .current_render_pipeline_bridge()
            .for_timestamp_resources(
                &self.query_set,
                &self.resolve_buffer,
                &self.readback_buffer,
                CopyTimingReadback {
                    encoder,
                    readback_size,
                },
            )?;
        self.readback_size = readback_size;
        Ok(Some(PendingGpuPassTimingReadback {
            _readback_buffer_handle: self.readback_buffer_handle,
            readback_buffer: self.readback_buffer,
            readback_size,
            timestamp_period_ns: self.timestamp_period_ns,
            entries: self.entries,
        }))
    }
}

struct ResolveTimingQueries<'a> {
    encoder: &'a mut CommandEncoder,
    query_count: u32,
}

impl CurrentRenderTimestampResourcesTerminal for ResolveTimingQueries<'_> {
    fn use_timestamp_resources(
        self,
        query_set: &QuerySet,
        resolve_buffer: &Buffer,
        _readback_buffer: &Buffer,
    ) {
        self.encoder
            .resolve_query_set(query_set, 0..self.query_count, resolve_buffer, 0);
    }
}

struct CopyTimingReadback<'a> {
    encoder: &'a mut CommandEncoder,
    readback_size: BufferAddress,
}

impl CurrentRenderTimestampResourcesTerminal for CopyTimingReadback<'_> {
    fn use_timestamp_resources(
        self,
        _query_set: &QuerySet,
        resolve_buffer: &Buffer,
        readback_buffer: &Buffer,
    ) {
        self.encoder.copy_buffer_to_buffer(
            resolve_buffer,
            0,
            readback_buffer,
            0,
            self.readback_size,
        );
    }
}

#[derive(Debug)]
pub(in crate::plugins::render::renderer) struct PendingGpuPassTimingReadback {
    _readback_buffer_handle: GpuBufferHandle,
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
        _readback_buffer_handle: _,
        readback_buffer,
        readback_size,
        timestamp_period_ns,
        entries,
    } = pending;
    let fallback_entries = entries.clone();
    let mut output = None;
    let bridge_result = context
        .current_render_pipeline_bridge()
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

    let data = slice.get_mapped_range();
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
        CurrentRenderTimestampWritesTerminal, GpuBufferUsage, GpuCapabilityFeature,
        GpuCapabilityProfile, GpuCapabilityRequirement, GpuContext, GpuContextDescriptor,
        GpuMemoryIntent, GpuPreferredFallback, GpuQueryKind, GpuQuerySetDescriptor,
        GpuResourceLifetime, GpuWorkResourceIdAllocator,
    };
    use crate::plugins::render::inspect::RenderTimingSource;
    use crate::plugins::render::renderer::resource_descriptors::{buffer_descriptor, owned_common};
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
        let readback_buffer = allocator
            .allocate_buffer_handle(
                buffer_descriptor(
                    "engine_test_gpu_timestamp_readback",
                    16,
                    [GpuBufferUsage::CopyDestination],
                    GpuResourceLifetime::Transient,
                    GpuMemoryIntent::Readback,
                )
                .expect("readback descriptor"),
            )
            .expect("readback handle");
        let mut frame =
            GpuPassTimingFrame::new(&context, &query_set, &resolve_buffer, &readback_buffer, 2)
                .expect("timestamp resources should realize")
                .expect("timestamp frame should allocate");
        let evidence = {
            let loan = context.current_render_device_queue();
            assert!(frame.activate(loan.queue));
            let indices = frame
                .register_pass(
                    GpuPassTimestampIndices { begin: 0, end: 1 },
                    1,
                    1,
                    "runtime.gpu",
                    "timestamp.empty_compute",
                    "compute",
                )
                .expect("timestamp pass should reserve queries");
            let writes = frame.timestamp_writes(indices);
            let mut encoder = loan
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("engine_test_gpu_timestamp_encoder"),
                });
            context
                .current_render_pipeline_bridge()
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
                    .encode_resolve(&context, &mut encoder)
                    .expect("timestamp resolve should encode")
            );
            let pending = frame
                .encode_readback_copy(&context, &mut encoder)
                .expect("timestamp readback copy should encode")
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
