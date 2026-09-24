use crate::runtime::publication::ProductPublicationOccurrence;
use anyhow::Result;
use product::{
    FieldProductDiagnostic, FieldProductDiagnosticCode, ProductIdentity, ProductJobId,
    ProductPublicationOutcome, ProductPublicationReport, ProductPublicationStatus,
    ratify_product_publication,
};

use runen_ecs::World;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductPublicationJournalEntry {
    pub publication_sequence: u64,
    pub schedule_label: &'static str,
    pub stage_sequence: u64,
    pub product_job_id: ProductJobId,
    pub status: ProductPublicationStatus,
    pub output_products: Vec<ProductIdentity>,
}

#[derive(Debug, Clone, Default, runen_ecs::Resource)]
pub struct ProductPublicationRuntimeResource {
    staged: Vec<ProductPublicationOutcome>,
    journal: Vec<ProductPublicationJournalEntry>,
    last_report: ProductPublicationReport,
}

impl ProductPublicationRuntimeResource {
    pub fn stage(&mut self, outcome: ProductPublicationOutcome) {
        self.staged.push(outcome);
    }

    pub fn stage_all(&mut self, outcomes: impl IntoIterator<Item = ProductPublicationOutcome>) {
        self.staged.extend(outcomes);
    }

    pub fn staged(&self) -> &[ProductPublicationOutcome] {
        &self.staged
    }

    pub fn journal(&self) -> &[ProductPublicationJournalEntry] {
        &self.journal
    }

    pub fn last_report(&self) -> &ProductPublicationReport {
        &self.last_report
    }

    pub fn publish_staged(
        &mut self,
        occurrence: &ProductPublicationOccurrence,
    ) -> ProductPublicationReport {
        let mut staged = std::mem::take(&mut self.staged);
        staged.sort_by_key(|outcome| (outcome.stage_sequence, outcome.product_job.job_id.raw()));

        let mut report = ProductPublicationReport::default();
        for outcome in staged {
            let ratification = ratify_product_publication(&outcome);
            if ratification.has_blocking_issues() {
                report.rejected_count = report.rejected_count.saturating_add(1);
                report.diagnostics.extend(ratification.iter().map(|issue| {
                    FieldProductDiagnostic::blocking(
                        FieldProductDiagnosticCode::FormationFailure,
                        format!("product publication rejected: {}", issue.message()),
                    )
                }));
                continue;
            }

            report.record(&outcome);
            self.journal.push(ProductPublicationJournalEntry {
                publication_sequence: occurrence.sequence(),
                schedule_label: occurrence.schedule_label(),
                stage_sequence: outcome.stage_sequence,
                product_job_id: outcome.product_job.job_id,
                status: outcome.status,
                output_products: outcome.output_product_ids(),
            });
        }
        self.last_report = report.clone();
        report
    }
}

pub fn publish_staged_product_outcomes(
    occurrence: &ProductPublicationOccurrence,
    world: &mut World,
) -> Result<()> {
    if let Ok(publications) = world.resource_mut::<ProductPublicationRuntimeResource>() {
        publications.publish_staged(occurrence);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use product::{
        ProductDescriptorCore, ProductFamily, ProductJobDescriptor, ProductKind, ProductLineage,
        ProductScaleBand, ProductScope,
    };

    fn occurrence(sequence: u64) -> ProductPublicationOccurrence {
        ProductPublicationOccurrence::new(sequence, "Update")
    }

    fn job(id: u64) -> ProductJobDescriptor {
        ProductJobDescriptor::new(
            ProductJobId::new(id),
            ProductKind::new("test_product"),
            "test.producer",
            ProductIdentity::new(id),
            ProductScope::non_spatial("test"),
            ProductScaleBand::Preview,
        )
    }

    fn descriptor(id: u64) -> ProductDescriptorCore {
        ProductDescriptorCore::new(
            ProductIdentity::new(id),
            ProductFamily::SurfaceSdf,
            ProductKind::new("test_product"),
            ProductScope::non_spatial("test"),
            ProductScaleBand::Preview,
            ProductLineage::new("test.producer", 1),
        )
    }

    #[test]
    fn staged_outcomes_publish_at_engine_publication_occurrence() {
        let mut resource = ProductPublicationRuntimeResource::default();
        resource.stage(ProductPublicationOutcome::ready(
            job(2),
            [descriptor(2)],
            10,
        ));

        assert_eq!(resource.journal().len(), 0);
        assert_eq!(resource.staged().len(), 1);

        let report = resource.publish_staged(&occurrence(4));

        assert_eq!(report.published_count, 1);
        assert_eq!(resource.staged().len(), 0);
        assert_eq!(resource.journal().len(), 1);
        assert_eq!(resource.journal()[0].publication_sequence, 4);
        assert_eq!(resource.journal()[0].schedule_label, "Update");
    }

    #[test]
    fn publication_journal_is_ordered_by_stage_sequence_then_job_id() {
        let mut resource = ProductPublicationRuntimeResource::default();
        resource.stage(ProductPublicationOutcome::ready(
            job(9),
            [descriptor(9)],
            20,
        ));
        resource.stage(ProductPublicationOutcome::ready(
            job(3),
            [descriptor(3)],
            10,
        ));
        resource.stage(ProductPublicationOutcome::ready(
            job(2),
            [descriptor(2)],
            10,
        ));

        resource.publish_staged(&occurrence(1));

        let ids = resource
            .journal()
            .iter()
            .map(|entry| entry.product_job_id.raw())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![2, 3, 9]);
    }

    #[test]
    fn invalid_publications_are_reported_and_not_journaled() {
        let mut resource = ProductPublicationRuntimeResource::default();
        resource.stage(ProductPublicationOutcome::ready(
            job(1),
            [descriptor(99)],
            1,
        ));

        let report = resource.publish_staged(&occurrence(1));

        assert_eq!(report.rejected_count, 1);
        assert_eq!(resource.journal().len(), 0);
        assert!(!report.diagnostics.is_empty());
    }
}
