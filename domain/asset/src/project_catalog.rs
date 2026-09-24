use serde::{Deserialize, Serialize};

use crate::{AssetKind, AssetSourceRoot, ImportSettings};

pub const ASSET_PROJECT_CATALOG_DESCRIPTOR_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetImportProfileDefault {
    pub asset_kind: AssetKind,
    pub profile_name: String,
    pub settings: ImportSettings,
}

impl AssetImportProfileDefault {
    pub fn new(
        asset_kind: AssetKind,
        profile_name: impl Into<String>,
        settings: ImportSettings,
    ) -> Self {
        Self {
            asset_kind,
            profile_name: profile_name.into(),
            settings,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetProjectCatalogDescriptor {
    pub version: u32,
    pub source_roots: Vec<AssetSourceRoot>,
    pub artifact_cache_root: String,
    pub field_product_cache_root: String,
    pub catalog_file_path: String,
    pub import_profile_defaults: Vec<AssetImportProfileDefault>,
}

impl AssetProjectCatalogDescriptor {
    pub fn new(
        source_roots: impl IntoIterator<Item = AssetSourceRoot>,
        artifact_cache_root: impl Into<String>,
        field_product_cache_root: impl Into<String>,
        catalog_file_path: impl Into<String>,
    ) -> Self {
        Self {
            version: ASSET_PROJECT_CATALOG_DESCRIPTOR_VERSION_V1,
            source_roots: source_roots.into_iter().collect(),
            artifact_cache_root: artifact_cache_root.into(),
            field_product_cache_root: field_product_cache_root.into(),
            catalog_file_path: catalog_file_path.into(),
            import_profile_defaults: Vec::new(),
        }
    }

    pub fn with_import_profile_default(mut self, default: AssetImportProfileDefault) -> Self {
        self.import_profile_defaults.push(default);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetSourceRootKind, asset_source_root_id};

    #[test]
    fn project_catalog_descriptor_preserves_source_roots_and_import_profiles() {
        let descriptor = AssetProjectCatalogDescriptor::new(
            [AssetSourceRoot::new(
                asset_source_root_id(1),
                AssetSourceRootKind::ProjectAssets,
                "Project assets",
                "assets",
            )],
            ".runenwerk/artifacts",
            ".runenwerk/field-products",
            "assets/catalog.ron",
        )
        .with_import_profile_default(AssetImportProfileDefault::new(
            AssetKind::Prefab,
            "default-prefab",
            ImportSettings::Prefab {
                descriptor_profile: "sdf-prefab-v2".to_string(),
            },
        ));

        assert_eq!(
            descriptor.version,
            ASSET_PROJECT_CATALOG_DESCRIPTOR_VERSION_V1
        );
        assert_eq!(descriptor.source_roots.len(), 1);
        assert_eq!(
            descriptor.import_profile_defaults[0].asset_kind,
            AssetKind::Prefab
        );
    }
}
