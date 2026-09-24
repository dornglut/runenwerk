use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use asset::{
    ArtifactPayloadKind, AssetArtifactDescriptor, AssetArtifactId, AssetDiagnosticCode,
    AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetKind, AssetSourceDescriptor,
    ForeignMeshMaterialRegionDescriptor, ImportPlan, ImportSettings,
    try_preserve_prior_valid_artifact,
};
use texture::{
    TextureChannelLayout, TextureColorSpace, TextureCompression, TextureDescriptor,
    TextureDimension, TextureExtent, TextureProductId,
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
    artifact_id: AssetArtifactId,
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
        artifact_id,
        plan.asset_id,
        expected.kind,
        payload_kind_for_import(expected.kind, &plan.settings, artifact_id),
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
        let Ok(artifact) = try_preserve_prior_valid_artifact(previous, diagnostic.clone()) else {
            return ImportJobOutcome {
                status: ImportJobStatus::Failed,
                artifact: None,
                diagnostics: vec![diagnostic],
            };
        };
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

fn payload_kind_for_import(
    kind: AssetKind,
    settings: &ImportSettings,
    artifact_id: AssetArtifactId,
) -> ArtifactPayloadKind {
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
        AssetKind::Texture2D | AssetKind::Texture3DVolume => {
            let descriptor = texture_descriptor_for_import(kind, settings, artifact_id);
            ArtifactPayloadKind::TextureProduct {
                descriptor_hash: descriptor.descriptor_hash().to_string(),
                descriptor,
                artifact_uri: None,
            }
        }
        AssetKind::ProceduralTexture => {
            let descriptor = texture_descriptor_for_import(kind, settings, artifact_id);
            ArtifactPayloadKind::GeneratedTextureProduct {
                descriptor_hash: descriptor.descriptor_hash().to_string(),
                descriptor,
                artifact_uri: None,
            }
        }
        AssetKind::Scene => ArtifactPayloadKind::SceneManifest,
        AssetKind::Shader => ArtifactPayloadKind::ShaderMetadata,
        AssetKind::UiDefinition => ArtifactPayloadKind::UiDefinition,
        AssetKind::ForeignMeshReferenceSource | AssetKind::ForeignMeshReferenceArtifact => {
            ArtifactPayloadKind::ForeignReference {
                format: foreign_reference_format(kind, settings),
                material_regions: foreign_mesh_material_regions_for_import(kind),
            }
        }
        _ => ArtifactPayloadKind::DiagnosticCapture,
    }
}

fn foreign_reference_format(kind: AssetKind, settings: &ImportSettings) -> String {
    match settings {
        ImportSettings::ForeignBlend { export_format, .. } => export_format.clone(),
        ImportSettings::ForeignGltf => "gltf".to_string(),
        _ => format!("{kind:?}"),
    }
}

fn foreign_mesh_material_regions_for_import(
    kind: AssetKind,
) -> Vec<ForeignMeshMaterialRegionDescriptor> {
    if kind != AssetKind::ForeignMeshReferenceArtifact {
        return Vec::new();
    }
    vec![ForeignMeshMaterialRegionDescriptor::source_material_slot(
        0,
        Some("Default material"),
    )]
}

fn texture_descriptor_for_import(
    kind: AssetKind,
    settings: &ImportSettings,
    artifact_id: AssetArtifactId,
) -> TextureDescriptor {
    let product_id = TextureProductId::new(artifact_id.raw());
    match (kind, settings) {
        (
            AssetKind::Texture3DVolume,
            ImportSettings::Texture3DVolume {
                resolution,
                color_space,
                compression,
            },
        ) => TextureDescriptor::new(
            product_id,
            format!("texture.artifact.{}", artifact_id.raw()),
            TextureDimension::Texture3DVolume,
            TextureExtent::new(resolution.width, resolution.height, resolution.depth),
        )
        .with_color_space(texture_color_space(*color_space))
        .with_compression(texture_compression(*compression)),
        (
            AssetKind::ProceduralTexture,
            ImportSettings::ProceduralTexture {
                resolution,
                color_space,
            },
        ) => TextureDescriptor::new(
            product_id,
            format!("texture.artifact.{}", artifact_id.raw()),
            TextureDimension::Texture2D,
            TextureExtent::new(resolution.width, resolution.height, resolution.depth),
        )
        .with_color_space(texture_color_space(*color_space)),
        (
            AssetKind::Texture2D,
            ImportSettings::Texture2D {
                color_space,
                compression,
            },
        ) => TextureDescriptor::new(
            product_id,
            format!("texture.artifact.{}", artifact_id.raw()),
            TextureDimension::Texture2D,
            TextureExtent::new(512, 512, 1),
        )
        .with_color_space(texture_color_space(*color_space))
        .with_compression(texture_compression(*compression)),
        (AssetKind::Texture3DVolume, _) => TextureDescriptor::new(
            product_id,
            format!("texture.artifact.{}", artifact_id.raw()),
            TextureDimension::Texture3DVolume,
            TextureExtent::new(64, 64, 64),
        )
        .with_color_space(TextureColorSpace::Data),
        _ => TextureDescriptor::new(
            product_id,
            format!("texture.artifact.{}", artifact_id.raw()),
            TextureDimension::Texture2D,
            TextureExtent::new(512, 512, 1),
        )
        .with_channel_layout(TextureChannelLayout::Rgba),
    }
}

fn texture_color_space(color_space: asset::TextureImportColorSpace) -> TextureColorSpace {
    match color_space {
        asset::TextureImportColorSpace::Linear => TextureColorSpace::Linear,
        asset::TextureImportColorSpace::Srgb => TextureColorSpace::Srgb,
        asset::TextureImportColorSpace::Data => TextureColorSpace::Data,
    }
}

fn texture_compression(compression: asset::TextureImportCompression) -> TextureCompression {
    match compression {
        asset::TextureImportCompression::Uncompressed => TextureCompression::Uncompressed,
        asset::TextureImportCompression::Bc5 => TextureCompression::Bc5,
        asset::TextureImportCompression::Bc7 => TextureCompression::Bc7,
        asset::TextureImportCompression::Astc => TextureCompression::Astc,
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
        ArtifactCacheKey, ArtifactValidity, FieldProductResolution, ImportSettings, SourceHash,
        asset_artifact_id, asset_id, asset_source_id, import_job_id,
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

        let outcome = run_import_job(
            &source,
            &plan,
            &root,
            &root.join(".cache"),
            asset_artifact_id(3),
            Some(&previous),
        )
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
                material_regions: vec![ForeignMeshMaterialRegionDescriptor::source_material_slot(
                    0,
                    Some("Default material"),
                )],
            },
            ArtifactCacheKey::new("previous-glb"),
        )
        .with_artifact_path("assets/.cache/previous.glb");

        let outcome = run_import_job(
            &source,
            &plan,
            &root,
            &root.join(".cache"),
            asset_artifact_id(7),
            Some(&previous),
        )
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

    #[test]
    fn foreign_mesh_import_exposes_stable_material_region_key() {
        let root = unique_temp_dir("runenwerk_gltf_material_regions");
        let source_path = root.join("assets/models/source.gltf");
        std::fs::create_dir_all(source_path.parent().unwrap()).expect("model dir should exist");
        std::fs::write(&source_path, b"placeholder").expect("source gltf should be writable");
        let source = AssetSourceDescriptor::new(
            asset_source_id(2),
            asset_id(1),
            AssetKind::ForeignMeshReferenceSource,
            "assets/models/source.gltf",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let plan = ImportPlan::deterministic(
            import_job_id(8),
            &source,
            ImportSettings::ForeignGltf,
            AssetKind::ForeignMeshReferenceArtifact,
        );

        let outcome = run_import_job(
            &source,
            &plan,
            &root,
            &root.join(".cache"),
            asset_artifact_id(8),
            None,
        )
        .expect("foreign gltf import should form a controlled placeholder artifact");

        assert_eq!(outcome.status, ImportJobStatus::Imported);
        let artifact = outcome.artifact.expect("artifact should be published");
        let ArtifactPayloadKind::ForeignReference {
            format,
            material_regions,
        } = artifact.payload_kind
        else {
            panic!("foreign mesh import should publish a foreign reference payload");
        };
        assert_eq!(format, "gltf");
        assert_eq!(material_regions.len(), 1);
        assert_eq!(material_regions[0].key.as_str(), "source_material_slot:0");
        assert!(
            !material_regions[0]
                .key_source
                .requires_weak_identity_diagnostic()
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
