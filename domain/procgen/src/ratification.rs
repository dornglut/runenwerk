//! File: domain/procgen/src/ratification.rs
//! Purpose: Procgen semantic ratification over graph-backed generator documents.

use std::collections::{BTreeMap, BTreeSet};

use graph::{GraphValidationError, validate_graph};
use ratification::{RatificationIssue, RatificationReport};

use crate::{
    ProcgenDocument, ProcgenNodeCatalog, ProcgenNodeKind, ProcgenOutputKind, ProcgenReservationId,
    ProcgenWriteTargetKind, parameter_hash_for_document,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcgenIssueCode {
    EmptyLabel,
    EmptySchemaVersion,
    EmptyGeneratorId,
    EmptyGeneratorVersion,
    EmptyWorldSeed,
    EmptySourceRevision,
    EmptyScope,
    ScopeWorldMismatch,
    GraphStructural,
    UnsupportedNode,
    DuplicateNodeParameters,
    MissingNodeParameters,
    NodeParameterKindMismatch,
    EmptyNodeParameterLabel,
    EmptySeedSalt,
    MissingWorldOpsOutput,
    MissingFieldProductOutput,
    EmptyWriteTargets,
    EmptyWriteTarget,
    DuplicateWriteTarget,
    InvalidWriteTargetBounds,
    InvalidMaterialChannel,
    EmptyOutputProduct,
    DuplicateOutputProduct,
    MissingWorldOpsOutputProduct,
    MissingFieldProductOutputProduct,
    EmptyLoweringVersion,
    InvalidFixedPointScale,
    EmptyCacheLineage,
    CacheLineageDrift,
    EmptyInputProduct,
    InputGenerationZero,
    EmptyReservationId,
    ReservationMissingWriteTarget,
    ReservationConflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProcgenIssueSubject {
    Document,
    Graph,
    Node(u64),
    Scope,
    InputProduct(u64),
    WriteTarget(String),
    OutputProduct(u64),
    Reservation(u64),
    CacheLineage,
}

pub type ProcgenRatificationReport = RatificationReport<ProcgenIssueCode, ProcgenIssueSubject>;

pub fn ratify_procgen_document(
    document: &ProcgenDocument,
    catalog: &ProcgenNodeCatalog,
) -> ProcgenRatificationReport {
    let mut report = ProcgenRatificationReport::new();

    ratify_document_identity(document, &mut report);
    ratify_graph(document, catalog, &mut report);
    ratify_inputs(document, &mut report);
    ratify_write_targets(document, &mut report);
    ratify_output_products(document, &mut report);
    ratify_lowering_and_cache(document, &mut report);
    ratify_reservations(document, &mut report);

    report
}

fn ratify_document_identity(document: &ProcgenDocument, report: &mut ProcgenRatificationReport) {
    if document.label.trim().is_empty() {
        report.push(issue(
            ProcgenIssueCode::EmptyLabel,
            ProcgenIssueSubject::Document,
            "procgen document label must not be empty",
        ));
    }
    if document.schema_version.trim().is_empty() {
        report.push(issue(
            ProcgenIssueCode::EmptySchemaVersion,
            ProcgenIssueSubject::Document,
            "procgen schema version must not be empty",
        ));
    }
    if document.generator_id.is_empty() {
        report.push(issue(
            ProcgenIssueCode::EmptyGeneratorId,
            ProcgenIssueSubject::Document,
            "procgen generator id must be non-zero",
        ));
    }
    if document.generator_version.trim().is_empty() {
        report.push(issue(
            ProcgenIssueCode::EmptyGeneratorVersion,
            ProcgenIssueSubject::Document,
            "procgen generator version must not be empty",
        ));
    }
    if document.world_seed.trim().is_empty() {
        report.push(issue(
            ProcgenIssueCode::EmptyWorldSeed,
            ProcgenIssueSubject::Document,
            "procgen world seed reference must not be empty",
        ));
    }
    if document.source_revision.trim().is_empty() {
        report.push(issue(
            ProcgenIssueCode::EmptySourceRevision,
            ProcgenIssueSubject::Document,
            "procgen document source revision must not be empty",
        ));
    }
    if !document.scope.is_bounded() {
        report.push(issue(
            ProcgenIssueCode::EmptyScope,
            ProcgenIssueSubject::Scope,
            "procgen scope must include at least one bounded region or chunk",
        ));
    }
    if !document.scope.all_ids_match_world() {
        report.push(issue(
            ProcgenIssueCode::ScopeWorldMismatch,
            ProcgenIssueSubject::Scope,
            "procgen scope ids must all belong to the declared world",
        ));
    }
}

fn ratify_graph(
    document: &ProcgenDocument,
    catalog: &ProcgenNodeCatalog,
    report: &mut ProcgenRatificationReport,
) {
    if let Err(error) = validate_graph(&document.graph) {
        report.push(issue(
            ProcgenIssueCode::GraphStructural,
            ProcgenIssueSubject::Graph,
            graph_error_message(&error),
        ));
    }

    let mut parameter_by_node = BTreeMap::new();
    for parameter in &document.node_parameters {
        if parameter_by_node
            .insert(parameter.node_id, parameter)
            .is_some()
        {
            report.push(issue(
                ProcgenIssueCode::DuplicateNodeParameters,
                ProcgenIssueSubject::Node(parameter.node_id.raw()),
                "procgen node parameters must be unique by graph node id",
            ));
        }
    }

    let mut has_world_ops_output = false;
    let mut has_field_product_output = false;
    for node in &document.graph.nodes {
        let Some(descriptor) = catalog.get(&node.name) else {
            report.push(issue(
                ProcgenIssueCode::UnsupportedNode,
                ProcgenIssueSubject::Node(node.id.raw()),
                format!("procgen node '{}' is not in the active catalog", node.name),
            ));
            continue;
        };
        match descriptor.kind {
            ProcgenNodeKind::WorldOpsOutput => has_world_ops_output = true,
            ProcgenNodeKind::FieldProductOutput => has_field_product_output = true,
            _ => {}
        }
        let Some(parameter) = parameter_by_node.get(&node.id) else {
            report.push(issue(
                ProcgenIssueCode::MissingNodeParameters,
                ProcgenIssueSubject::Node(node.id.raw()),
                "procgen graph node is missing procgen-owned parameters",
            ));
            continue;
        };
        if parameter.kind != descriptor.kind {
            report.push(issue(
                ProcgenIssueCode::NodeParameterKindMismatch,
                ProcgenIssueSubject::Node(node.id.raw()),
                "procgen node parameter kind must match the active catalog descriptor",
            ));
        }
        if parameter.label.trim().is_empty() {
            report.push(issue(
                ProcgenIssueCode::EmptyNodeParameterLabel,
                ProcgenIssueSubject::Node(node.id.raw()),
                "procgen node parameter label must not be empty",
            ));
        }
        if matches!(
            parameter.kind,
            ProcgenNodeKind::HeightNoise | ProcgenNodeKind::MaterialRule
        ) && parameter.seed_salt.trim().is_empty()
        {
            report.push(issue(
                ProcgenIssueCode::EmptySeedSalt,
                ProcgenIssueSubject::Node(node.id.raw()),
                "lowering nodes must declare a deterministic seed salt",
            ));
        }
    }

    if !has_world_ops_output {
        report.push(issue(
            ProcgenIssueCode::MissingWorldOpsOutput,
            ProcgenIssueSubject::Graph,
            "procgen graph must contain a world operation output node",
        ));
    }
    if !has_field_product_output {
        report.push(issue(
            ProcgenIssueCode::MissingFieldProductOutput,
            ProcgenIssueSubject::Graph,
            "procgen graph must contain a field product output node",
        ));
    }
}

fn ratify_inputs(document: &ProcgenDocument, report: &mut ProcgenRatificationReport) {
    for input in &document.input_products {
        if input.product_id.is_empty() {
            report.push(issue(
                ProcgenIssueCode::EmptyInputProduct,
                ProcgenIssueSubject::InputProduct(input.product_id.raw()),
                "procgen input product identities must be non-zero",
            ));
        }
        if input.generation == 0 {
            report.push(issue(
                ProcgenIssueCode::InputGenerationZero,
                ProcgenIssueSubject::InputProduct(input.product_id.raw()),
                "procgen input products must carry non-zero generations",
            ));
        }
    }
}

fn ratify_write_targets(document: &ProcgenDocument, report: &mut ProcgenRatificationReport) {
    if document.write_targets.is_empty() {
        report.push(issue(
            ProcgenIssueCode::EmptyWriteTargets,
            ProcgenIssueSubject::Document,
            "procgen documents must declare at least one write target",
        ));
    }

    let mut target_ids = BTreeSet::new();
    for target in &document.write_targets {
        if target.target_id.trim().is_empty() {
            report.push(issue(
                ProcgenIssueCode::EmptyWriteTarget,
                ProcgenIssueSubject::WriteTarget(target.target_id.clone()),
                "procgen write target id must not be empty",
            ));
        }
        if !target_ids.insert(target.target_id.as_str()) {
            report.push(issue(
                ProcgenIssueCode::DuplicateWriteTarget,
                ProcgenIssueSubject::WriteTarget(target.target_id.clone()),
                "procgen write target ids must be unique",
            ));
        }
        if !target.has_valid_bounds() {
            report.push(issue(
                ProcgenIssueCode::InvalidWriteTargetBounds,
                ProcgenIssueSubject::WriteTarget(target.target_id.clone()),
                "procgen write target bounds must have non-zero volume",
            ));
        }
        if target.kind == ProcgenWriteTargetKind::MaterialChannel {
            match target.material_channel {
                Some(channel) if channel < 16 => {}
                _ => report.push(issue(
                    ProcgenIssueCode::InvalidMaterialChannel,
                    ProcgenIssueSubject::WriteTarget(target.target_id.clone()),
                    "material write targets must declare a material channel in 0..16",
                )),
            }
        }
    }
}

fn ratify_output_products(document: &ProcgenDocument, report: &mut ProcgenRatificationReport) {
    let mut product_ids = BTreeSet::new();
    let mut has_world_ops = false;
    let mut has_field_product = false;
    for output in &document.output_products {
        if output.product_id.is_empty() || output.label.trim().is_empty() {
            report.push(issue(
                ProcgenIssueCode::EmptyOutputProduct,
                ProcgenIssueSubject::OutputProduct(output.product_id.raw()),
                "procgen output products must declare non-zero ids and non-empty labels",
            ));
        }
        if !product_ids.insert(output.product_id) {
            report.push(issue(
                ProcgenIssueCode::DuplicateOutputProduct,
                ProcgenIssueSubject::OutputProduct(output.product_id.raw()),
                "procgen output product identities must be unique",
            ));
        }
        match output.kind {
            ProcgenOutputKind::WorldOpsWindow => has_world_ops = true,
            ProcgenOutputKind::FieldProductCandidate => has_field_product = true,
        }
    }
    if !has_world_ops {
        report.push(issue(
            ProcgenIssueCode::MissingWorldOpsOutputProduct,
            ProcgenIssueSubject::Document,
            "procgen documents must declare a world operation window output product",
        ));
    }
    if !has_field_product {
        report.push(issue(
            ProcgenIssueCode::MissingFieldProductOutputProduct,
            ProcgenIssueSubject::Document,
            "procgen documents must declare a field product candidate output product",
        ));
    }
}

fn ratify_lowering_and_cache(document: &ProcgenDocument, report: &mut ProcgenRatificationReport) {
    if document.lowering_policy.lowering_version.trim().is_empty() {
        report.push(issue(
            ProcgenIssueCode::EmptyLoweringVersion,
            ProcgenIssueSubject::Document,
            "procgen lowering version must not be empty",
        ));
    }
    if document.lowering_policy.fixed_point_scale <= 0 {
        report.push(issue(
            ProcgenIssueCode::InvalidFixedPointScale,
            ProcgenIssueSubject::Document,
            "procgen lowering fixed-point scale must be positive",
        ));
    }
    if document.cache_lineage.parameter_hash.trim().is_empty() {
        report.push(issue(
            ProcgenIssueCode::EmptyCacheLineage,
            ProcgenIssueSubject::CacheLineage,
            "procgen cache lineage must include a parameter hash",
        ));
    } else {
        let expected = parameter_hash_for_document(document);
        if document.cache_lineage.parameter_hash != expected {
            report.push(issue(
                ProcgenIssueCode::CacheLineageDrift,
                ProcgenIssueSubject::CacheLineage,
                "procgen cache lineage parameter hash must match document parameters",
            ));
        }
    }
}

fn ratify_reservations(document: &ProcgenDocument, report: &mut ProcgenRatificationReport) {
    let write_target_ids = document
        .write_targets
        .iter()
        .map(|target| target.target_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut reservation_ids = BTreeSet::new();

    for reservation in &document.reservations {
        if reservation.reservation_id == ProcgenReservationId::default() {
            report.push(issue(
                ProcgenIssueCode::EmptyReservationId,
                ProcgenIssueSubject::Reservation(reservation.reservation_id.raw()),
                "procgen reservations must use non-zero ids",
            ));
        }
        if !reservation_ids.insert(reservation.reservation_id) {
            report.push(issue(
                ProcgenIssueCode::EmptyReservationId,
                ProcgenIssueSubject::Reservation(reservation.reservation_id.raw()),
                "procgen reservation ids must be unique",
            ));
        }
        if !write_target_ids.contains(reservation.target_id.as_str()) {
            report.push(issue(
                ProcgenIssueCode::ReservationMissingWriteTarget,
                ProcgenIssueSubject::Reservation(reservation.reservation_id.raw()),
                "procgen reservation must reference a declared write target",
            ));
        }
    }

    for (index, reservation) in document.reservations.iter().enumerate() {
        for other in document.reservations.iter().skip(index + 1) {
            if reservation.kind == other.kind
                && reservation.material_channel == other.material_channel
                && reservation.overlaps(other)
            {
                report.push(issue(
                    ProcgenIssueCode::ReservationConflict,
                    ProcgenIssueSubject::Reservation(reservation.reservation_id.raw()),
                    "procgen reservations for the same target class must not overlap",
                ));
            }
        }
    }
}

fn issue(
    code: ProcgenIssueCode,
    subject: ProcgenIssueSubject,
    message: impl Into<String>,
) -> RatificationIssue<ProcgenIssueCode, ProcgenIssueSubject> {
    RatificationIssue::error(code, subject, message)
}

fn graph_error_message(error: &GraphValidationError) -> String {
    match error {
        GraphValidationError::DuplicateNodeId(id) => format!("duplicate node id {}", id.raw()),
        GraphValidationError::DuplicatePortId(id) => format!("duplicate port id {}", id.raw()),
        GraphValidationError::DuplicateEdgeId(id) => format!("duplicate edge id {}", id.raw()),
        GraphValidationError::MissingNode(id) => format!("missing node {}", id.raw()),
        GraphValidationError::MissingPort { edge_id, port_id } => {
            format!(
                "edge {} references missing port {}",
                edge_id.raw(),
                port_id.raw()
            )
        }
        GraphValidationError::EdgeDirectionInvalid { edge_id, .. } => {
            format!("edge {} has invalid port directions", edge_id.raw())
        }
        GraphValidationError::PortTypeMismatch { edge_id, .. } => {
            format!("edge {} has mismatched port types", edge_id.raw())
        }
        GraphValidationError::DirectedCycleDetected => "directed cycle detected".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use graph::NodeId;
    use product::ProductIdentity;

    use super::*;
    use crate::{
        ProcgenInputProduct, ProcgenNodeKind, ProcgenNodeParameters, ProcgenOutputKind,
        ProcgenOutputProduct, ProcgenReservation, ProcgenReservationId, ProcgenWriteTarget,
        test_fixtures::{bounds, valid_document},
    };

    #[test]
    fn valid_bounded_terrain_material_document_ratifies() {
        let document = valid_document();

        let report = ratify_procgen_document(&document, &ProcgenNodeCatalog::first_slice());

        assert!(report.is_accepted());
    }

    #[test]
    fn invalid_identity_scope_and_lineage_fields_reject() {
        let mut document = valid_document();
        document.label.clear();
        document.schema_version.clear();
        document.generator_id = Default::default();
        document.generator_version.clear();
        document.world_seed.clear();
        document.source_revision.clear();
        document.scope.chunk_ids.clear();
        document.scope.region_ids.clear();
        document.cache_lineage.parameter_hash = "stale".to_string();

        let report = ratify_procgen_document(&document, &ProcgenNodeCatalog::first_slice());

        for code in [
            ProcgenIssueCode::EmptyLabel,
            ProcgenIssueCode::EmptySchemaVersion,
            ProcgenIssueCode::EmptyGeneratorId,
            ProcgenIssueCode::EmptyGeneratorVersion,
            ProcgenIssueCode::EmptyWorldSeed,
            ProcgenIssueCode::EmptySourceRevision,
            ProcgenIssueCode::EmptyScope,
            ProcgenIssueCode::CacheLineageDrift,
        ] {
            assert!(report.iter().any(|issue| issue.code() == &code), "{code:?}");
        }
    }

    #[test]
    fn unsupported_or_mismatched_nodes_reject() {
        let mut document = valid_document();
        document.graph.nodes[0].name = "procgen.cave_mask".to_string();
        document.node_parameters[1] =
            ProcgenNodeParameters::new(NodeId::new(2), ProcgenNodeKind::HeightNoise, "bad")
                .with_seed_salt("bad");
        document.refresh_cache_lineage();

        let report = ratify_procgen_document(&document, &ProcgenNodeCatalog::first_slice());

        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProcgenIssueCode::UnsupportedNode)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProcgenIssueCode::NodeParameterKindMismatch)
        );
    }

    #[test]
    fn missing_outputs_and_empty_targets_reject() {
        let mut document = valid_document();
        document
            .graph
            .nodes
            .retain(|node| node.id != NodeId::new(4));
        document
            .output_products
            .retain(|output| output.kind != ProcgenOutputKind::FieldProductCandidate);
        document.write_targets.clear();
        document.refresh_cache_lineage();

        let report = ratify_procgen_document(&document, &ProcgenNodeCatalog::first_slice());

        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProcgenIssueCode::MissingFieldProductOutput)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProcgenIssueCode::MissingFieldProductOutputProduct)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProcgenIssueCode::EmptyWriteTargets)
        );
    }

    #[test]
    fn invalid_inputs_write_targets_outputs_and_reservations_reject() {
        let mut document = valid_document()
            .with_input_product(ProcgenInputProduct::new(ProductIdentity::new(0), 0))
            .with_write_target(ProcgenWriteTarget::material_channel(
                "density-main",
                bounds(),
                17,
            ))
            .with_output_product(ProcgenOutputProduct::new(
                ProductIdentity::new(8001),
                ProcgenOutputKind::WorldOpsWindow,
                "duplicate",
            ));
        document.reservations.push(ProcgenReservation::from_target(
            ProcgenReservationId::default(),
            &document.write_targets[0],
        ));
        document.reservations.push(ProcgenReservation::from_target(
            ProcgenReservationId::new(99),
            &document.write_targets[0],
        ));
        document.refresh_cache_lineage();

        let report = ratify_procgen_document(&document, &ProcgenNodeCatalog::first_slice());

        for code in [
            ProcgenIssueCode::EmptyInputProduct,
            ProcgenIssueCode::InputGenerationZero,
            ProcgenIssueCode::DuplicateWriteTarget,
            ProcgenIssueCode::InvalidMaterialChannel,
            ProcgenIssueCode::DuplicateOutputProduct,
            ProcgenIssueCode::EmptyReservationId,
            ProcgenIssueCode::ReservationConflict,
        ] {
            assert!(report.iter().any(|issue| issue.code() == &code), "{code:?}");
        }
    }
}
