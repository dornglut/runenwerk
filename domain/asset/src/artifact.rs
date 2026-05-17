use serde::{Deserialize, Serialize};
use texture::TextureDescriptor;

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

impl ArtifactValidity {
    pub const fn preserves_prior_valid(self) -> bool {
        matches!(self, Self::FailedPreserved)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetArtifactPreservationError {
    PreviousArtifactNotValid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArtifactPayloadKind {
    FormedFieldProduct {
        product_id: String,
    },
    WorldSdfPayload {
        chunk_count: u32,
    },
    FormedMaterialProduct {
        product_id: String,
    },
    TextureProduct {
        descriptor: TextureDescriptor,
        descriptor_hash: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        artifact_uri: Option<String>,
    },
    GeneratedTextureProduct {
        descriptor: TextureDescriptor,
        descriptor_hash: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        artifact_uri: Option<String>,
    },
    ForeignReference {
        format: String,
    },
    SceneManifest,
    ShaderMetadata,
    UiDefinition,
    DiagnosticCapture,
    RuntimePackage {
        package_id: String,
    },
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

pub fn preserve_prior_valid_artifact(
    previous: &AssetArtifactDescriptor,
    diagnostic: AssetDiagnosticRecord,
) -> AssetArtifactDescriptor {
    try_preserve_prior_valid_artifact(previous, diagnostic)
        .expect("prior-valid artifact preservation requires a valid previous artifact")
}

pub fn try_preserve_prior_valid_artifact(
    previous: &AssetArtifactDescriptor,
    diagnostic: AssetDiagnosticRecord,
) -> Result<AssetArtifactDescriptor, AssetArtifactPreservationError> {
    if previous.validity != ArtifactValidity::Valid {
        return Err(AssetArtifactPreservationError::PreviousArtifactNotValid);
    }
    Ok(previous
        .clone()
        .with_validity(ArtifactValidity::FailedPreserved)
        .with_diagnostic(diagnostic))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetDiagnosticCode, AssetDiagnosticRecord, asset_artifact_id, asset_id};
    use texture::{TextureDescriptor, TextureDimension, TextureExtent, TextureProductId};

    #[test]
    fn preserve_prior_valid_artifact_keeps_identity_and_attaches_diagnostic() {
        let previous = AssetArtifactDescriptor::new(
            asset_artifact_id(7),
            asset_id(2),
            AssetKind::FormedFieldProduct,
            ArtifactPayloadKind::FormedFieldProduct {
                product_id: "previous".to_string(),
            },
            ArtifactCacheKey::new("previous"),
        )
        .with_artifact_path(".runenwerk/artifacts/previous.ron");
        let diagnostic = AssetDiagnosticRecord::error(
            AssetDiagnosticCode::SourceMissing,
            "source file is missing",
        );

        let preserved = preserve_prior_valid_artifact(&previous, diagnostic);

        assert_eq!(preserved.artifact_id, previous.artifact_id);
        assert_eq!(preserved.validity, ArtifactValidity::FailedPreserved);
        assert!(preserved.validity.preserves_prior_valid());
        assert_eq!(preserved.diagnostics.len(), 1);
    }

    #[test]
    fn checked_preservation_rejects_non_valid_previous_artifacts() {
        let previous = AssetArtifactDescriptor::new(
            asset_artifact_id(7),
            asset_id(2),
            AssetKind::FormedFieldProduct,
            ArtifactPayloadKind::FormedFieldProduct {
                product_id: "previous".to_string(),
            },
            ArtifactCacheKey::new("previous"),
        )
        .with_validity(ArtifactValidity::Stale);
        let diagnostic = AssetDiagnosticRecord::error(
            AssetDiagnosticCode::SourceMissing,
            "source file is missing",
        );

        assert_eq!(
            try_preserve_prior_valid_artifact(&previous, diagnostic),
            Err(AssetArtifactPreservationError::PreviousArtifactNotValid)
        );
    }

    #[test]
    fn texture_payload_carries_typed_descriptor_identity() {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(9),
            "albedo",
            TextureDimension::Texture2D,
            TextureExtent::new(64, 64, 1),
        );
        let payload = ArtifactPayloadKind::TextureProduct {
            descriptor_hash: descriptor.descriptor_hash().to_string(),
            descriptor: descriptor.clone(),
            artifact_uri: Some(".runenwerk/artifacts/albedo.ktx2".to_string()),
        };

        let ArtifactPayloadKind::TextureProduct {
            descriptor: carried,
            descriptor_hash,
            artifact_uri,
        } = payload
        else {
            panic!("expected typed texture payload");
        };

        assert_eq!(carried.product_id, TextureProductId::new(9));
        assert_eq!(descriptor_hash, descriptor.descriptor_hash());
        assert_eq!(
            artifact_uri.as_deref(),
            Some(".runenwerk/artifacts/albedo.ktx2")
        );
    }
}
