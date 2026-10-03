use editor_definition::{
    EditorWorkspaceHostDefinition, EditorWorkspaceLayoutDefinition,
    EditorWorkspacePanelTabDefinition, EditorWorkspaceSplitAxisDefinition,
};
use ui_surface::{SessionRetentionClass, SurfaceCapabilitySet};

use crate::{
    EditorToolSuite, PanelKind, ProviderFamilyDefinition, ProviderFamilyId,
    SCENE_WORKSPACE_PROFILE_ID, SuiteRef, SurfaceRef, ToolSuiteRegistry, ToolSurfaceCreationPolicy,
    ToolSurfaceDefinition, ToolSurfaceRole, ToolSurfaceRoute,
};

use super::{EditorCompositionRuntime, form_editor_profile_composition};

const TEST_SURFACE_KEY: &str = "runenwerk.diagnostics.diagnostics";

pub(crate) fn test_editor_composition_runtime() -> EditorCompositionRuntime {
    let provider_family = ProviderFamilyId::new("runenwerk.test").unwrap();
    let registry = ToolSuiteRegistry::new(vec![EditorToolSuite::new(
        SuiteRef::from_stable_key("runenwerk.test").unwrap(),
        "Test",
        vec![ProviderFamilyDefinition::new(
            provider_family.clone(),
            "Test",
        )],
        vec![ToolSurfaceDefinition::new(
            SurfaceRef::from_stable_key(TEST_SURFACE_KEY).unwrap(),
            "Diagnostics",
            ToolSurfaceRole::Primary,
            PanelKind::Diagnostics,
            provider_family,
            ToolSurfaceRoute::ProviderOwnedLocal,
            SurfaceCapabilitySet::new(true, true, true, false),
            SessionRetentionClass::Restorable,
            ToolSurfaceCreationPolicy::MultipleInstances,
        )],
    )])
    .unwrap();

    let tab = |id: &str| EditorWorkspacePanelTabDefinition {
        id: format!("{id}.tab"),
        label: "Diagnostics".to_string(),
        tool_surface: TEST_SURFACE_KEY.to_string(),
    };
    let stack = |id: &str| EditorWorkspaceHostDefinition::TabStack {
        id: id.to_string(),
        tabs: vec![tab(id)],
        active_tab: Some(format!("{id}.tab")),
    };
    let layout = EditorWorkspaceLayoutDefinition {
        id: "runenwerk.test.composition".to_string(),
        label: "Test Composition".to_string(),
        root: EditorWorkspaceHostDefinition::Split {
            id: "root".to_string(),
            axis: EditorWorkspaceSplitAxisDefinition::Horizontal,
            fraction: 0.5,
            first: Box::new(stack("left")),
            second: Box::new(stack("right")),
        },
        floating_hosts: Vec::new(),
    };

    form_editor_profile_composition(SCENE_WORKSPACE_PROFILE_ID, &layout, registry.surfaces())
        .expect("current composition-native test layout should form")
}
