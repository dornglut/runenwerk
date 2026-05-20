use super::diagnostics::material_diagnostic;
use super::*;

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

pub fn catalog_with_material_artifacts(
    catalog: &AssetCatalog,
    artifacts: impl IntoIterator<Item = AssetArtifactDescriptor>,
) -> Result<AssetCatalog, Vec<AssetDiagnosticRecord>> {
    let mut candidate = catalog.clone();
    for artifact in artifacts {
        candidate.insert_artifact(artifact);
    }
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

pub(super) fn material_artifact_path(artifact_id: &asset::AssetArtifactId) -> String {
    format!(
        ".runenwerk/artifacts/material-preview-{}.artifact.ron",
        artifact_id.raw()
    )
}

pub(super) fn material_shader_artifact_path(cache_key: &ArtifactCacheKey) -> String {
    content_addressed_artifact_path("material-shader", cache_key, "wgsl")
}

pub(super) fn material_scene_shader_artifact_path(cache_key: &ArtifactCacheKey) -> String {
    content_addressed_artifact_path("material-scene-shader", cache_key, "wgsl")
}

pub(super) fn content_addressed_artifact_path(
    prefix: &str,
    cache_key: &ArtifactCacheKey,
    ext: &str,
) -> String {
    let digest = blake3::hash(cache_key.as_str().as_bytes());
    format!(
        ".runenwerk/artifacts/generated/{prefix}/{}.{}",
        digest.to_hex(),
        ext
    )
}

pub(super) fn write_material_shader_artifact(path: &Path, wgsl: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, wgsl)?;
    Ok(())
}

pub(super) fn canonical_shader_registry_path(project_root: &Path, relative_path: &str) -> String {
    project_root
        .join(relative_path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[allow(dead_code)]
pub(super) fn project_relative_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[allow(dead_code)]
pub(super) fn absolute_source_path(project_root: &Path, source: &AssetSourceDescriptor) -> PathBuf {
    project_root.join(&source.relative_path)
}
