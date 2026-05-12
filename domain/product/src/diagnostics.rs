use serde::{Deserialize, Serialize};

use crate::{ProductConsumerClass, ProductFamily, ProductIdentity, ProductScaleBand};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldProductDiagnosticSeverity {
    Info,
    Warning,
    Error,
    Blocking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldProductDiagnosticCode {
    MissingProduct,
    DeclaredNotFormed,
    RetiredSelected,
    StaleProduct,
    PotentiallyStale,
    GenerationMismatch,
    NonResident,
    PendingResidency,
    FallbackUsed,
    GhostSummaryUsed,
    FormationFailure,
    FailedPreservedOutput,
    RebuildBudgetExhausted,
    MissingDependency,
    AmbiguousLineage,
    UndeclaredDependency,
    UnsupportedConsumerRequest,
    InvalidScaleBand,
    StrictFallbackRejected,
    VisualOnlyUsedForStrictQuery,
    DerivedProductUsedAsAuthority,
    GhostSummaryUsedForAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldProductDiagnostic {
    pub code: FieldProductDiagnosticCode,
    pub severity: FieldProductDiagnosticSeverity,
    pub product_id: Option<ProductIdentity>,
    pub family: Option<ProductFamily>,
    pub scale_band: Option<ProductScaleBand>,
    pub consumer_class: Option<ProductConsumerClass>,
    pub generation: Option<u64>,
    pub message: String,
    pub cause: String,
    pub suggested_action: String,
    pub related_products: Vec<ProductIdentity>,
}

impl FieldProductDiagnostic {
    pub fn new(
        code: FieldProductDiagnosticCode,
        severity: FieldProductDiagnosticSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            product_id: None,
            family: None,
            scale_band: None,
            consumer_class: None,
            generation: None,
            message: message.into(),
            cause: String::new(),
            suggested_action: String::new(),
            related_products: Vec::new(),
        }
    }

    pub fn blocking(code: FieldProductDiagnosticCode, message: impl Into<String>) -> Self {
        Self::new(code, FieldProductDiagnosticSeverity::Blocking, message)
    }

    pub fn for_product(mut self, product_id: ProductIdentity) -> Self {
        self.product_id = Some(product_id);
        self
    }
}
