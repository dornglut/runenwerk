//! File: apps/runenwerk_editor/src/shell/tool_suites/ui_lab_tool_suite.rs
//! Purpose: UI Lab tool-suite metadata for reusable UI proof surfaces.

use editor_shell::{
    EditorToolSuite, PanelKind, ProviderFamilyDefinition, ProviderFamilyId, SuiteRef, ToolSuiteId,
    ToolSurfaceRole, ToolSurfaceRoute,
};

use super::{UI_LAB_INTERACTION_STORY_SURFACE_KEY, stable_tool_surface_definition};

pub fn ui_lab_tool_suite() -> EditorToolSuite {
    let suite_id =
        ToolSuiteId::new("runenwerk.ui_lab").expect("compiled-in UI Lab suite id should be valid");
    let provider_family = ProviderFamilyId::new(suite_id.as_str())
        .expect("compiled-in UI Lab provider family should be valid");

    EditorToolSuite::new(
        SuiteRef::new(suite_id),
        "UI Lab",
        vec![ProviderFamilyDefinition::new(
            provider_family.clone(),
            "UI Lab",
        )],
        vec![stable_tool_surface_definition(
            UI_LAB_INTERACTION_STORY_SURFACE_KEY,
            "Interaction Story Lab",
            ToolSurfaceRole::Preview,
            PanelKind::Diagnostics,
            ToolSurfaceRoute::ProviderOwnedLocal,
            provider_family,
        )],
    )
}
