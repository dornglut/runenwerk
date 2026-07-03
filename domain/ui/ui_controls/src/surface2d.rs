//! Renderer-neutral Surface2D declarations for reusable control packages.
//!
//! `ui_controls` may describe 2D coordinate, navigation, transform, bounds,
//! input, accessibility, overlay, diagnostic, and budget evidence needed by a
//! reusable surface. It does not own graph semantics, timeline semantics,
//! renderer backends, product state mutation, editor commands, app composition,
//! authored UI mutation, or plugin framework behavior.

use serde::{Deserialize, Serialize};
use ui_program::RouteCapability;
use ui_schema::UiSchemaShape;

use crate::package::ids::ControlKindId;
use crate::{
    ControlCompiler, ControlContribution, ControlDef, ControlField, ControlFieldGroup,
    ControlModuleDescriptor, ControlPreset, ControlStyleRole, ControlThemeGroup,
    ControlVisualState,
};

pub const SURFACE2D_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.surface2d";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlSurface2DInputMode {
    KeyboardPan,
    KeyboardZoom,
    KeyboardFitContent,
    PointerCapture,
    WheelScroll,
    TrackpadPinchStatus,
    TouchPanZoomStatus,
    ControllerNavigationStatus,
}

impl ControlSurface2DInputMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::KeyboardPan => "keyboard-pan",
            Self::KeyboardZoom => "keyboard-zoom",
            Self::KeyboardFitContent => "keyboard-fit-content",
            Self::PointerCapture => "pointer-capture",
            Self::WheelScroll => "wheel-scroll",
            Self::TrackpadPinchStatus => "trackpad-pinch-status",
            Self::TouchPanZoomStatus => "touch-pan-zoom-status",
            Self::ControllerNavigationStatus => "controller-navigation-status",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlSurface2DLayerKind {
    Background,
    Grid,
    DiagnosticOverlay,
    SelectionBox,
}

impl ControlSurface2DLayerKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Grid => "grid",
            Self::DiagnosticOverlay => "diagnostic-overlay",
            Self::SelectionBox => "selection-box",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlSurface2DBudgetEvidenceKind {
    TransformProjection,
    PanZoomUpdate,
    HoverCoordinateUpdate,
    SelectionRectangleUpdate,
    FitContentCalculation,
    LargeContentBoundsProjection,
    RuntimeReportGeneration,
    StaticMountReportGeneration,
    PrimitiveCount,
}

impl ControlSurface2DBudgetEvidenceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TransformProjection => "transform-projection",
            Self::PanZoomUpdate => "pan-zoom-update",
            Self::HoverCoordinateUpdate => "hover-coordinate-update",
            Self::SelectionRectangleUpdate => "selection-rectangle-update",
            Self::FitContentCalculation => "fit-content-calculation",
            Self::LargeContentBoundsProjection => "large-content-bounds-projection",
            Self::RuntimeReportGeneration => "runtime-report-generation",
            Self::StaticMountReportGeneration => "static-mount-report-generation",
            Self::PrimitiveCount => "primitive-count",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlSurface2DAccessibilitySupport {
    pub keyboard_pan: bool,
    pub keyboard_zoom: bool,
    pub keyboard_fit_content: bool,
    pub focus_visible: bool,
    pub inspection_readable_name_and_bounds: bool,
    pub reduced_motion_navigation: bool,
}

impl ControlSurface2DAccessibilitySupport {
    pub const fn complete() -> Self {
        Self {
            keyboard_pan: true,
            keyboard_zoom: true,
            keyboard_fit_content: true,
            focus_visible: true,
            inspection_readable_name_and_bounds: true,
            reduced_motion_navigation: true,
        }
    }

    pub const fn is_complete(&self) -> bool {
        self.keyboard_pan
            && self.keyboard_zoom
            && self.keyboard_fit_content
            && self.focus_visible
            && self.inspection_readable_name_and_bounds
            && self.reduced_motion_navigation
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlSurface2DInteractionSupport {
    pub content_bounds: bool,
    pub viewport_bounds: bool,
    pub world_screen_transform: bool,
    pub pan_zoom_state: bool,
    pub fit_content: bool,
    pub hover_coordinate: bool,
    pub selection_rectangle: bool,
    pub pointer_capture: bool,
    pub gesture_cancel_commit: bool,
    pub invalid_transform_diagnostic: bool,
}

impl ControlSurface2DInteractionSupport {
    pub const fn complete() -> Self {
        Self {
            content_bounds: true,
            viewport_bounds: true,
            world_screen_transform: true,
            pan_zoom_state: true,
            fit_content: true,
            hover_coordinate: true,
            selection_rectangle: true,
            pointer_capture: true,
            gesture_cancel_commit: true,
            invalid_transform_diagnostic: true,
        }
    }

    pub const fn is_complete(&self) -> bool {
        self.content_bounds
            && self.viewport_bounds
            && self.world_screen_transform
            && self.pan_zoom_state
            && self.fit_content
            && self.hover_coordinate
            && self.selection_rectangle
            && self.pointer_capture
            && self.gesture_cancel_commit
            && self.invalid_transform_diagnostic
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlSurface2DDescriptor {
    pub control_kind_id: ControlKindId,
    pub input_modes: Vec<ControlSurface2DInputMode>,
    pub layer_kinds: Vec<ControlSurface2DLayerKind>,
    pub budget_evidence: Vec<ControlSurface2DBudgetEvidenceKind>,
    pub accessibility: ControlSurface2DAccessibilitySupport,
    pub interaction: ControlSurface2DInteractionSupport,
    #[serde(default = "default_true")]
    pub proof_required: bool,
    #[serde(default)]
    pub renderer_backend_required: bool,
    #[serde(default)]
    pub executes_host_commands: bool,
    #[serde(default)]
    pub mutates_product_state: bool,
    #[serde(default)]
    pub graph_or_timeline_semantics: bool,
}

impl ControlSurface2DDescriptor {
    pub fn new(control_kind_id: ControlKindId) -> Self {
        Self {
            control_kind_id,
            input_modes: vec![
                ControlSurface2DInputMode::KeyboardPan,
                ControlSurface2DInputMode::KeyboardZoom,
                ControlSurface2DInputMode::KeyboardFitContent,
                ControlSurface2DInputMode::PointerCapture,
                ControlSurface2DInputMode::WheelScroll,
                ControlSurface2DInputMode::TrackpadPinchStatus,
                ControlSurface2DInputMode::TouchPanZoomStatus,
                ControlSurface2DInputMode::ControllerNavigationStatus,
            ],
            layer_kinds: vec![
                ControlSurface2DLayerKind::Background,
                ControlSurface2DLayerKind::Grid,
                ControlSurface2DLayerKind::DiagnosticOverlay,
                ControlSurface2DLayerKind::SelectionBox,
            ],
            budget_evidence: vec![
                ControlSurface2DBudgetEvidenceKind::TransformProjection,
                ControlSurface2DBudgetEvidenceKind::PanZoomUpdate,
                ControlSurface2DBudgetEvidenceKind::HoverCoordinateUpdate,
                ControlSurface2DBudgetEvidenceKind::SelectionRectangleUpdate,
                ControlSurface2DBudgetEvidenceKind::FitContentCalculation,
                ControlSurface2DBudgetEvidenceKind::LargeContentBoundsProjection,
                ControlSurface2DBudgetEvidenceKind::RuntimeReportGeneration,
                ControlSurface2DBudgetEvidenceKind::StaticMountReportGeneration,
                ControlSurface2DBudgetEvidenceKind::PrimitiveCount,
            ],
            accessibility: ControlSurface2DAccessibilitySupport::complete(),
            interaction: ControlSurface2DInteractionSupport::complete(),
            proof_required: true,
            renderer_backend_required: false,
            executes_host_commands: false,
            mutates_product_state: false,
            graph_or_timeline_semantics: false,
        }
    }

    pub fn summary(&self) -> ControlSurface2DSupportSummary {
        ControlSurface2DSupportSummary::from_descriptor(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlSurface2DSupportSummary {
    pub control_kind_id: ControlKindId,
    pub surface2d_supported: bool,
    pub input_modes: Vec<String>,
    pub layer_kinds: Vec<String>,
    pub budget_evidence: Vec<String>,
    pub accessibility_complete: bool,
    pub interaction_complete: bool,
    pub renderer_backend_required: bool,
    pub executes_host_commands: bool,
    pub mutates_product_state: bool,
    pub graph_or_timeline_semantics: bool,
}

impl ControlSurface2DSupportSummary {
    pub fn from_descriptor(descriptor: &ControlSurface2DDescriptor) -> Self {
        let mut input_modes = descriptor
            .input_modes
            .iter()
            .map(|mode| mode.as_str().to_owned())
            .collect::<Vec<_>>();
        input_modes.sort();
        input_modes.dedup();
        let mut layer_kinds = descriptor
            .layer_kinds
            .iter()
            .map(|kind| kind.as_str().to_owned())
            .collect::<Vec<_>>();
        layer_kinds.sort();
        layer_kinds.dedup();
        let mut budget_evidence = descriptor
            .budget_evidence
            .iter()
            .map(|kind| kind.as_str().to_owned())
            .collect::<Vec<_>>();
        budget_evidence.sort();
        budget_evidence.dedup();
        Self {
            control_kind_id: descriptor.control_kind_id.clone(),
            surface2d_supported: descriptor.proof_required,
            input_modes,
            layer_kinds,
            budget_evidence,
            accessibility_complete: descriptor.accessibility.is_complete(),
            interaction_complete: descriptor.interaction.is_complete(),
            renderer_backend_required: descriptor.renderer_backend_required,
            executes_host_commands: descriptor.executes_host_commands,
            mutates_product_state: descriptor.mutates_product_state,
            graph_or_timeline_semantics: descriptor.graph_or_timeline_semantics,
        }
    }

    pub fn inspection_facts(&self) -> Vec<ControlSurface2DInspectionFact> {
        vec![
            ControlSurface2DInspectionFact::new(
                "surface2d.supported",
                bool_string(self.surface2d_supported),
            ),
            ControlSurface2DInspectionFact::new("surface2d.input_modes", self.input_modes.join(",")),
            ControlSurface2DInspectionFact::new("surface2d.layers", self.layer_kinds.join(",")),
            ControlSurface2DInspectionFact::new(
                "surface2d.budget_evidence",
                self.budget_evidence.join(","),
            ),
            ControlSurface2DInspectionFact::new(
                "surface2d.accessibility_complete",
                bool_string(self.accessibility_complete),
            ),
            ControlSurface2DInspectionFact::new(
                "surface2d.interaction_complete",
                bool_string(self.interaction_complete),
            ),
            ControlSurface2DInspectionFact::new(
                "surface2d.renderer_backend_required",
                bool_string(self.renderer_backend_required),
            ),
            ControlSurface2DInspectionFact::new(
                "surface2d.executes_host_commands",
                bool_string(self.executes_host_commands),
            ),
            ControlSurface2DInspectionFact::new(
                "surface2d.mutates_product_state",
                bool_string(self.mutates_product_state),
            ),
            ControlSurface2DInspectionFact::new(
                "surface2d.graph_or_timeline_semantics",
                bool_string(self.graph_or_timeline_semantics),
            ),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlSurface2DInspectionFact {
    pub key: String,
    pub value: String,
}

impl ControlSurface2DInspectionFact {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

pub fn control_contribution() -> ControlContribution {
    ControlDef::builder(
        "surface2d",
        "Surface2D",
        ControlPreset::Surface2D,
        RouteCapability::new("runenwerk.ui.controls.surface2d.navigate"),
    )
    .with_description("Reusable renderer-neutral 2D coordinate and navigation surface descriptor with package, catalog, inspection, runtime proof, and static mount evidence.")
    .with_category("base-control")
    .with_tag("surface2d")
    .with_tag("coordinate-navigation")
    .with_field_group(ControlFieldGroup::properties([
        ControlField::required("surface_id", UiSchemaShape::StableIdRef),
        ControlField::required("content_width", UiSchemaShape::Number),
        ControlField::required("content_height", UiSchemaShape::Number),
        ControlField::required("viewport_width", UiSchemaShape::Number),
        ControlField::required("viewport_height", UiSchemaShape::Number),
    ]))
    .with_field_group(ControlFieldGroup::state([
        ControlField::optional("pan_x", UiSchemaShape::Number),
        ControlField::optional("pan_y", UiSchemaShape::Number),
        ControlField::optional("zoom", UiSchemaShape::Number),
        ControlField::optional("focused", UiSchemaShape::Bool),
    ]))
    .with_field_group(ControlFieldGroup::event_payload([
        ControlField::required("intent", UiSchemaShape::String),
        ControlField::optional("screen_x", UiSchemaShape::Number),
        ControlField::optional("screen_y", UiSchemaShape::Number),
        ControlField::optional("cancel", UiSchemaShape::Bool),
    ]))
    .with_theme_group(
        ControlThemeGroup::surface("surface2d")
            .with_style(ControlStyleRole::Container, "surface")
            .with_style(ControlStyleRole::FocusRing, "surface-focus-ring")
            .with_optional_visual_state(ControlVisualState::Focused),
    )
    .build_contribution()
}

pub fn control_module() -> ControlModuleDescriptor {
    ControlCompiler::new().compile_module(&control_contribution())
}

fn default_true() -> bool {
    true
}

fn bool_string(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
