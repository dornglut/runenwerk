//! File: domain/texture/src/generated.rs
//! Purpose: Generated texture product cache and lineage descriptors.

use product::{
    ProductAuthorityClass, ProductConsumerClass, ProductDescriptorCore, ProductFamily,
    ProductFreshness, ProductIdentity, ProductKind, ProductLineage, ProductQueryPolicy,
    ProductRebuildPolicy, ProductResidency, ProductRetentionPolicy, ProductScaleBand, ProductScope,
};

use crate::{TextureDescriptor, TextureDimension};

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
    pub product_core: ProductDescriptorCore,
    pub cache_key: TextureCacheKey,
    pub lineage: TextureSourceLineage,
}

impl GeneratedTextureProduct {
    pub fn new(
        descriptor: TextureDescriptor,
        cache_key: TextureCacheKey,
        lineage: TextureSourceLineage,
    ) -> Self {
        let product_core = texture_product_core(&descriptor, &cache_key, &lineage);
        Self {
            descriptor,
            product_core,
            cache_key,
            lineage,
        }
    }
}

fn texture_product_core(
    descriptor: &TextureDescriptor,
    cache_key: &TextureCacheKey,
    lineage: &TextureSourceLineage,
) -> ProductDescriptorCore {
    let mut product_lineage =
        ProductLineage::new(lineage.producer.as_str(), 1).with_source_revision(cache_key.as_str());
    product_lineage = product_lineage.with_source_key(format!(
        "texture_source:{}:{}",
        lineage.producer, lineage.source_revision
    ));
    for dependency in &lineage.dependencies {
        product_lineage =
            product_lineage.with_source_key(format!("texture_dependency:{dependency}"));
    }

    let mut core = ProductDescriptorCore::new(
        ProductIdentity::new(descriptor.product_id.raw()),
        ProductFamily::Texture,
        ProductKind::new(texture_product_kind(descriptor)),
        ProductScope::non_spatial(format!("texture:{}", descriptor.label)),
        ProductScaleBand::Preview,
        product_lineage,
    );
    core.freshness = ProductFreshness::Current;
    core.residency = ProductResidency::NotApplicable;
    core.consumer_class = ProductConsumerClass::Renderer;
    core.authority_class = ProductAuthorityClass::DeterministicDerived;
    core.retention_policy = ProductRetentionPolicy::Cacheable;
    core.rebuild_policy = ProductRebuildPolicy::Budgeted;
    core.query_policy = ProductQueryPolicy::VisualFallbackAllowed;
    core
}

fn texture_product_kind(descriptor: &TextureDescriptor) -> String {
    let dimension = match descriptor.dimension {
        TextureDimension::Texture2D => "texture_2d",
        TextureDimension::Texture3DVolume => "texture_3d_volume",
    };
    format!("{dimension}:{}", descriptor.label)
}
