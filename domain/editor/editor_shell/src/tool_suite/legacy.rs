//! File: domain/editor/editor_shell/src/tool_suite/legacy.rs
//! Purpose: Compatibility candidates from legacy enum surfaces to stable keys.
//!
//! `ToolSurfaceKind` is not normal tool-surface authority after Option C1-C4.
//! Normal workspace state, mutations, profiles, projection, provider requests,
//! provider matching, and V5 persistence use `ToolSurfaceStableKey` as their
//! authority. This module is the explicit compatibility boundary for legacy
//! enum callers, V1-V4 persisted layout migration, V5 legacy metadata
//! validation, authored legacy-key adapters, and tests that prove those
//! boundary paths still work.
//!
//! The reverse stable-key to `ToolSurfaceKind` bridge exists only while old
//! callers are retired or quarantined. Do not use it for new stable-key-first
//! code paths.

use crate::{PersistedToolSurfaceKindV2, ToolSurfaceKind};

use super::{ToolSurfaceDefinition, ToolSurfaceRegistry, ToolSurfaceStableKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyToolSurfaceStableKeyCandidate {
    pub kind: ToolSurfaceKind,
    pub persisted_kind: PersistedToolSurfaceKindV2,
    pub stable_key: &'static str,
}

pub const SAVEABLE_TOOL_SURFACE_STABLE_KEY_CANDIDATES: &[LegacyToolSurfaceStableKeyCandidate] = &[
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::Outliner,
        persisted_kind: PersistedToolSurfaceKindV2::Outliner,
        stable_key: "runenwerk.scene.outliner",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::EntityTable,
        persisted_kind: PersistedToolSurfaceKindV2::EntityTable,
        stable_key: "runenwerk.scene.entity_table",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::Viewport,
        persisted_kind: PersistedToolSurfaceKindV2::Viewport,
        stable_key: "runenwerk.scene.viewport",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::Inspector,
        persisted_kind: PersistedToolSurfaceKindV2::Inspector,
        stable_key: "runenwerk.scene.inspector",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::Console,
        persisted_kind: PersistedToolSurfaceKindV2::Console,
        stable_key: "runenwerk.editor.console",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::EditorDesignOutliner,
        persisted_kind: PersistedToolSurfaceKindV2::EditorDesignOutliner,
        stable_key: "runenwerk.editor_design.definition_outliner",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::UiHierarchy,
        persisted_kind: PersistedToolSurfaceKindV2::UiHierarchy,
        stable_key: "runenwerk.editor_design.ui_hierarchy",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::UiCanvas,
        persisted_kind: PersistedToolSurfaceKindV2::UiCanvas,
        stable_key: "runenwerk.editor_design.ui_canvas",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::StyleInspector,
        persisted_kind: PersistedToolSurfaceKindV2::StyleInspector,
        stable_key: "runenwerk.editor_design.style_inspector",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::Bindings,
        persisted_kind: PersistedToolSurfaceKindV2::Bindings,
        stable_key: "runenwerk.editor_design.bindings",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::DockLayoutPreview,
        persisted_kind: PersistedToolSurfaceKindV2::DockLayoutPreview,
        stable_key: "runenwerk.editor_design.dock_layout_preview",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::ThemeEditor,
        persisted_kind: PersistedToolSurfaceKindV2::ThemeEditor,
        stable_key: "runenwerk.editor_design.theme_editor",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::ShortcutEditor,
        persisted_kind: PersistedToolSurfaceKindV2::ShortcutEditor,
        stable_key: "runenwerk.editor_design.shortcut_editor",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::MenuEditor,
        persisted_kind: PersistedToolSurfaceKindV2::MenuEditor,
        stable_key: "runenwerk.editor_design.menu_editor",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::DefinitionValidation,
        persisted_kind: PersistedToolSurfaceKindV2::DefinitionValidation,
        stable_key: "runenwerk.editor_design.definition_validation",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::CommandDiff,
        persisted_kind: PersistedToolSurfaceKindV2::CommandDiff,
        stable_key: "runenwerk.editor_design.command_diff",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::AssetBrowser,
        persisted_kind: PersistedToolSurfaceKindV2::AssetBrowser,
        stable_key: "runenwerk.assets.browser",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::ImportInspector,
        persisted_kind: PersistedToolSurfaceKindV2::ImportInspector,
        stable_key: "runenwerk.assets.import_inspector",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::FieldProductViewer,
        persisted_kind: PersistedToolSurfaceKindV2::FieldProductViewer,
        stable_key: "runenwerk.field_world.product_viewer",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::SdfBrushBrowser,
        persisted_kind: PersistedToolSurfaceKindV2::SdfBrushBrowser,
        stable_key: "runenwerk.field_world.sdf_brush_browser",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::GraphCanvas,
        persisted_kind: PersistedToolSurfaceKindV2::GraphCanvas,
        stable_key: "runenwerk.graph.canvas",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::Diagnostics,
        persisted_kind: PersistedToolSurfaceKindV2::Diagnostics,
        stable_key: "runenwerk.diagnostics.diagnostics",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::RuntimeDebug,
        persisted_kind: PersistedToolSurfaceKindV2::RuntimeDebug,
        stable_key: "runenwerk.diagnostics.runtime_debug",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::FieldLayerStack,
        persisted_kind: PersistedToolSurfaceKindV2::FieldLayerStack,
        stable_key: "runenwerk.field_world.layer_stack",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::SdfGraphCanvas,
        persisted_kind: PersistedToolSurfaceKindV2::SdfGraphCanvas,
        stable_key: "runenwerk.field_world.sdf_graph_canvas",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::MaterialGraphCanvas,
        persisted_kind: PersistedToolSurfaceKindV2::MaterialGraphCanvas,
        stable_key: "runenwerk.material_lab.graph_canvas",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::MaterialInspector,
        persisted_kind: PersistedToolSurfaceKindV2::MaterialInspector,
        stable_key: "runenwerk.material_lab.inspector",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::MaterialPreview,
        persisted_kind: PersistedToolSurfaceKindV2::MaterialPreview,
        stable_key: "runenwerk.material_lab.preview",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::TextureViewer,
        persisted_kind: PersistedToolSurfaceKindV2::TextureViewer,
        stable_key: "runenwerk.texture.viewer_2d",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::VolumeTextureViewer,
        persisted_kind: PersistedToolSurfaceKindV2::VolumeTextureViewer,
        stable_key: "runenwerk.texture.viewer_3d",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::ProcgenGraphCanvas,
        persisted_kind: PersistedToolSurfaceKindV2::ProcgenGraphCanvas,
        stable_key: "runenwerk.procgen.graph_canvas",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::ProcgenPreview,
        persisted_kind: PersistedToolSurfaceKindV2::ProcgenPreview,
        stable_key: "runenwerk.procgen.preview",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::GameplayGraphCanvas,
        persisted_kind: PersistedToolSurfaceKindV2::GameplayGraphCanvas,
        stable_key: "runenwerk.gameplay.graph_canvas",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::GameplayCompilerDiagnostics,
        persisted_kind: PersistedToolSurfaceKindV2::GameplayCompilerDiagnostics,
        stable_key: "runenwerk.gameplay.compiler_diagnostics",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::ParticleGraphCanvas,
        persisted_kind: PersistedToolSurfaceKindV2::ParticleGraphCanvas,
        stable_key: "runenwerk.particle.graph_canvas",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::ParticlePreview,
        persisted_kind: PersistedToolSurfaceKindV2::ParticlePreview,
        stable_key: "runenwerk.particle.preview",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::PhysicsAuthoring,
        persisted_kind: PersistedToolSurfaceKindV2::PhysicsAuthoring,
        stable_key: "runenwerk.physics.authoring",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::PhysicsDebug,
        persisted_kind: PersistedToolSurfaceKindV2::PhysicsDebug,
        stable_key: "runenwerk.physics.debug",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::Timeline,
        persisted_kind: PersistedToolSurfaceKindV2::Timeline,
        stable_key: "runenwerk.animation.timeline",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::CurveEditor,
        persisted_kind: PersistedToolSurfaceKindV2::CurveEditor,
        stable_key: "runenwerk.animation.curve_editor",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::AnimationGraphCanvas,
        persisted_kind: PersistedToolSurfaceKindV2::AnimationGraphCanvas,
        stable_key: "runenwerk.animation.graph_canvas",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::SimulationPreview,
        persisted_kind: PersistedToolSurfaceKindV2::SimulationPreview,
        stable_key: "runenwerk.simulation.preview",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::SimulationDiagnostics,
        persisted_kind: PersistedToolSurfaceKindV2::SimulationDiagnostics,
        stable_key: "runenwerk.simulation.diagnostics",
    },
    LegacyToolSurfaceStableKeyCandidate {
        kind: ToolSurfaceKind::Placeholder,
        persisted_kind: PersistedToolSurfaceKindV2::Placeholder,
        stable_key: "runenwerk.diagnostics.placeholder",
    },
];

pub fn saveable_tool_surface_stable_key_candidates()
-> &'static [LegacyToolSurfaceStableKeyCandidate] {
    SAVEABLE_TOOL_SURFACE_STABLE_KEY_CANDIDATES
}

/// Legacy boundary helper for callers that still receive `ToolSurfaceKind`.
///
/// Stable-key-first code should already carry `ToolSurfaceStableKey` and should
/// not route through this helper.
pub fn stable_key_for_tool_surface_kind(kind: ToolSurfaceKind) -> Option<ToolSurfaceStableKey> {
    stable_key_candidate_for_kind(kind).map(|candidate| stable_key(candidate.stable_key))
}

/// V1-V4 persistence migration helper.
pub fn stable_key_for_persisted_tool_surface_kind_v2(
    kind: PersistedToolSurfaceKindV2,
) -> Option<ToolSurfaceStableKey> {
    stable_key_candidate_for_persisted_kind(kind).map(|candidate| stable_key(candidate.stable_key))
}

/// Temporary reverse compatibility bridge for legacy boundary callers while old
/// enum-facing APIs are quarantined.
///
/// Do not broaden this by guessing future suite ownership. Once live workspace
/// identity and external APIs no longer need legacy metadata, this helper should
/// be removed or moved behind a migration-only boundary.
pub fn tool_surface_kind_for_stable_key(key: &ToolSurfaceStableKey) -> Option<ToolSurfaceKind> {
    stable_key_candidate_for_key(key.as_str()).map(|candidate| candidate.kind)
}

pub fn stable_key_candidate_for_kind(
    kind: ToolSurfaceKind,
) -> Option<&'static LegacyToolSurfaceStableKeyCandidate> {
    SAVEABLE_TOOL_SURFACE_STABLE_KEY_CANDIDATES
        .iter()
        .find(|candidate| candidate.kind == kind)
}

pub fn stable_key_candidate_for_persisted_kind(
    kind: PersistedToolSurfaceKindV2,
) -> Option<&'static LegacyToolSurfaceStableKeyCandidate> {
    SAVEABLE_TOOL_SURFACE_STABLE_KEY_CANDIDATES
        .iter()
        .find(|candidate| candidate.persisted_kind == kind)
}

pub fn stable_key_candidate_for_key(
    stable_key: &str,
) -> Option<&'static LegacyToolSurfaceStableKeyCandidate> {
    SAVEABLE_TOOL_SURFACE_STABLE_KEY_CANDIDATES
        .iter()
        .find(|candidate| candidate.stable_key == stable_key)
}

pub fn resolve_legacy_tool_surface_kind<'a>(
    registry: &'a ToolSurfaceRegistry,
    kind: ToolSurfaceKind,
) -> LegacyToolSurfaceResolution<'a> {
    let Some(key) = stable_key_for_tool_surface_kind(kind) else {
        return LegacyToolSurfaceResolution::UnmappedLegacySurface { kind };
    };

    match registry.get(&key) {
        Some(definition) => LegacyToolSurfaceResolution::Resolved { key, definition },
        None => LegacyToolSurfaceResolution::UnregisteredLegacySurface { key },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyToolSurfaceResolution<'a> {
    Resolved {
        key: ToolSurfaceStableKey,
        definition: &'a ToolSurfaceDefinition,
    },
    UnregisteredLegacySurface {
        key: ToolSurfaceStableKey,
    },
    UnmappedLegacySurface {
        kind: ToolSurfaceKind,
    },
}

fn stable_key(value: &str) -> ToolSurfaceStableKey {
    ToolSurfaceStableKey::new(value).expect("hard-coded stable key should be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EditorToolSuite, ProviderFamilyDefinition, ProviderFamilyId, ToolSuiteId,
        ToolSuiteRegistry, ToolSurfacePersistence, ToolSurfaceRole, ToolSurfaceRoute,
    };

    #[test]
    fn legacy_tool_surface_kind_maps_material_lab_to_stable_keys() {
        assert_eq!(
            stable_key_for_tool_surface_kind(ToolSurfaceKind::MaterialGraphCanvas)
                .expect("material graph key")
                .as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
        assert_eq!(
            stable_key_for_tool_surface_kind(ToolSurfaceKind::MaterialInspector)
                .expect("material inspector key")
                .as_str(),
            "runenwerk.material_lab.inspector"
        );
        assert_eq!(
            stable_key_for_tool_surface_kind(ToolSurfaceKind::MaterialPreview)
                .expect("material preview key")
                .as_str(),
            "runenwerk.material_lab.preview"
        );
    }

    #[test]
    fn legacy_tool_surface_kind_maps_current_fixed_layout_to_stable_keys() {
        assert_eq!(
            stable_key_for_tool_surface_kind(ToolSurfaceKind::Outliner)
                .expect("outliner key")
                .as_str(),
            "runenwerk.scene.outliner"
        );
        assert_eq!(
            stable_key_for_tool_surface_kind(ToolSurfaceKind::EntityTable)
                .expect("entity table key")
                .as_str(),
            "runenwerk.scene.entity_table"
        );
        assert_eq!(
            stable_key_for_tool_surface_kind(ToolSurfaceKind::Viewport)
                .expect("viewport key")
                .as_str(),
            "runenwerk.scene.viewport"
        );
        assert_eq!(
            stable_key_for_tool_surface_kind(ToolSurfaceKind::Inspector)
                .expect("inspector key")
                .as_str(),
            "runenwerk.scene.inspector"
        );
        assert_eq!(
            stable_key_for_tool_surface_kind(ToolSurfaceKind::Console)
                .expect("console key")
                .as_str(),
            "runenwerk.editor.console"
        );
    }

    #[test]
    fn persisted_tool_surface_kind_v2_maps_material_lab_to_stable_keys() {
        assert_eq!(
            stable_key_for_persisted_tool_surface_kind_v2(
                PersistedToolSurfaceKindV2::MaterialGraphCanvas
            )
            .expect("material graph key")
            .as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
        assert_eq!(
            stable_key_for_persisted_tool_surface_kind_v2(
                PersistedToolSurfaceKindV2::MaterialInspector
            )
            .expect("material inspector key")
            .as_str(),
            "runenwerk.material_lab.inspector"
        );
        assert_eq!(
            stable_key_for_persisted_tool_surface_kind_v2(
                PersistedToolSurfaceKindV2::MaterialPreview
            )
            .expect("material preview key")
            .as_str(),
            "runenwerk.material_lab.preview"
        );
    }

    #[test]
    fn stable_keys_map_back_to_material_lab_legacy_kinds() {
        assert_eq!(
            tool_surface_kind_for_stable_key(
                &ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap()
            ),
            Some(ToolSurfaceKind::MaterialGraphCanvas)
        );
        assert_eq!(
            tool_surface_kind_for_stable_key(
                &ToolSurfaceStableKey::new("runenwerk.material_lab.inspector").unwrap()
            ),
            Some(ToolSurfaceKind::MaterialInspector)
        );
        assert_eq!(
            tool_surface_kind_for_stable_key(
                &ToolSurfaceStableKey::new("runenwerk.material_lab.preview").unwrap()
            ),
            Some(ToolSurfaceKind::MaterialPreview)
        );
        assert_eq!(
            tool_surface_kind_for_stable_key(
                &ToolSurfaceStableKey::new("runenwerk.procgen.graph_canvas").unwrap()
            ),
            Some(ToolSurfaceKind::ProcgenGraphCanvas)
        );
    }

    #[test]
    fn every_saveable_tool_surface_kind_has_stable_key_candidate() {
        for candidate in saveable_tool_surface_stable_key_candidates() {
            assert_eq!(
                stable_key_for_tool_surface_kind(candidate.kind)
                    .expect("saveable kind should map")
                    .as_str(),
                candidate.stable_key
            );
            assert_eq!(
                stable_key_for_persisted_tool_surface_kind_v2(candidate.persisted_kind)
                    .expect("saveable persisted kind should map")
                    .as_str(),
                candidate.stable_key
            );
            assert_eq!(
                tool_surface_kind_for_stable_key(
                    &ToolSurfaceStableKey::new(candidate.stable_key).unwrap()
                ),
                Some(candidate.kind)
            );
        }
    }

    #[test]
    fn known_legacy_candidate_without_registered_surface_is_unregistered_not_unknown() {
        let registry = ToolSuiteRegistry::new(Vec::new()).expect("empty registry should be valid");
        let key = stable_key("runenwerk.material_lab.graph_canvas");

        assert_eq!(
            registry.surfaces().resolve(&key),
            crate::ToolSurfaceResolution::UnknownKey { key: key.clone() }
        );
        assert_eq!(
            resolve_legacy_tool_surface_kind(
                registry.surfaces(),
                ToolSurfaceKind::MaterialGraphCanvas
            ),
            LegacyToolSurfaceResolution::UnregisteredLegacySurface { key }
        );
    }

    #[test]
    fn placeholder_uses_explicit_fallback_stable_key() {
        let registry = ToolSuiteRegistry::new(Vec::new()).expect("empty registry should be valid");
        let key = stable_key("runenwerk.diagnostics.placeholder");

        assert_eq!(
            stable_key_for_tool_surface_kind(ToolSurfaceKind::Placeholder),
            Some(key.clone())
        );
        assert_eq!(
            resolve_legacy_tool_surface_kind(registry.surfaces(), ToolSurfaceKind::Placeholder),
            LegacyToolSurfaceResolution::UnregisteredLegacySurface { key }
        );
    }

    #[test]
    fn registered_legacy_candidate_resolves_to_definition() {
        let registry = ToolSuiteRegistry::new(vec![material_lab_suite_with_graph_canvas()])
            .expect("material lab suite should be valid");

        let resolution = resolve_legacy_tool_surface_kind(
            registry.surfaces(),
            ToolSurfaceKind::MaterialGraphCanvas,
        );

        match resolution {
            LegacyToolSurfaceResolution::Resolved { key, definition } => {
                assert_eq!(key.as_str(), "runenwerk.material_lab.graph_canvas");
                assert_eq!(definition.label, "Material Graph");
            }
            other => panic!("expected resolved legacy surface, got {other:?}"),
        }
    }

    fn material_lab_suite_with_graph_canvas() -> EditorToolSuite {
        let provider_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        EditorToolSuite {
            suite_id: ToolSuiteId::new("runenwerk.material_lab").unwrap(),
            label: "Material Lab".to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family.clone(),
                label: "Material Lab".to_string(),
            }],
            surfaces: vec![ToolSurfaceDefinition {
                key: stable_key("runenwerk.material_lab.graph_canvas"),
                label: "Material Graph".to_string(),
                role: ToolSurfaceRole::Primary,
                panel_kind: crate::PanelKind::MaterialGraphCanvas,
                provider_family,
                route: ToolSurfaceRoute::ProviderOwnedGraphCanvas,
                persistence: ToolSurfacePersistence::StableKey,
            }],
        }
    }
}
