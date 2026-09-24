use serde::{Deserialize, Serialize};

use crate::{FieldProductDiagnostic, ProductDescriptorCore, ProductIdentity, ProductJobDescriptor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductPublicationStatus {
    Ready,
    FailedPreserved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductPublicationOutcome {
    pub product_job: ProductJobDescriptor,
    pub output_descriptors: Vec<ProductDescriptorCore>,
    pub diagnostics: Vec<FieldProductDiagnostic>,
    pub status: ProductPublicationStatus,
    pub stage_sequence: u64,
}

impl ProductPublicationOutcome {
    pub fn ready(
        product_job: ProductJobDescriptor,
        output_descriptors: impl IntoIterator<Item = ProductDescriptorCore>,
        stage_sequence: u64,
    ) -> Self {
        Self {
            product_job,
            output_descriptors: output_descriptors.into_iter().collect(),
            diagnostics: Vec::new(),
            status: ProductPublicationStatus::Ready,
            stage_sequence,
        }
    }

    pub fn failed_preserved(
        product_job: ProductJobDescriptor,
        output_descriptors: impl IntoIterator<Item = ProductDescriptorCore>,
        diagnostics: impl IntoIterator<Item = FieldProductDiagnostic>,
        stage_sequence: u64,
    ) -> Self {
        Self {
            product_job,
            output_descriptors: output_descriptors.into_iter().collect(),
            diagnostics: diagnostics.into_iter().collect(),
            status: ProductPublicationStatus::FailedPreserved,
            stage_sequence,
        }
    }

    pub fn rejected(
        product_job: ProductJobDescriptor,
        diagnostics: impl IntoIterator<Item = FieldProductDiagnostic>,
        stage_sequence: u64,
    ) -> Self {
        Self {
            product_job,
            output_descriptors: Vec::new(),
            diagnostics: diagnostics.into_iter().collect(),
            status: ProductPublicationStatus::Rejected,
            stage_sequence,
        }
    }

    pub fn output_product_ids(&self) -> Vec<ProductIdentity> {
        self.output_descriptors
            .iter()
            .map(|descriptor| descriptor.identity)
            .collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductPublicationReport {
    pub published_count: usize,
    pub failed_preserved_count: usize,
    pub rejected_count: usize,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

impl ProductPublicationReport {
    pub fn record(&mut self, outcome: &ProductPublicationOutcome) {
        match outcome.status {
            ProductPublicationStatus::Ready => {
                self.published_count = self.published_count.saturating_add(1);
            }
            ProductPublicationStatus::FailedPreserved => {
                self.failed_preserved_count = self.failed_preserved_count.saturating_add(1);
            }
            ProductPublicationStatus::Rejected => {
                self.rejected_count = self.rejected_count.saturating_add(1);
            }
        }
        self.diagnostics.extend(outcome.diagnostics.iter().cloned());
    }
}
