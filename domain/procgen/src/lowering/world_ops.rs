//! File: domain/procgen/src/lowering/world_ops.rs
//! Purpose: Deterministic lowering from procgen documents to world operation windows.

use world_ops::{Operation, OperationId, OperationRecord};

use crate::{
    ProcgenCandidateId, ProcgenChangedRegion, ProcgenDocument, ProcgenExplanationEntry,
    ProcgenNodeCatalog, ProcgenRealization, ProcgenRealizationId, ProcgenWriteTarget,
    ProcgenWriteTargetKind,
    determinism::{determinism_key_for_document, stable_nonzero_hash64, write_target_kind_name},
    ratification::{ProcgenRatificationReport, ratify_procgen_document},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenWorldOpsLoweringResult {
    pub report: ProcgenRatificationReport,
    pub realization: Option<ProcgenRealization>,
}

pub fn lower_procgen_to_world_ops(
    document: &ProcgenDocument,
    catalog: &ProcgenNodeCatalog,
) -> ProcgenWorldOpsLoweringResult {
    let report = ratify_procgen_document(document, catalog);
    if report.has_blocking_issues() {
        return ProcgenWorldOpsLoweringResult {
            report,
            realization: None,
        };
    }

    let determinism_key = determinism_key_for_document(document);
    let candidate_id = ProcgenCandidateId::new(stable_nonzero_hash64([
        "procgen.candidate",
        determinism_key.as_str(),
    ]));
    let realization_id = ProcgenRealizationId::new(stable_nonzero_hash64([
        "procgen.realization",
        determinism_key.as_str(),
    ]));
    let mut targets = document.write_targets.clone();
    targets.sort_by_key(canonical_target_key);

    let operation_records = targets
        .iter()
        .enumerate()
        .map(|(index, target)| {
            operation_for_target(document, target, determinism_key.as_str(), index)
        })
        .collect::<Vec<_>>();
    let changed_regions = targets
        .iter()
        .map(|target| ProcgenChangedRegion {
            target_id: target.target_id.clone(),
            bounds_q: target.bounds_q,
            product_id: document
                .output_products
                .iter()
                .find(|output| output.kind == crate::ProcgenOutputKind::FieldProductCandidate)
                .map(|output| output.product_id),
        })
        .collect::<Vec<_>>();
    let explanations = operation_records
        .iter()
        .map(|record| {
            ProcgenExplanationEntry::new(
                format!("operation:{}", record.op_id.0),
                format!(
                    "lowered deterministic procgen target into {:?}",
                    record.operation
                ),
            )
        })
        .collect::<Vec<_>>();

    ProcgenWorldOpsLoweringResult {
        report,
        realization: Some(ProcgenRealization {
            realization_id,
            candidate_id,
            operation_records,
            changed_regions,
            explanations,
            determinism_key: determinism_key.0,
        }),
    }
}

fn operation_for_target(
    document: &ProcgenDocument,
    target: &ProcgenWriteTarget,
    determinism_key: &str,
    index: usize,
) -> OperationRecord {
    let target_key = canonical_target_key(target);
    let index_string = index.to_string();
    let seed = stable_nonzero_hash64([
        "procgen.operation.seed",
        determinism_key,
        target_key.as_str(),
        index_string.as_str(),
    ]);
    let payload = canonical_payload(document, target, determinism_key, index);
    let operation = match target.kind {
        ProcgenWriteTargetKind::DensityField => Operation::DensityFieldDeform {
            bounds_q: target.bounds_q,
            payload,
        },
        ProcgenWriteTargetKind::MaterialChannel => {
            let channel = target
                .material_channel
                .expect("ratified material targets declare channel");
            Operation::MaterialFieldEdit {
                bounds_q: target.bounds_q,
                channel_mask: 1u16 << channel,
                payload,
            }
        }
    };

    OperationRecord {
        op_id: OperationId(stable_nonzero_hash64([
            "procgen.operation.id",
            determinism_key,
            target_key.as_str(),
            index_string.as_str(),
        ])),
        base_world_revision: document.lowering_policy.base_world_revision,
        planet_id: document.scope.world_id,
        operation,
        affected_bounds_q: target.bounds_q,
        deterministic_seed: seed,
    }
}

fn canonical_payload(
    document: &ProcgenDocument,
    target: &ProcgenWriteTarget,
    determinism_key: &str,
    index: usize,
) -> Vec<u8> {
    format!(
        "procgen.payload|document={}|generator={}|source={}|key={}|target={}|kind={}|index={}",
        document.document_id.raw(),
        document.generator_id.raw(),
        document.source_revision,
        determinism_key,
        target.target_id,
        write_target_kind_name(target.kind),
        index
    )
    .into_bytes()
}

fn canonical_target_key(target: &ProcgenWriteTarget) -> String {
    format!(
        "{}:{:?}:{}:{}:{}:{}:{}:{}:{:?}",
        target.target_id,
        target.kind,
        target.bounds_q.min.x,
        target.bounds_q.min.y,
        target.bounds_q.min.z,
        target.bounds_q.max.x,
        target.bounds_q.max.y,
        target.bounds_q.max.z,
        target.material_channel
    )
}

#[cfg(test)]
mod tests {
    use product::ProductIdentity;
    use world_ops::Operation;

    use super::*;
    use crate::{ProcgenInputProduct, ProcgenNodeCatalog, test_fixtures::valid_document};

    #[test]
    fn valid_document_lowers_to_deterministic_world_ops_window() {
        let document = valid_document();

        let result = lower_procgen_to_world_ops(&document, &ProcgenNodeCatalog::first_slice());

        assert!(result.report.is_accepted());
        let realization = result.realization.expect("valid document should lower");
        assert_eq!(realization.operation_records.len(), 2);
        assert!(
            realization
                .operation_records
                .iter()
                .any(|record| matches!(record.operation, Operation::DensityFieldDeform { .. }))
        );
        assert!(
            realization
                .operation_records
                .iter()
                .any(|record| matches!(record.operation, Operation::MaterialFieldEdit { .. }))
        );
        assert_eq!(realization.changed_regions.len(), 2);
        assert_eq!(realization.explanations.len(), 2);
    }

    #[test]
    fn identical_inputs_produce_identical_operation_records_and_explanations() {
        let first = valid_document();
        let second = valid_document();

        let first_realization =
            lower_procgen_to_world_ops(&first, &ProcgenNodeCatalog::first_slice())
                .realization
                .unwrap();
        let second_realization =
            lower_procgen_to_world_ops(&second, &ProcgenNodeCatalog::first_slice())
                .realization
                .unwrap();

        assert_eq!(first_realization, second_realization);
    }

    #[test]
    fn upstream_generation_changes_deterministic_operation_ids() {
        let first = valid_document();
        let mut second = valid_document();
        second.input_products.clear();
        second = second.with_input_product(ProcgenInputProduct::new(ProductIdentity::new(77), 13));
        second.refresh_cache_lineage();

        let first_realization =
            lower_procgen_to_world_ops(&first, &ProcgenNodeCatalog::first_slice())
                .realization
                .unwrap();
        let second_realization =
            lower_procgen_to_world_ops(&second, &ProcgenNodeCatalog::first_slice())
                .realization
                .unwrap();

        assert_ne!(
            first_realization.operation_records[0].op_id,
            second_realization.operation_records[0].op_id
        );
        assert_ne!(
            first_realization.determinism_key,
            second_realization.determinism_key
        );
    }
}
