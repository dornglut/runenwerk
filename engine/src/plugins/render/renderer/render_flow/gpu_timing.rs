use super::*;
use crate::plugins::gpu::{GpuBufferHandle, GpuContext, GpuQuerySetHandle, GpuReadbackId};

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
    query_capacity: u32,
    entries: Vec<GpuPassTimingEntry>,
}

impl GpuPassTimingFrame {
    pub fn new(
        _context: &GpuContext,
        query_set_handle: &GpuQuerySetHandle,
        resolve_buffer_handle: &GpuBufferHandle,
        _readback_id: GpuReadbackId,
        query_capacity: u32,
    ) -> Result<Self> {
        if query_capacity == 0 {
            anyhow::bail!(
                "GPU timing frame requires nonzero query capacity after logical timing admission"
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
        Ok(Self {
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

    pub fn pending_evidence(self) -> Vec<RenderPassTimingEvidence> {
        self.entries
            .into_iter()
            .map(|entry| {
                RenderPassTimingEvidence::gpu_diagnostic(
                    Some(entry.frame_index),
                    Some(entry.render_surface_id),
                    entry.flow_id,
                    entry.pass_id,
                    entry.pass_kind,
                    RenderGpuTimingDiagnostic::readback_pending(
                        "GPU timestamp submission was accepted; readback is pending",
                    ),
                )
            })
            .collect()
    }
}
