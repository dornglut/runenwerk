use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    FieldProductDiagnostic, ProductDescriptorCore, ProductIdentity, ProductKind,
    ProductProducerKey, ProductScaleBand, ProductSourceKey,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProductCacheKey(String);

impl ProductCacheKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn from_identity(identity: &ProductCacheIdentity) -> Self {
        Self(identity.stable_key())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProductCacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductCacheIdentity {
    pub product_id: ProductIdentity,
    pub product_kind: ProductKind,
    pub scale_band: ProductScaleBand,
    pub descriptor_generation: u64,
    pub source_revision: Option<String>,
    pub formation_version: Option<String>,
    pub producer: ProductProducerKey,
    pub source_keys: Vec<ProductSourceKey>,
    pub upstream_products: Vec<ProductIdentity>,
}

impl ProductCacheIdentity {
    pub fn from_descriptor(descriptor: &ProductDescriptorCore) -> Self {
        let mut source_keys = descriptor.lineage.source_keys.clone();
        source_keys.sort();
        source_keys.dedup();
        let mut upstream_products = descriptor.lineage.upstream_products.clone();
        upstream_products.sort();
        upstream_products.dedup();

        Self {
            product_id: descriptor.identity,
            product_kind: descriptor.kind.clone(),
            scale_band: descriptor.scale_band,
            descriptor_generation: descriptor.lineage.generation,
            source_revision: descriptor.lineage.source_revision.clone(),
            formation_version: descriptor.lineage.producer_version.clone(),
            producer: descriptor.lineage.producer.clone(),
            source_keys,
            upstream_products,
        }
    }

    pub fn cache_key(&self) -> ProductCacheKey {
        ProductCacheKey::from_identity(self)
    }

    pub fn stable_key(&self) -> String {
        let source_keys = self
            .source_keys
            .iter()
            .map(|key| keyed_part(key.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        let upstream_products = self
            .upstream_products
            .iter()
            .map(|product| product.raw().to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "product-cache:v1|product={}|kind={}|scale={:?}|generation={}|source_revision={}|formation_version={}|producer={}|source_keys=[{}]|upstream=[{}]",
            self.product_id.raw(),
            keyed_part(self.product_kind.as_str()),
            self.scale_band,
            self.descriptor_generation,
            optional_part(self.source_revision.as_deref()),
            optional_part(self.formation_version.as_deref()),
            keyed_part(self.producer.as_str()),
            source_keys,
            upstream_products,
        )
    }
}

fn optional_part(value: Option<&str>) -> String {
    value.map(keyed_part).unwrap_or_else(|| "-".to_string())
}

fn keyed_part(value: &str) -> String {
    format!("{}:{value}", value.len())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductCacheDecisionKind {
    Hit,
    Miss,
    Stale,
    Rejected,
    WriteFailed,
    PreservedLastGood,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductCacheDecision {
    pub kind: ProductCacheDecisionKind,
    pub key: ProductCacheKey,
    pub product_id: Option<ProductIdentity>,
    pub stored_generation: Option<u64>,
    pub requested_generation: Option<u64>,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

impl ProductCacheDecision {
    pub fn new(kind: ProductCacheDecisionKind, key: ProductCacheKey) -> Self {
        Self {
            kind,
            key,
            product_id: None,
            stored_generation: None,
            requested_generation: None,
            diagnostics: Vec::new(),
        }
    }

    pub fn for_identity(mut self, identity: &ProductCacheIdentity) -> Self {
        self.product_id = Some(identity.product_id);
        self.requested_generation = Some(identity.descriptor_generation);
        self
    }

    pub fn with_stored_generation(mut self, generation: u64) -> Self {
        self.stored_generation = Some(generation);
        self
    }

    pub fn with_diagnostics(
        mut self,
        diagnostics: impl IntoIterator<Item = FieldProductDiagnostic>,
    ) -> Self {
        self.diagnostics = diagnostics.into_iter().collect();
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ProductDescriptorCore, ProductFamily, ProductIdentity, ProductKind, ProductLineage,
        ProductScaleBand, ProductScope,
    };

    use super::ProductCacheIdentity;

    #[test]
    fn product_cache_identity_captures_descriptor_lineage() {
        let mut descriptor = ProductDescriptorCore::new(
            ProductIdentity::new(7),
            ProductFamily::Texture,
            ProductKind::new("drawing.ink_tile.rgba8"),
            ProductScope::non_spatial("tile"),
            ProductScaleBand::Preview,
            ProductLineage::new("drawing.ink_tile", 42)
                .with_source_key("drawing.ink_tile.preview:1")
                .with_source_revision("5")
                .with_upstream_product(ProductIdentity::new(3)),
        );
        descriptor.lineage.producer_version = Some("2".to_string());

        let identity = ProductCacheIdentity::from_descriptor(&descriptor);

        assert_eq!(identity.product_id, ProductIdentity::new(7));
        assert_eq!(identity.descriptor_generation, 42);
        assert_eq!(identity.source_revision.as_deref(), Some("5"));
        assert_eq!(identity.formation_version.as_deref(), Some("2"));
        assert!(identity.stable_key().contains("scale=Preview"));
        assert!(
            identity
                .stable_key()
                .contains("producer=16:drawing.ink_tile")
        );
        assert_eq!(identity.cache_key().as_str(), identity.stable_key());
    }
}
