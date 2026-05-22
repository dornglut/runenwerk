use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::plugins::render::renderer) struct GpuPassTimestampIndices {
    pub begin: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::plugins::render::renderer) struct GpuPassTimestampWrites<'a> {
    pub query_set: &'a QuerySet,
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
    query_set: QuerySet,
    resolve_buffer: Buffer,
    readback_buffer: Buffer,
    query_capacity: u32,
    query_count: u32,
    readback_size: BufferAddress,
    timestamp_period_ns: f32,
    entries: Vec<GpuPassTimingEntry>,
}

impl GpuPassTimingFrame {
    pub fn new(device: &Device, queue: &Queue, pass_capacity: usize) -> Option<Self> {
        if pass_capacity == 0 {
            return None;
        }
        let timestamp_period_ns = queue.get_timestamp_period();
        if timestamp_period_ns <= 0.0 {
            return None;
        }
        let query_capacity = pass_capacity.checked_mul(2)?.try_into().ok()?;
        if query_capacity == 0 {
            return None;
        }
        let readback_size = u64::from(query_capacity) * u64::from(QUERY_SIZE);
        let query_set = device.create_query_set(&QuerySetDescriptor {
            label: Some("engine_render_gpu_pass_timestamps"),
            ty: QueryType::Timestamp,
            count: query_capacity,
        });
        let resolve_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_render_gpu_pass_timestamp_resolve"),
            size: readback_size,
            usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_render_gpu_pass_timestamp_readback"),
            size: readback_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Some(Self {
            query_set,
            resolve_buffer,
            readback_buffer,
            query_capacity,
            query_count: 0,
            readback_size,
            timestamp_period_ns,
            entries: Vec::new(),
        })
    }

    pub fn reserve_pass(
        &mut self,
        frame_index: u64,
        render_surface_id: u64,
        flow_id: impl Into<String>,
        pass_id: impl Into<String>,
        pass_kind: impl Into<String>,
    ) -> Option<GpuPassTimestampIndices> {
        let begin = self.query_count;
        let end = begin.checked_add(1)?;
        if end >= self.query_capacity {
            return None;
        }
        self.query_count = self.query_count.saturating_add(2);
        let indices = GpuPassTimestampIndices { begin, end };
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

    pub fn timestamp_writes(&self, indices: GpuPassTimestampIndices) -> GpuPassTimestampWrites<'_> {
        GpuPassTimestampWrites {
            query_set: &self.query_set,
            indices,
        }
    }

    pub fn resolve(mut self, encoder: &mut CommandEncoder) -> Option<PendingGpuPassTimingReadback> {
        if self.query_count == 0 {
            return None;
        }
        let readback_size = u64::from(self.query_count) * u64::from(QUERY_SIZE);
        encoder.resolve_query_set(
            &self.query_set,
            0..self.query_count,
            &self.resolve_buffer,
            0,
        );
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &self.readback_buffer,
            0,
            readback_size,
        );
        self.readback_size = readback_size;
        Some(PendingGpuPassTimingReadback {
            readback_buffer: self.readback_buffer,
            readback_size,
            timestamp_period_ns: self.timestamp_period_ns,
            entries: self.entries,
        })
    }
}

#[derive(Debug)]
pub(in crate::plugins::render::renderer) struct PendingGpuPassTimingReadback {
    readback_buffer: Buffer,
    readback_size: BufferAddress,
    timestamp_period_ns: f32,
    entries: Vec<GpuPassTimingEntry>,
}

pub(in crate::plugins::render::renderer) fn read_gpu_pass_timing_evidence(
    device: &Device,
    pending: PendingGpuPassTimingReadback,
) -> Vec<RenderPassTimingEvidence> {
    let PendingGpuPassTimingReadback {
        readback_buffer,
        readback_size,
        timestamp_period_ns,
        entries,
    } = pending;
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
    use crate::plugins::render::backend::request_device_and_queue;
    use crate::plugins::render::inspect::RenderTimingSource;
    use pollster::block_on;

    #[test]
    #[ignore = "runtime evidence test: requires a local WGPU adapter and may depend on driver timestamp-query support"]
    fn render_gpu_timing_runtime_query_readback_reports_measured_or_unsupported() {
        let instance = runtime_gpu_timing_probe_instance();
        let adapter = match block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })) {
            Ok(adapter) => adapter,
            Err(err) => {
                println!("runtime GPU timing evidence: no adapter available: {err}");
                return;
            }
        };
        let adapter_info = adapter.get_info();
        println!(
            "runtime GPU timing evidence: backend={:?} adapter={}",
            adapter_info.backend, adapter_info.name
        );
        let (device, queue, timing_capabilities) =
            block_on(request_device_and_queue(&adapter)).expect("device request should succeed");
        if !timing_capabilities.timestamp_query {
            println!("runtime GPU timing evidence: timestamp queries unsupported by adapter");
            return;
        }

        let mut frame =
            GpuPassTimingFrame::new(&device, &queue, 1).expect("timestamp frame should allocate");
        let indices = frame
            .reserve_pass(1, 1, "runtime.gpu", "timestamp.empty_compute", "compute")
            .expect("timestamp pass should reserve queries");
        let writes = frame.timestamp_writes(indices);
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_test_gpu_timestamp_encoder"),
        });
        {
            let _pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("engine_test_gpu_timestamp_compute_pass"),
                timestamp_writes: Some(ComputePassTimestampWrites {
                    query_set: writes.query_set,
                    beginning_of_pass_write_index: Some(writes.indices.begin),
                    end_of_pass_write_index: Some(writes.indices.end),
                }),
            });
        }
        let pending = frame
            .resolve(&mut encoder)
            .expect("timestamp queries should resolve");
        queue.submit(std::iter::once(encoder.finish()));

        let evidence = read_gpu_pass_timing_evidence(&device, pending);
        println!("runtime GPU timing evidence: {evidence:?}");
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].source, RenderTimingSource::GpuTimestampQuery);
        assert_eq!(
            evidence[0].gpu_capability,
            RenderGpuTimingCapability::Supported
        );
        assert!(evidence[0].millis.is_some());
    }

    fn runtime_gpu_timing_probe_instance() -> Instance {
        let descriptor = InstanceDescriptor {
            backends: runtime_gpu_timing_probe_backends(),
            ..InstanceDescriptor::default()
        }
        .with_env();
        Instance::new(&descriptor)
    }

    #[cfg(target_os = "windows")]
    fn runtime_gpu_timing_probe_backends() -> Backends {
        // Keep the headless proof on a deterministic backend by default; WGPU_BACKEND can still
        // override this when a backend-specific driver issue is being investigated.
        Backends::VULKAN
    }

    #[cfg(not(target_os = "windows"))]
    fn runtime_gpu_timing_probe_backends() -> Backends {
        Backends::PRIMARY
    }
}
