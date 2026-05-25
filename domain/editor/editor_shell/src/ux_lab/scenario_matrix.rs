//! File: domain/editor/editor_shell/src/ux_lab/scenario_matrix.rs
//! Purpose: Editor UX Lab scenario matrix contracts.

use std::collections::BTreeSet;

use crate::VisibleWidgetState;
use ui_definition::UiRecipeStateVariantId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorUxDensity {
    Comfortable,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorUxViewportClass {
    Narrow,
    Standard,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorUxInputModality {
    Pointer,
    Keyboard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorUxScenarioMatrix {
    pub densities: Vec<EditorUxDensity>,
    pub viewport_classes: Vec<EditorUxViewportClass>,
    pub input_modalities: Vec<EditorUxInputModality>,
    pub required_widget_states: BTreeSet<VisibleWidgetState>,
    pub design_system_state_variants: BTreeSet<UiRecipeStateVariantId>,
    pub expected_diagnostics: Vec<&'static str>,
}

impl EditorUxScenarioMatrix {
    pub fn baseline(required_widget_states: impl IntoIterator<Item = VisibleWidgetState>) -> Self {
        Self {
            densities: vec![EditorUxDensity::Comfortable],
            viewport_classes: vec![EditorUxViewportClass::Standard],
            input_modalities: vec![
                EditorUxInputModality::Pointer,
                EditorUxInputModality::Keyboard,
            ],
            required_widget_states: required_widget_states.into_iter().collect(),
            design_system_state_variants: BTreeSet::new(),
            expected_diagnostics: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.densities.is_empty()
            || self.viewport_classes.is_empty()
            || self.input_modalities.is_empty()
    }
}
