use product::{ProductPublicationOutcome, QuerySnapshotProductDescriptor};

use crate::runtime::jobs::executor::RuntimeJobExecutorResource;
use crate::runtime::jobs::types::{RuntimeJobCompletion, RuntimeJobStatus};
use crate::runtime::{ProductPublicationRuntimeResource, QuerySnapshotRuntimeResource};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeProductJobOutput {
    pub publication_outcomes: Vec<ProductPublicationOutcome>,
    pub query_snapshots: Vec<QuerySnapshotProductDescriptor>,
}

impl RuntimeProductJobOutput {
    pub fn new(
        publication_outcomes: impl IntoIterator<Item = ProductPublicationOutcome>,
        query_snapshots: impl IntoIterator<Item = QuerySnapshotProductDescriptor>,
    ) -> Self {
        Self {
            publication_outcomes: publication_outcomes.into_iter().collect(),
            query_snapshots: query_snapshots.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeProductJobStageReport {
    pub completed_count: usize,
    pub failed_count: usize,
    pub stale_count: usize,
    pub staged_publication_count: usize,
    pub staged_query_snapshot_count: usize,
}

pub fn stage_completed_product_jobs(
    executor: &mut RuntimeJobExecutorResource,
    publications: &mut ProductPublicationRuntimeResource,
    snapshots: &mut QuerySnapshotRuntimeResource,
) -> RuntimeProductJobStageReport {
    let mut completions = executor.drain_completed::<RuntimeProductJobOutput>();
    completions.sort_by_key(product_job_completion_order_key);
    let mut report = RuntimeProductJobStageReport::default();
    for completion in completions {
        stage_one_completion(completion, publications, snapshots, &mut report);
    }
    report
}

fn product_job_completion_order_key(
    completion: &RuntimeJobCompletion<RuntimeProductJobOutput>,
) -> (u64, u64, u64) {
    let stage_sequence = completion
        .output
        .as_ref()
        .and_then(|output| {
            output
                .publication_outcomes
                .iter()
                .map(|outcome| outcome.stage_sequence)
                .min()
        })
        .unwrap_or(completion.handle.submission_sequence);
    (
        completion.handle.generation.raw(),
        stage_sequence,
        completion.product_job.job_id.raw(),
    )
}

fn stage_one_completion(
    completion: RuntimeJobCompletion<RuntimeProductJobOutput>,
    publications: &mut ProductPublicationRuntimeResource,
    snapshots: &mut QuerySnapshotRuntimeResource,
    report: &mut RuntimeProductJobStageReport,
) {
    match completion.status {
        RuntimeJobStatus::Completed => {
            report.completed_count = report.completed_count.saturating_add(1);
            if let Some(output) = completion.output {
                let mut publication_outcomes = output.publication_outcomes;
                publication_outcomes.sort_by_key(|outcome| {
                    (
                        completion.handle.generation.raw(),
                        outcome.stage_sequence,
                        outcome.product_job.job_id.raw(),
                    )
                });
                let mut query_snapshots = output.query_snapshots;
                query_snapshots.sort_by_key(|snapshot| {
                    (
                        completion.handle.generation.raw(),
                        snapshot.product_id().raw(),
                        snapshot.source_generation,
                        snapshot.response_generation,
                    )
                });
                report.staged_publication_count = report
                    .staged_publication_count
                    .saturating_add(publication_outcomes.len());
                report.staged_query_snapshot_count = report
                    .staged_query_snapshot_count
                    .saturating_add(query_snapshots.len());
                publications.stage_all(publication_outcomes);
                snapshots.stage_all(query_snapshots);
            }
        }
        RuntimeJobStatus::Failed => {
            report.failed_count = report.failed_count.saturating_add(1);
            publications.stage(ProductPublicationOutcome::rejected(
                completion.product_job,
                completion.diagnostics,
                completion.handle.submission_sequence,
            ));
            report.staged_publication_count = report.staged_publication_count.saturating_add(1);
        }
        RuntimeJobStatus::Stale => {
            report.stale_count = report.stale_count.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use product::{
        ProductDescriptorCore, ProductFamily, ProductIdentity, ProductJobDescriptor, ProductJobId,
        ProductKind, ProductLineage, ProductQueryPolicy, ProductScaleBand, ProductScope,
        QuerySnapshotProductDescriptor,
    };

    use crate::runtime::jobs::{
        RuntimeJob, RuntimeJobExecutorResource, RuntimeJobGeneration, RuntimeJobResult,
    };
    use crate::runtime::{ProductPublicationRuntimeResource, QuerySnapshotRuntimeResource};

    use super::*;

    struct ProductTestJob;

    impl RuntimeJob for ProductTestJob {
        type Output = RuntimeProductJobOutput;

        fn product_job(&self) -> ProductJobDescriptor {
            product_job()
        }

        fn generation(&self) -> RuntimeJobGeneration {
            RuntimeJobGeneration::new(1)
        }

        fn execute(self) -> RuntimeJobResult<Self::Output> {
            let descriptor = descriptor();
            Ok(RuntimeProductJobOutput::new(
                [ProductPublicationOutcome::ready(
                    product_job(),
                    [descriptor.clone()],
                    1,
                )],
                [QuerySnapshotProductDescriptor::new(
                    descriptor,
                    1,
                    1,
                    ProductQueryPolicy::StrictCurrentOnly,
                )],
            ))
        }
    }

    struct OrderedProductTestJob {
        job_id: u64,
        generation: u64,
        stage_sequence: u64,
    }

    impl RuntimeJob for OrderedProductTestJob {
        type Output = RuntimeProductJobOutput;

        fn product_job(&self) -> ProductJobDescriptor {
            product_job_with_id(self.job_id)
        }

        fn generation(&self) -> RuntimeJobGeneration {
            RuntimeJobGeneration::new(self.generation)
        }

        fn execute(self) -> RuntimeJobResult<Self::Output> {
            let descriptor = descriptor_with_id(self.job_id, self.generation);
            Ok(RuntimeProductJobOutput::new(
                [ProductPublicationOutcome::ready(
                    product_job_with_id(self.job_id),
                    [descriptor],
                    self.stage_sequence,
                )],
                [],
            ))
        }
    }

    #[test]
    fn runtime_job_product_helper_stages_completed_publications_and_snapshots() {
        let mut executor = RuntimeJobExecutorResource::default();
        executor.submit(ProductTestJob).unwrap();
        let mut publications = ProductPublicationRuntimeResource::default();
        let mut snapshots = QuerySnapshotRuntimeResource::default();

        let report = stage_completed_product_jobs(&mut executor, &mut publications, &mut snapshots);

        assert_eq!(report.completed_count, 1);
        assert_eq!(report.staged_publication_count, 1);
        assert_eq!(report.staged_query_snapshot_count, 1);
        assert_eq!(publications.staged().len(), 1);
        assert_eq!(snapshots.staged().len(), 1);
    }

    #[test]
    fn runtime_job_product_helper_orders_staged_publications_by_generation_stage_and_job() {
        let mut executor = RuntimeJobExecutorResource::default();
        executor
            .submit(OrderedProductTestJob {
                job_id: 2,
                generation: 2,
                stage_sequence: 10,
            })
            .unwrap();
        executor
            .submit(OrderedProductTestJob {
                job_id: 1,
                generation: 1,
                stage_sequence: 20,
            })
            .unwrap();
        let mut publications = ProductPublicationRuntimeResource::default();
        let mut snapshots = QuerySnapshotRuntimeResource::default();

        stage_completed_product_jobs(&mut executor, &mut publications, &mut snapshots);

        let staged_job_ids = publications
            .staged()
            .iter()
            .map(|outcome| outcome.product_job.job_id.raw())
            .collect::<Vec<_>>();
        assert_eq!(staged_job_ids, vec![1, 2]);
    }

    fn product_job() -> ProductJobDescriptor {
        product_job_with_id(100)
    }

    fn product_job_with_id(id: u64) -> ProductJobDescriptor {
        ProductJobDescriptor::new(
            ProductJobId::new(id),
            ProductKind::new("runtime_product_test"),
            "engine.runtime.test",
            ProductIdentity::new(id),
            ProductScope::non_spatial("runtime-product-test"),
            ProductScaleBand::Preview,
        )
    }

    fn descriptor() -> ProductDescriptorCore {
        descriptor_with_id(100, 1)
    }

    fn descriptor_with_id(id: u64, generation: u64) -> ProductDescriptorCore {
        ProductDescriptorCore::new(
            ProductIdentity::new(id),
            ProductFamily::SurfaceSdf,
            ProductKind::new("runtime_product_test"),
            ProductScope::non_spatial("runtime-product-test"),
            ProductScaleBand::Preview,
            ProductLineage::new("engine.runtime.test", generation),
        )
    }
}
