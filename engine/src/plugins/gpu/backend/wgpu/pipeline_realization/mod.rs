//! Context/device-generation-bound G4C3 pipeline realization.

mod compute;
mod diagnostics;
mod publication;
mod records;
mod registry;
mod render;
mod render_lowering;
mod render_mapping;
mod render_validation;

pub(crate) use records::{ComputePipelineRealizationRecord, RenderPipelineRealizationRecord};

use super::{WgpuDeviceHealth, WgpuErrorAttributionGate};
use crate::plugins::gpu::GpuContextAffinity;
use compute::{ComputePipelineRequestKey, ComputeRecord};
use diagnostics::PipelineCacheDiagnosticRegistry;
use registry::SingleFlightRegistry;
use render::RenderRecord;
use render_validation::RenderPipelineRequestKey;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

const DEFAULT_MAX_PIPELINE_REALIZATION_RECORDS: NonZeroUsize =
    NonZeroUsize::new(16_384).expect("the default pipeline realization-record bound is nonzero");
const G4C3_WGPU_PIPELINE_COMPATIBILITY_REVISION: u32 = 1;

/// The sole private G4C3 owner for one admitted WGPU device generation.
pub(crate) struct PipelineRealizationState {
    affinity: GpuContextAffinity,
    max_records: NonZeroUsize,
    compute: Arc<Mutex<SingleFlightRegistry<ComputePipelineRequestKey, ComputeRecord>>>,
    render: Arc<Mutex<SingleFlightRegistry<RenderPipelineRequestKey, RenderRecord>>>,
    cache_diagnostics: Mutex<PipelineCacheDiagnosticRegistry>,
    health: Arc<WgpuDeviceHealth>,
    error_attribution_gate: Arc<WgpuErrorAttributionGate>,
}

impl PipelineRealizationState {
    pub(crate) fn new(
        affinity: GpuContextAffinity,
        health: Arc<WgpuDeviceHealth>,
        error_attribution_gate: Arc<WgpuErrorAttributionGate>,
    ) -> Self {
        Self {
            affinity,
            max_records: DEFAULT_MAX_PIPELINE_REALIZATION_RECORDS,
            compute: Arc::new(Mutex::new(SingleFlightRegistry::default())),
            render: Arc::new(Mutex::new(SingleFlightRegistry::default())),
            cache_diagnostics: Mutex::new(PipelineCacheDiagnosticRegistry::default()),
            health,
            error_attribution_gate,
        }
    }
}

impl core::fmt::Debug for PipelineRealizationState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PipelineRealizationState")
            .field("affinity", &self.affinity)
            .field(
                "compute_records",
                &self
                    .compute
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .total_len(),
            )
            .field(
                "render_records",
                &self
                    .render
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .total_len(),
            )
            .field(
                "cache_diagnostics",
                &*self
                    .cache_diagnostics
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
            .field("max_records_per_family", &self.max_records)
            .finish_non_exhaustive()
    }
}
