use crate::plugins::render::{
    RenderGpuResidencyEntry, RenderGpuResidencyJournalEntry, RenderGpuResidencyResource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGpuResidencyInspection {
    pub resident_count: usize,
    pub allocated_count: usize,
    pub preserved_count: usize,
    pub invalidated_count: usize,
    pub evicted_count: usize,
    pub rejected_count: usize,
    pub diagnostic_count: usize,
    pub entries: Vec<RenderGpuResidencyInspectionEntry>,
    pub journal: Vec<RenderGpuResidencyJournalInspectionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGpuResidencyInspectionEntry {
    pub product_id: u64,
    pub generation: u64,
    pub requested_residency: String,
    pub priority: i32,
    pub hard_pin: bool,
    pub status: String,
    pub cache_id: String,
    pub diagnostic_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGpuResidencyJournalInspectionEntry {
    pub action: String,
    pub product_id: u64,
    pub generation: u64,
    pub requested_residency: String,
    pub priority: i32,
    pub hard_pin: bool,
    pub cache_id: Option<String>,
    pub diagnostic_count: usize,
}

pub fn inspect_render_gpu_residency(
    residency: &RenderGpuResidencyResource,
) -> RenderGpuResidencyInspection {
    let summary = residency.last_summary();
    RenderGpuResidencyInspection {
        resident_count: summary.resident_count,
        allocated_count: summary.allocated_count,
        preserved_count: summary.preserved_count,
        invalidated_count: summary.invalidated_count,
        evicted_count: summary.evicted_count,
        rejected_count: summary.rejected_count,
        diagnostic_count: summary.diagnostic_count,
        entries: residency.entries().values().map(inspect_entry).collect(),
        journal: residency
            .journal()
            .iter()
            .map(inspect_journal_entry)
            .collect(),
    }
}

fn inspect_entry(entry: &RenderGpuResidencyEntry) -> RenderGpuResidencyInspectionEntry {
    RenderGpuResidencyInspectionEntry {
        product_id: entry.product_id.raw(),
        generation: entry.generation,
        requested_residency: format!("{:?}", entry.requested_residency),
        priority: entry.priority,
        hard_pin: entry.hard_pin,
        status: format!("{:?}", entry.status),
        cache_id: entry.cache_handle.to_string(),
        diagnostic_count: entry.diagnostics.len(),
    }
}

fn inspect_journal_entry(
    entry: &RenderGpuResidencyJournalEntry,
) -> RenderGpuResidencyJournalInspectionEntry {
    RenderGpuResidencyJournalInspectionEntry {
        action: format!("{:?}", entry.action),
        product_id: entry.product_id.raw(),
        generation: entry.generation,
        requested_residency: format!("{:?}", entry.requested_residency),
        priority: entry.priority,
        hard_pin: entry.hard_pin,
        cache_id: entry.cache_handle.map(|handle| handle.to_string()),
        diagnostic_count: entry.diagnostics.len(),
    }
}
