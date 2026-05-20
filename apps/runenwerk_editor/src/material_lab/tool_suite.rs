//! File: apps/runenwerk_editor/src/material_lab/tool_suite.rs
//! Purpose: Material Lab tool-suite declaration.

use editor_shell::{
    EditorToolSuite, PanelKind, ProviderFamilyDefinition, ProviderFamilyId, ToolSuiteId,
    ToolSurfaceDefinition, ToolSurfacePersistence, ToolSurfaceRole, ToolSurfaceRoute,
    ToolSurfaceStableKey,
};

pub fn material_lab_tool_suite() -> EditorToolSuite {
    let suite_id = ToolSuiteId::new("runenwerk.material_lab").unwrap();
    let provider_family_id = ProviderFamilyId::new("runenwerk.material_lab").unwrap();

    EditorToolSuite {
        suite_id,
        label: "Material Lab".to_string(),
        provider_families: vec![ProviderFamilyDefinition {
            id: provider_family_id.clone(),
            label: "Material Lab".to_string(),
        }],
        surfaces: vec![
            material_lab_surface(
                "runenwerk.material_lab.graph_canvas",
                "Material Graph",
                ToolSurfaceRole::Primary,
                PanelKind::MaterialGraphCanvas,
                provider_family_id.clone(),
                ToolSurfaceRoute::ProviderOwnedGraphCanvas,
            ),
            material_lab_surface(
                "runenwerk.material_lab.inspector",
                "Material Inspector",
                ToolSurfaceRole::Inspector,
                PanelKind::MaterialInspector,
                provider_family_id.clone(),
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
            material_lab_surface(
                "runenwerk.material_lab.preview",
                "Material Preview",
                ToolSurfaceRole::Preview,
                PanelKind::MaterialPreview,
                provider_family_id,
                ToolSurfaceRoute::ProviderOwnedLocal,
            ),
        ],
    }
}

fn material_lab_surface(
    key: &str,
    label: &str,
    role: ToolSurfaceRole,
    panel_kind: PanelKind,
    provider_family: ProviderFamilyId,
    route: ToolSurfaceRoute,
) -> ToolSurfaceDefinition {
    ToolSurfaceDefinition {
        key: ToolSurfaceStableKey::new(key).unwrap(),
        label: label.to_string(),
        role,
        panel_kind,
        provider_family,
        route,
        persistence: ToolSurfacePersistence::StableKey,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_lab_tool_suite_registers_three_stable_surfaces() {
        let suite = material_lab_tool_suite();

        let keys = suite
            .surfaces
            .iter()
            .map(|surface| surface.key.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            keys,
            vec![
                "runenwerk.material_lab.graph_canvas",
                "runenwerk.material_lab.inspector",
                "runenwerk.material_lab.preview",
            ]
        );
    }

    #[test]
    fn material_lab_surfaces_are_stable_key_native() {
        let suite = material_lab_tool_suite();

        assert_eq!(suite.suite_id.as_str(), "runenwerk.material_lab");
        assert!(suite.surfaces.iter().all(|surface| {
            surface.persistence == ToolSurfacePersistence::StableKey
                && surface.provider_family.as_str() == "runenwerk.material_lab"
        }));
    }

    #[test]
    fn material_lab_tool_suite_uses_provider_owned_graph_route_for_graph_canvas() {
        let suite = material_lab_tool_suite();
        let graph_canvas = suite
            .surfaces
            .iter()
            .find(|surface| surface.key.as_str() == "runenwerk.material_lab.graph_canvas")
            .expect("graph canvas should be declared");

        assert_eq!(graph_canvas.label, "Material Graph");
        assert_eq!(graph_canvas.role, ToolSurfaceRole::Primary);
        assert_eq!(
            graph_canvas.route,
            ToolSurfaceRoute::ProviderOwnedGraphCanvas
        );
        assert_eq!(graph_canvas.persistence, ToolSurfacePersistence::StableKey);
    }

    #[test]
    fn material_lab_graph_canvas_has_no_legacy_tool_surface_kind() {
        let source = include_str!("tool_suite.rs");

        assert!(!source.contains(concat!("Tool", "SurfaceKind::MaterialGraphCanvas")));
    }

    #[test]
    fn material_lab_inspector_has_no_legacy_tool_surface_kind() {
        let source = include_str!("tool_suite.rs");

        assert!(!source.contains(concat!("Tool", "SurfaceKind::MaterialInspector")));
    }

    #[test]
    fn material_lab_preview_has_no_legacy_tool_surface_kind() {
        let source = include_str!("tool_suite.rs");

        assert!(!source.contains(concat!("Tool", "SurfaceKind::MaterialPreview")));
    }

    #[test]
    fn material_lab_tool_suite_declares_provider_family() {
        let suite = material_lab_tool_suite();

        assert_eq!(suite.suite_id.as_str(), "runenwerk.material_lab");
        assert_eq!(suite.provider_families.len(), 1);
        assert_eq!(
            suite.provider_families[0].id.as_str(),
            "runenwerk.material_lab"
        );
        assert!(
            suite
                .surfaces
                .iter()
                .all(|surface| surface.provider_family.as_str() == "runenwerk.material_lab")
        );
    }
}
