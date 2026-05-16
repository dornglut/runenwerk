use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use asset::{
    ArtifactCacheKey, ArtifactValidity, AssetArtifactDescriptor, AssetArtifactId, AssetCatalog,
    AssetDiagnosticCode, AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetId, AssetKind,
    AssetSourceDescriptor, ImportJobId, ImportPlan, asset_artifact_id, deterministic_cache_key,
    import_job_id, ratify_asset_catalog, ratify_asset_import_plan_against_source,
};
use serde::{Deserialize, Serialize};

use crate::asset_pipeline::{EditorAssetProjectSession, ImportJobStatus, run_import_job};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EditorImportExecutionLedger {
    entries: Vec<EditorImportExecutionLedgerEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorImportExecutionLedgerEntry {
    pub cache_key: ArtifactCacheKey,
    pub job_id: ImportJobId,
    pub artifact_id: AssetArtifactId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorImportExecutionOutcome {
    pub status: ImportJobStatus,
    pub plan: Option<ImportPlan>,
    pub artifact: Option<AssetArtifactDescriptor>,
    pub diagnostics: Vec<AssetDiagnosticRecord>,
}

impl EditorImportExecutionLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[EditorImportExecutionLedgerEntry] {
        &self.entries
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Ok(Self::default());
        }
        let source = std::fs::read_to_string(path).with_context(|| {
            format!("failed to read import execution ledger: {}", path.display())
        })?;
        ron::from_str(&source).with_context(|| {
            format!(
                "failed to decode import execution ledger: {}",
                path.display()
            )
        })
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create import execution ledger parent directory: {}",
                    parent.display()
                )
            })?;
        }
        let config = ron::ser::PrettyConfig::new()
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let ron = ron::ser::to_string_pretty(self, config)
            .context("failed to encode import execution ledger as RON")?;
        std::fs::write(path, ron).with_context(|| {
            format!(
                "failed to write import execution ledger: {}",
                path.display()
            )
        })
    }

    pub fn validate_against_catalog(&self, catalog: &AssetCatalog) -> Vec<AssetDiagnosticRecord> {
        let mut diagnostics = Vec::new();
        let mut job_ids = BTreeSet::new();
        let mut artifact_ids = BTreeSet::new();
        let mut ledger_cache_keys: BTreeMap<&ArtifactCacheKey, &EditorImportExecutionLedgerEntry> =
            BTreeMap::new();
        for entry in &self.entries {
            if !job_ids.insert(entry.job_id) {
                diagnostics.push(blocking_identity_diagnostic(format!(
                    "duplicate import job id in ledger: {}",
                    entry.job_id.raw()
                )));
            }
            if !artifact_ids.insert(entry.artifact_id) {
                diagnostics.push(blocking_identity_diagnostic(format!(
                    "duplicate artifact id in ledger: {}",
                    entry.artifact_id.raw()
                )));
            }
            if ledger_cache_keys.insert(&entry.cache_key, entry).is_some() {
                diagnostics.push(blocking_identity_diagnostic(format!(
                    "duplicate cache key in ledger: {}",
                    entry.cache_key.as_str()
                )));
            }
        }

        let mut catalog_cache_keys: BTreeMap<&ArtifactCacheKey, AssetArtifactId> = BTreeMap::new();
        for artifact in catalog.artifacts.values() {
            if let Some(existing) =
                catalog_cache_keys.insert(&artifact.cache_key, artifact.artifact_id)
            {
                diagnostics.push(blocking_identity_diagnostic(format!(
                    "catalog cache-key collision {} between artifacts {} and {}",
                    artifact.cache_key.as_str(),
                    existing.raw(),
                    artifact.artifact_id.raw()
                )));
            }
        }

        for entry in &self.entries {
            if let Some(catalog_artifact_id) = catalog_cache_keys.get(&entry.cache_key)
                && *catalog_artifact_id != entry.artifact_id
            {
                diagnostics.push(blocking_identity_diagnostic(format!(
                    "ledger/catalog disagreement for cache key {}: ledger artifact {} catalog artifact {}",
                    entry.cache_key.as_str(),
                    entry.artifact_id.raw(),
                    catalog_artifact_id.raw()
                )));
            }
            if let Some(catalog_artifact) = catalog.artifact(entry.artifact_id)
                && catalog_artifact.cache_key != entry.cache_key
            {
                diagnostics.push(blocking_identity_diagnostic(format!(
                    "ledger/catalog disagreement for artifact {}: ledger cache key {} catalog cache key {}",
                    entry.artifact_id.raw(),
                    entry.cache_key.as_str(),
                    catalog_artifact.cache_key.as_str()
                )));
            }
        }

        diagnostics
    }

    pub fn resolve_or_allocate(
        &mut self,
        cache_key: ArtifactCacheKey,
        catalog: &AssetCatalog,
    ) -> Result<EditorImportExecutionLedgerEntry, Vec<AssetDiagnosticRecord>> {
        let diagnostics = self.validate_against_catalog(catalog);
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        if let Some(entry) = self
            .entries
            .iter()
            .find(|entry| entry.cache_key == cache_key)
            .cloned()
        {
            return Ok(entry);
        }
        let max_job_id = self
            .entries
            .iter()
            .map(|entry| entry.job_id.raw())
            .max()
            .unwrap_or(0);
        let max_ledger_artifact_id = self
            .entries
            .iter()
            .map(|entry| entry.artifact_id.raw())
            .max()
            .unwrap_or(0);
        let max_catalog_artifact_id = catalog
            .artifacts
            .keys()
            .map(|artifact_id| artifact_id.raw())
            .max()
            .unwrap_or(0);
        if max_job_id == u64::MAX || max_ledger_artifact_id.max(max_catalog_artifact_id) == u64::MAX
        {
            return Err(vec![blocking_identity_diagnostic(
                "import execution ledger id allocation overflow",
            )]);
        }
        let next_job_id = max_job_id + 1;
        let next_artifact_id = max_ledger_artifact_id.max(max_catalog_artifact_id) + 1;
        let entry = EditorImportExecutionLedgerEntry {
            cache_key,
            job_id: import_job_id(next_job_id),
            artifact_id: asset_artifact_id(next_artifact_id),
        };
        self.entries.push(entry.clone());
        self.entries
            .sort_by(|left, right| left.cache_key.cmp(&right.cache_key));
        Ok(entry)
    }
}

pub fn execute_import_for_asset(
    catalog: &AssetCatalog,
    session: &mut EditorAssetProjectSession,
    asset_id: AssetId,
) -> EditorImportExecutionOutcome {
    match EditorImportExecutionLedger::load_from_path(&session.import_ledger_path()) {
        Ok(ledger) => session.replace_import_ledger(ledger),
        Err(error) => {
            return rejected_import(format!(
                "failed to load import execution ledger; import publication blocked: {error}"
            ));
        }
    }

    let Some(source) = primary_source_for_asset(catalog, asset_id).cloned() else {
        return rejected_import(format!("asset {} has no source descriptor", asset_id.raw()));
    };
    let recipe = match session
        .import_profile_registry()
        .resolve_for_source(&source)
    {
        Ok(recipe) => recipe,
        Err(diagnostics) => {
            return EditorImportExecutionOutcome {
                status: ImportJobStatus::Failed,
                plan: None,
                artifact: None,
                diagnostics,
            };
        }
    };
    let cache_key =
        deterministic_cache_key(&source, &recipe.settings, recipe.expected_artifact_kind);
    let entry = match session
        .import_ledger_mut()
        .resolve_or_allocate(cache_key, catalog)
    {
        Ok(entry) => entry,
        Err(diagnostics) => {
            return EditorImportExecutionOutcome {
                status: ImportJobStatus::Failed,
                plan: None,
                artifact: None,
                diagnostics,
            };
        }
    };
    if let Err(error) = session
        .import_ledger()
        .save_to_path(&session.import_ledger_path())
    {
        return rejected_import(format!(
            "failed to persist import execution ledger: {error}"
        ));
    }

    let expected_kind = recipe.expected_artifact_kind;
    let plan = ImportPlan::deterministic(entry.job_id, &source, recipe.settings, expected_kind);
    let report = ratify_asset_import_plan_against_source(&plan, &source);
    if report.has_blocking_issues() {
        return EditorImportExecutionOutcome {
            status: ImportJobStatus::Failed,
            plan: Some(plan),
            artifact: None,
            diagnostics: report
                .issues()
                .iter()
                .map(|issue| {
                    AssetDiagnosticRecord::error(
                        AssetDiagnosticCode::RatificationRejected,
                        format!(
                            "import plan rejected {:?}: {}",
                            issue.code(),
                            issue.message()
                        ),
                    )
                })
                .collect(),
        };
    }

    let previous_valid_artifact =
        previous_valid_artifact_for_import(catalog, &source, expected_kind);
    let outcome = match run_import_job(
        &source,
        &plan,
        session.project_root(),
        &session.artifact_cache_root(),
        entry.artifact_id,
        previous_valid_artifact,
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            return rejected_import(format!("import job execution failed: {error}"));
        }
    };

    EditorImportExecutionOutcome {
        status: outcome.status,
        plan: Some(plan),
        artifact: outcome.artifact,
        diagnostics: outcome.diagnostics,
    }
}

pub fn catalog_with_import_artifact(
    catalog: &AssetCatalog,
    artifact: AssetArtifactDescriptor,
) -> Result<AssetCatalog, Vec<AssetDiagnosticRecord>> {
    let mut candidate = catalog.clone();
    candidate.insert_artifact(artifact);
    let report = ratify_asset_catalog(&candidate);
    if report.has_blocking_issues() {
        return Err(report
            .issues()
            .iter()
            .map(|issue| {
                AssetDiagnosticRecord::error(
                    AssetDiagnosticCode::RatificationRejected,
                    format!(
                        "import publication rejected {:?}: {}",
                        issue.code(),
                        issue.message()
                    ),
                )
            })
            .collect());
    }
    Ok(candidate)
}

fn primary_source_for_asset(
    catalog: &AssetCatalog,
    asset_id: AssetId,
) -> Option<&AssetSourceDescriptor> {
    let record = catalog.asset(asset_id)?;
    record
        .primary_source_id
        .and_then(|source_id| catalog.source(source_id))
        .or_else(|| {
            catalog
                .sources
                .values()
                .find(|source| source.asset_id == asset_id)
        })
}

fn previous_valid_artifact_for_import<'a>(
    catalog: &'a AssetCatalog,
    source: &AssetSourceDescriptor,
    expected_kind: AssetKind,
) -> Option<&'a AssetArtifactDescriptor> {
    let record = catalog.asset(source.asset_id)?;
    record
        .artifact_ids
        .iter()
        .rev()
        .filter_map(|artifact_id| catalog.artifact(*artifact_id))
        .find(|artifact| {
            artifact.kind == expected_kind && artifact.validity == ArtifactValidity::Valid
        })
}

fn rejected_import(message: impl Into<String>) -> EditorImportExecutionOutcome {
    EditorImportExecutionOutcome {
        status: ImportJobStatus::Failed,
        plan: None,
        artifact: None,
        diagnostics: vec![AssetDiagnosticRecord::new(
            AssetDiagnosticCode::RatificationRejected,
            AssetDiagnosticSeverity::Error,
            message,
        )],
    }
}

fn blocking_identity_diagnostic(message: impl Into<String>) -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::error(AssetDiagnosticCode::CatalogDuplicateId, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{
        ArtifactPayloadKind, AssetProjectCatalogDescriptor, AssetRecord, AssetSourceDescriptor,
        AssetSourceRoot, AssetSourceRootKind, FieldProductResolution, ImportSettings, SourceHash,
        asset_id, asset_source_id, asset_source_root_id,
    };
    use editor_persistence::{
        ProjectFileV3, ProjectImportProfileDefaultV3, ProjectImportProfileDefinitionV3,
    };

    #[test]
    fn ledger_reuses_identity_for_identical_cache_key() {
        let catalog = AssetCatalog::new();
        let mut ledger = EditorImportExecutionLedger::default();
        let key = ArtifactCacheKey::new("same");

        let first = ledger
            .resolve_or_allocate(key.clone(), &catalog)
            .expect("first identity should allocate");
        let second = ledger
            .resolve_or_allocate(key, &catalog)
            .expect("same key should reuse identity");

        assert_eq!(first, second);
        assert_eq!(ledger.len(), 1);
    }

    #[test]
    fn ledger_allocates_new_ids_above_ledger_and_catalog_maxima() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_artifact(AssetArtifactDescriptor::new(
            asset_artifact_id(40),
            asset_id(1),
            AssetKind::Scene,
            ArtifactPayloadKind::SceneManifest,
            ArtifactCacheKey::new("catalog"),
        ));
        let mut ledger = EditorImportExecutionLedger {
            entries: vec![EditorImportExecutionLedgerEntry {
                cache_key: ArtifactCacheKey::new("old"),
                job_id: import_job_id(9),
                artifact_id: asset_artifact_id(10),
            }],
        };

        let entry = ledger
            .resolve_or_allocate(ArtifactCacheKey::new("new"), &catalog)
            .expect("identity should allocate above maxima");

        assert_eq!(entry.job_id.raw(), 10);
        assert_eq!(entry.artifact_id.raw(), 41);
    }

    #[test]
    fn ledger_rejects_duplicate_job_artifact_and_catalog_disagreement() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_artifact(AssetArtifactDescriptor::new(
            asset_artifact_id(8),
            asset_id(1),
            AssetKind::Scene,
            ArtifactPayloadKind::SceneManifest,
            ArtifactCacheKey::new("same"),
        ));
        let ledger = EditorImportExecutionLedger {
            entries: vec![
                EditorImportExecutionLedgerEntry {
                    cache_key: ArtifactCacheKey::new("same"),
                    job_id: import_job_id(2),
                    artifact_id: asset_artifact_id(7),
                },
                EditorImportExecutionLedgerEntry {
                    cache_key: ArtifactCacheKey::new("other"),
                    job_id: import_job_id(2),
                    artifact_id: asset_artifact_id(7),
                },
            ],
        };

        let diagnostics = ledger.validate_against_catalog(&catalog);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("duplicate import job id") })
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("duplicate artifact id") })
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("ledger/catalog disagreement") })
        );
    }

    #[test]
    fn ledger_rejects_artifact_id_reused_for_different_catalog_cache_key() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_artifact(AssetArtifactDescriptor::new(
            asset_artifact_id(8),
            asset_id(1),
            AssetKind::Scene,
            ArtifactPayloadKind::SceneManifest,
            ArtifactCacheKey::new("catalog"),
        ));
        let ledger = EditorImportExecutionLedger {
            entries: vec![EditorImportExecutionLedgerEntry {
                cache_key: ArtifactCacheKey::new("ledger"),
                job_id: import_job_id(2),
                artifact_id: asset_artifact_id(8),
            }],
        };

        let diagnostics = ledger.validate_against_catalog(&catalog);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("ledger/catalog disagreement for artifact 8")
        }));
    }

    #[test]
    fn import_execution_preserves_prior_valid_artifact_on_controlled_failure() {
        let root = unique_temp_dir("orchestrated_import_preserve");
        let mut session = EditorAssetProjectSession::new(
            &root,
            AssetProjectCatalogDescriptor::new(
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
        let asset_id = asset_id(1);
        let source_id = asset_source_id(1);
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "field", "Field", AssetKind::SdfGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(
            AssetSourceDescriptor::new(
                source_id,
                asset_id,
                AssetKind::SdfGraph,
                "assets/missing.ron",
            )
            .with_hash(SourceHash::new("sha256", "missing")),
        );
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(2),
                asset_id,
                AssetKind::FormedFieldProduct,
                ArtifactPayloadKind::FormedFieldProduct {
                    product_id: "prior".to_string(),
                },
                ArtifactCacheKey::new("prior"),
            )
            .with_source(source_id, asset::asset_source_revision_id(1))
            .with_artifact_path(".runenwerk/artifacts/prior.ron"),
        );

        let outcome = execute_import_for_asset(&catalog, &mut session, asset_id);

        assert_eq!(outcome.status, ImportJobStatus::FailedPreserved);
        assert_eq!(
            outcome.artifact.as_ref().map(|artifact| artifact.validity),
            Some(ArtifactValidity::FailedPreserved)
        );
        assert_eq!(session.import_ledger().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn import_execution_uses_project_resolved_recipe() {
        let root = unique_temp_dir("orchestrated_import_project_recipe");
        let mut project = ProjectFileV3::new("project.test", "Test");
        project
            .import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                AssetKind::SdfGraph,
                "high",
                ImportSettings::SdfGraph {
                    resolution: FieldProductResolution::new(128, 128, 1),
                },
                AssetKind::FormedFieldProduct,
            ));
        project
            .import_profile_defaults
            .push(ProjectImportProfileDefaultV3::new(
                AssetKind::SdfGraph,
                "high",
            ));
        let mut session = EditorAssetProjectSession::from_project_file(&root, &project)
            .expect("project session should resolve profile defaults");
        std::fs::create_dir_all(root.join("assets")).expect("asset dir should be writable");
        std::fs::write(root.join("assets/field.ron"), "()").expect("source should be writable");
        let mut catalog = AssetCatalog::new();
        let asset_id = asset_id(1);
        let source_id = asset_source_id(1);
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "field", "Field", AssetKind::SdfGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::SdfGraph,
            "assets/field.ron",
        ));

        let outcome = execute_import_for_asset(&catalog, &mut session, asset_id);

        assert_eq!(outcome.status, ImportJobStatus::Imported);
        assert_eq!(
            outcome.plan.as_ref().map(|plan| &plan.settings),
            Some(&ImportSettings::SdfGraph {
                resolution: FieldProductResolution::new(128, 128, 1),
            })
        );
        assert_eq!(session.import_ledger().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn malformed_import_ledger_blocks_publication_with_diagnostic() {
        let root = unique_temp_dir("orchestrated_import_bad_ledger");
        let mut session = EditorAssetProjectSession::new(
            &root,
            AssetProjectCatalogDescriptor::new(
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
        std::fs::create_dir_all(root.join(".runenwerk")).expect("ledger dir should be writable");
        std::fs::write(session.import_ledger_path(), "not valid ron")
            .expect("bad ledger should be writable");
        let mut catalog = AssetCatalog::new();
        let asset_id = asset_id(5);
        let source_id = asset_source_id(6);
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "scene", "Scene", AssetKind::Scene)
                .with_primary_source(source_id),
        );
        catalog.insert_source(
            AssetSourceDescriptor::new(source_id, asset_id, AssetKind::Scene, "assets/scene.ron")
                .with_hash(SourceHash::new("sha256", "abc")),
        );

        let outcome = execute_import_for_asset(&catalog, &mut session, asset_id);

        assert_eq!(outcome.status, ImportJobStatus::Failed);
        assert!(outcome.artifact.is_none());
        assert!(
            outcome.diagnostics[0]
                .message
                .contains("failed to load import execution ledger")
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
