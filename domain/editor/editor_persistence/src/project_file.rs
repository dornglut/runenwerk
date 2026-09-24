use serde::{Deserialize, Serialize};

pub const PROJECT_FILE_VERSION_V1: u32 = 1;
pub const PROJECT_FILE_VERSION_V2: u32 = 2;
pub const PROJECT_FILE_VERSION_V3: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSceneEntryV1 {
    pub scene_id: String,
    pub display_name: String,
    pub file_path: String,
}

impl ProjectSceneEntryV1 {
    pub fn new(
        scene_id: impl Into<String>,
        display_name: impl Into<String>,
        file_path: impl Into<String>,
    ) -> Self {
        Self {
            scene_id: scene_id.into(),
            display_name: display_name.into(),
            file_path: file_path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFileV1 {
    pub version: u32,
    pub project_name: String,
    pub startup_scene_id: Option<String>,
    pub scenes: Vec<ProjectSceneEntryV1>,
}

impl ProjectFileV1 {
    pub fn new(
        project_name: impl Into<String>,
        startup_scene_id: Option<String>,
        scenes: Vec<ProjectSceneEntryV1>,
    ) -> Self {
        Self {
            version: PROJECT_FILE_VERSION_V1,
            project_name: project_name.into(),
            startup_scene_id,
            scenes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAssetSourceRootV2 {
    pub root_id: asset::AssetSourceRootId,
    pub kind: asset::AssetSourceRootKind,
    pub display_name: String,
    pub relative_path: String,
}

impl ProjectAssetSourceRootV2 {
    pub fn new(
        root_id: asset::AssetSourceRootId,
        kind: asset::AssetSourceRootKind,
        display_name: impl Into<String>,
        relative_path: impl Into<String>,
    ) -> Self {
        Self {
            root_id,
            kind,
            display_name: display_name.into(),
            relative_path: relative_path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAssetEntryV2 {
    pub asset_id: asset::AssetId,
    pub source_id: asset::AssetSourceId,
    pub kind: asset::AssetKind,
    pub stable_name: String,
    pub display_name: String,
    pub source_path: String,
}

impl ProjectAssetEntryV2 {
    pub fn new(
        asset_id: asset::AssetId,
        source_id: asset::AssetSourceId,
        kind: asset::AssetKind,
        stable_name: impl Into<String>,
        display_name: impl Into<String>,
        source_path: impl Into<String>,
    ) -> Self {
        Self {
            asset_id,
            source_id,
            kind,
            stable_name: stable_name.into(),
            display_name: display_name.into(),
            source_path: source_path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenDocumentRestorationPolicyV2 {
    RestoreLastSession,
    StartupDocumentOnly,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectImportProfileDefaultV2 {
    pub asset_kind: asset::AssetKind,
    pub profile_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFileV2 {
    pub version: u32,
    pub project_id: String,
    pub project_name: String,
    pub asset_source_roots: Vec<ProjectAssetSourceRootV2>,
    pub artifact_cache_root: String,
    pub field_product_cache_root: String,
    pub catalog_file_path: String,
    pub startup_document_id: Option<String>,
    pub default_workspace_profile_id: Option<u64>,
    pub open_document_restoration_policy: OpenDocumentRestorationPolicyV2,
    pub import_profile_defaults: Vec<ProjectImportProfileDefaultV2>,
    pub compatibility_version: u32,
    pub migrated_assets: Vec<ProjectAssetEntryV2>,
}

impl ProjectFileV2 {
    pub fn new(project_id: impl Into<String>, project_name: impl Into<String>) -> Self {
        Self {
            version: PROJECT_FILE_VERSION_V2,
            project_id: project_id.into(),
            project_name: project_name.into(),
            asset_source_roots: vec![
                ProjectAssetSourceRootV2::new(
                    asset::asset_source_root_id(1),
                    asset::AssetSourceRootKind::ProjectAssets,
                    "Project assets",
                    "assets",
                ),
                ProjectAssetSourceRootV2::new(
                    asset::asset_source_root_id(2),
                    asset::AssetSourceRootKind::GameAssets,
                    "Game assets",
                    "game/assets",
                ),
            ],
            artifact_cache_root: ".runenwerk/artifacts".to_string(),
            field_product_cache_root: ".runenwerk/field-products".to_string(),
            catalog_file_path: "assets/catalog.ron".to_string(),
            startup_document_id: None,
            default_workspace_profile_id: Some(1),
            open_document_restoration_policy: OpenDocumentRestorationPolicyV2::RestoreLastSession,
            import_profile_defaults: Vec::new(),
            compatibility_version: 1,
            migrated_assets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectImportProfileDefaultV3 {
    pub asset_kind: asset::AssetKind,
    pub profile_name: String,
}

impl ProjectImportProfileDefaultV3 {
    pub fn new(asset_kind: asset::AssetKind, profile_name: impl Into<String>) -> Self {
        Self {
            asset_kind,
            profile_name: profile_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectImportProfileDefinitionV3 {
    pub asset_kind: asset::AssetKind,
    pub profile_name: String,
    pub settings: asset::ImportSettings,
    pub expected_artifact_kind: asset::AssetKind,
}

impl ProjectImportProfileDefinitionV3 {
    pub fn new(
        asset_kind: asset::AssetKind,
        profile_name: impl Into<String>,
        settings: asset::ImportSettings,
        expected_artifact_kind: asset::AssetKind,
    ) -> Self {
        Self {
            asset_kind,
            profile_name: profile_name.into(),
            settings,
            expected_artifact_kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFileV3 {
    pub version: u32,
    pub project_id: String,
    pub project_name: String,
    pub asset_source_roots: Vec<ProjectAssetSourceRootV2>,
    pub artifact_cache_root: String,
    pub field_product_cache_root: String,
    pub catalog_file_path: String,
    pub startup_document_id: Option<String>,
    pub default_workspace_profile_id: Option<u64>,
    pub open_document_restoration_policy: OpenDocumentRestorationPolicyV2,
    pub import_profile_definitions: Vec<ProjectImportProfileDefinitionV3>,
    pub import_profile_defaults: Vec<ProjectImportProfileDefaultV3>,
    pub compatibility_version: u32,
    pub migrated_assets: Vec<ProjectAssetEntryV2>,
}

impl ProjectFileV3 {
    pub fn new(project_id: impl Into<String>, project_name: impl Into<String>) -> Self {
        let v2 = ProjectFileV2::new(project_id, project_name);
        migrate_project_file_v2_to_v3(v2)
    }
}

pub fn migrate_project_file_v1_to_v2(project: ProjectFileV1) -> ProjectFileV2 {
    let mut migrated = ProjectFileV2::new(
        format!("project.{}", stable_project_token(&project.project_name)),
        project.project_name,
    );
    migrated.startup_document_id = project
        .startup_scene_id
        .as_deref()
        .map(|scene_id| format!("scene:{scene_id}"));
    migrated.migrated_assets = project
        .scenes
        .into_iter()
        .enumerate()
        .map(|(index, scene)| {
            let raw = index as u64 + 1;
            ProjectAssetEntryV2::new(
                asset::asset_id(raw),
                asset::asset_source_id(raw),
                asset::AssetKind::Scene,
                scene.scene_id,
                scene.display_name,
                scene.file_path,
            )
        })
        .collect();
    migrated
}

pub fn migrate_project_file_v2_to_v3(project: ProjectFileV2) -> ProjectFileV3 {
    ProjectFileV3 {
        version: PROJECT_FILE_VERSION_V3,
        project_id: project.project_id,
        project_name: project.project_name,
        asset_source_roots: project.asset_source_roots,
        artifact_cache_root: project.artifact_cache_root,
        field_product_cache_root: project.field_product_cache_root,
        catalog_file_path: project.catalog_file_path,
        startup_document_id: project.startup_document_id,
        default_workspace_profile_id: project.default_workspace_profile_id,
        open_document_restoration_policy: project.open_document_restoration_policy,
        import_profile_definitions: Vec::new(),
        import_profile_defaults: project
            .import_profile_defaults
            .into_iter()
            .map(|default| {
                ProjectImportProfileDefaultV3::new(default.asset_kind, default.profile_name)
            })
            .collect(),
        compatibility_version: project.compatibility_version,
        migrated_assets: project.migrated_assets,
    }
}

pub fn migrate_project_file_v1_to_v3(project: ProjectFileV1) -> ProjectFileV3 {
    migrate_project_file_v2_to_v3(migrate_project_file_v1_to_v2(project))
}

fn stable_project_token(project_name: &str) -> String {
    let token = project_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let token = token.trim_matches('-').to_string();
    if token.is_empty() {
        "untitled".to_string()
    } else {
        token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_migration_preserves_scene_startup_as_document_id() {
        let v1 = ProjectFileV1::new(
            "Demo Project",
            Some("hub_stub".to_string()),
            vec![ProjectSceneEntryV1::new(
                "hub_stub",
                "Hub",
                "assets/scenes/hub_stub.ron",
            )],
        );

        let v2 = migrate_project_file_v1_to_v2(v1);

        assert_eq!(v2.version, PROJECT_FILE_VERSION_V2);
        assert_eq!(v2.startup_document_id.as_deref(), Some("scene:hub_stub"));
        assert_eq!(v2.migrated_assets.len(), 1);
        assert_eq!(v2.migrated_assets[0].kind, asset::AssetKind::Scene);
    }

    #[test]
    fn v1_migration_to_v3_preserves_scene_startup_and_uses_v3_version() {
        let v1 = ProjectFileV1::new(
            "Demo Project",
            Some("hub_stub".to_string()),
            vec![ProjectSceneEntryV1::new(
                "hub_stub",
                "Hub",
                "assets/scenes/hub_stub.ron",
            )],
        );

        let v3 = migrate_project_file_v1_to_v3(v1);

        assert_eq!(v3.version, PROJECT_FILE_VERSION_V3);
        assert_eq!(v3.startup_document_id.as_deref(), Some("scene:hub_stub"));
        assert_eq!(v3.migrated_assets.len(), 1);
        assert!(v3.import_profile_definitions.is_empty());
    }

    #[test]
    fn v2_migration_to_v3_preserves_profile_defaults_as_selection_policy() {
        let mut v2 = ProjectFileV2::new("project.demo", "Demo");
        v2.import_profile_defaults
            .push(ProjectImportProfileDefaultV2 {
                asset_kind: asset::AssetKind::SdfGraph,
                profile_name: "default".to_string(),
            });

        let v3 = migrate_project_file_v2_to_v3(v2);

        assert_eq!(v3.version, PROJECT_FILE_VERSION_V3);
        assert_eq!(v3.import_profile_defaults.len(), 1);
        assert_eq!(
            v3.import_profile_defaults[0].asset_kind,
            asset::AssetKind::SdfGraph
        );
        assert_eq!(v3.import_profile_defaults[0].profile_name, "default");
        assert!(v3.import_profile_definitions.is_empty());
    }

    #[test]
    fn project_file_v3_preserves_profile_definitions_and_defaults() {
        let mut v3 = ProjectFileV3::new("project.demo", "Demo");
        v3.import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                asset::AssetKind::Texture2D,
                "ui",
                asset::ImportSettings::Texture2D {
                    color_space: asset::TextureImportColorSpace::Srgb,
                    compression: asset::TextureImportCompression::Bc7,
                },
                asset::AssetKind::Texture2D,
            ));
        v3.import_profile_defaults
            .push(ProjectImportProfileDefaultV3::new(
                asset::AssetKind::Texture2D,
                "ui",
            ));

        assert_eq!(v3.version, PROJECT_FILE_VERSION_V3);
        assert_eq!(v3.import_profile_definitions[0].profile_name, "ui");
        assert_eq!(v3.import_profile_defaults[0].profile_name, "ui");
    }
}
