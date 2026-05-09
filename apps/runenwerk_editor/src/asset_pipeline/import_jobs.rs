use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use asset::{
    ArtifactPayloadKind, ArtifactValidity, AssetArtifactDescriptor, AssetDiagnosticCode,
    AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetKind, AssetSourceDescriptor, ImportPlan,
    ImportSettings, asset_artifact_id,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportJobStatus {
    Imported,
    Failed,
    FailedPreserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportJobOutcome {
    pub status: ImportJobStatus,
    pub artifact: Option<AssetArtifactDescriptor>,
    pub diagnostics: Vec<AssetDiagnosticRecord>,
}

pub fn run_import_job(
    source: &AssetSourceDescriptor,
    plan: &ImportPlan,
    project_root: &Path,
    cache_root: &Path,
    previous_valid_artifact: Option<&AssetArtifactDescriptor>,
) -> Result<ImportJobOutcome> {
    if source.asset_id != plan.asset_id || source.source_id != plan.source_id {
        bail!("import plan does not match the provided asset source descriptor");
    }

    let source_path = project_root.join(&source.relative_path);
    let source_error = if !source_path.is_file() {
        Some(AssetDiagnosticRecord::error(
            AssetDiagnosticCode::SourceMissing,
            format!("asset source is missing: {}", source.relative_path),
        ))
    } else if source.source_hash != plan.source_hash {
        Some(AssetDiagnosticRecord::error(
            AssetDiagnosticCode::SourceHashMismatch,
            "asset source hash no longer matches the import plan",
        ))
    } else {
        missing_tool_diagnostic(&plan.settings)
    };

    if let Some(diagnostic) = source_error {
        return Ok(failed_import_outcome(previous_valid_artifact, diagnostic));
    }

    std::fs::create_dir_all(cache_root).with_context(|| {
        format!(
            "failed to create asset artifact cache root: {}",
            cache_root.display()
        )
    })?;

    let expected = plan
        .expected_artifacts
        .iter()
        .find(|artifact| artifact.required)
        .or_else(|| plan.expected_artifacts.first())
        .context("import plan must declare at least one expected artifact")?;
    let artifact_path = artifact_path_for_cache_key(project_root, cache_root, plan);
    let artifact = AssetArtifactDescriptor::new(
        asset_artifact_id(plan.job_id.raw()),
        plan.asset_id,
        expected.kind,
        payload_kind_for_import(expected.kind),
        expected.cache_key.clone(),
    )
    .with_source(source.source_id, source.revision_id)
    .with_artifact_path(project_relative_path(project_root, &artifact_path));

    Ok(ImportJobOutcome {
        status: ImportJobStatus::Imported,
        artifact: Some(artifact),
        diagnostics: Vec::new(),
    })
}

fn failed_import_outcome(
    previous_valid_artifact: Option<&AssetArtifactDescriptor>,
    diagnostic: AssetDiagnosticRecord,
) -> ImportJobOutcome {
    if let Some(previous) = previous_valid_artifact {
        let artifact = previous
            .clone()
            .with_validity(ArtifactValidity::FailedPreserved)
            .with_diagnostic(diagnostic.clone());
        ImportJobOutcome {
            status: ImportJobStatus::FailedPreserved,
            artifact: Some(artifact),
            diagnostics: vec![diagnostic],
        }
    } else {
        ImportJobOutcome {
            status: ImportJobStatus::Failed,
            artifact: None,
            diagnostics: vec![diagnostic],
        }
    }
}

fn missing_tool_diagnostic(settings: &ImportSettings) -> Option<AssetDiagnosticRecord> {
    match settings {
        ImportSettings::ForeignBlend {
            blender_executable: None,
            ..
        } => Some(AssetDiagnosticRecord::new(
            AssetDiagnosticCode::ImportToolMissing,
            AssetDiagnosticSeverity::Error,
            "Blender import requires a configured blender executable",
        )),
        _ => None,
    }
}

fn payload_kind_for_import(kind: AssetKind) -> ArtifactPayloadKind {
    match kind {
        AssetKind::FormedFieldProduct => ArtifactPayloadKind::FormedFieldProduct {
            product_id: "pending_field_product".to_string(),
        },
        AssetKind::WorldSdfChunkPageArtifact => {
            ArtifactPayloadKind::WorldSdfPayload { chunk_count: 0 }
        }
        AssetKind::Material | AssetKind::ProceduralMaterial => {
            ArtifactPayloadKind::FormedMaterialProduct {
                product_id: "pending_material_product".to_string(),
            }
        }
        AssetKind::Texture2D | AssetKind::Texture3DVolume => ArtifactPayloadKind::TextureProduct {
            product_id: "pending_texture_product".to_string(),
            dimension: format!("{kind:?}"),
        },
        AssetKind::ProceduralTexture => ArtifactPayloadKind::GeneratedTextureProduct {
            product_id: "pending_generated_texture".to_string(),
        },
        AssetKind::Scene => ArtifactPayloadKind::SceneManifest,
        AssetKind::Shader => ArtifactPayloadKind::ShaderMetadata,
        AssetKind::UiDefinition => ArtifactPayloadKind::UiDefinition,
        AssetKind::ForeignMeshReferenceSource | AssetKind::ForeignMeshReferenceArtifact => {
            ArtifactPayloadKind::ForeignReference {
                format: format!("{kind:?}"),
            }
        }
        _ => ArtifactPayloadKind::DiagnosticCapture,
    }
}

fn artifact_path_for_cache_key(
    project_root: &Path,
    cache_root: &Path,
    plan: &ImportPlan,
) -> PathBuf {
    let file_name = format!(
        "{}.artifact.ron",
        sanitize_cache_key(plan.cache_key.as_str())
    );
    let cache_path = cache_root.join(file_name);
    if cache_path.is_absolute() {
        cache_path
    } else {
        project_root.join(cache_path)
    }
}

fn sanitize_cache_key(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn project_relative_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{
        ArtifactCacheKey, FieldProductResolution, ImportSettings, SourceHash, asset_id,
        asset_source_id, import_job_id,
    };

    #[test]
    fn failed_import_preserves_previous_valid_artifact() {
        let root = unique_temp_dir("runenwerk_missing_import_source");
        let source = AssetSourceDescriptor::new(
            asset_source_id(2),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/fields/missing.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let plan = ImportPlan::deterministic(
            import_job_id(3),
            &source,
            ImportSettings::SdfGraph {
                resolution: FieldProductResolution::new(32, 32, 1),
            },
            AssetKind::FormedFieldProduct,
        );
        let previous = AssetArtifactDescriptor::new(
            asset_artifact_id(5),
            source.asset_id,
            AssetKind::FormedFieldProduct,
            ArtifactPayloadKind::FormedFieldProduct {
                product_id: "previous".to_string(),
            },
            ArtifactCacheKey::new("previous"),
        )
        .with_artifact_path("assets/.cache/previous.artifact.ron");

        let outcome = run_import_job(&source, &plan, &root, &root.join(".cache"), Some(&previous))
            .expect("missing source should report a controlled import failure");

        assert_eq!(outcome.status, ImportJobStatus::FailedPreserved);
        assert_eq!(
            outcome.artifact.as_ref().map(|artifact| artifact.validity),
            Some(ArtifactValidity::FailedPreserved)
        );
        assert_eq!(
            outcome.diagnostics[0].code,
            AssetDiagnosticCode::SourceMissing
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn foreign_blend_without_tool_reports_missing_tool_and_preserves_previous_artifact() {
        let root = unique_temp_dir("runenwerk_blend_missing_tool");
        let source_path = root.join("assets/models/source.blend");
        std::fs::create_dir_all(source_path.parent().unwrap()).expect("model dir should exist");
        std::fs::write(&source_path, b"placeholder").expect("source blend should be writable");
        let source = AssetSourceDescriptor::new(
            asset_source_id(2),
            asset_id(1),
            AssetKind::ForeignMeshReferenceSource,
            "assets/models/source.blend",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let plan = ImportPlan::deterministic(
            import_job_id(7),
            &source,
            ImportSettings::ForeignBlend {
                blender_executable: None,
                export_format: "glb".to_string(),
            },
            AssetKind::ForeignMeshReferenceArtifact,
        );
        let previous = AssetArtifactDescriptor::new(
            asset_artifact_id(9),
            source.asset_id,
            AssetKind::ForeignMeshReferenceArtifact,
            ArtifactPayloadKind::ForeignReference {
                format: "glb".to_string(),
            },
            ArtifactCacheKey::new("previous-glb"),
        )
        .with_artifact_path("assets/.cache/previous.glb");

        let outcome = run_import_job(&source, &plan, &root, &root.join(".cache"), Some(&previous))
            .expect("missing Blender tool should be a controlled import diagnostic");

        assert_eq!(outcome.status, ImportJobStatus::FailedPreserved);
        assert_eq!(
            outcome.diagnostics[0].code,
            AssetDiagnosticCode::ImportToolMissing
        );
        assert_eq!(
            outcome.artifact.as_ref().map(|artifact| artifact.validity),
            Some(ArtifactValidity::FailedPreserved)
        );
        let _ = std::fs::remove_dir_all(root);
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
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
