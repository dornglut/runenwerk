//! Context/device-generation-bound G4C3 pipeline realization.

mod compute;
mod publication;
mod records;
mod registry;

pub(crate) use records::ComputePipelineRealizationRecord;

use super::{WgpuDeviceHealth, WgpuErrorAttributionGate};
use crate::plugins::gpu::GpuContextAffinity;
use compute::{ComputePipelineRequestKey, ComputeRecord};
use registry::SingleFlightRegistry;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

const DEFAULT_MAX_PIPELINE_REALIZATION_RECORDS: NonZeroUsize =
    NonZeroUsize::new(16_384).expect("the default pipeline realization-record bound is nonzero");
pub(super) const G4C3_WGPU_PIPELINE_COMPATIBILITY_REVISION: u32 = 1;

/// The sole private G4C3 owner for one admitted WGPU device generation.
pub(crate) struct PipelineRealizationState {
    pub(super) affinity: GpuContextAffinity,
    pub(super) max_records: NonZeroUsize,
    pub(super) compute: Arc<Mutex<SingleFlightRegistry<ComputePipelineRequestKey, ComputeRecord>>>,
    pub(super) health: Arc<WgpuDeviceHealth>,
    pub(super) error_attribution_gate: Arc<WgpuErrorAttributionGate>,
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
            .field("max_records_per_family", &self.max_records)
            .finish_non_exhaustive()
    }
}
