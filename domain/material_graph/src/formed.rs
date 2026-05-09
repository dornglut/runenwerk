//! File: domain/material_graph/src/formed.rs
//! Purpose: Formed material product descriptors and source maps.

use graph::NodeId;

use crate::{MaterialGraphDocumentId, MaterialOutputTarget, MaterialProductId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialCacheKey(pub String);

impl MaterialCacheKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialSpecializationFragment(pub String);

impl MaterialSpecializationFragment {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialParameterKind {
    Scalar,
    Vector2,
    Vector3,
    Vector4,
    Texture2D,
    Texture3D,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialParameterDescriptor {
    pub key: String,
    pub kind: MaterialParameterKind,
}

impl MaterialParameterDescriptor {
    pub fn new(key: impl Into<String>, kind: MaterialParameterKind) -> Self {
        Self {
            key: key.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialSourceMapEntry {
    pub node_id: NodeId,
    pub role: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct MaterialSourceMap {
    pub entries: Vec<MaterialSourceMapEntry>,
}

impl MaterialSourceMap {
    pub fn from_nodes(nodes: impl IntoIterator<Item = NodeId>) -> Self {
        Self {
            entries: nodes
                .into_iter()
                .map(|node_id| MaterialSourceMapEntry {
                    node_id,
                    role: "semantic_node".to_string(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FormedMaterialProduct {
    pub product_id: MaterialProductId,
    pub source_document_id: MaterialGraphDocumentId,
    pub output_target: MaterialOutputTarget,
    pub parameters: Vec<MaterialParameterDescriptor>,
    pub source_map: MaterialSourceMap,
    pub specialization_fragment: MaterialSpecializationFragment,
    pub cache_key: MaterialCacheKey,
}

impl FormedMaterialProduct {
    pub fn new(
        product_id: MaterialProductId,
        source_document_id: MaterialGraphDocumentId,
        output_target: MaterialOutputTarget,
        cache_key: MaterialCacheKey,
    ) -> Self {
        Self {
            product_id,
            source_document_id,
            output_target,
            parameters: Vec::new(),
            source_map: MaterialSourceMap::default(),
            specialization_fragment: MaterialSpecializationFragment::new("material.first_slice"),
            cache_key,
        }
    }
}
