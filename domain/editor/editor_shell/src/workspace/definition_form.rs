//! File: domain/editor/editor_shell/src/workspace/definition_form.rs
//! Purpose: Resolve authored editor tool-surface references at the shell boundary.

use crate::{
    ToolSurfaceKind, stable_key_for_tool_surface_kind,
    tool_suite::{ToolSurfaceDefinition, ToolSurfaceRegistry, ToolSurfaceStableKey},
    tool_surface_kind_from_definition_key,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoredToolSurfaceResolution<'a> {
    RegistryBacked {
        stable_surface_key: ToolSurfaceStableKey,
        definition: &'a ToolSurfaceDefinition,
    },
    Legacy {
        tool_surface_kind: ToolSurfaceKind,
        stable_surface_key: Option<ToolSurfaceStableKey>,
    },
    UnknownStableSurfaceKey {
        stable_surface_key: ToolSurfaceStableKey,
    },
    UnknownAuthoredSurface {
        authored_key: String,
    },
}

pub fn resolve_authored_tool_surface_reference<'a>(
    authored_key: &str,
    registry: Option<&'a ToolSurfaceRegistry>,
) -> AuthoredToolSurfaceResolution<'a> {
    if let Some(registry) = registry
        && authored_key.contains('.')
        && let Ok(stable_surface_key) = ToolSurfaceStableKey::new(authored_key.to_string())
    {
        return match registry.get(&stable_surface_key) {
            Some(definition) => AuthoredToolSurfaceResolution::RegistryBacked {
                stable_surface_key,
                definition,
            },
            None => AuthoredToolSurfaceResolution::UnknownStableSurfaceKey { stable_surface_key },
        };
    }

    match tool_surface_kind_from_definition_key(authored_key) {
        Some(tool_surface_kind) => AuthoredToolSurfaceResolution::Legacy {
            tool_surface_kind,
            stable_surface_key: stable_key_for_tool_surface_kind(tool_surface_kind),
        },
        None => AuthoredToolSurfaceResolution::UnknownAuthoredSurface {
            authored_key: authored_key.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EditorToolSuite, ProviderFamilyDefinition, ProviderFamilyId, ToolSuiteId,
        ToolSuiteRegistry, ToolSurfacePersistence, ToolSurfaceRole, ToolSurfaceRoute,
    };

    #[test]
    fn authored_stable_surface_key_resolves_through_registry() {
        let registry = material_lab_registry();

        let resolution = resolve_authored_tool_surface_reference(
            "runenwerk.material_lab.graph_canvas",
            Some(registry.surfaces()),
        );

        match resolution {
            AuthoredToolSurfaceResolution::RegistryBacked {
                stable_surface_key,
                definition,
            } => {
                assert_eq!(
                    stable_surface_key.as_str(),
                    "runenwerk.material_lab.graph_canvas"
                );
                assert_eq!(definition.label, "Material Graph");
            }
            other => panic!("expected registry-backed resolution, got {other:?}"),
        }
    }

    #[test]
    fn authored_legacy_surface_key_still_resolves_through_legacy_path() {
        let registry = material_lab_registry();

        let resolution = resolve_authored_tool_surface_reference(
            "material_graph_canvas",
            Some(registry.surfaces()),
        );

        assert_eq!(
            resolution,
            AuthoredToolSurfaceResolution::Legacy {
                tool_surface_kind: ToolSurfaceKind::MaterialGraphCanvas,
                stable_surface_key: Some(
                    ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap()
                ),
            }
        );
    }

    #[test]
    fn unknown_authored_stable_surface_key_fails_closed() {
        let registry = material_lab_registry();
        let unknown_key = ToolSurfaceStableKey::new("runenwerk.material_lab.unknown").unwrap();

        let resolution = resolve_authored_tool_surface_reference(
            "runenwerk.material_lab.unknown",
            Some(registry.surfaces()),
        );

        assert_eq!(
            resolution,
            AuthoredToolSurfaceResolution::UnknownStableSurfaceKey {
                stable_surface_key: unknown_key
            }
        );
    }

    #[test]
    fn unknown_authored_legacy_surface_key_remains_unknown() {
        let registry = material_lab_registry();

        let resolution =
            resolve_authored_tool_surface_reference("missing_surface", Some(registry.surfaces()));

        assert_eq!(
            resolution,
            AuthoredToolSurfaceResolution::UnknownAuthoredSurface {
                authored_key: "missing_surface".to_string()
            }
        );
    }

    #[test]
    fn stable_key_resolution_does_not_silently_fallback_to_legacy() {
        let registry = material_lab_registry();
        let unknown_key =
            ToolSurfaceStableKey::new("runenwerk.material_lab.material_graph_canvas").unwrap();

        let resolution = resolve_authored_tool_surface_reference(
            "runenwerk.material_lab.material_graph_canvas",
            Some(registry.surfaces()),
        );

        assert_eq!(
            resolution,
            AuthoredToolSurfaceResolution::UnknownStableSurfaceKey {
                stable_surface_key: unknown_key
            }
        );
    }

    fn material_lab_registry() -> ToolSuiteRegistry {
        let provider_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        ToolSuiteRegistry::new(vec![EditorToolSuite {
            suite_id: ToolSuiteId::new("runenwerk.material_lab").unwrap(),
            label: "Material Lab".to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family.clone(),
                label: "Material Lab".to_string(),
            }],
            surfaces: vec![
                material_lab_surface(
                    "runenwerk.material_lab.graph_canvas",
                    "Material Graph",
                    ToolSurfaceRole::Primary,
                    provider_family.clone(),
                    ToolSurfaceRoute::ProviderOwnedGraphCanvas,
                ),
                material_lab_surface(
                    "runenwerk.material_lab.inspector",
                    "Material Inspector",
                    ToolSurfaceRole::Inspector,
                    provider_family.clone(),
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
                material_lab_surface(
                    "runenwerk.material_lab.preview",
                    "Material Preview",
                    ToolSurfaceRole::Preview,
                    provider_family,
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
            ],
        }])
        .expect("material lab registry fixture should be valid")
    }

    fn material_lab_surface(
        key: &str,
        label: &str,
        role: ToolSurfaceRole,
        provider_family: ProviderFamilyId,
        route: ToolSurfaceRoute,
    ) -> ToolSurfaceDefinition {
        ToolSurfaceDefinition {
            key: ToolSurfaceStableKey::new(key).unwrap(),
            label: label.to_string(),
            role,
            panel_kind: match role {
                ToolSurfaceRole::Primary => crate::PanelKind::MaterialGraphCanvas,
                ToolSurfaceRole::Inspector => crate::PanelKind::MaterialInspector,
                ToolSurfaceRole::Preview => crate::PanelKind::MaterialPreview,
            },
            provider_family,
            route,
            persistence: ToolSurfacePersistence::StableKey,
            capabilities: ui_surface::SurfaceCapabilitySet::new(true, true, true, false),
            session_retention: ui_surface::SessionRetentionClass::Restorable,
            creation_policy: crate::ToolSurfaceCreationPolicy::SingletonPerWorkspace,
            target_profile_compatibility: crate::ToolSurfaceTargetProfileCompatibility::AllProfiles,
            legacy_compatibility_key: None,
        }
    }
}
