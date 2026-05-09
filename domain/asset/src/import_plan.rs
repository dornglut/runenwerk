use serde::{Deserialize, Serialize};

use crate::{
    ArtifactCacheKey, AssetDiagnosticRecord, AssetId, AssetKind, AssetSourceDescriptor,
    AssetSourceId, ImportJobId, ImportSettings, SourceHash,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedArtifact {
    pub kind: AssetKind,
    pub cache_key: ArtifactCacheKey,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportValidationRequirement {
    pub code: String,
    pub description: String,
}

impl ImportValidationRequirement {
    pub fn new(code: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPlan {
    pub job_id: ImportJobId,
    pub asset_id: AssetId,
    pub source_id: AssetSourceId,
    pub source_hash: Option<SourceHash>,
    pub settings: ImportSettings,
    pub expected_artifacts: Vec<ExpectedArtifact>,
    pub dependencies: Vec<AssetId>,
    pub cache_key: ArtifactCacheKey,
    pub validation_requirements: Vec<ImportValidationRequirement>,
    pub expected_diagnostics: Vec<AssetDiagnosticRecord>,
}

impl ImportPlan {
    pub fn deterministic(
        job_id: ImportJobId,
        source: &AssetSourceDescriptor,
        settings: ImportSettings,
        expected_artifact_kind: AssetKind,
    ) -> Self {
        let cache_key = deterministic_cache_key(source, &settings);
        Self {
            job_id,
            asset_id: source.asset_id,
            source_id: source.source_id,
            source_hash: source.source_hash.clone(),
            settings,
            expected_artifacts: vec![ExpectedArtifact {
                kind: expected_artifact_kind,
                cache_key: cache_key.clone(),
                required: true,
            }],
            dependencies: Vec::new(),
            cache_key,
            validation_requirements: vec![ImportValidationRequirement::new(
                "source_hash_matches_plan",
                "source hash must match the deterministic import plan",
            )],
            expected_diagnostics: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, asset_id: AssetId) -> Self {
        self.dependencies.push(asset_id);
        self.dependencies.sort();
        self.dependencies.dedup();
        self
    }
}

pub fn deterministic_cache_key(
    source: &AssetSourceDescriptor,
    settings: &ImportSettings,
) -> ArtifactCacheKey {
    let hash = source
        .source_hash
        .as_ref()
        .map(|hash| format!("{}:{}", hash.algorithm, hash.value))
        .unwrap_or_else(|| "unhashed".to_string());
    ArtifactCacheKey::new(format!(
        "asset-{}-source-{}-{}-{}",
        source.asset_id.raw(),
        source.source_id.raw(),
        settings.stable_kind_label(),
        hash
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssetKind, FieldProductResolution, ImportSettings, SourceHash, asset_id, asset_source_id,
        import_job_id,
    };

    #[test]
    fn import_plan_cache_key_is_deterministic() {
        let source = AssetSourceDescriptor::new(
            asset_source_id(2),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/fields/test.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let settings = ImportSettings::SdfGraph {
            resolution: FieldProductResolution::new(64, 64, 1),
        };

        let first = ImportPlan::deterministic(
            import_job_id(3),
            &source,
            settings.clone(),
            AssetKind::FormedFieldProduct,
        );
        let second = ImportPlan::deterministic(
            import_job_id(3),
            &source,
            settings,
            AssetKind::FormedFieldProduct,
        );

        assert_eq!(first.cache_key, second.cache_key);
        assert_eq!(first.expected_artifacts, second.expected_artifacts);
    }
}
