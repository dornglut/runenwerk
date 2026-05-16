use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    AssetArtifactDescriptor, AssetArtifactId, AssetDependencyGraph, AssetId, AssetKind,
    AssetRevisionId, AssetSourceDescriptor, AssetSourceId, AssetSourceRoot, AssetSourceRootId,
    asset_revision_id,
};

pub const ASSET_CATALOG_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRecord {
    pub asset_id: AssetId,
    pub stable_name: String,
    pub display_name: String,
    pub kind: AssetKind,
    pub primary_source_id: Option<AssetSourceId>,
    pub artifact_ids: Vec<AssetArtifactId>,
    pub revision_id: AssetRevisionId,
}

impl AssetRecord {
    pub fn new(
        asset_id: AssetId,
        stable_name: impl Into<String>,
        display_name: impl Into<String>,
        kind: AssetKind,
    ) -> Self {
        Self {
            asset_id,
            stable_name: stable_name.into(),
            display_name: display_name.into(),
            kind,
            primary_source_id: None,
            artifact_ids: Vec::new(),
            revision_id: asset_revision_id(1),
        }
    }

    pub fn with_primary_source(mut self, source_id: AssetSourceId) -> Self {
        self.primary_source_id = Some(source_id);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetCatalog {
    pub version: u32,
    pub records: BTreeMap<AssetId, AssetRecord>,
    #[serde(default)]
    pub source_roots: BTreeMap<AssetSourceRootId, AssetSourceRoot>,
    pub sources: BTreeMap<AssetSourceId, AssetSourceDescriptor>,
    pub artifacts: BTreeMap<AssetArtifactId, AssetArtifactDescriptor>,
    pub dependency_graph: AssetDependencyGraph,
}

impl Default for AssetCatalog {
    fn default() -> Self {
        Self {
            version: ASSET_CATALOG_VERSION_V1,
            records: BTreeMap::new(),
            source_roots: BTreeMap::new(),
            sources: BTreeMap::new(),
            artifacts: BTreeMap::new(),
            dependency_graph: AssetDependencyGraph::new(),
        }
    }
}

impl AssetCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_asset_record(&mut self, record: AssetRecord) -> Option<AssetRecord> {
        self.records.insert(record.asset_id, record)
    }

    pub fn insert_source_root(&mut self, root: AssetSourceRoot) -> Option<AssetSourceRoot> {
        self.source_roots.insert(root.root_id, root)
    }

    pub fn insert_source(
        &mut self,
        source: AssetSourceDescriptor,
    ) -> Option<AssetSourceDescriptor> {
        self.sources.insert(source.source_id, source)
    }

    pub fn insert_artifact(
        &mut self,
        artifact: AssetArtifactDescriptor,
    ) -> Option<AssetArtifactDescriptor> {
        if let Some(record) = self.records.get_mut(&artifact.asset_id)
            && !record.artifact_ids.contains(&artifact.artifact_id)
        {
            record.artifact_ids.push(artifact.artifact_id);
        }
        self.artifacts.insert(artifact.artifact_id, artifact)
    }

    pub fn asset(&self, asset_id: AssetId) -> Option<&AssetRecord> {
        self.records.get(&asset_id)
    }

    pub fn source(&self, source_id: AssetSourceId) -> Option<&AssetSourceDescriptor> {
        self.sources.get(&source_id)
    }

    pub fn source_root(&self, root_id: AssetSourceRootId) -> Option<&AssetSourceRoot> {
        self.source_roots.get(&root_id)
    }

    pub fn artifact(&self, artifact_id: AssetArtifactId) -> Option<&AssetArtifactDescriptor> {
        self.artifacts.get(&artifact_id)
    }

    pub fn assets(&self) -> impl Iterator<Item = &AssetRecord> {
        self.records.values()
    }
}
