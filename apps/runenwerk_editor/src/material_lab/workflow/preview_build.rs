use super::*;
use super::artifact_io::{
    canonical_shader_registry_path, catalog_with_material_artifacts, material_artifact_path,
    material_scene_shader_artifact_path, material_shader_artifact_path, write_material_shader_artifact,
};
use super::diagnostics::{material_diagnostic, material_graph_diagnostic};
use super::source_resolution::resolve_material_source_for_asset;

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

    let source = match resolve_material_source_for_asset(catalog, asset_id).cloned() {
        Ok(source) => source,
        Err(diagnostic) => {
            return preserved_or_blocked(catalog, asset_id, diagnostic);
        }
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
    let Some(executable_ir) = product.executable_ir.as_ref() else {
        return blocked_preview("material graph lowering did not produce executable material IR");
    };
    let resolved_resources = match resolve_material_resources(catalog, executable_ir) {
        Ok(resources) => resources,
        Err(diagnostics) => return preserved_or_blocked_many(catalog, asset_id, diagnostics),
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
    let executable_ir = product
        .executable_ir
        .as_ref()
        .expect("executable IR checked before ledger allocation");
    let compiled_shader = match compile_material_shader(MaterialShaderCompileRequest {
        ir: executable_ir,
        fixture: MaterialPreviewFixture::Sphere,
    }) {
        Ok(shader) => shader,
        Err(error) => {
            return preserved_or_blocked(
                catalog,
                asset_id,
                material_diagnostic(
                    AssetDiagnosticCode::RatificationRejected,
                    format!("material shader compilation failed: {error}"),
                ),
            );
        }
    };
    let shader_cache_key = ArtifactCacheKey::new(format!(
        "material-shader-v1:{}:{}",
        artifact_cache_key.as_str(),
        compiled_shader.identity
    ));
    let scene_shader_cache_key = ArtifactCacheKey::new(format!(
        "material-scene-shader-v1:{}:{}",
        artifact_cache_key.as_str(),
        compiled_shader.scene_identity
    ));
    let shader_entry = match session
        .import_ledger_mut()
        .resolve_or_allocate(shader_cache_key.clone(), catalog)
    {
        Ok(entry) => entry,
        Err(diagnostics) => {
            return EditorMaterialPreviewBuildOutcome {
                publication: None,
                diagnostics,
            };
        }
    };
    let scene_shader_entry = match session
        .import_ledger_mut()
        .resolve_or_allocate(scene_shader_cache_key.clone(), catalog)
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
            "failed to persist import execution ledger after shader allocation: {error}"
        ));
    }
    let shader_relative_path = material_shader_artifact_path(&shader_cache_key);
    let shader_absolute_path = session.project_root().join(&shader_relative_path);
    if let Err(error) = write_material_shader_artifact(&shader_absolute_path, &compiled_shader.wgsl)
    {
        return blocked_preview(format!(
            "failed to write generated material shader artifact: {error}"
        ));
    }
    let scene_shader_relative_path = material_scene_shader_artifact_path(&scene_shader_cache_key);
    let scene_shader_absolute_path = session.project_root().join(&scene_shader_relative_path);
    if let Err(error) =
        write_material_shader_artifact(&scene_shader_absolute_path, &compiled_shader.scene_wgsl)
    {
        return blocked_preview(format!(
            "failed to write generated material scene shader artifact: {error}"
        ));
    }
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
    let shader_artifact = AssetArtifactDescriptor::new(
        shader_entry.artifact_id,
        source.asset_id,
        AssetKind::Shader,
        ArtifactPayloadKind::ShaderMetadata,
        shader_cache_key.clone(),
    )
    .with_source(source.source_id, source.revision_id)
    .with_artifact_path(shader_relative_path.clone());
    let scene_shader_artifact = AssetArtifactDescriptor::new(
        scene_shader_entry.artifact_id,
        source.asset_id,
        AssetKind::Shader,
        ArtifactPayloadKind::ShaderMetadata,
        scene_shader_cache_key.clone(),
    )
    .with_source(source.source_id, source.revision_id)
    .with_artifact_path(scene_shader_relative_path.clone());
    if let Err(diagnostics) = catalog_with_material_artifacts(
        catalog,
        [
            artifact.clone(),
            shader_artifact.clone(),
            scene_shader_artifact.clone(),
        ],
    ) {
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
        material_recipe.renderer_parameter_profile,
        shader_entry.artifact_id,
        shader_cache_key,
        canonical_shader_registry_path(session.project_root(), &shader_relative_path),
        compiled_shader.identity,
        scene_shader_entry.artifact_id,
        scene_shader_cache_key,
        canonical_shader_registry_path(session.project_root(), &scene_shader_relative_path),
        compiled_shader.scene_identity,
        resolved_resources,
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
            vec![shader_artifact, scene_shader_artifact],
        )),
        diagnostics: Vec::new(),
    }
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

