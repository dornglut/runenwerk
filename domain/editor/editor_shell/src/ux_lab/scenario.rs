//! File: domain/editor/editor_shell/src/ux_lab/scenario.rs
//! Purpose: Editor UX scenario identity, args, and interaction contracts.

use crate::{
    EditorUxDensity, EditorUxDesignSystemEvidence, EditorUxGraphCanvasEvidence,
    EditorUxInputModality, EditorUxProductPatternEvidence, EditorUxRegisteredSurfaceEvidence,
    EditorUxScenarioMatrix, EditorUxViewportClass, EditorUxWorkbenchEvidence, ToolSurfaceReadiness,
    UiNode, VisibleWidgetScanRequirement, WidgetId,
};
use ui_surface::SurfaceDefinitionId;
use ui_widgets::PrimitiveWidgetScenarioKind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EditorUxScenarioId(String);

impl EditorUxScenarioId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorUxScenarioKind {
    PrimitiveWidget(PrimitiveWidgetScenarioKind),
    ProductPattern,
    RegisteredSurface(SurfaceDefinitionId),
    HostScenario,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorUxScenarioArgs {
    pub density: EditorUxDensity,
    pub viewport_class: EditorUxViewportClass,
    pub input_modality: EditorUxInputModality,
}

impl Default for EditorUxScenarioArgs {
    fn default() -> Self {
        Self {
            density: EditorUxDensity::Comfortable,
            viewport_class: EditorUxViewportClass::Standard,
            input_modality: EditorUxInputModality::Pointer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorUxScenarioInteraction {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: EditorUxScenarioInteractionKind,
    pub target_widget_id: Option<WidgetId>,
}

impl EditorUxScenarioInteraction {
    pub fn new(
        id: &'static str,
        label: &'static str,
        kind: EditorUxScenarioInteractionKind,
        target_widget_id: Option<WidgetId>,
    ) -> Self {
        Self {
            id,
            label,
            kind,
            target_widget_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorUxScenarioInteractionKind {
    PointerActivate,
    KeyboardActivate,
    TextEntry,
    FocusTraversal,
    ScenarioCapture,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorUxScenario {
    pub id: EditorUxScenarioId,
    pub label: String,
    pub kind: EditorUxScenarioKind,
    pub args: EditorUxScenarioArgs,
    pub readiness: ToolSurfaceReadiness,
    pub scenario_matrix: EditorUxScenarioMatrix,
    pub design_system_evidence: Option<EditorUxDesignSystemEvidence>,
    pub graph_canvas_evidence: Option<EditorUxGraphCanvasEvidence>,
    pub product_pattern_evidence: Option<EditorUxProductPatternEvidence>,
    pub registered_surface_evidence: Option<EditorUxRegisteredSurfaceEvidence>,
    pub workbench_evidence: Option<EditorUxWorkbenchEvidence>,
    pub interactions: Vec<EditorUxScenarioInteraction>,
    pub root: UiNode,
    pub scan_requirement: VisibleWidgetScanRequirement,
}

impl EditorUxScenario {
    pub fn new(
        id: EditorUxScenarioId,
        label: impl Into<String>,
        kind: EditorUxScenarioKind,
        readiness: ToolSurfaceReadiness,
        scenario_matrix: EditorUxScenarioMatrix,
        root: UiNode,
        scan_requirement: VisibleWidgetScanRequirement,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            kind,
            args: EditorUxScenarioArgs::default(),
            readiness,
            scenario_matrix,
            design_system_evidence: None,
            graph_canvas_evidence: None,
            product_pattern_evidence: None,
            registered_surface_evidence: None,
            workbench_evidence: None,
            interactions: Vec::new(),
            root,
            scan_requirement,
        }
    }

    pub fn with_interactions(
        mut self,
        interactions: impl IntoIterator<Item = EditorUxScenarioInteraction>,
    ) -> Self {
        self.interactions = interactions.into_iter().collect();
        self
    }

    pub fn with_design_system_evidence(mut self, evidence: EditorUxDesignSystemEvidence) -> Self {
        self.scenario_matrix
            .design_system_state_variants
            .extend(evidence.state_variants.iter().cloned());
        self.design_system_evidence = Some(evidence);
        self
    }

    pub fn with_workbench_evidence(mut self, evidence: EditorUxWorkbenchEvidence) -> Self {
        self.workbench_evidence = Some(evidence);
        self
    }

    pub fn with_graph_canvas_evidence(mut self, evidence: EditorUxGraphCanvasEvidence) -> Self {
        self.graph_canvas_evidence = Some(evidence);
        self
    }

    pub fn with_product_pattern_evidence(
        mut self,
        evidence: EditorUxProductPatternEvidence,
    ) -> Self {
        self.product_pattern_evidence = Some(evidence);
        self
    }

    pub fn with_registered_surface_evidence(
        mut self,
        evidence: EditorUxRegisteredSurfaceEvidence,
    ) -> Self {
        self.registered_surface_evidence = Some(evidence);
        self
    }
}
