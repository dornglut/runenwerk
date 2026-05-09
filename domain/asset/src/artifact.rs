use serde::{Deserialize, Serialize};

use crate::{
    AssetArtifactId, AssetArtifactRevisionId, AssetDiagnosticRecord, AssetId, AssetKind,
    AssetSourceId, AssetSourceRevisionId, asset_artifact_revision_id,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArtifactCacheKey(pub String);

impl ArtifactCacheKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactValidity {
    Valid,
    Stale,
    FailedPreserved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArtifactPayloadKind {
    FormedFieldProduct { product_id: String },
    WorldSdfPayload { chunk_count: u32 },
    ForeignReference { format: String },
    SceneManifest,
    ShaderMetadata,
    UiDefinition,
    DiagnosticCapture,
    RuntimePackage { package_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetArtifactDescriptor {
    pub artifact_id: AssetArtifactId,
    pub asset_id: AssetId,
    pub source_id: Option<AssetSourceId>,
    pub kind: AssetKind,
    pub payload_kind: ArtifactPayloadKind,
    pub cache_key: ArtifactCacheKey,
    pub artifact_path: Option<String>,
    pub source_revision_id: Option<AssetSourceRevisionId>,
    pub artifact_revision_id: AssetArtifactRevisionId,
    pub validity: ArtifactValidity,
    pub diagnostics: Vec<AssetDiagnosticRecord>,
}

impl AssetArtifactDescriptor {
    pub fn new(
        artifact_id: AssetArtifactId,
        asset_id: AssetId,
        kind: AssetKind,
        payload_kind: ArtifactPayloadKind,
        cache_key: ArtifactCacheKey,
    ) -> Self {
        Self {
            artifact_id,
            asset_id,
            source_id: None,
            kind,
            payload_kind,
            cache_key,
            artifact_path: None,
            source_revision_id: None,
            artifact_revision_id: asset_artifact_revision_id(1),
            validity: ArtifactValidity::Valid,
            diagnostics: Vec::new(),
        }
    }

    pub fn with_source(
        mut self,
        source_id: AssetSourceId,
        revision_id: AssetSourceRevisionId,
    ) -> Self {
        self.source_id = Some(source_id);
        self.source_revision_id = Some(revision_id);
        self
    }

    pub fn with_artifact_path(mut self, artifact_path: impl Into<String>) -> Self {
        self.artifact_path = Some(artifact_path.into());
        self
    }

    pub fn with_validity(mut self, validity: ArtifactValidity) -> Self {
        self.validity = validity;
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: AssetDiagnosticRecord) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }
}
