//! Catalog-backed model/mesh material region projection.

use std::collections::BTreeSet;

use asset::{
    ArtifactPayloadKind, ArtifactValidity, AssetArtifactDescriptor, AssetArtifactId, AssetCatalog,
    AssetId, AssetKind, AssetRecord, AssetSourceId, AssetSourceRevisionId,
};
use editor_scene::{
    SceneMeshMaterialRegionId, SceneModelMeshMaterialRegionSourceId, SceneModelMeshSourceId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogModelMeshMaterialRegion {
    pub asset_id: AssetId,
    pub asset_stable_name: String,
    pub asset_display_name: String,
    pub artifact_id: AssetArtifactId,
    pub source_id: Option<AssetSourceId>,
    pub source_revision_id: Option<AssetSourceRevisionId>,
    pub source_revision: Option<String>,
    pub material_region_key: String,
    pub display_name: String,
    pub artifact_validity: ArtifactValidity,
    pub blocking_diagnostic: Option<String>,
    pub identity_diagnostic: Option<String>,
}

impl CatalogModelMeshMaterialRegion {
    pub fn is_assignable(&self) -> bool {
        self.artifact_validity == ArtifactValidity::Valid && self.blocking_diagnostic.is_none()
    }

    pub fn display_diagnostic(&self) -> Option<String> {
        self.blocking_diagnostic
            .clone()
            .or_else(|| self.identity_diagnostic.clone())
    }

    pub fn scene_material_region(&self) -> Result<SceneModelMeshMaterialRegionSourceId, String> {
        if let Some(diagnostic) = &self.blocking_diagnostic {
            return Err(diagnostic.clone());
        }
        let source_id = self.source_id.ok_or_else(|| {
            format!(
                "foreign mesh artifact {} has no stable source identity",
                self.artifact_id.raw()
            )
        })?;
        let mut source = SceneModelMeshSourceId::new(self.asset_id, source_id);
        if let Some(revision_id) = self.source_revision_id {
            source = source.with_source_revision_id(revision_id);
        }
        if let Some(revision) = &self.source_revision {
            source = source.with_source_revision(revision.clone());
        }
        let material_region = SceneMeshMaterialRegionId::new(self.material_region_key.clone())?;
        Ok(SceneModelMeshMaterialRegionSourceId::new(
            source,
            material_region,
        ))
    }
}

pub(crate) fn catalog_model_mesh_material_regions(
    catalog: &AssetCatalog,
) -> Vec<CatalogModelMeshMaterialRegion> {
    let mut regions = Vec::new();
    for record in catalog.assets() {
        let mut seen_region_keys = BTreeSet::new();
        for artifact_id in record.artifact_ids.iter().rev() {
            let Some(artifact) = catalog.artifact(*artifact_id) else {
                continue;
            };
            for region in model_mesh_regions_for_artifact(catalog, record, artifact) {
                if seen_region_keys.insert(region.material_region_key.clone()) {
                    regions.push(region);
                }
            }
        }
    }
    regions.sort_by(|left, right| {
        left.asset_stable_name
            .cmp(&right.asset_stable_name)
            .then_with(|| left.artifact_id.cmp(&right.artifact_id))
            .then_with(|| left.material_region_key.cmp(&right.material_region_key))
    });
    regions
}

pub(crate) fn resolve_catalog_model_mesh_material_region(
    catalog: &AssetCatalog,
    asset_id: AssetId,
    material_region_key: &str,
) -> Result<CatalogModelMeshMaterialRegion, String> {
    let record = catalog.asset(asset_id).ok_or_else(|| {
        format!(
            "foreign mesh asset {} is not present in the asset catalog",
            asset_id.raw()
        )
    })?;
    if !matches!(
        record.kind,
        AssetKind::ForeignMeshReferenceSource | AssetKind::ForeignMeshReferenceArtifact
    ) {
        return Err(format!(
            "asset {} is {:?}, not a foreign mesh reference",
            asset_id.raw(),
            record.kind
        ));
    }

    for artifact_id in record.artifact_ids.iter().rev() {
        let Some(artifact) = catalog.artifact(*artifact_id) else {
            continue;
        };
        for region in model_mesh_regions_for_artifact(catalog, record, artifact) {
            if region.material_region_key != material_region_key {
                continue;
            }
            if region.artifact_validity != ArtifactValidity::Valid {
                return Err(format!(
                    "foreign mesh material region '{}' belongs to artifact {} with validity {:?}",
                    material_region_key,
                    region.artifact_id.raw(),
                    region.artifact_validity
                ));
            }
            if let Some(diagnostic) = &region.blocking_diagnostic {
                return Err(diagnostic.clone());
            }
            return Ok(region);
        }
    }

    Err(format!(
        "foreign mesh asset {} does not expose material region '{}'",
        asset_id.raw(),
        material_region_key
    ))
}

fn model_mesh_regions_for_artifact(
    catalog: &AssetCatalog,
    record: &AssetRecord,
    artifact: &AssetArtifactDescriptor,
) -> Vec<CatalogModelMeshMaterialRegion> {
    if artifact.kind != AssetKind::ForeignMeshReferenceArtifact {
        return Vec::new();
    }
    let ArtifactPayloadKind::ForeignReference {
        material_regions, ..
    } = &artifact.payload_kind
    else {
        return Vec::new();
    };

    material_regions
        .iter()
        .map(|region| {
            let material_region_key = region.key.as_str().to_string();
            let key_diagnostic = SceneMeshMaterialRegionId::new(material_region_key.clone())
                .err()
                .map(|error| {
                    format!(
                        "foreign mesh material region '{material_region_key}' is invalid: {error}"
                    )
                });
            let artifact_asset_diagnostic = (artifact.asset_id != record.asset_id).then(|| {
                format!(
                    "foreign mesh artifact {} belongs to asset {}, expected asset {}",
                    artifact.artifact_id.raw(),
                    artifact.asset_id.raw(),
                    record.asset_id.raw()
                )
            });
            let source_diagnostic = source_identity_diagnostic(catalog, record, artifact);
            let blocking_diagnostic = key_diagnostic
                .or(artifact_asset_diagnostic)
                .or(source_diagnostic);
            let identity_diagnostic =
                region.key_source.requires_weak_identity_diagnostic().then(|| {
                    format!(
                        "foreign mesh material region '{}' uses deterministic fallback identity; source-authored material slot metadata is preferred",
                        material_region_key
                    )
                });
            CatalogModelMeshMaterialRegion {
                asset_id: record.asset_id,
                asset_stable_name: record.stable_name.clone(),
                asset_display_name: record.display_name.clone(),
                artifact_id: artifact.artifact_id,
                source_id: artifact.source_id,
                source_revision_id: artifact.source_revision_id,
                source_revision: source_revision_label(catalog, artifact.source_id),
                material_region_key,
                display_name: region.display_name.clone(),
                artifact_validity: artifact.validity,
                blocking_diagnostic,
                identity_diagnostic,
            }
        })
        .collect()
}

fn source_identity_diagnostic(
    catalog: &AssetCatalog,
    record: &AssetRecord,
    artifact: &AssetArtifactDescriptor,
) -> Option<String> {
    let Some(source_id) = artifact.source_id else {
        return Some(format!(
            "foreign mesh artifact {} has no stable source identity",
            artifact.artifact_id.raw()
        ));
    };
    let Some(source) = catalog.source(source_id) else {
        return Some(format!(
            "foreign mesh artifact {} references missing catalog source {}",
            artifact.artifact_id.raw(),
            source_id.raw()
        ));
    };
    if source.asset_id != record.asset_id {
        return Some(format!(
            "foreign mesh artifact {} references source {} owned by asset {}, expected asset {}",
            artifact.artifact_id.raw(),
            source_id.raw(),
            source.asset_id.raw(),
            record.asset_id.raw()
        ));
    }
    if source.kind != AssetKind::ForeignMeshReferenceSource {
        return Some(format!(
            "foreign mesh artifact {} references source {} with kind {:?}, expected ForeignMeshReferenceSource",
            artifact.artifact_id.raw(),
            source_id.raw(),
            source.kind
        ));
    }
    None
}

fn source_revision_label(
    catalog: &AssetCatalog,
    source_id: Option<AssetSourceId>,
) -> Option<String> {
    let source = catalog.source(source_id?)?;
    source
        .source_hash
        .as_ref()
        .map(|hash| format!("{}:{}", hash.algorithm, hash.value))
}
