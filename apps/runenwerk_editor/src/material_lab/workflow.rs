use std::path::{Path, PathBuf};

use anyhow::Result;
use asset::{
    ArtifactCacheKey, ArtifactPayloadKind, AssetArtifactDescriptor, AssetCatalog,
    AssetDiagnosticCode, AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetId, AssetKind,
    AssetSourceDescriptor, ImportPlan, deterministic_cache_key, ratify_asset_catalog,
    ratify_asset_import_plan_against_source, try_preserve_prior_valid_artifact,
};
use material_graph::{
    MaterialGraphDocument, MaterialGraphIssueCode, MaterialGraphIssueSubject, MaterialOutputTarget,
    lower_material_graph,
};
use product::ProductPublicationOutcome;

use crate::editor_app::RunenwerkEditorApp;
use crate::material_lab::{
    EditorMaterialPreviewProduct, EditorMaterialPreviewPublication, ResolvedMaterialLoweringRecipe,
    material_document_id_for_source, material_product_id_for_import_job,
    previous_valid_material_artifact, read_material_graph_document, write_material_graph_document,
};

impl RunenwerkEditorApp {
    pub fn select_material_asset(&mut self, asset_id: AssetId) {
        self.material_lab_runtime_mut()
            .select_material_asset(Some(asset_id));
        self.append_console_line(format!("[material] selected asset {}", asset_id.raw()));
    }

    pub fn clear_material_diagnostics(&mut self) {
        self.material_lab_runtime_mut().clear_diagnostics();
        self.append_console_line("[material] diagnostics cleared");
    }

    pub fn rebuild_selected_material_preview(&mut self) -> Result<()> {
        let Some(asset_id) = self.material_lab_runtime().selected_material_asset_id() else {
            let diagnostic = material_diagnostic(
                AssetDiagnosticCode::RatificationRejected,
                "no selected material asset to build",
            );
            self.record_material_workflow_diagnostics([diagnostic]);
            return Ok(());
        };
        self.rebuild_material_preview(asset_id)
    }

    pub fn rebuild_material_preview(&mut self, asset_id: AssetId) -> Result<()> {
        if self.asset_project_session.is_none() {
            self.record_missing_asset_project_session("build material preview");
            return Ok(());
        }

        self.material_lab_runtime_mut()
            .select_material_asset(Some(asset_id));
        let catalog_snapshot = self.asset_catalog_runtime().catalog().clone();
        let outcome = {
            let session = self
                .asset_project_session
                .as_mut()
                .expect("asset project session checked above");
            rebuild_material_preview_for_asset(&catalog_snapshot, session, asset_id)
        };

        self.record_material_workflow_diagnostics(outcome.diagnostics.clone());
        if let Some(publication) = outcome.publication {
            self.queue_material_preview_publication(publication);
            self.material_lab_runtime_mut()
                .set_workflow_status("preview publication queued");
        } else {
            self.material_lab_runtime_mut()
                .set_workflow_status("preview build blocked");
        }
        Ok(())
    }

    pub fn load_material_graph_document_for_asset(
        &self,
        asset_id: AssetId,
    ) -> Result<MaterialGraphDocument> {
        let session = self.asset_project_session().ok_or_else(|| {
            anyhow::anyhow!("cannot load material graph document: no asset project session")
        })?;
        let source = material_source_for_asset(self.asset_catalog_runtime().catalog(), asset_id)
            .ok_or_else(|| {
                anyhow::anyhow!("asset {} has no material graph source", asset_id.raw())
            })?;
        read_material_graph_document(&session.project_root().join(&source.relative_path))
    }

    pub fn write_material_graph_document_for_asset(
        &self,
        asset_id: AssetId,
        document: &MaterialGraphDocument,
    ) -> Result<()> {
        let session = self.asset_project_session().ok_or_else(|| {
            anyhow::anyhow!("cannot write material graph document: no asset project session")
        })?;
        let source = material_source_for_asset(self.asset_catalog_runtime().catalog(), asset_id)
            .ok_or_else(|| {
                anyhow::anyhow!("asset {} has no material graph source", asset_id.raw())
            })?;
        write_material_graph_document(
            &session.project_root().join(&source.relative_path),
            document,
        )
    }

    fn record_material_workflow_diagnostics(
        &mut self,
        diagnostics: impl IntoIterator<Item = AssetDiagnosticRecord>,
    ) {
        let diagnostics = diagnostics.into_iter().collect::<Vec<_>>();
        for diagnostic in &diagnostics {
            self.asset_catalog_runtime_mut()
                .record_diagnostic(diagnostic.clone());
        }
        self.material_lab_runtime_mut()
            .record_diagnostics(diagnostics);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorMaterialPreviewBuildOutcome {
    pub publication: Option<EditorMaterialPreviewPublication>,
    pub diagnostics: Vec<AssetDiagnosticRecord>,
}

pub fn rebuild_material_preview_for_asset(
    catalog: &AssetCatalog,
    session: &mut crate::asset_pipeline::EditorAssetProjectSession,
    asset_id: AssetId,
) -> EditorMaterialPreviewBuildOutcome {
    match crate::asset_pipeline::EditorImportExecutionLedger::load_from_path(
        &session.import_ledger_path(),
    ) {
        Ok(ledger) => session.replace_import_ledger(ledger),
        Err(error) => {
            return blocked_preview(format!(
                "failed to load import execution ledger; material preview publication blocked: {error}"
            ));
        }
    }

    let Some(source) = material_source_for_asset(catalog, asset_id).cloned() else {
        return blocked_preview(format!(
            "asset {} has no material graph source descriptor",
            asset_id.raw()
        ));
    };

    let recipe = match session
        .import_profile_registry()
        .resolve_for_source(&source)
    {
        Ok(recipe) => recipe,
        Err(diagnostics) => {
            return EditorMaterialPreviewBuildOutcome {
                publication: None,
                diagnostics,
            };
        }
    };
    let material_recipe = match ResolvedMaterialLoweringRecipe::resolve(&source, &recipe) {
        Ok(recipe) => recipe,
        Err(diagnostics) => {
            return EditorMaterialPreviewBuildOutcome {
                publication: None,
                diagnostics,
            };
        }
    };

    let source_path = session.project_root().join(&source.relative_path);
    let document = match read_material_graph_document(&source_path) {
        Ok(document) => document,
        Err(error) => {
            return preserved_or_blocked(
                catalog,
                asset_id,
                material_diagnostic(
                    AssetDiagnosticCode::SourceMissing,
                    format!(
                        "material graph source could not be read: {} ({error})",
                        source.relative_path
                    ),
                ),
            );
        }
    };

    let expected_document_id = material_document_id_for_source(source.asset_id, source.source_id);
    if document.document_id != expected_document_id {
        return preserved_or_blocked(
            catalog,
            asset_id,
            material_diagnostic(
                AssetDiagnosticCode::RatificationRejected,
                format!(
                    "material graph document id {} does not match asset/source identity {}",
                    document.document_id.raw(),
                    expected_document_id.raw()
                ),
            ),
        );
    }
    if let Err(diagnostic) = material_recipe.validate_document_output_target(document.output_target)
    {
        return preserved_or_blocked(catalog, asset_id, diagnostic);
    }

    let node_catalog = material_recipe.node_catalog();
    let lowering = lower_material_graph(&document, &node_catalog);
    if lowering.report.has_blocking_issues() {
        let diagnostics = lowering
            .report
            .issues()
            .iter()
            .map(|issue| {
                material_graph_diagnostic(
                    issue.code(),
                    issue.subject(),
                    issue.message().to_string(),
                )
            })
            .collect::<Vec<_>>();
        return preserved_or_blocked_many(catalog, asset_id, diagnostics);
    }
    let Some(product) = lowering.product else {
        return blocked_preview("material graph lowering produced no formed material product");
    };

    let import_cache_key = deterministic_cache_key(
        &source,
        &recipe.settings,
        material_recipe.expected_artifact_kind,
    );
    let artifact_cache_key = ArtifactCacheKey::new(format!(
        "{}:{}:formed_material={}",
        import_cache_key.as_str(),
        material_recipe.cache_key_component,
        product.cache_key.as_str()
    ));
    let entry = match session
        .import_ledger_mut()
        .resolve_or_allocate(artifact_cache_key.clone(), catalog)
    {
        Ok(entry) => entry,
        Err(diagnostics) => {
            return EditorMaterialPreviewBuildOutcome {
                publication: None,
                diagnostics,
            };
        }
    };
    if let Err(error) = session
        .import_ledger()
        .save_to_path(&session.import_ledger_path())
    {
        return blocked_preview(format!(
            "failed to persist import execution ledger: {error}"
        ));
    }

    let product = product.with_product_id(material_product_id_for_import_job(entry.job_id));
    let plan = ImportPlan::deterministic(
        entry.job_id,
        &source,
        recipe.settings.clone(),
        material_recipe.expected_artifact_kind,
    );
    let report = ratify_asset_import_plan_against_source(&plan, &source);
    if report.has_blocking_issues() {
        return EditorMaterialPreviewBuildOutcome {
            publication: None,
            diagnostics: report
                .issues()
                .iter()
                .map(|issue| {
                    material_diagnostic(
                        AssetDiagnosticCode::RatificationRejected,
                        format!(
                            "material import plan rejected {:?}: {}",
                            issue.code(),
                            issue.message()
                        ),
                    )
                })
                .collect(),
        };
    }
    let Some(product_job) = plan.product_job.clone() else {
        return blocked_preview("material preview import plan did not create a product job");
    };

    let artifact = AssetArtifactDescriptor::new(
        entry.artifact_id,
        source.asset_id,
        material_recipe.expected_artifact_kind,
        ArtifactPayloadKind::FormedMaterialProduct {
            product_id: product.product_id.raw().to_string(),
        },
        artifact_cache_key.clone(),
    )
    .with_source(source.source_id, source.revision_id)
    .with_artifact_path(material_artifact_path(&entry.artifact_id));
    if let Err(diagnostics) = catalog_with_material_artifact(catalog, artifact.clone()) {
        return EditorMaterialPreviewBuildOutcome {
            publication: None,
            diagnostics,
        };
    }

    let preview = EditorMaterialPreviewProduct::new(
        source.asset_id,
        source.source_id,
        entry.artifact_id,
        artifact_cache_key,
        product.clone(),
    );
    let publication = ProductPublicationOutcome::ready(
        product_job,
        [product.product_core.clone()],
        entry.job_id.raw(),
    );

    EditorMaterialPreviewBuildOutcome {
        publication: Some(EditorMaterialPreviewPublication::ready(
            publication,
            preview,
            artifact,
        )),
        diagnostics: Vec::new(),
    }
}

pub fn material_source_for_asset(
    catalog: &AssetCatalog,
    asset_id: AssetId,
) -> Option<&AssetSourceDescriptor> {
    let record = catalog.asset(asset_id)?;
    if record.kind != AssetKind::MaterialGraph {
        return None;
    }
    record
        .primary_source_id
        .and_then(|source_id| catalog.source(source_id))
        .or_else(|| {
            catalog
                .sources
                .values()
                .find(|source| source.asset_id == asset_id)
        })
        .filter(|source| source.kind == AssetKind::MaterialGraph)
}

pub fn default_material_graph_document_for_source(
    asset_id: AssetId,
    source: &AssetSourceDescriptor,
    label: impl Into<String>,
) -> MaterialGraphDocument {
    default_material_graph_document_for_source_with_target(
        asset_id,
        source,
        label,
        MaterialOutputTarget::PbrPreview,
    )
}

pub fn default_material_graph_document_for_source_with_target(
    asset_id: AssetId,
    source: &AssetSourceDescriptor,
    label: impl Into<String>,
    output_target: MaterialOutputTarget,
) -> MaterialGraphDocument {
    use graph::{
        CyclePolicy, GraphDefinition, GraphId, NodeDefinition, NodeId, PortDefinition,
        PortDirection, PortId, PortTypeId,
    };
    MaterialGraphDocument::new(
        material_document_id_for_source(asset_id, source.source_id),
        label,
        GraphDefinition::new(
            GraphId::new(1),
            "material.preview",
            CyclePolicy::RejectDirectedCycles,
            [NodeDefinition::new(
                NodeId::new(1),
                "pbr.output",
                [PortDefinition::new(
                    PortId::new(1),
                    "base_color",
                    PortDirection::Input,
                    PortTypeId::new(1),
                )],
            )],
            [],
        ),
        output_target,
    )
}

pub fn catalog_with_material_artifact(
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
                material_diagnostic(
                    AssetDiagnosticCode::RatificationRejected,
                    format!(
                        "material preview catalog publication rejected {:?}: {}",
                        issue.code(),
                        issue.message()
                    ),
                )
            })
            .collect());
    }
    Ok(candidate)
}

fn preserved_or_blocked(
    catalog: &AssetCatalog,
    asset_id: AssetId,
    diagnostic: AssetDiagnosticRecord,
) -> EditorMaterialPreviewBuildOutcome {
    preserved_or_blocked_many(catalog, asset_id, vec![diagnostic])
}

fn preserved_or_blocked_many(
    catalog: &AssetCatalog,
    asset_id: AssetId,
    diagnostics: Vec<AssetDiagnosticRecord>,
) -> EditorMaterialPreviewBuildOutcome {
    let Some(first) = diagnostics.first().cloned() else {
        return blocked_preview("material preview build failed without diagnostics");
    };
    let publication = previous_valid_material_artifact(catalog, asset_id)
        .and_then(|previous| try_preserve_prior_valid_artifact(previous, first).ok())
        .map(EditorMaterialPreviewPublication::failed_preserved);
    EditorMaterialPreviewBuildOutcome {
        publication,
        diagnostics,
    }
}

fn blocked_preview(message: impl Into<String>) -> EditorMaterialPreviewBuildOutcome {
    EditorMaterialPreviewBuildOutcome {
        publication: None,
        diagnostics: vec![material_diagnostic(
            AssetDiagnosticCode::RatificationRejected,
            message,
        )],
    }
}

fn material_graph_diagnostic(
    code: &MaterialGraphIssueCode,
    subject: &MaterialGraphIssueSubject,
    message: String,
) -> AssetDiagnosticRecord {
    material_diagnostic(
        AssetDiagnosticCode::RatificationRejected,
        format!("material graph rejected {code:?} {subject:?}: {message}"),
    )
}

fn material_diagnostic(
    code: AssetDiagnosticCode,
    message: impl Into<String>,
) -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::new(code, AssetDiagnosticSeverity::Error, message)
}

fn material_artifact_path(artifact_id: &asset::AssetArtifactId) -> String {
    format!(
        ".runenwerk/artifacts/material-preview-{}.artifact.ron",
        artifact_id.raw()
    )
}

#[allow(dead_code)]
fn project_relative_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[allow(dead_code)]
fn absolute_source_path(project_root: &Path, source: &AssetSourceDescriptor) -> PathBuf {
    project_root.join(&source.relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{
        AssetProjectCatalogDescriptor, AssetRecord, AssetSourceDescriptor, AssetSourceRoot,
        AssetSourceRootKind, ImportSettings, SourceHash, asset_artifact_id, asset_id,
        asset_source_id, asset_source_root_id,
    };
    use editor_persistence::{
        ProjectFileV3, ProjectImportProfileDefaultV3, ProjectImportProfileDefinitionV3,
    };

    #[test]
    fn material_preview_build_lowers_source_document_and_queues_publication() {
        let root = unique_temp_dir("material_preview_build");
        let asset_id = asset_id(1);
        let source_id = asset_source_id(2);
        let mut project = ProjectFileV3::new("project.material", "Material");
        project
            .import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                AssetKind::MaterialGraph,
                "render",
                ImportSettings::MaterialGraph {
                    lowering_target: "render_material".to_string(),
                },
                AssetKind::Material,
            ));
        project
            .import_profile_defaults
            .push(ProjectImportProfileDefaultV3::new(
                AssetKind::MaterialGraph,
                "render",
            ));
        let mut session =
            crate::asset_pipeline::EditorAssetProjectSession::from_project_file(&root, &project)
                .expect("project session should form");
        let source = AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/rock.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let document = default_material_graph_document_for_source_with_target(
            asset_id,
            &source,
            "Rock",
            MaterialOutputTarget::RenderMaterial,
        );
        write_material_graph_document(&root.join(&source.relative_path), &document)
            .expect("source document should write");
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(source);

        let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

        assert!(outcome.diagnostics.is_empty());
        let publication = outcome.publication.expect("preview should queue");
        assert_eq!(publication.status, product::ProductPublicationStatus::Ready);
        assert_eq!(
            publication.preview.as_ref().map(|preview| preview.asset_id),
            Some(asset_id)
        );
        assert_eq!(session.import_ledger().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn material_recipe_rejection_blocks_before_ledger_allocation() {
        let root = unique_temp_dir("material_recipe_rejected");
        let asset_id = asset_id(1);
        let source_id = asset_source_id(2);
        let mut project = ProjectFileV3::new("project.material", "Material");
        project
            .import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                AssetKind::MaterialGraph,
                "bad",
                ImportSettings::MaterialGraph {
                    lowering_target: String::new(),
                },
                AssetKind::Material,
            ));
        project
            .import_profile_defaults
            .push(ProjectImportProfileDefaultV3::new(
                AssetKind::MaterialGraph,
                "bad",
            ));
        let mut session =
            crate::asset_pipeline::EditorAssetProjectSession::from_project_file(&root, &project)
                .expect("project session should form");
        let source = AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/rock.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(source);

        let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

        assert!(outcome.publication.is_none());
        assert_eq!(session.import_ledger().len(), 0);
        assert!(
            outcome.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == AssetDiagnosticCode::ImportProfileRejected
            })
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn changed_material_recipe_allocates_new_ledger_entry() {
        let root = unique_temp_dir("material_recipe_cache_split");
        let asset_id = asset_id(1);
        let source_id = asset_source_id(2);
        let source = AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/rock.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(source.clone());

        let mut preview_session =
            crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
                &root,
                &material_project_with_recipe("preview", "preview"),
            )
            .expect("preview project session should form");
        let preview_document = default_material_graph_document_for_source_with_target(
            asset_id,
            &source,
            "Rock",
            MaterialOutputTarget::PbrPreview,
        );
        write_material_graph_document(&root.join(&source.relative_path), &preview_document)
            .expect("preview source document should write");
        let preview = rebuild_material_preview_for_asset(&catalog, &mut preview_session, asset_id);
        assert!(preview.diagnostics.is_empty(), "{:?}", preview.diagnostics);
        assert_eq!(preview_session.import_ledger().len(), 1);

        let mut render_session =
            crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
                &root,
                &material_project_with_recipe("render", "render_material"),
            )
            .expect("render project session should form");
        let render_document = default_material_graph_document_for_source_with_target(
            asset_id,
            &source,
            "Rock",
            MaterialOutputTarget::RenderMaterial,
        );
        write_material_graph_document(&root.join(&source.relative_path), &render_document)
            .expect("render source document should write");
        let render = rebuild_material_preview_for_asset(&catalog, &mut render_session, asset_id);

        assert!(render.diagnostics.is_empty(), "{:?}", render.diagnostics);
        assert_eq!(render_session.import_ledger().len(), 2);
        let cache_keys = render_session
            .import_ledger()
            .entries()
            .iter()
            .map(|entry| entry.cache_key.clone())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(cache_keys.len(), 2);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_material_graph_preserves_prior_valid_artifact() {
        let root = unique_temp_dir("material_preview_preserve");
        let asset_id = asset_id(1);
        let source_id = asset_source_id(2);
        let mut session = crate::asset_pipeline::EditorAssetProjectSession::new(
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
        let source = AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/missing.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(source);
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(9),
                asset_id,
                AssetKind::Material,
                ArtifactPayloadKind::FormedMaterialProduct {
                    product_id: "3".to_string(),
                },
                ArtifactCacheKey::new("prior"),
            )
            .with_validity(asset::ArtifactValidity::Valid),
        );

        let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

        assert!(!outcome.diagnostics.is_empty());
        let publication = outcome.publication.expect("prior valid should preserve");
        assert_eq!(
            publication.artifact.validity,
            asset::ArtifactValidity::FailedPreserved
        );
        assert_eq!(publication.artifact.artifact_id, asset_artifact_id(9));
        let _ = std::fs::remove_dir_all(root);
    }

    fn material_project_with_recipe(profile_name: &str, lowering_target: &str) -> ProjectFileV3 {
        let mut project = ProjectFileV3::new("project.material", "Material");
        project
            .import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                AssetKind::MaterialGraph,
                profile_name,
                ImportSettings::MaterialGraph {
                    lowering_target: lowering_target.to_string(),
                },
                AssetKind::Material,
            ));
        project
            .import_profile_defaults
            .push(ProjectImportProfileDefaultV3::new(
                AssetKind::MaterialGraph,
                profile_name,
            ));
        project
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
