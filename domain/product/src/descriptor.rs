use serde::{Deserialize, Serialize};

use crate::{
    FieldProductDiagnostic, ProductAuthorityClass, ProductConsumerClass, ProductFreshness,
    ProductIdentity, ProductProducerKey, ProductQueryPolicy, ProductRebuildPolicy,
    ProductResidency, ProductRetentionPolicy, ProductSourceKey,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductFamily {
    SurfaceSdf,
    Material,
    Substance,
    Flow,
    Water,
    Wetness,
    Liquid,
    Process,
    Influence,
    Perception,
    Navigation,
    Gameplay,
    Radiance,
    Atmosphere,
    TimeCelestial,
    Collision,
    Movement,
    Interaction,
    StrictQuery,
    Expression,
    Provenance,
    Diagnostics,
    Texture,
    FamilySpecific,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProductKind(pub String);

impl ProductKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProductScope {
    Field {
        chunk_ids: Vec<String>,
        region_ids: Vec<String>,
    },
    Chunk {
        chunk_ids: Vec<String>,
    },
    Region {
        region_ids: Vec<String>,
    },
    ClipmapWindow {
        window_id: String,
    },
    View {
        view_id: String,
    },
    Sector {
        sector_ids: Vec<String>,
    },
    Portal {
        portal_ids: Vec<String>,
    },
    Basin {
        basin_id: String,
    },
    RiverSegment {
        segment_id: String,
    },
    NonSpatial {
        label: String,
    },
}

impl ProductScope {
    pub fn field(
        chunk_ids: impl IntoIterator<Item = impl Into<String>>,
        region_ids: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let mut chunk_ids = chunk_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut region_ids = region_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        chunk_ids.sort();
        chunk_ids.dedup();
        region_ids.sort();
        region_ids.dedup();
        Self::Field {
            chunk_ids,
            region_ids,
        }
    }

    pub fn chunks(chunk_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut chunk_ids = chunk_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        chunk_ids.sort();
        chunk_ids.dedup();
        Self::Chunk { chunk_ids }
    }

    pub fn regions(region_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut region_ids = region_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        region_ids.sort();
        region_ids.dedup();
        Self::Region { region_ids }
    }

    pub fn non_spatial(label: impl Into<String>) -> Self {
        Self::NonSpatial {
            label: label.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Field {
                chunk_ids,
                region_ids,
            } => {
                chunk_ids.iter().all(|value| value.trim().is_empty())
                    && region_ids.iter().all(|value| value.trim().is_empty())
            }
            Self::Chunk { chunk_ids } => chunk_ids.iter().all(|value| value.trim().is_empty()),
            Self::Region { region_ids } => region_ids.iter().all(|value| value.trim().is_empty()),
            Self::ClipmapWindow { window_id }
            | Self::View { view_id: window_id }
            | Self::Basin {
                basin_id: window_id,
            }
            | Self::RiverSegment {
                segment_id: window_id,
            }
            | Self::NonSpatial { label: window_id } => window_id.trim().is_empty(),
            Self::Sector { sector_ids } => sector_ids.iter().all(|value| value.trim().is_empty()),
            Self::Portal { portal_ids } => portal_ids.iter().all(|value| value.trim().is_empty()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductScaleBand {
    Near,
    Mid,
    Far,
    Summary,
    Preview,
    Final,
    CollisionStrictQuery,
    Offline,
    FamilySpecific,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductLineage {
    pub source_keys: Vec<ProductSourceKey>,
    pub source_revision: Option<String>,
    pub upstream_products: Vec<ProductIdentity>,
    pub producer: ProductProducerKey,
    pub producer_version: Option<String>,
    pub generation: u64,
}

impl ProductLineage {
    pub fn new(producer: impl Into<String>, generation: u64) -> Self {
        Self {
            source_keys: Vec::new(),
            source_revision: None,
            upstream_products: Vec::new(),
            producer: ProductProducerKey::new(producer),
            producer_version: None,
            generation,
        }
    }

    pub fn with_source_key(mut self, source_key: impl Into<String>) -> Self {
        self.source_keys.push(ProductSourceKey::new(source_key));
        self.source_keys.sort();
        self.source_keys.dedup();
        self
    }

    pub fn with_source_revision(mut self, revision: impl Into<String>) -> Self {
        self.source_revision = Some(revision.into());
        self
    }

    pub fn with_upstream_product(mut self, product_id: ProductIdentity) -> Self {
        self.upstream_products.push(product_id);
        self.upstream_products.sort();
        self.upstream_products.dedup();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductDescriptorCore {
    pub identity: ProductIdentity,
    pub family: ProductFamily,
    pub kind: ProductKind,
    pub scope: ProductScope,
    pub scale_band: ProductScaleBand,
    pub lineage: ProductLineage,
    pub freshness: ProductFreshness,
    pub residency: ProductResidency,
    pub consumer_class: ProductConsumerClass,
    pub authority_class: ProductAuthorityClass,
    pub retention_policy: ProductRetentionPolicy,
    pub rebuild_policy: ProductRebuildPolicy,
    pub query_policy: ProductQueryPolicy,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

impl ProductDescriptorCore {
    pub fn new(
        identity: ProductIdentity,
        family: ProductFamily,
        kind: ProductKind,
        scope: ProductScope,
        scale_band: ProductScaleBand,
        lineage: ProductLineage,
    ) -> Self {
        Self {
            identity,
            family,
            kind,
            scope,
            scale_band,
            lineage,
            freshness: ProductFreshness::Current,
            residency: ProductResidency::NotApplicable,
            consumer_class: ProductConsumerClass::Tooling,
            authority_class: ProductAuthorityClass::DeterministicDerived,
            retention_policy: ProductRetentionPolicy::RetainWhileReferenced,
            rebuild_policy: ProductRebuildPolicy::Budgeted,
            query_policy: ProductQueryPolicy::OwnerCustom,
            diagnostics: Vec::new(),
        }
    }

    pub fn query_policy_allows_consumption(&self) -> bool {
        self.query_policy
            .allows(self.freshness, self.residency, self.authority_class)
    }

    pub fn cache_identity(&self) -> crate::ProductCacheIdentity {
        crate::ProductCacheIdentity::from_descriptor(self)
    }
}
