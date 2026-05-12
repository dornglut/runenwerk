use serde::{Deserialize, Serialize};

use crate::{
    FieldProductDiagnostic, ProductConsumerClass, ProductDescriptorCore, ProductFreshness,
    ProductIdentity, ProductQueryPolicy, ProductScope,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuerySnapshotInvalidationPolicy {
    InvalidateOnSourceGenerationChange,
    RetainForFrame,
    RetainForTick,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuerySnapshotProductDescriptor {
    pub descriptor: ProductDescriptorCore,
    pub source_generation: u64,
    pub response_generation: u64,
    pub scope: ProductScope,
    pub freshness: ProductFreshness,
    pub consumer_class: ProductConsumerClass,
    pub requested_policy: ProductQueryPolicy,
    pub invalidation_policy: QuerySnapshotInvalidationPolicy,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

impl QuerySnapshotProductDescriptor {
    pub fn new(
        descriptor: ProductDescriptorCore,
        source_generation: u64,
        response_generation: u64,
        requested_policy: ProductQueryPolicy,
    ) -> Self {
        Self {
            scope: descriptor.scope.clone(),
            freshness: descriptor.freshness,
            consumer_class: descriptor.consumer_class,
            descriptor,
            source_generation,
            response_generation,
            requested_policy,
            invalidation_policy:
                QuerySnapshotInvalidationPolicy::InvalidateOnSourceGenerationChange,
            diagnostics: Vec::new(),
        }
    }

    pub fn product_id(&self) -> ProductIdentity {
        self.descriptor.identity
    }
}
