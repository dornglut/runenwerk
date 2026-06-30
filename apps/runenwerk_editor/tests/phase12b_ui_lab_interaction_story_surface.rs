use editor_shell::{ToolSurfaceRoute, ToolSurfaceStableKey, tool_surface_kind_for_stable_key};
use runenwerk_editor::shell::RunenwerkWorkbenchHost;

#[test]
fn ui_lab_interaction_story_surface_is_registry_backed_without_legacy_kind() {
    let host = RunenwerkWorkbenchHost::full_editor()
        .expect("full editor workbench should validate UI Lab provider support");
    let key = ToolSurfaceStableKey::new("runenwerk.ui_lab.interaction_story")
        .expect("UI Lab stable key should be valid");
    let surface = host
        .tool_surface_registry()
        .get(&key)
        .expect("UI Lab interaction story surface should be registered");

    assert_eq!(surface.label, "Interaction Story Lab");
    assert_eq!(surface.provider_family.as_str(), "runenwerk.ui_lab");
    assert_eq!(surface.route, ToolSurfaceRoute::ProviderOwnedLocal);
    assert_eq!(tool_surface_kind_for_stable_key(&key), None);
}
