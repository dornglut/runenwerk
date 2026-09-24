use std::collections::{BTreeMap, BTreeSet};

use asset::{
    AssetDiagnosticCode, AssetDiagnosticRecord, AssetKind, AssetSourceDescriptor,
    FieldProductResolution, ImportSettings, TextureImportColorSpace, TextureImportCompression,
    TextureProductResolution,
};
use editor_persistence::{ProjectFileV3, ProjectImportProfileDefinitionV3};

pub const DEFAULT_IMPORT_PROFILE_NAME: &str = "default";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EditorImportProfileKey {
    pub asset_kind: AssetKind,
    pub profile_name: String,
}

impl EditorImportProfileKey {
    pub fn new(asset_kind: AssetKind, profile_name: impl Into<String>) -> Self {
        Self {
            asset_kind,
            profile_name: profile_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorImportRecipe {
    pub key: EditorImportProfileKey,
    pub settings: ImportSettings,
    pub expected_artifact_kind: AssetKind,
}

impl EditorImportRecipe {
    pub fn new(
        asset_kind: AssetKind,
        profile_name: impl Into<String>,
        settings: ImportSettings,
        expected_artifact_kind: AssetKind,
    ) -> Self {
        Self {
            key: EditorImportProfileKey::new(asset_kind, profile_name),
            settings,
            expected_artifact_kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditorImportProfileRegistry {
    recipes: BTreeMap<EditorImportProfileKey, EditorImportRecipe>,
    defaults: BTreeMap<AssetKind, String>,
    diagnostics: Vec<AssetDiagnosticRecord>,
}

impl Default for EditorImportProfileRegistry {
    fn default() -> Self {
        Self::built_in()
    }
}

impl EditorImportProfileRegistry {
    pub fn built_in() -> Self {
        let mut registry = Self {
            recipes: BTreeMap::new(),
            defaults: BTreeMap::new(),
            diagnostics: Vec::new(),
        };
        for recipe in built_in_default_recipes() {
            registry
                .defaults
                .insert(recipe.key.asset_kind, recipe.key.profile_name.clone());
            registry.recipes.insert(recipe.key.clone(), recipe);
        }
        registry
    }

    pub fn from_project_file(project: &ProjectFileV3) -> Self {
        let mut registry = Self::built_in();
        registry.extend_project_definitions(&project.import_profile_definitions);
        registry.extend_project_defaults(project);
        registry
    }

    pub fn diagnostics(&self) -> &[AssetDiagnosticRecord] {
        &self.diagnostics
    }

    pub fn recipe(&self, key: &EditorImportProfileKey) -> Option<&EditorImportRecipe> {
        self.recipes.get(key)
    }

    pub fn resolve_for_source(
        &self,
        source: &AssetSourceDescriptor,
    ) -> Result<EditorImportRecipe, Vec<AssetDiagnosticRecord>> {
        if !self.diagnostics.is_empty() {
            return Err(self.diagnostics.clone());
        }
        let Some(profile_name) = self.defaults.get(&source.kind) else {
            return Err(vec![import_profile_diagnostic(format!(
                "no import profile default is registered for source kind {:?}",
                source.kind
            ))]);
        };
        let key = EditorImportProfileKey::new(source.kind, profile_name.clone());
        let Some(recipe) = self.recipes.get(&key) else {
            return Err(vec![import_profile_diagnostic(format!(
                "import profile default {:?}/{} has no recipe",
                source.kind, profile_name
            ))]);
        };
        if !recipe.settings.supports_source_kind(source.kind) {
            return Err(vec![import_profile_diagnostic(format!(
                "import profile {:?}/{} settings do not support source kind {:?}",
                source.kind, profile_name, source.kind
            ))]);
        }
        if !recipe
            .settings
            .supports_artifact_kind(recipe.expected_artifact_kind)
        {
            return Err(vec![import_profile_diagnostic(format!(
                "import profile {:?}/{} settings do not support artifact kind {:?}",
                source.kind, profile_name, recipe.expected_artifact_kind
            ))]);
        }
        Ok(recipe.clone())
    }

    fn extend_project_definitions(&mut self, definitions: &[ProjectImportProfileDefinitionV3]) {
        let mut seen = BTreeSet::new();
        for definition in definitions {
            let key =
                EditorImportProfileKey::new(definition.asset_kind, definition.profile_name.clone());
            if !seen.insert(key.clone()) {
                self.diagnostics.push(import_profile_diagnostic(format!(
                    "duplicate project import profile {:?}/{}",
                    definition.asset_kind, definition.profile_name
                )));
                continue;
            }
            if definition.profile_name.trim().is_empty() {
                self.diagnostics.push(import_profile_diagnostic(format!(
                    "project import profile for {:?} has an empty name",
                    definition.asset_kind
                )));
                continue;
            }
            if !definition
                .settings
                .supports_source_kind(definition.asset_kind)
            {
                self.diagnostics.push(import_profile_diagnostic(format!(
                    "project import profile {:?}/{} settings do not support source kind {:?}",
                    definition.asset_kind, definition.profile_name, definition.asset_kind
                )));
                continue;
            }
            if !definition
                .settings
                .supports_artifact_kind(definition.expected_artifact_kind)
            {
                self.diagnostics.push(import_profile_diagnostic(format!(
                    "project import profile {:?}/{} settings do not support artifact kind {:?}",
                    definition.asset_kind,
                    definition.profile_name,
                    definition.expected_artifact_kind
                )));
                continue;
            }
            self.recipes.insert(
                key,
                EditorImportRecipe::new(
                    definition.asset_kind,
                    definition.profile_name.clone(),
                    definition.settings.clone(),
                    definition.expected_artifact_kind,
                ),
            );
        }
    }

    fn extend_project_defaults(&mut self, project: &ProjectFileV3) {
        let mut seen = BTreeSet::new();
        for default in &project.import_profile_defaults {
            if !seen.insert(default.asset_kind) {
                self.diagnostics.push(import_profile_diagnostic(format!(
                    "duplicate project import profile default for {:?}",
                    default.asset_kind
                )));
                continue;
            }
            if default.profile_name.trim().is_empty() {
                self.diagnostics.push(import_profile_diagnostic(format!(
                    "project import profile default for {:?} has an empty name",
                    default.asset_kind
                )));
                continue;
            }
            let key = EditorImportProfileKey::new(default.asset_kind, default.profile_name.clone());
            if !self.recipes.contains_key(&key) {
                self.diagnostics.push(import_profile_diagnostic(format!(
                    "project import profile default {:?}/{} has no recipe",
                    default.asset_kind, default.profile_name
                )));
                continue;
            }
            self.defaults
                .insert(default.asset_kind, default.profile_name.clone());
        }
    }
}

pub fn import_profile_diagnostic(message: impl Into<String>) -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::error(AssetDiagnosticCode::ImportProfileRejected, message)
}

fn built_in_default_recipes() -> Vec<EditorImportRecipe> {
    vec![
        EditorImportRecipe::new(
            AssetKind::SdfGraph,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::SdfGraph {
                resolution: FieldProductResolution::new(64, 64, 1),
            },
            AssetKind::FormedFieldProduct,
        ),
        EditorImportRecipe::new(
            AssetKind::SdfBrushLayer,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::SdfBrushLayer {
                resolution: FieldProductResolution::new(64, 64, 1),
            },
            AssetKind::FormedFieldProduct,
        ),
        EditorImportRecipe::new(
            AssetKind::FieldWorldDefinition,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::FieldWorldDefinition {
                resolution: FieldProductResolution::new(64, 64, 1),
            },
            AssetKind::FormedFieldProduct,
        ),
        EditorImportRecipe::new(
            AssetKind::MaterialGraph,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::MaterialGraph {
                lowering_target: "preview".to_string(),
            },
            AssetKind::Material,
        ),
        EditorImportRecipe::new(
            AssetKind::Material,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::Material {
                product_target: "preview".to_string(),
            },
            AssetKind::Material,
        ),
        EditorImportRecipe::new(
            AssetKind::Prefab,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::Prefab {
                descriptor_profile: "sdf-prefab-v2".to_string(),
            },
            AssetKind::Prefab,
        ),
        EditorImportRecipe::new(
            AssetKind::ProceduralTexture,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::ProceduralTexture {
                resolution: TextureProductResolution::new(512, 512, 1),
                color_space: TextureImportColorSpace::Srgb,
            },
            AssetKind::ProceduralTexture,
        ),
        EditorImportRecipe::new(
            AssetKind::Texture2D,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::Texture2D {
                color_space: TextureImportColorSpace::Srgb,
                compression: TextureImportCompression::Uncompressed,
            },
            AssetKind::Texture2D,
        ),
        EditorImportRecipe::new(
            AssetKind::Texture3DVolume,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::Texture3DVolume {
                resolution: TextureProductResolution::new(64, 64, 64),
                color_space: TextureImportColorSpace::Data,
                compression: TextureImportCompression::Uncompressed,
            },
            AssetKind::Texture3DVolume,
        ),
        EditorImportRecipe::new(
            AssetKind::ForeignMeshReferenceSource,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::ForeignBlend {
                blender_executable: None,
                export_format: "glb".to_string(),
            },
            AssetKind::ForeignMeshReferenceArtifact,
        ),
        EditorImportRecipe::new(
            AssetKind::Scene,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::Scene,
            AssetKind::Scene,
        ),
        EditorImportRecipe::new(
            AssetKind::Shader,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::Shader,
            AssetKind::Shader,
        ),
        EditorImportRecipe::new(
            AssetKind::UiDefinition,
            DEFAULT_IMPORT_PROFILE_NAME,
            ImportSettings::UiDefinition,
            AssetKind::UiDefinition,
        ),
        raw_ron_recipe(AssetKind::Graph),
        raw_ron_recipe(AssetKind::Theme),
        raw_ron_recipe(AssetKind::Menu),
        raw_ron_recipe(AssetKind::Shortcut),
        raw_ron_recipe(AssetKind::WorkspaceDefinition),
        raw_ron_recipe(AssetKind::EditorDefinition),
    ]
}

fn raw_ron_recipe(asset_kind: AssetKind) -> EditorImportRecipe {
    EditorImportRecipe::new(
        asset_kind,
        DEFAULT_IMPORT_PROFILE_NAME,
        ImportSettings::RawRon {
            schema_hint: Some(asset_kind_label(asset_kind).to_string()),
        },
        asset_kind,
    )
}

fn asset_kind_label(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Scene => "scene",
        AssetKind::Prefab => "prefab",
        AssetKind::SdfGraph => "sdf_graph",
        AssetKind::SdfBrushLayer => "sdf_brush_layer",
        AssetKind::FieldWorldDefinition => "field_world_definition",
        AssetKind::WorldEditLog => "world_edit_log",
        AssetKind::FieldMaterialChannelSet => "field_material_channel_set",
        AssetKind::FormedFieldProduct => "formed_field_product",
        AssetKind::WorldSdfChunkPageArtifact => "world_sdf_chunk_page_artifact",
        AssetKind::ClipmapBrickmapProduct => "clipmap_brickmap_product",
        AssetKind::MaterialGraph => "material_graph",
        AssetKind::Material => "material",
        AssetKind::ProceduralMaterial => "procedural_material",
        AssetKind::ProceduralTexture => "procedural_texture",
        AssetKind::Texture2D => "texture_2d",
        AssetKind::Texture3DVolume => "texture_3d_volume",
        AssetKind::GameplayGraph => "gameplay_graph",
        AssetKind::GameplayRuleTrigger => "gameplay_rule_trigger",
        AssetKind::GameplayAbility => "gameplay_ability",
        AssetKind::GameplayQuest => "gameplay_quest",
        AssetKind::GameplayAtrIrProduct => "gameplay_atr_ir_product",
        AssetKind::GameplayEcsLoweringProduct => "gameplay_ecs_lowering_product",
        AssetKind::ParticleGraph => "particle_graph",
        AssetKind::ParticleEmitter => "particle_emitter",
        AssetKind::PhysicsConfig => "physics_config",
        AssetKind::AnimationClip => "animation_clip",
        AssetKind::AnimationGraph => "animation_graph",
        AssetKind::ProcgenGraph => "procgen_graph",
        AssetKind::UiLayout => "ui_layout",
        AssetKind::UiDefinition => "ui_definition",
        AssetKind::Graph => "graph",
        AssetKind::Script => "script",
        AssetKind::Shader => "shader",
        AssetKind::Theme => "theme",
        AssetKind::Menu => "menu",
        AssetKind::Shortcut => "shortcut",
        AssetKind::WorkspaceDefinition => "workspace_definition",
        AssetKind::EditorDefinition => "editor_definition",
        AssetKind::DiagnosticsCapture => "diagnostics_capture",
        AssetKind::ForeignMeshReferenceSource => "foreign_mesh_reference_source",
        AssetKind::ForeignMeshReferenceArtifact => "foreign_mesh_reference_artifact",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{asset_id, asset_source_id};
    use editor_persistence::{ProjectImportProfileDefaultV3, ProjectImportProfileDefinitionV3};

    #[test]
    fn built_in_default_resolves_for_empty_project() {
        let project = ProjectFileV3::new("project.test", "Test");
        let registry = EditorImportProfileRegistry::from_project_file(&project);
        let source = AssetSourceDescriptor::new(
            asset_source_id(1),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/field.ron",
        );

        let recipe = registry
            .resolve_for_source(&source)
            .expect("built-in default should resolve");

        assert_eq!(recipe.key.profile_name, DEFAULT_IMPORT_PROFILE_NAME);
        assert_eq!(recipe.expected_artifact_kind, AssetKind::FormedFieldProduct);
    }

    #[test]
    fn project_profile_definition_overrides_built_in_by_exact_key() {
        let mut project = ProjectFileV3::new("project.test", "Test");
        project
            .import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                AssetKind::SdfGraph,
                DEFAULT_IMPORT_PROFILE_NAME,
                ImportSettings::SdfGraph {
                    resolution: FieldProductResolution::new(128, 128, 1),
                },
                AssetKind::FormedFieldProduct,
            ));
        let registry = EditorImportProfileRegistry::from_project_file(&project);
        let source = AssetSourceDescriptor::new(
            asset_source_id(1),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/field.ron",
        );

        let recipe = registry
            .resolve_for_source(&source)
            .expect("project override should resolve");

        assert_eq!(
            recipe.settings,
            ImportSettings::SdfGraph {
                resolution: FieldProductResolution::new(128, 128, 1),
            }
        );
    }

    #[test]
    fn missing_project_default_recipe_blocks_resolution() {
        let mut project = ProjectFileV3::new("project.test", "Test");
        project
            .import_profile_defaults
            .push(ProjectImportProfileDefaultV3::new(
                AssetKind::SdfGraph,
                "missing",
            ));
        let registry = EditorImportProfileRegistry::from_project_file(&project);
        let source = AssetSourceDescriptor::new(
            asset_source_id(1),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/field.ron",
        );

        let diagnostics = registry
            .resolve_for_source(&source)
            .expect_err("missing profile should be rejected");

        assert_eq!(
            diagnostics[0].code,
            AssetDiagnosticCode::ImportProfileRejected
        );
    }

    #[test]
    fn incompatible_project_profile_definition_is_diagnostic() {
        let mut project = ProjectFileV3::new("project.test", "Test");
        project
            .import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                AssetKind::SdfGraph,
                "bad",
                ImportSettings::Texture2D {
                    color_space: TextureImportColorSpace::Srgb,
                    compression: TextureImportCompression::Uncompressed,
                },
                AssetKind::Texture2D,
            ));

        let registry = EditorImportProfileRegistry::from_project_file(&project);

        assert_eq!(
            registry.diagnostics()[0].code,
            AssetDiagnosticCode::ImportProfileRejected
        );
    }
}
