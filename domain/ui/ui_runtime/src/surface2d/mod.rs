use ui_controls::{ControlInspectionSection, SURFACE2D_CONTROL_KIND_ID, runenwerk_control_package};
use ui_math::{UiPoint, UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, RectPrimitive, StrokePrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId,
    UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};

pub const BASE_CONTROLS_SURFACE2D_PROOF_ID: &str = "base-controls.surface2d.proof";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface2DTransform {
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
}

impl Surface2DTransform {
    pub const fn new(pan_x: f32, pan_y: f32, zoom: f32) -> Self {
        Self { pan_x, pan_y, zoom }
    }

    pub fn is_valid(self) -> bool {
        self.zoom.is_finite() && self.zoom > 0.0 && self.pan_x.is_finite() && self.pan_y.is_finite()
    }

    pub fn world_to_screen(self, point: UiPoint) -> Option<UiPoint> {
        self.is_valid()
            .then(|| UiPoint::new(point.x * self.zoom + self.pan_x, point.y * self.zoom + self.pan_y))
    }

    pub fn screen_to_world(self, point: UiPoint) -> Option<UiPoint> {
        self.is_valid()
            .then(|| UiPoint::new((point.x - self.pan_x) / self.zoom, (point.y - self.pan_y) / self.zoom))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Surface2DBoundaryCounters {
    pub side_effect_count: u32,
    pub semantic_write_count: u32,
    pub backend_resource_count: u32,
}

impl Surface2DBoundaryCounters {
    pub const fn clean(self) -> bool {
        self.side_effect_count == 0 && self.semantic_write_count == 0 && self.backend_resource_count == 0
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

pub fn base_controls_surface2d_report() -> Surface2DProofReport {
    let package = runenwerk_control_package();
    let catalog = ui_controls::ControlCatalogIndex::from_packages([&package]);
    let inspection = ui_controls::BaseControlsPlugin::new().inspection();
    let descriptor = package
        .surface2d_descriptors
        .iter()
        .find(|descriptor| descriptor.control_kind_id.as_str() == SURFACE2D_CONTROL_KIND_ID)
        .expect("base controls must carry Surface2D descriptor");
    let transform = Surface2DTransform::new(32.0, 48.0, 1.5);
    let invalid_transform = Surface2DTransform::new(0.0, 0.0, 0.0);
    let hover_screen = UiPoint::new(182.0, 198.0);
    let hover_world = transform
        .screen_to_world(hover_screen)
        .expect("valid transform maps screen to world");
    let summary = descriptor.summary();

    Surface2DProofReport {
        proof_id: BASE_CONTROLS_SURFACE2D_PROOF_ID.to_owned(),
        descriptor_evidence: vec![format!(
            "{} inputs:{} layers:{} budgets:{}",
            descriptor.control_kind_id.as_str(),
            descriptor.input_modes.len(),
            descriptor.layer_kinds.len(),
            descriptor.budget_evidence.len()
        )],
        transform_evidence: vec![
            format!("world-to-screen:{:?}", transform.world_to_screen(UiPoint::new(10.0, 20.0))),
            format!("screen-to-world:{:.2},{:.2}", hover_world.x, hover_world.y),
        ],
        navigation_evidence: vec![format!("pan:{},{}", transform.pan_x, transform.pan_y), format!("zoom:{}", transform.zoom), "fit-content".to_owned()],
        hover_evidence: vec![format!("hover:{:.1},{:.1}", hover_world.x, hover_world.y)],
        selection_evidence: vec!["selection-box".to_owned()],
        pointer_capture_evidence: vec!["pointer-capture".to_owned()],
        gesture_evidence: vec!["gesture-cancel-commit".to_owned()],
        accessibility_input_evidence: summary.input_modes.iter().map(|mode| format!("input:{mode}")).collect(),
        budget_evidence: summary.budget_evidence.iter().map(|budget| format!("budget:{budget}")).collect(),
        diagnostic_evidence: vec![format!("invalid-transform:{}", invalid_transform.world_to_screen(UiPoint::new(1.0, 1.0)).is_none())],
        catalog_projection_evidence: catalog.entries.iter().filter(|entry| entry.surface2d_supported).map(|entry| format!("{}:{}", entry.control_kind_id, entry.surface2d_input_modes.len())).collect(),
        inspection_projection_evidence: inspection.controls.iter().filter_map(|control| control.fact(ControlInspectionSection::Surface2D, "surface2d.supported").map(|value| format!("{}:{value}", control.control_kind_id))).collect(),
        static_mount_expectations: vec!["surface".to_owned(), "background".to_owned(), "grid".to_owned(), "selection".to_owned(), "diagnostic".to_owned(), "stable-order".to_owned()],
        boundary_counters: Surface2DBoundaryCounters::default(),
    }
}

pub fn base_controls_surface2d_proof_frame() -> Surface2DProofRenderFrame {
    surface2d_report_to_frame(base_controls_surface2d_report())
}

pub fn surface2d_report_to_frame(report: Surface2DProofReport) -> Surface2DProofRenderFrame {
    let mut primitives = Vec::new();
    let mut order = 0_u32;
    let viewport = UiRect::new(32.0, 32.0, 640.0, 420.0);
    primitives.push(background(viewport, &mut order));
    primitives.extend(grid(viewport, &mut order));
    primitives.push(selection_outline(UiRect::new(104.0, 120.0, 192.0, 144.0), &mut order));
    primitives.push(diagnostic_overlay(viewport.inset(ui_math::UiInsets::all(8.0)), &mut order));
    let primitive_count = primitives.len();
    let mut surface = UiSurface::new(UiSurfaceId(16), UiSize::new(704.0, 484.0));
    surface.push_layer(UiLayer::with_primitives(UiLayerId(0), primitives));
    let summary = render_summary(&report, primitive_count);
    Surface2DProofRenderFrame { proof_id: report.proof_id, frame: UiFrame::with_surfaces(vec![surface]), summary }
}

fn render_summary(report: &Surface2DProofReport, primitive_count: usize) -> Surface2DProofRenderSummary {
    Surface2DProofRenderSummary {
        descriptor_rows: report.descriptor_evidence.len(),
        transform_rows: report.transform_evidence.len(),
        navigation_rows: report.navigation_evidence.len(),
        hover_rows: report.hover_evidence.len(),
        selection_rows: report.selection_evidence.len(),
        pointer_capture_rows: report.pointer_capture_evidence.len(),
        gesture_rows: report.gesture_evidence.len(),
        accessibility_input_rows: report.accessibility_input_evidence.len(),
        budget_rows: report.budget_evidence.len(),
        diagnostic_rows: report.diagnostic_evidence.len(),
        catalog_rows: report.catalog_projection_evidence.len(),
        inspection_rows: report.inspection_projection_evidence.len(),
        primitive_count,
        has_background: true,
        has_grid: true,
        has_selection_outline: true,
        has_diagnostic_overlay: true,
        boundary_clean: report.boundary_counters.clean(),
    }
}

fn background(rect: UiRect, order: &mut u32) -> UiPrimitive {
    RectPrimitive::new(rect, 6.0, UiPaint::rgba(0.06, 0.07, 0.08, 1.0), UiDrawKey::new(1600, None), sort_key(order)).into()
}

fn grid(rect: UiRect, order: &mut u32) -> Vec<UiPrimitive> {
    let mut primitives = Vec::new();
    for x in [rect.x + 80.0, rect.x + 160.0, rect.x + 240.0, rect.x + 320.0, rect.x + 400.0, rect.x + 480.0, rect.x + 560.0] {
        primitives.push(StrokePrimitive::new([UiPoint::new(x, rect.y), UiPoint::new(x, rect.y + rect.height)], 1.0, UiPaint::rgba(0.18, 0.2, 0.22, 1.0), UiDrawKey::new(1601, None), sort_key(order)).with_clip(rect).into());
    }
    for y in [rect.y + 70.0, rect.y + 140.0, rect.y + 210.0, rect.y + 280.0, rect.y + 350.0] {
        primitives.push(StrokePrimitive::new([UiPoint::new(rect.x, y), UiPoint::new(rect.x + rect.width, y)], 1.0, UiPaint::rgba(0.18, 0.2, 0.22, 1.0), UiDrawKey::new(1602, None), sort_key(order)).with_clip(rect).into());
    }
    primitives
}

fn selection_outline(rect: UiRect, order: &mut u32) -> UiPrimitive {
    BorderPrimitive::new(rect, 0.0, 2.0, UiPaint::rgba(0.7, 0.9, 1.0, 1.0), UiDrawKey::new(1603, None), sort_key(order)).into()
}

fn diagnostic_overlay(rect: UiRect, order: &mut u32) -> UiPrimitive {
    BorderPrimitive::new(rect, 6.0, 2.0, UiPaint::rgba(1.0, 0.55, 0.2, 1.0), UiDrawKey::new(1604, None), sort_key(order)).into()
}

fn sort_key(order: &mut u32) -> UiSortKey {
    let key = UiSortKey::new(0, 0, *order);
    *order += 1;
    key
}
