use std::path::{Path, PathBuf};

use asset::{
    AssetDiagnosticCode, AssetDiagnosticRecord, AssetImportProfileDefault,
    AssetProjectCatalogDescriptor, AssetSourceRoot, ratify_asset_project_catalog_descriptor,
};
use editor_persistence::ProjectFileV3;

use crate::asset_pipeline::{EditorImportExecutionLedger, EditorImportProfileRegistry};

#[derive(Debug, Clone)]
pub struct EditorAssetProjectSession {
    project_root: PathBuf,
    descriptor: AssetProjectCatalogDescriptor,
    import_profile_registry: EditorImportProfileRegistry,
    import_ledger: EditorImportExecutionLedger,
    catalog_load_status: Option<String>,
    catalog_save_status: Option<String>,
    import_status: Option<String>,
}

impl EditorAssetProjectSession {
    pub fn new(
        project_root: impl Into<PathBuf>,
        descriptor: AssetProjectCatalogDescriptor,
    ) -> Self {
        Self::new_with_import_profile_registry(
            project_root,
            descriptor,
            EditorImportProfileRegistry::built_in(),
        )
    }

    pub fn new_with_import_profile_registry(
        project_root: impl Into<PathBuf>,
        descriptor: AssetProjectCatalogDescriptor,
        import_profile_registry: EditorImportProfileRegistry,
    ) -> Self {
        Self {
            project_root: project_root.into(),
            descriptor,
            import_profile_registry,
            import_ledger: EditorImportExecutionLedger::default(),
            catalog_load_status: None,
            catalog_save_status: None,
            import_status: None,
        }
    }

    pub fn from_project_file(
        project_root: impl Into<PathBuf>,
        project: &ProjectFileV3,
    ) -> Result<Self, Vec<AssetDiagnosticRecord>> {
        let import_profile_registry = EditorImportProfileRegistry::from_project_file(project);
        if !import_profile_registry.diagnostics().is_empty() {
            return Err(import_profile_registry.diagnostics().to_vec());
        }
        let descriptor = project_asset_catalog_descriptor(project, &import_profile_registry)?;
        Ok(Self::new_with_import_profile_registry(
            project_root,
            descriptor,
            import_profile_registry,
        ))
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn descriptor(&self) -> &AssetProjectCatalogDescriptor {
        &self.descriptor
    }

    pub fn import_profile_registry(&self) -> &EditorImportProfileRegistry {
        &self.import_profile_registry
    }

    pub fn import_ledger(&self) -> &EditorImportExecutionLedger {
        &self.import_ledger
    }

    pub fn import_ledger_mut(&mut self) -> &mut EditorImportExecutionLedger {
        &mut self.import_ledger
    }

    pub fn replace_import_ledger(&mut self, ledger: EditorImportExecutionLedger) {
        self.import_ledger = ledger;
    }

    pub fn catalog_path(&self) -> PathBuf {
        self.project_root.join(&self.descriptor.catalog_file_path)
    }

    pub fn artifact_cache_root(&self) -> PathBuf {
        self.project_root.join(&self.descriptor.artifact_cache_root)
    }

    pub fn import_ledger_path(&self) -> PathBuf {
        self.project_root.join(".runenwerk/import-jobs.ron")
    }

    pub fn set_catalog_load_status(&mut self, status: impl Into<String>) {
        self.catalog_load_status = Some(status.into());
    }

    pub fn set_catalog_save_status(&mut self, status: impl Into<String>) {
        self.catalog_save_status = Some(status.into());
    }

    pub fn set_import_status(&mut self, status: impl Into<String>) {
        self.import_status = Some(status.into());
    }

    pub fn status_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("project root: {}", self.project_root.display()),
            format!("catalog: {}", self.descriptor.catalog_file_path),
            format!("artifact cache: {}", self.descriptor.artifact_cache_root),
            format!(
                "import ledger: .runenwerk/import-jobs.ron entries={}",
                self.import_ledger.len()
            ),
        ];
        if let Some(status) = &self.catalog_load_status {
            lines.push(format!("last load: {status}"));
        }
        if let Some(status) = &self.catalog_save_status {
            lines.push(format!("last save: {status}"));
        }
        if let Some(status) = &self.import_status {
            lines.push(format!("last import: {status}"));
        }
        lines
    }
}

pub fn project_asset_catalog_descriptor(
    project: &ProjectFileV3,
    import_profile_registry: &EditorImportProfileRegistry,
) -> Result<AssetProjectCatalogDescriptor, Vec<AssetDiagnosticRecord>> {
    let mut descriptor = AssetProjectCatalogDescriptor::new(
        project.asset_source_roots.iter().map(|root| {
            AssetSourceRoot::new(
                root.root_id,
                root.kind,
                root.display_name.clone(),
                root.relative_path.clone(),
            )
        }),
        project.artifact_cache_root.clone(),
        project.field_product_cache_root.clone(),
        project.catalog_file_path.clone(),
    );
    for default in &project.import_profile_defaults {
        let key = crate::asset_pipeline::EditorImportProfileKey::new(
            default.asset_kind,
            default.profile_name.clone(),
        );
        let Some(recipe) = import_profile_registry.recipe(&key) else {
            return Err(vec![AssetDiagnosticRecord::error(
                AssetDiagnosticCode::ImportProfileRejected,
                format!(
                    "project catalog default {:?}/{} has no resolved import recipe",
                    default.asset_kind, default.profile_name
                ),
            )]);
        };
        descriptor = descriptor.with_import_profile_default(AssetImportProfileDefault::new(
            default.asset_kind,
            default.profile_name.clone(),
            recipe.settings.clone(),
        ));
    }
    let report = ratify_asset_project_catalog_descriptor(&descriptor);
    if report.has_blocking_issues() {
        return Err(report
            .issues()
            .iter()
            .map(|issue| {
                AssetDiagnosticRecord::error(
                    AssetDiagnosticCode::RatificationRejected,
                    format!(
                        "asset project descriptor rejected {:?}: {}",
                        issue.code(),
                        issue.message()
                    ),
                )
            })
            .collect());
    }
    Ok(descriptor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_file_v3_forms_asset_project_descriptor_without_app_io() {
        let project = ProjectFileV3::new("project.test", "Test");
        let session = EditorAssetProjectSession::from_project_file("C:/project", &project)
            .expect("project session should form");

        assert_eq!(session.descriptor().catalog_file_path, "assets/catalog.ron");
        assert_eq!(session.descriptor().source_roots.len(), 2);
        assert!(session.status_lines()[0].contains("project root"));
    }

    #[test]
    fn project_file_v3_resolves_profile_defaults_into_descriptor() {
        let mut project = ProjectFileV3::new("project.test", "Test");
        project.import_profile_defaults.push(
            editor_persistence::ProjectImportProfileDefaultV3::new(
                asset::AssetKind::SdfGraph,
                crate::asset_pipeline::DEFAULT_IMPORT_PROFILE_NAME,
            ),
        );

        let session = EditorAssetProjectSession::from_project_file("C:/project", &project)
            .expect("built-in profile default should resolve");

        assert_eq!(session.descriptor().import_profile_defaults.len(), 1);
        assert_eq!(
            session.descriptor().import_profile_defaults[0].asset_kind,
            asset::AssetKind::SdfGraph
        );
    }
}
