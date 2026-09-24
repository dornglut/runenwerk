//! File: domain/procgen/src/products.rs
//! Purpose: Product job and publication DTO builders for procgen outputs.

use product::{
    ProductAccessDescriptor, ProductAuthorityClass, ProductConsumerClass, ProductDescriptorCore,
    ProductDeterminismClass, ProductFamily, ProductFreshness, ProductJobAccess, ProductJobAffinity,
    ProductJobBudgetClass, ProductJobDescriptor, ProductJobFailurePolicy, ProductJobId,
    ProductKind, ProductLineage, ProductPublicationOutcome, ProductQueryPolicy, ProductResidency,
    ProductRetentionPolicy, ProductScaleBand,
};
use world_sdf::FieldPreviewProduct;

use crate::{
    ProcgenBudgetClass, ProcgenDocument, ProcgenOutputKind, ProcgenRetentionClass,
    determinism::{determinism_key_for_document, stable_nonzero_hash64},
    ratification::ratify_procgen_document,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenProductContracts {
    pub product_job: ProductJobDescriptor,
    pub output_descriptors: Vec<ProductDescriptorCore>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenFormedPreviewProductContracts {
    pub product_job: ProductJobDescriptor,
    pub output_descriptors: Vec<ProductDescriptorCore>,
}

pub fn build_procgen_product_contracts(
    document: &ProcgenDocument,
    catalog: &crate::ProcgenNodeCatalog,
) -> Option<ProcgenProductContracts> {
    let report = ratify_procgen_document(document, catalog);
    if report.has_blocking_issues() {
        return None;
    }

    let output_descriptors = procgen_output_product_descriptors(document);
    Some(ProcgenProductContracts {
        product_job: procgen_product_job(document, &output_descriptors),
        output_descriptors,
    })
}

pub fn procgen_output_product_descriptors(
    document: &ProcgenDocument,
) -> Vec<ProductDescriptorCore> {
    let determinism_key = determinism_key_for_document(document);
    let generation = stable_nonzero_hash64([
        "procgen.product.generation",
        determinism_key.as_str(),
        document.cache_lineage.parameter_hash.as_str(),
    ]);
    let scope = document.scope.product_scope();
    let mut outputs = document.output_products.clone();
    outputs.sort();
    outputs
        .into_iter()
        .map(|output| {
            let (family, kind) = match output.kind {
                ProcgenOutputKind::WorldOpsWindow => (
                    ProductFamily::FamilySpecific,
                    ProductKind::new("procgen.world_ops_window"),
                ),
                ProcgenOutputKind::FieldProductCandidate => (
                    ProductFamily::SurfaceSdf,
                    ProductKind::new("procgen.field_product_candidate"),
                ),
            };
            let mut lineage = ProductLineage::new(procgen_producer_key(document), generation)
                .with_source_key(format!("procgen.document.{}", document.document_id.raw()))
                .with_source_revision(document.source_revision.clone());
            lineage.producer_version = Some(document.generator_version.clone());
            for input in &document.input_products {
                lineage = lineage.with_upstream_product(input.product_id);
            }
            let mut descriptor = ProductDescriptorCore::new(
                output.product_id,
                family,
                kind,
                scope.clone(),
                scale_band_for_document(document),
                lineage,
            );
            descriptor.freshness = ProductFreshness::Current;
            descriptor.residency = ProductResidency::NotApplicable;
            descriptor.consumer_class = ProductConsumerClass::Tooling;
            descriptor.authority_class = ProductAuthorityClass::DeterministicDerived;
            descriptor.retention_policy = retention_policy_for_document(document);
            descriptor.query_policy = ProductQueryPolicy::StrictCurrentOnly;
            descriptor
        })
        .collect()
}

pub fn build_procgen_publication_outcome(
    document: &ProcgenDocument,
    catalog: &crate::ProcgenNodeCatalog,
    stage_sequence: u64,
) -> Option<ProductPublicationOutcome> {
    let contracts = build_procgen_product_contracts(document, catalog)?;
    Some(ProductPublicationOutcome::ready(
        contracts.product_job,
        contracts.output_descriptors,
        stage_sequence,
    ))
}

pub fn build_procgen_formed_preview_product_contracts(
    document: &ProcgenDocument,
    catalog: &crate::ProcgenNodeCatalog,
    preview_products: &[FieldPreviewProduct],
) -> Option<ProcgenFormedPreviewProductContracts> {
    if preview_products.is_empty() {
        return None;
    }
    let report = ratify_procgen_document(document, catalog);
    if report.has_blocking_issues() {
        return None;
    }

    let output_descriptors = procgen_formed_preview_product_descriptors(document, preview_products);
    Some(ProcgenFormedPreviewProductContracts {
        product_job: procgen_product_job(document, &output_descriptors),
        output_descriptors,
    })
}

pub fn procgen_formed_preview_product_descriptors(
    document: &ProcgenDocument,
    preview_products: &[FieldPreviewProduct],
) -> Vec<ProductDescriptorCore> {
    let mut descriptors = procgen_output_product_descriptors(document)
        .into_iter()
        .filter(|descriptor| descriptor.kind.as_str() == "procgen.world_ops_window")
        .collect::<Vec<_>>();
    let mut preview_descriptors = preview_products
        .iter()
        .map(|product| product.descriptor.product_core())
        .collect::<Vec<_>>();
    preview_descriptors.sort_by_key(|descriptor| descriptor.identity);
    descriptors.extend(preview_descriptors);
    descriptors
}

pub fn build_procgen_formed_preview_publication_outcome(
    document: &ProcgenDocument,
    catalog: &crate::ProcgenNodeCatalog,
    preview_products: &[FieldPreviewProduct],
    stage_sequence: u64,
) -> Option<ProductPublicationOutcome> {
    let contracts =
        build_procgen_formed_preview_product_contracts(document, catalog, preview_products)?;
    Some(ProductPublicationOutcome::ready(
        contracts.product_job,
        contracts.output_descriptors,
        stage_sequence,
    ))
}

fn procgen_product_job(
    document: &ProcgenDocument,
    output_descriptors: &[ProductDescriptorCore],
) -> ProductJobDescriptor {
    let determinism_key = determinism_key_for_document(document);
    let output_products = output_descriptors
        .iter()
        .map(|descriptor| descriptor.identity)
        .collect::<Vec<_>>();
    let primary_output = output_products[0];
    let mut job = ProductJobDescriptor::new(
        ProductJobId::new(stable_nonzero_hash64([
            "procgen.product.job",
            determinism_key.as_str(),
        ])),
        ProductKind::new("procgen.bounded_region_terrain_material"),
        procgen_producer_key(document),
        primary_output,
        document.scope.product_scope(),
        scale_band_for_document(document),
    );
    job.input_products = document
        .input_products
        .iter()
        .map(|input| input.product_id)
        .collect();
    job.output_products = output_products.clone();
    job.access = ProductJobAccess {
        products: document
            .input_products
            .iter()
            .map(|input| ProductAccessDescriptor::read(input.product_id))
            .chain(
                output_products
                    .into_iter()
                    .map(ProductAccessDescriptor::write),
            )
            .collect(),
    };
    job.budget_class = match document.execution_policy.budget_class {
        ProcgenBudgetClass::RuntimePreview => ProductJobBudgetClass::Interactive,
        ProcgenBudgetClass::OfflineBake => ProductJobBudgetClass::Offline,
    };
    job.priority = match document.execution_policy.budget_class {
        ProcgenBudgetClass::RuntimePreview => 50,
        ProcgenBudgetClass::OfflineBake => 0,
    };
    job.affinity = ProductJobAffinity::Worker;
    job.determinism = ProductDeterminismClass::AuthoritativeDeterministic;
    job.authority_class = ProductAuthorityClass::DeterministicDerived;
    job.failure_policy = ProductJobFailurePolicy::PreserveLastValidWithDiagnostic;
    job
}

fn procgen_producer_key(document: &ProcgenDocument) -> String {
    format!("procgen.generator.{}", document.generator_id.raw())
}

fn scale_band_for_document(document: &ProcgenDocument) -> ProductScaleBand {
    match document.execution_policy.budget_class {
        ProcgenBudgetClass::RuntimePreview => ProductScaleBand::Preview,
        ProcgenBudgetClass::OfflineBake => ProductScaleBand::Offline,
    }
}

fn retention_policy_for_document(document: &ProcgenDocument) -> ProductRetentionPolicy {
    match document.execution_policy.retention_class {
        ProcgenRetentionClass::SessionCandidate => ProductRetentionPolicy::SessionLocal,
        ProcgenRetentionClass::RetainedBake => ProductRetentionPolicy::Cacheable,
    }
}

#[cfg(test)]
mod tests {
    use product::{ratify_product_job, ratify_product_publication};

    use super::*;
    use crate::{
        ProcgenFieldPreviewPolicy, ProcgenNodeCatalog, form_procgen_field_preview_products,
        test_fixtures::valid_document,
    };

    #[test]
    fn product_contracts_pass_existing_product_ratifiers() {
        let document = valid_document();
        let contracts =
            build_procgen_product_contracts(&document, &ProcgenNodeCatalog::first_slice())
                .expect("valid procgen document should form product contracts");

        assert!(!ratify_product_job(&contracts.product_job).has_blocking_issues());
        assert_eq!(contracts.output_descriptors.len(), 2);
        let outcome = ProductPublicationOutcome::ready(
            contracts.product_job,
            contracts.output_descriptors,
            1,
        );
        assert!(!ratify_product_publication(&outcome).has_blocking_issues());
    }

    #[test]
    fn identical_inputs_produce_identical_product_contracts() {
        let first = valid_document();
        let second = valid_document();

        let first_contracts =
            build_procgen_product_contracts(&first, &ProcgenNodeCatalog::first_slice()).unwrap();
        let second_contracts =
            build_procgen_product_contracts(&second, &ProcgenNodeCatalog::first_slice()).unwrap();

        assert_eq!(first_contracts, second_contracts);
    }

    #[test]
    fn seed_changes_product_job_identity_and_generations() {
        let first = valid_document();
        let mut second = valid_document();
        second.world_seed = "world-seed:beta".to_string();
        second.refresh_cache_lineage();

        let first_contracts =
            build_procgen_product_contracts(&first, &ProcgenNodeCatalog::first_slice()).unwrap();
        let second_contracts =
            build_procgen_product_contracts(&second, &ProcgenNodeCatalog::first_slice()).unwrap();

        assert_ne!(
            first_contracts.product_job.job_id,
            second_contracts.product_job.job_id
        );
        assert_ne!(
            first_contracts.output_descriptors[0].lineage.generation,
            second_contracts.output_descriptors[0].lineage.generation
        );
    }

    #[test]
    fn formed_preview_product_contracts_replace_generic_field_candidate_descriptor() {
        let document = valid_document();
        let formation = form_procgen_field_preview_products(
            &document,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );
        let contracts = build_procgen_formed_preview_product_contracts(
            &document,
            &ProcgenNodeCatalog::first_slice(),
            &formation.products,
        )
        .expect("formed preview contracts should build");

        assert_eq!(contracts.output_descriptors.len(), 3);
        assert!(
            contracts
                .output_descriptors
                .iter()
                .any(|descriptor| descriptor.kind.as_str() == "procgen.world_ops_window")
        );
        assert!(
            contracts
                .output_descriptors
                .iter()
                .any(|descriptor| descriptor.kind.as_str() == "scalar_distance")
        );
        assert!(
            contracts
                .output_descriptors
                .iter()
                .any(|descriptor| descriptor.kind.as_str() == "material_channel")
        );
        assert!(
            contracts
                .output_descriptors
                .iter()
                .all(|descriptor| descriptor.kind.as_str() != "procgen.field_product_candidate")
        );
        assert!(!ratify_product_job(&contracts.product_job).has_blocking_issues());
    }
}
