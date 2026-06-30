//! File: apps/runenwerk_editor/src/shell/tool_suites/mod.rs
//! Purpose: Compiled-in tool-suite metadata declarations for current editor surfaces.

use editor_shell::{
    EditorToolSuite, PanelKind, ProviderFamilyDefinition, ProviderFamilyId, SuiteRef, SurfaceRef,
    ToolSuiteId, ToolSurfaceCreationPolicy, ToolSurfaceDefinition, ToolSurfaceKind,
    ToolSurfaceRole, ToolSurfaceRoute, ToolSurfaceStableKey, editor_surface_definitions,
    panel_kind_for_tool_surface_kind, stable_key_for_tool_surface_kind,
    tool_surface_capability_set, tool_surface_definition_id, tool_surface_kind_definition_key,
    tool_surface_session_retention_class,
};
use ui_surface::{SessionRetentionClass, SurfaceCapabilitySet};

pub mod asset_tool_suite;
pub mod core_tool_suite;
pub mod diagnostics_tool_suite;
pub mod editor_design_tool_suite;
pub mod field_sdf_tool_suite;
pub mod gameplay_tool_suite;
pub mod graph_tool_suite;
pub mod procgen_tool_suite;
pub mod simulation_tool_suite;
pub mod texture_tool_suite;
pub mod ui_lab_tool_suite;

pub(crate) const SCENE_OUTLINER_SURFACE_KEY: &str = "runenwerk.scene.outliner";
pub(crate) const SCENE_ENTITY_TABLE_SURFACE_KEY: &str = "runenwerk.scene.entity_table";
pub(crate) const SCENE_VIEWPORT_SURFACE_KEY: &str = "runenwerk.scene.viewport";
pub(crate) const SCENE_INSPECTOR_SURFACE_KEY: &str = "runenwerk.scene.inspector";
pub(crate) const EDITOR_CONSOLE_SURFACE_KEY: &str = "runenwerk.editor.console";
pub(crate) const EDITOR_DESIGN_SURFACE_KEYS: &[&str] = &[
    "runenwerk.editor_design.definition_outliner",
    "runenwerk.editor_design.ui_hierarchy",
    "runenwerk.editor_design.ui_canvas",
    "runenwerk.editor_design.style_inspector",
    "runenwerk.editor_design.bindings",
    "runenwerk.editor_design.dock_layout_preview",
    "runenwerk.editor_design.theme_editor",
    "runenwerk.editor_design.shortcut_editor",
    "runenwerk.editor_design.menu_editor",
    "runenwerk.editor_design.definition_validation",
    "runenwerk.editor_design.command_diff",
];
pub(crate) const ASSET_BROWSER_SURFACE_KEY: &str = "runenwerk.assets.browser";
pub(crate) const IMPORT_INSPECTOR_SURFACE_KEY: &str = "runenwerk.assets.import_inspector";
pub(crate) const FIELD_PRODUCT_VIEWER_SURFACE_KEY: &str = "runenwerk.field_world.product_viewer";
pub(crate) const SDF_BRUSH_BROWSER_SURFACE_KEY: &str = "runenwerk.field_world.sdf_brush_browser";
pub(crate) const FIELD_LAYER_STACK_SURFACE_KEY: &str = "runenwerk.field_world.layer_stack";
pub(crate) const SDF_GRAPH_CANVAS_SURFACE_KEY: &str = "runenwerk.field_world.sdf_graph_canvas";
pub(crate) const DIAGNOSTICS_SURFACE_KEYS: &[&str] = &[
    "runenwerk.diagnostics.diagnostics",
    "runenwerk.diagnostics.runtime_debug",
    "runenwerk.diagnostics.placeholder",
];
pub(crate) const TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY: &str =
    "runenwerk.diagnostics.tool_suite_registry_inspector";
pub(crate) const UI_LAB_INTERACTION_STORY_SURFACE_KEY: &str =
    "runenwerk.ui_lab.interaction_story";
pub(crate) const MATERIAL_GRAPH_CANVAS_SURFACE_KEY: &str = "runenwerk.material_lab.graph_canvas";
pub(crate) const MATERIAL_INSPECTOR_SURFACE_KEY: &str = "runenwerk.material_lab.inspector";
pub(crate) const MATERIAL_PREVIEW_SURFACE_KEY: &str = "runenwerk.material_lab.preview";
pub(crate) const TEXTURE_VIEWER_2D_SURFACE_KEY: &str = "runenwerk.texture.viewer_2d";
pub(crate) const TEXTURE_VIEWER_3D_SURFACE_KEY: &str = "runenwerk.texture.viewer_3d";
pub(crate) const PROCGEN_GRAPH_CANVAS_SURFACE_KEY: &str = "runenwerk.procgen.graph_canvas";
pub(crate) const PROCGEN_PREVIEW_SURFACE_KEY: &str = "runenwerk.procgen.preview";

pub fn runenwerk_shell_tool_suites() -> Vec<EditorToolSuite> {
    vec![
        core_tool_suite::scene_tool_suite(),
        core_tool_suite::editor_core_tool_suite(),
        editor_design_tool_suite::editor_design_tool_suite(),
        asset_tool_suite::asset_tool_suite(),
        field_sdf_tool_suite::field_sdf_tool_suite(),
        graph_tool_suite::graph_tool_suite(),
        diagnostics_tool_suite::diagnostics_tool_suite(),
        ui_lab_tool_suite::ui_lab_tool_suite(),
        texture_tool_suite::texture_tool_suite(),
        procgen_tool_suite::procgen_tool_suite(),
        gameplay_tool_suite::gameplay_tool_suite(),
        gameplay_tool_suite::particle_tool_suite(),
        gameplay_tool_suite::physics_tool_suite(),
        simulation_tool_suite::animation_tool_suite(),
        simulation_tool_suite::simulation_tool_suite(),
    ]
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToolSuiteSurface {
    pub kind: ToolSurfaceKind,
    pub role: ToolSurfaceRole,
    pub route: ToolSurfaceRoute,
}

pub(crate) fn tool_suite(
    suite_id: &str,
    label: &str,
    surfaces: &[ToolSuiteSurface],
) -> EditorToolSuite {
    let suite_id = ToolSuiteId::new(suite_id).expect("compiled-in suite id should be valid");
    let provider_family_id = ProviderFamilyId::new(suite_id.as_str())
        .expect("compiled-in provider family should be valid");

    EditorToolSuite::new(
        SuiteRef::new(suite_id),
        label,
        vec![ProviderFamilyDefinition::new(
            provider_family_id.clone(),
            label,
        )],
        surfaces
            .iter()
            .map(|surface| tool_surface_definition(*surface, provider_family_id.clone()))
            .collect(),
    )
}

fn tool_surface_definition(
    surface: ToolSuiteSurface,
    provider_family: ProviderFamilyId,
) -> ToolSurfaceDefinition {
    ToolSurfaceDefinition::new(
        SurfaceRef::new(
            stable_key_for_tool_surface_kind(surface.kind)
                .expect("saveable compiled-in surface should have a stable key"),
        ),
        surface_label(surface.kind),
        surface.role,
        panel_kind_for_tool_surface_kind(surface.kind),
        provider_family,
        surface.route,
        tool_surface_capability_set(surface.kind),
        tool_surface_session_retention_class(surface.kind),
        ToolSurfaceCreationPolicy::SingletonPerWorkspace,
    )
    .with_legacy_compatibility_key(tool_surface_kind_definition_key(surface.kind))
}

pub(crate) fn stable_tool_surface_definition(
    stable_key: &str,
    label: &str,
    role: ToolSurfaceRole,
    panel_kind: PanelKind,
    route: ToolSurfaceRoute,
    provider_family: ProviderFamilyId,
) -> ToolSurfaceDefinition {
    ToolSurfaceDefinition::new(
        SurfaceRef::new(
            ToolSurfaceStableKey::new(stable_key)
                .expect("compiled-in stable tool surface key should be valid"),
        ),
        label,
        role,
        panel_kind,
        provider_family,
        route,
        stable_surface_capabilities(panel_kind),
        SessionRetentionClass::Restorable,
        ToolSurfaceCreationPolicy::SingletonPerWorkspace,
    )
}

fn stable_surface_capabilities(panel_kind: PanelKind) -> SurfaceCapabilitySet {
    match panel_kind {
        PanelKind::Placeholder => SurfaceCapabilitySet::new(true, false, false, false),
        PanelKind::Console
        | PanelKind::Diagnostics
        | PanelKind::RuntimeDebug
        | PanelKind::GameplayCompilerDiagnostics
        | PanelKind::PhysicsDebug
        | PanelKind::SimulationDiagnostics => SurfaceCapabilitySet::new(true, true, false, false),
        _ => SurfaceCapabilitySet::new(true, true, true, false),
    }
}

fn surface_label(kind: ToolSurfaceKind) -> &'static str {
    let definition_id = tool_surface_definition_id(kind);
    editor_surface_definitions()
        .into_iter()
        .find(|definition| definition.id == definition_id)
        .map(|definition| definition.display_name)
        .expect("every compiled-in surface should have a legacy surface definition")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use editor_shell::{
        ToolSurfacePersistence, saveable_tool_surface_stable_key_candidates,
        tool_surface_kind_for_stable_key,
    };

    use super::*;

    #[test]
    fn shell_tool_suites_have_unique_stable_keys() {
        let mut keys = BTreeSet::new();

        for suite in runenwerk_shell_tool_suites() {
            for surface in suite.surfaces {
                assert!(
                    keys.insert(surface.key),
                    "duplicate shell tool-suite stable key"
                );
            }
        }
    }

    #[test]
    fn shell_tool_suites_cover_non_material_saveable_candidates() {
        let suites = runenwerk_shell_tool_suites();
        let registered_keys = suites
            .iter()
            .flat_map(|suite| suite.surfaces.iter())
            .map(|surface| surface.key.as_str())
            .collect::<BTreeSet<_>>();

        for candidate in saveable_tool_surface_stable_key_candidates() {
            if candidate.stable_key.starts_with("runenwerk.material_lab.") {
                continue;
            }
            assert!(
                registered_keys.contains(candidate.stable_key),
                "missing shell suite metadata for {}",
                candidate.stable_key
            );
        }
    }

    #[test]
    fn non_material_graph_surfaces_are_not_marked_provider_owned_graph_routes_yet() {
        let graph_routes = runenwerk_shell_tool_suites()
            .into_iter()
            .flat_map(|suite| suite.surfaces.into_iter())
            .filter(|surface| surface.route == ToolSurfaceRoute::ProviderOwnedGraphCanvas)
            .map(|surface| surface.key.as_str().to_string())
            .collect::<Vec<_>>();

        assert_eq!(graph_routes, Vec::<String>::new());
    }

    #[test]
    fn diagnostics_suite_registers_tool_suite_registry_inspector_surface() {
        let suite = diagnostics_tool_suite::diagnostics_tool_suite();
        let surface = suite
            .surfaces
            .iter()
            .find(|surface| surface.key.as_str() == TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY)
            .expect("Tool Suite Registry Inspector surface should be registered");

        assert_eq!(surface.label, "Tool Suite Registry Inspector");
        assert_eq!(surface.provider_family.as_str(), "runenwerk.diagnostics");
    }

    #[test]
    fn ui_lab_suite_registers_interaction_story_surface() {
        let suite = ui_lab_tool_suite::ui_lab_tool_suite();
        let surface = suite
            .surfaces
            .iter()
            .find(|surface| surface.key.as_str() == UI_LAB_INTERACTION_STORY_SURFACE_KEY)
            .expect("UI Lab interaction story surface should be registered");

        assert_eq!(suite.suite_id.as_str(), "runenwerk.ui_lab");
        assert_eq!(surface.label, "Interaction Story Lab");
        assert_eq!(surface.provider_family.as_str(), "runenwerk.ui_lab");
    }

    #[test]
    fn inspector_surface_has_no_legacy_tool_surface_kind() {
        let suite = diagnostics_tool_suite::diagnostics_tool_suite();
        let surface = suite
            .surfaces
            .iter()
            .find(|surface| surface.key.as_str() == TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY)
            .expect("Tool Suite Registry Inspector surface should be registered");

        assert_eq!(tool_surface_kind_for_stable_key(&surface.key), None);
    }

    #[test]
    fn ui_lab_interaction_story_surface_has_no_legacy_tool_surface_kind() {
        let suite = ui_lab_tool_suite::ui_lab_tool_suite();
        let surface = suite
            .surfaces
            .iter()
            .find(|surface| surface.key.as_str() == UI_LAB_INTERACTION_STORY_SURFACE_KEY)
            .expect("UI Lab interaction story surface should be registered");

        assert_eq!(tool_surface_kind_for_stable_key(&surface.key), None);
    }

    #[test]
    fn inspector_surface_uses_provider_owned_local_route() {
        let suite = diagnostics_tool_suite::diagnostics_tool_suite();
        let surface = suite
            .surfaces
            .iter()
            .find(|surface| surface.key.as_str() == TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY)
            .expect("Tool Suite Registry Inspector surface should be registered");

        assert_eq!(surface.route, ToolSurfaceRoute::ProviderOwnedLocal);
        assert_eq!(surface.persistence, ToolSurfacePersistence::StableKey);
    }

    #[test]
    fn ui_lab_interaction_story_surface_uses_provider_owned_local_route() {
        let suite = ui_lab_tool_suite::ui_lab_tool_suite();
        let surface = suite
            .surfaces
            .iter()
            .find(|surface| surface.key.as_str() == UI_LAB_INTERACTION_STORY_SURFACE_KEY)
            .expect("UI Lab interaction story surface should be registered");

        assert_eq!(surface.route, ToolSurfaceRoute::ProviderOwnedLocal);
        assert_eq!(surface.persistence, ToolSurfacePersistence::StableKey);
    }
}
