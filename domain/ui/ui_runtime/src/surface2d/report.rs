use ui_render_data::UiFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Surface2DBoundaryCounters {
    pub side_effect_count: u32,
    pub semantic_write_count: u32,
    pub backend_resource_count: u32,
}

impl Surface2DBoundaryCounters {
    pub const fn clean(self) -> bool {
        self.side_effect_count == 0
            && self.semantic_write_count == 0
            && self.backend_resource_count == 0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Surface2DProofReport {
    pub proof_id: String,
    pub descriptor_evidence: Vec<String>,
    pub transform_evidence: Vec<String>,
    pub navigation_evidence: Vec<String>,
    pub hover_evidence: Vec<String>,
    pub selection_evidence: Vec<String>,
    pub pointer_capture_evidence: Vec<String>,
    pub gesture_evidence: Vec<String>,
    pub accessibility_input_evidence: Vec<String>,
    pub budget_evidence: Vec<String>,
    pub diagnostic_evidence: Vec<String>,
    pub catalog_projection_evidence: Vec<String>,
    pub inspection_projection_evidence: Vec<String>,
    pub static_mount_expectations: Vec<String>,
    pub boundary_counters: Surface2DBoundaryCounters,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Surface2DProofRenderFrame {
    pub proof_id: String,
    pub frame: UiFrame,
    pub summary: Surface2DProofRenderSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface2DProofRenderSummary {
    pub descriptor_rows: usize,
    pub transform_rows: usize,
    pub navigation_rows: usize,
    pub hover_rows: usize,
    pub selection_rows: usize,
    pub pointer_capture_rows: usize,
    pub gesture_rows: usize,
    pub accessibility_input_rows: usize,
    pub budget_rows: usize,
    pub diagnostic_rows: usize,
    pub catalog_rows: usize,
    pub inspection_rows: usize,
    pub primitive_count: usize,
    pub has_background: bool,
    pub has_grid: bool,
    pub has_selection_outline: bool,
    pub has_diagnostic_overlay: bool,
    pub boundary_clean: bool,
}
