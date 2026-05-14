use serde::{Deserialize, Serialize};

use crate::{
    ProductDescriptorCore, ProductIdentity, ProductKind, ProductProducerKey, ProductScaleBand,
    ProductSourceKey,
};

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
    }
}
