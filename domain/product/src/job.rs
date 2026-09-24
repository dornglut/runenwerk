use serde::{Deserialize, Serialize};

use crate::{
    FieldProductDiagnostic, ProductAuthorityClass, ProductIdentity, ProductJobId, ProductKind,
    ProductProducerKey, ProductScaleBand, ProductScope,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductAccessMode {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductAccessDescriptor {
    pub product_id: ProductIdentity,
    pub mode: ProductAccessMode,
}

impl ProductAccessDescriptor {
    pub const fn read(product_id: ProductIdentity) -> Self {
        Self {
            product_id,
            mode: ProductAccessMode::Read,
        }
    }

    pub const fn write(product_id: ProductIdentity) -> Self {
        Self {
            product_id,
            mode: ProductAccessMode::Write,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductJobAccess {
    pub products: Vec<ProductAccessDescriptor>,
}

impl ProductJobAccess {
    pub fn with_read(mut self, product_id: ProductIdentity) -> Self {
        self.products
            .push(ProductAccessDescriptor::read(product_id));
        self
    }

    pub fn with_write(mut self, product_id: ProductIdentity) -> Self {
        self.products
            .push(ProductAccessDescriptor::write(product_id));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductJobBudgetClass {
    Interactive,
    Background,
    GpuUpload,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductJobAffinity {
    Worker,
    MainThread,
    Background,
    GpuAdjacentPrepare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductDeterminismClass {
    AuthoritativeDeterministic,
    DeterministicLocal,
    VisualOnlyNondeterministicAllowed,
    BackgroundNondeterministicAllowed,
    OfflineDeterministicPreferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductJobFailurePolicy {
    FailPublication,
    PreserveLastValidWithDiagnostic,
    PublishDiagnosticOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductJobDescriptor {
    pub job_id: ProductJobId,
    pub kind: ProductKind,
    pub producer: ProductProducerKey,
    pub input_products: Vec<ProductIdentity>,
    pub output_products: Vec<ProductIdentity>,
    pub scope: ProductScope,
    pub scale_band: ProductScaleBand,
    pub access: ProductJobAccess,
    pub budget_class: ProductJobBudgetClass,
    pub priority: i32,
    pub affinity: ProductJobAffinity,
    pub determinism: ProductDeterminismClass,
    pub authority_class: ProductAuthorityClass,
    pub failure_policy: ProductJobFailurePolicy,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

impl ProductJobDescriptor {
    pub fn new(
        job_id: ProductJobId,
        kind: ProductKind,
        producer: impl Into<String>,
        output_product: ProductIdentity,
        scope: ProductScope,
        scale_band: ProductScaleBand,
    ) -> Self {
        Self {
            job_id,
            kind,
            producer: ProductProducerKey::new(producer),
            input_products: Vec::new(),
            output_products: vec![output_product],
            scope,
            scale_band,
            access: ProductJobAccess::default().with_write(output_product),
            budget_class: ProductJobBudgetClass::Background,
            priority: 0,
            affinity: ProductJobAffinity::Worker,
            determinism: ProductDeterminismClass::DeterministicLocal,
            authority_class: ProductAuthorityClass::DeterministicDerived,
            failure_policy: ProductJobFailurePolicy::PreserveLastValidWithDiagnostic,
            diagnostics: Vec::new(),
        }
    }
}
