use product::FieldProductDiagnostic;
use spatial::ChunkId;
use world_sdf::SdfPageCoord3;

use crate::plugins::render::features::world::sdf_residency::{
    RenderSdfBrickAtlasRecord, RenderSdfChunkResidencyEntry, RenderSdfClipmapWindowRecord,
    RenderSdfPageResidencyRecord, RenderSdfResidencyResource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfResidencyInspection {
    pub addressable_product_count: usize,
    pub selected_product_count: usize,
    pub requested_product_count: usize,
    pub resident_product_count: usize,
    pub resident_page_count: usize,
    pub resident_brick_count: usize,
    pub clipmap_window_count: usize,
    pub invalidated_product_count: usize,
    pub rejected_product_count: usize,
    pub resident_bytes: u64,
    pub upload_bytes: u64,
    pub budget: RenderSdfResidencyBudgetInspection,
    pub diagnostic_count: usize,
    pub diagnostics: Vec<RenderSdfResidencyDiagnosticInspection>,
    pub entries: Vec<RenderSdfChunkResidencyInspectionEntry>,
    pub clipmap_windows: Vec<RenderSdfClipmapWindowInspectionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfResidencyBudgetInspection {
    pub page_status: String,
    pub brick_status: String,
    pub resident_byte_status: String,
    pub upload_byte_status: String,
    pub clipmap_page_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfResidencyDiagnosticInspection {
    pub code: String,
    pub severity: String,
    pub product_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfChunkResidencyInspectionEntry {
    pub product_id: u64,
    pub product_generation: u64,
    pub scale_band: String,
    pub freshness: String,
    pub source_residency: String,
    pub authority_class: String,
    pub query_policy: String,
    pub requested_residency: String,
    pub priority: i32,
    pub hard_pin: bool,
    pub status: String,
    pub chunk_id: String,
    pub chunk_revision: u64,
    pub chunk_generation: u64,
    pub checksum: u64,
    pub cache_generation: u64,
    pub invalidated: bool,
    pub page_count: usize,
    pub brick_count: usize,
    pub resident_bytes: u64,
    pub upload_bytes: u64,
    pub diagnostic_count: usize,
    pub pages: Vec<RenderSdfPageResidencyInspectionEntry>,
    pub brick_atlas: Vec<RenderSdfBrickAtlasInspectionEntry>,
    pub clipmap_window: RenderSdfClipmapWindowInspectionEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfPageResidencyInspectionEntry {
    pub page_coord: [i16; 3],
    pub page_generation: u64,
    pub brick_count: usize,
    pub resident_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfBrickAtlasInspectionEntry {
    pub page_coord: [i16; 3],
    pub brick_coord: [u8; 3],
    pub occupancy_mask: u8,
    pub material_channel_mask: u16,
    pub surface_band_present: bool,
    pub resident_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfClipmapWindowInspectionEntry {
    pub scale_band: String,
    pub chunk_count: usize,
    pub page_count: usize,
    pub brick_count: usize,
    pub resident_bytes: u64,
    pub page_budget_status: String,
    pub brick_budget_status: String,
    pub resident_byte_budget_status: String,
}

pub fn inspect_render_sdf_residency(
    residency: &RenderSdfResidencyResource,
) -> RenderSdfResidencyInspection {
    let summary = residency.last_summary();
    RenderSdfResidencyInspection {
        addressable_product_count: summary.addressable_product_count,
        selected_product_count: summary.selected_product_count,
        requested_product_count: summary.requested_product_count,
        resident_product_count: summary.resident_product_count,
        resident_page_count: summary.resident_page_count,
        resident_brick_count: summary.resident_brick_count,
        clipmap_window_count: summary.clipmap_window_count,
        invalidated_product_count: summary.invalidated_product_count,
        rejected_product_count: summary.rejected_product_count,
        resident_bytes: summary.resident_bytes,
        upload_bytes: summary.upload_bytes,
        budget: RenderSdfResidencyBudgetInspection {
            page_status: summary.page_budget_status.as_str().to_string(),
            brick_status: summary.brick_budget_status.as_str().to_string(),
            resident_byte_status: summary.resident_byte_budget_status.as_str().to_string(),
            upload_byte_status: summary.upload_byte_budget_status.as_str().to_string(),
            clipmap_page_status: summary.clipmap_page_budget_status.as_str().to_string(),
        },
        diagnostic_count: summary.diagnostic_count,
        diagnostics: residency
            .diagnostics()
            .iter()
            .map(inspect_diagnostic)
            .collect(),
        entries: residency.entries().values().map(inspect_entry).collect(),
        clipmap_windows: residency
            .clipmap_windows()
            .iter()
            .map(inspect_window)
            .collect(),
    }
}

fn inspect_entry(entry: &RenderSdfChunkResidencyEntry) -> RenderSdfChunkResidencyInspectionEntry {
    RenderSdfChunkResidencyInspectionEntry {
        product_id: entry.product_id.raw(),
        product_generation: entry.product_generation,
        scale_band: format!("{:?}", entry.scale_band),
        freshness: format!("{:?}", entry.freshness),
        source_residency: format!("{:?}", entry.source_residency),
        authority_class: format!("{:?}", entry.authority_class),
        query_policy: format!("{:?}", entry.query_policy),
        requested_residency: format!("{:?}", entry.requested_residency),
        priority: entry.priority,
        hard_pin: entry.hard_pin,
        status: entry.status.as_str().to_string(),
        chunk_id: inspect_chunk_id(entry.chunk_id),
        chunk_revision: entry.chunk_revision,
        chunk_generation: entry.chunk_generation,
        checksum: entry.checksum,
        cache_generation: entry.cache_generation,
        invalidated: entry.invalidated,
        page_count: entry.page_records.len(),
        brick_count: entry.brick_atlas_records.len(),
        resident_bytes: entry.resident_bytes,
        upload_bytes: entry.upload_bytes,
        diagnostic_count: entry.diagnostics.len(),
        pages: entry.page_records.iter().map(inspect_page).collect(),
        brick_atlas: entry
            .brick_atlas_records
            .iter()
            .map(inspect_brick)
            .collect(),
        clipmap_window: inspect_window(&entry.clipmap_window),
    }
}

fn inspect_page(record: &RenderSdfPageResidencyRecord) -> RenderSdfPageResidencyInspectionEntry {
    RenderSdfPageResidencyInspectionEntry {
        page_coord: inspect_page_coord(record.page_coord),
        page_generation: record.page_generation,
        brick_count: record.brick_count,
        resident_bytes: record.resident_bytes,
    }
}

fn inspect_brick(record: &RenderSdfBrickAtlasRecord) -> RenderSdfBrickAtlasInspectionEntry {
    RenderSdfBrickAtlasInspectionEntry {
        page_coord: inspect_page_coord(record.page_coord),
        brick_coord: record.brick_coord,
        occupancy_mask: record.occupancy_mask,
        material_channel_mask: record.material_channel_mask,
        surface_band_present: record.surface_band_present,
        resident_bytes: record.resident_bytes,
    }
}

fn inspect_window(record: &RenderSdfClipmapWindowRecord) -> RenderSdfClipmapWindowInspectionEntry {
    RenderSdfClipmapWindowInspectionEntry {
        scale_band: format!("{:?}", record.scale_band),
        chunk_count: record.chunk_count,
        page_count: record.page_count,
        brick_count: record.brick_count,
        resident_bytes: record.resident_bytes,
        page_budget_status: record.page_budget_status.as_str().to_string(),
        brick_budget_status: record.brick_budget_status.as_str().to_string(),
        resident_byte_budget_status: record.resident_byte_budget_status.as_str().to_string(),
    }
}

fn inspect_diagnostic(
    diagnostic: &FieldProductDiagnostic,
) -> RenderSdfResidencyDiagnosticInspection {
    RenderSdfResidencyDiagnosticInspection {
        code: format!("{:?}", diagnostic.code),
        severity: format!("{:?}", diagnostic.severity),
        product_id: diagnostic.product_id.map(|product_id| product_id.raw()),
        message: diagnostic.message.clone(),
    }
}

fn inspect_page_coord(coord: SdfPageCoord3) -> [i16; 3] {
    [coord.x, coord.y, coord.z]
}

fn inspect_chunk_id(chunk_id: ChunkId) -> String {
    format!(
        "world:{}:{}:{}:{}",
        chunk_id.world_id.0, chunk_id.coord.x, chunk_id.coord.y, chunk_id.coord.z
    )
}
