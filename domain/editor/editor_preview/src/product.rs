use asset::{AssetArtifactId, AssetArtifactRevisionId, AssetId, AssetKind};
use serde::{Deserialize, Serialize};
use world_sdf::{FieldProductId, SdfChunkPayload, WorldSdfPayloadRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProductKind {
    Scene,
    FieldProduct,
    ProcgenPreview,
    WorldSdfPayload,
    Material,
    Texture,
    Shader,
    UiDefinition,
    UnsupportedFutureKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProductRef {
    pub kind: RuntimeProductKind,
    pub asset_kind: Option<AssetKind>,
    pub asset_id: Option<AssetId>,
    pub artifact_id: Option<AssetArtifactId>,
    pub artifact_revision: Option<AssetArtifactRevisionId>,
    pub field_product_id: Option<FieldProductId>,
    pub label: String,
    pub payload_refs: Vec<WorldSdfPayloadRef>,
}

impl RuntimeProductRef {
    pub fn new(kind: RuntimeProductKind, label: impl Into<String>) -> Self {
        Self {
            kind,
            asset_kind: None,
            asset_id: None,
            artifact_id: None,
            artifact_revision: None,
            field_product_id: None,
            label: label.into(),
            payload_refs: Vec::new(),
        }
    }

    pub fn for_asset(mut self, asset_id: AssetId, asset_kind: AssetKind) -> Self {
        self.asset_id = Some(asset_id);
        self.asset_kind = Some(asset_kind);
        self
    }

    pub fn with_artifact(
        mut self,
        artifact_id: AssetArtifactId,
        revision: AssetArtifactRevisionId,
    ) -> Self {
        self.artifact_id = Some(artifact_id);
        self.artifact_revision = Some(revision);
        self
    }

    pub fn with_field_product(mut self, product_id: FieldProductId) -> Self {
        self.field_product_id = Some(product_id);
        self
    }

    pub fn with_payload_refs(mut self, payload_refs: Vec<WorldSdfPayloadRef>) -> Self {
        self.payload_refs = payload_refs;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSdfPayloadPackage {
    pub product_ref: RuntimeProductRef,
    pub payloads: Vec<SdfChunkPayload>,
}

impl WorldSdfPayloadPackage {
    pub fn new(product_ref: RuntimeProductRef, payloads: Vec<SdfChunkPayload>) -> Self {
        Self {
            product_ref,
            payloads,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProductPayload {
    Descriptor(RuntimeProductRef),
    WorldSdf(WorldSdfPayloadPackage),
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::asset_id;

    #[test]
    fn product_ref_keeps_asset_identity_typed() {
        let product = RuntimeProductRef::new(RuntimeProductKind::Scene, "scene")
            .for_asset(asset_id(42), AssetKind::Scene);

        assert_eq!(product.asset_id, Some(asset_id(42)));
        assert_eq!(product.asset_kind, Some(AssetKind::Scene));
    }
}
