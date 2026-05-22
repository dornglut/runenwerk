use crate::plugins::render::{
    RenderGpuResidencyEntry, RenderGpuResidencyJournalEntry, RenderGpuResidencyResource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGpuResidencyInspection {
    pub addressable_count: usize,
    pub selected_count: usize,
    pub requested_count: usize,
    pub accepted_count: usize,
    pub resident_count: usize,
    pub allocated_count: usize,
    pub preserved_count: usize,
    pub invalidated_count: usize,
    pub evicted_count: usize,
    pub rejected_count: usize,
    pub resident_bytes: u64,
    pub upload_bytes: u64,
    pub budget: RenderGpuResidencyBudgetInspection,
    pub diagnostic_count: usize,
    pub entries: Vec<RenderGpuResidencyInspectionEntry>,
    pub journal: Vec<RenderGpuResidencyJournalInspectionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGpuResidencyBudgetInspection {
    pub max_resident_entries: usize,
    pub max_resident_bytes: u64,
    pub max_upload_bytes_per_frame: u64,
    pub resident_entry_status: String,
    pub resident_byte_status: String,
    pub upload_byte_status: String,
    pub hard_pinned_over_entry_budget: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGpuResidencyInspectionEntry {
    pub product_id: u64,
    pub generation: u64,
    pub source_scale_band: String,
    pub source_freshness: String,
    pub source_residency: String,
    pub source_authority_class: String,
    pub source_query_policy: String,
    pub requested_residency: String,
    pub priority: i32,
    pub hard_pin: bool,
    pub status: String,
    pub cache_id: String,
    pub resident_bytes: u64,
    pub upload_bytes: u64,
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
    pub resident_bytes: u64,
    pub upload_bytes: u64,
    pub diagnostic_count: usize,
}

pub fn inspect_render_gpu_residency(
    residency: &RenderGpuResidencyResource,
) -> RenderGpuResidencyInspection {
    let summary = residency.last_summary();
    RenderGpuResidencyInspection {
        addressable_count: summary.addressable_count,
        selected_count: summary.selected_count,
        requested_count: summary.requested_count,
        accepted_count: summary.accepted_count,
        resident_count: summary.resident_count,
        allocated_count: summary.allocated_count,
        preserved_count: summary.preserved_count,
        invalidated_count: summary.invalidated_count,
        evicted_count: summary.evicted_count,
        rejected_count: summary.rejected_count,
        resident_bytes: summary.resident_bytes,
        upload_bytes: summary.upload_bytes,
        budget: RenderGpuResidencyBudgetInspection {
            max_resident_entries: summary.max_resident_entries,
            max_resident_bytes: summary.max_resident_bytes,
            max_upload_bytes_per_frame: summary.max_upload_bytes_per_frame,
            resident_entry_status: summary.resident_entry_budget_status.as_str().to_string(),
            resident_byte_status: summary.resident_byte_budget_status.as_str().to_string(),
            upload_byte_status: summary.upload_byte_budget_status.as_str().to_string(),
            hard_pinned_over_entry_budget: summary.hard_pinned_over_entry_budget,
        },
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
        source_scale_band: format!("{:?}", entry.source.scale_band),
        source_freshness: format!("{:?}", entry.source.freshness),
        source_residency: format!("{:?}", entry.source.product_residency),
        source_authority_class: format!("{:?}", entry.source.authority_class),
        source_query_policy: format!("{:?}", entry.source.query_policy),
        requested_residency: format!("{:?}", entry.requested_residency),
        priority: entry.priority,
        hard_pin: entry.hard_pin,
        status: format!("{:?}", entry.status),
        cache_id: entry.cache_handle.to_string(),
        resident_bytes: entry.resident_bytes,
        upload_bytes: entry.upload_bytes,
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
        resident_bytes: entry.resident_bytes,
        upload_bytes: entry.upload_bytes,
        diagnostic_count: entry.diagnostics.len(),
    }
}
