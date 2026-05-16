use std::path::Path;

use anyhow::{Context, Result};
use asset::{
    AssetCatalog, AssetDiagnosticCode, AssetDiagnosticRecord, AssetDiagnosticSeverity,
    ratify_asset_catalog,
};

use crate::asset_pipeline::EditorAssetProjectSession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogPersistenceOutcome {
    pub accepted_catalog: Option<AssetCatalog>,
    pub diagnostics: Vec<AssetDiagnosticRecord>,
}

impl CatalogPersistenceOutcome {
    fn accepted(catalog: AssetCatalog) -> Self {
        Self {
            accepted_catalog: Some(catalog),
            diagnostics: Vec::new(),
        }
    }

    fn rejected(diagnostics: Vec<AssetDiagnosticRecord>) -> Self {
        Self {
            accepted_catalog: None,
            diagnostics,
        }
    }
}

pub fn load_project_catalog(
    session: &EditorAssetProjectSession,
) -> Result<CatalogPersistenceOutcome> {
    let path = session.catalog_path();
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CatalogPersistenceOutcome::rejected(vec![
                AssetDiagnosticRecord::error(
                    AssetDiagnosticCode::SourceMissing,
                    format!("asset catalog is missing: {}", path.display()),
                ),
            ]));
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read asset catalog: {}", path.display()));
        }
    };
    let catalog = match ron::from_str::<AssetCatalog>(&source) {
        Ok(catalog) => catalog,
        Err(error) => {
            return Ok(CatalogPersistenceOutcome::rejected(vec![
                AssetDiagnosticRecord::new(
                    AssetDiagnosticCode::RatificationRejected,
                    AssetDiagnosticSeverity::Error,
                    format!("asset catalog decode failed: {error}"),
                ),
            ]));
        }
    };
    let report = ratify_asset_catalog(&catalog);
    if report.has_blocking_issues() {
        return Ok(CatalogPersistenceOutcome::rejected(
            report
                .issues()
                .iter()
                .map(|issue| {
                    AssetDiagnosticRecord::error(
                        AssetDiagnosticCode::RatificationRejected,
                        format!(
                            "catalog ratification rejected {:?}: {}",
                            issue.code(),
                            issue.message()
                        ),
                    )
                })
                .collect(),
        ));
    }
    Ok(CatalogPersistenceOutcome::accepted(catalog))
}

pub fn save_project_catalog(
    session: &EditorAssetProjectSession,
    catalog: &AssetCatalog,
) -> Result<Vec<AssetDiagnosticRecord>> {
    let report = ratify_asset_catalog(catalog);
    if report.has_blocking_issues() {
        return Ok(report
            .issues()
            .iter()
            .map(|issue| {
                AssetDiagnosticRecord::error(
                    AssetDiagnosticCode::RatificationRejected,
                    format!(
                        "catalog save rejected {:?}: {}",
                        issue.code(),
                        issue.message()
                    ),
                )
            })
            .collect());
    }
    let path = session.catalog_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create asset catalog parent directory: {}",
                parent.display()
            )
        })?;
    }
    let ron = encode_catalog_ron(catalog)?;
    std::fs::write(&path, ron)
        .with_context(|| format!("failed to write asset catalog: {}", path.display()))?;
    Ok(Vec::new())
}

fn encode_catalog_ron(catalog: &AssetCatalog) -> Result<String> {
    let config = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    ron::ser::to_string_pretty(catalog, config).context("failed to encode AssetCatalog as RON")
}

pub fn catalog_path_exists(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_pipeline::EditorAssetProjectSession;
    use asset::{
        AssetRecord, AssetSourceRoot, AssetSourceRootKind, asset_id, asset_source_root_id,
    };

    #[test]
    fn catalog_load_save_round_trips_ratified_catalog() {
        let root = unique_temp_dir("catalog_round_trip");
        let session = EditorAssetProjectSession::new(
            &root,
            asset::AssetProjectCatalogDescriptor::new(
                [AssetSourceRoot::new(
                    asset_source_root_id(1),
                    AssetSourceRootKind::ProjectAssets,
                    "Project assets",
                    "assets",
                )],
                ".runenwerk/artifacts",
                ".runenwerk/field-products",
                "assets/catalog.ron",
            ),
        );
        let mut catalog = AssetCatalog::new();
        catalog.insert_source_root(AssetSourceRoot::new(
            asset_source_root_id(1),
            AssetSourceRootKind::ProjectAssets,
            "Project assets",
            "assets",
        ));
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(1),
            "scene",
            "Scene",
            asset::AssetKind::Scene,
        ));

        let diagnostics = save_project_catalog(&session, &catalog).expect("save should not fail");
        assert!(diagnostics.is_empty());
        let loaded = load_project_catalog(&session).expect("load should not fail");

        assert_eq!(loaded.accepted_catalog, Some(catalog));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn missing_catalog_reports_controlled_diagnostic_without_replacing_catalog() {
        let root = unique_temp_dir("catalog_missing");
        let session = EditorAssetProjectSession::new(
            &root,
            asset::AssetProjectCatalogDescriptor::new(
                [],
                ".runenwerk/artifacts",
                ".runenwerk/field-products",
                "assets/catalog.ron",
            ),
        );

        let loaded = load_project_catalog(&session).expect("missing file is controlled");

        assert!(loaded.accepted_catalog.is_none());
        assert_eq!(
            loaded.diagnostics[0].code,
            AssetDiagnosticCode::SourceMissing
        );
        let _ = std::fs::remove_dir_all(root);
    }

    fn unique_temp_dir(label: &str) -> std::path::PathBuf {
        let mut root = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        root.push(format!("{label}_{nanos}"));
        std::fs::create_dir_all(&root).expect("temp dir should be creatable");
        root
    }
}
