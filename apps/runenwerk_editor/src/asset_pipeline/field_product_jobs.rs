use std::path::{Path, PathBuf};

use anyhow::Result;
use asset::{
    ArtifactPayloadKind, AssetArtifactDescriptor, AssetKind, AssetSourceDescriptor, ImportPlan,
    ImportSettings, asset_artifact_id, ratify_asset_artifact,
};
use spatial::{ChunkCoord3, ChunkId, WorldId};
use world_sdf::{
    FieldProductCandidate, FieldProductDescriptor, FieldProductId, FieldProductKind,
    FieldProductLineage, FieldProductScope, ratify_field_product_candidate,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldProductJobOutcome {
    pub candidate: FieldProductCandidate,
    pub artifact: AssetArtifactDescriptor,
    pub field_product_ratified: bool,
    pub asset_artifact_ratified: bool,
}

pub fn run_field_product_job(
    source: &AssetSourceDescriptor,
    plan: &ImportPlan,
    project_root: &Path,
    cache_root: &Path,
) -> Result<FieldProductJobOutcome> {
    let chunk = ChunkId::new(WorldId(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
    let mut lineage = FieldProductLineage::new(
        source.revision_id.raw(),
        "runenwerk_editor.field_product_jobs",
    );
    lineage.source_asset_id = Some(source.asset_id.raw());
    let mut descriptor = FieldProductDescriptor::new(
        FieldProductId(plan.job_id.raw()),
        product_kind_for_import(&plan.settings),
        FieldProductScope::from_chunks([chunk]),
        lineage,
    );
    descriptor.scale_band = scale_band_for_import(&plan.settings);

    let candidate = FieldProductCandidate::new(descriptor);
    let artifact_path = field_product_artifact_path(project_root, cache_root, plan);
    let artifact = AssetArtifactDescriptor::new(
        asset_artifact_id(plan.job_id.raw()),
        source.asset_id,
        AssetKind::FormedFieldProduct,
        ArtifactPayloadKind::FormedFieldProduct {
            product_id: candidate.descriptor.product_id.0.to_string(),
        },
        plan.cache_key.clone(),
    )
    .with_source(source.source_id, source.revision_id)
    .with_artifact_path(project_relative_path(project_root, &artifact_path));

    let field_report = ratify_field_product_candidate(&candidate);
    let artifact_report = ratify_asset_artifact(&artifact);

    Ok(FieldProductJobOutcome {
        candidate,
        artifact,
        field_product_ratified: !field_report.has_blocking_issues(),
        asset_artifact_ratified: !artifact_report.has_blocking_issues(),
    })
}

fn product_kind_for_import(settings: &ImportSettings) -> FieldProductKind {
    match settings {
        ImportSettings::WorldSdfProduct { .. } => FieldProductKind::WorldSdfChunkPages,
        ImportSettings::SdfBrushLayer { .. } => FieldProductKind::OccupancySupport,
        _ => FieldProductKind::ScalarDistance,
    }
}

fn scale_band_for_import(settings: &ImportSettings) -> String {
    match settings {
        ImportSettings::WorldSdfProduct { scale_band, .. } => scale_band.clone(),
        _ => "preview".to_string(),
    }
}

fn field_product_artifact_path(
    project_root: &Path,
    cache_root: &Path,
    plan: &ImportPlan,
) -> PathBuf {
    let file_name = format!(
        "{}.field-product.ron",
        plan.cache_key
            .as_str()
            .chars()
            .map(
                |ch| if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                    ch
                } else {
                    '_'
                }
            )
            .collect::<String>()
    );
    let path = cache_root.join(file_name);
    if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    }
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
        FieldProductResolution, ImportSettings, SourceHash, asset_id, asset_source_id,
        import_job_id,
    };

    #[test]
    fn sdf_source_forms_ratified_field_product_candidate() {
        let root = unique_temp_dir("runenwerk_field_product_job");
        let source = AssetSourceDescriptor::new(
            asset_source_id(2),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/fields/brush.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let plan = ImportPlan::deterministic(
            import_job_id(3),
            &source,
            ImportSettings::SdfGraph {
                resolution: FieldProductResolution::new(64, 64, 1),
            },
            AssetKind::FormedFieldProduct,
        );

        let outcome = run_field_product_job(&source, &plan, &root, &root.join(".cache"))
            .expect("field product job should form a preview candidate");

        assert!(outcome.field_product_ratified);
        assert!(outcome.asset_artifact_ratified);
        assert_eq!(
            outcome.artifact.payload_kind,
            ArtifactPayloadKind::FormedFieldProduct {
                product_id: "3".to_string()
            }
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
