//! File: domain/texture/src/generated.rs
//! Purpose: Generated texture product cache and lineage descriptors.

use crate::TextureDescriptor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureCacheKey(pub String);

impl TextureCacheKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureSourceLineage {
    pub producer: String,
    pub source_revision: String,
    pub dependencies: Vec<String>,
}

impl TextureSourceLineage {
    pub fn new(producer: impl Into<String>, source_revision: impl Into<String>) -> Self {
        Self {
            producer: producer.into(),
            source_revision: source_revision.into(),
            dependencies: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self.dependencies.sort();
        self.dependencies.dedup();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeneratedTextureProduct {
    pub descriptor: TextureDescriptor,
    pub cache_key: TextureCacheKey,
    pub lineage: TextureSourceLineage,
}

impl GeneratedTextureProduct {
    pub fn new(
        descriptor: TextureDescriptor,
        cache_key: TextureCacheKey,
        lineage: TextureSourceLineage,
    ) -> Self {
        Self {
            descriptor,
            cache_key,
            lineage,
        }
    }
}
