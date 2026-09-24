use serde::{Deserialize, Serialize};

use crate::{
    AssetId, AssetKind, AssetSourceId, AssetSourceRevisionId, AssetSourceRootId,
    asset_source_revision_id,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetSourceRootKind {
    ProjectAssets,
    GameAssets,
    ExternalReference,
    GeneratedCache,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSourceRoot {
    pub root_id: AssetSourceRootId,
    pub kind: AssetSourceRootKind,
    pub display_name: String,
    pub relative_path: String,
}

impl AssetSourceRoot {
    pub fn new(
        root_id: AssetSourceRootId,
        kind: AssetSourceRootKind,
        display_name: impl Into<String>,
        relative_path: impl Into<String>,
    ) -> Self {
        Self {
            root_id,
            kind,
            display_name: display_name.into(),
            relative_path: relative_path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceHash {
    pub algorithm: String,
    pub value: String,
}

impl SourceHash {
    pub fn new(algorithm: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AssetImporterId(String);

impl AssetImporterId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssetSourceProvenance {
    Authored,
    ExternalReference { tool: Option<String> },
    Migrated { from_version: u32 },
    Generated { producer: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthoredFieldDocumentKind {
    SdfGraph,
    SdfBrushLayer,
    FieldWorldDefinition,
    WorldEditLog,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSourceDescriptor {
    pub source_id: AssetSourceId,
    pub asset_id: AssetId,
    pub kind: AssetKind,
    pub source_root_id: Option<AssetSourceRootId>,
    pub relative_path: String,
    pub source_hash: Option<SourceHash>,
    #[serde(default)]
    pub importer_id: Option<AssetImporterId>,
    pub provenance: AssetSourceProvenance,
    pub authored_field_document_kind: Option<AuthoredFieldDocumentKind>,
    pub revision_id: AssetSourceRevisionId,
}

impl AssetSourceDescriptor {
    pub fn new(
        source_id: AssetSourceId,
        asset_id: AssetId,
        kind: AssetKind,
        relative_path: impl Into<String>,
    ) -> Self {
        let authored_field_document_kind = match kind {
            AssetKind::SdfGraph => Some(AuthoredFieldDocumentKind::SdfGraph),
            AssetKind::SdfBrushLayer => Some(AuthoredFieldDocumentKind::SdfBrushLayer),
            AssetKind::FieldWorldDefinition => {
                Some(AuthoredFieldDocumentKind::FieldWorldDefinition)
            }
            AssetKind::WorldEditLog => Some(AuthoredFieldDocumentKind::WorldEditLog),
            _ => None,
        };
        Self {
            source_id,
            asset_id,
            kind,
            source_root_id: None,
            relative_path: relative_path.into(),
            source_hash: None,
            importer_id: None,
            provenance: AssetSourceProvenance::Authored,
            authored_field_document_kind,
            revision_id: asset_source_revision_id(1),
        }
    }

    pub fn with_hash(mut self, source_hash: SourceHash) -> Self {
        self.source_hash = Some(source_hash);
        self
    }

    pub fn with_source_root(mut self, source_root_id: AssetSourceRootId) -> Self {
        self.source_root_id = Some(source_root_id);
        self
    }

    pub fn with_importer(mut self, importer_id: AssetImporterId) -> Self {
        self.importer_id = Some(importer_id);
        self
    }

    pub fn with_provenance(mut self, provenance: AssetSourceProvenance) -> Self {
        self.provenance = provenance;
        self
    }
}
