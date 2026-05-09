//! File: domain/editor/editor_scene/src/sdf_authoring/projection.rs
//! Purpose: Read-only editor projection DTOs for SDF operation surfaces.

use crate::{
    SdfOperationDocument, SdfOperationEntryId, SdfOperationIssue, SdfOperationLayerId,
    SdfOperationWindowCandidate, lower_sdf_operation_document,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfOperationLayerProjection {
    pub layer_id: SdfOperationLayerId,
    pub display_name: String,
    pub enabled: bool,
    pub operation_count: usize,
    pub enabled_operation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfOperationEntryProjection {
    pub layer_id: SdfOperationLayerId,
    pub operation_id: SdfOperationEntryId,
    pub display_name: String,
    pub enabled: bool,
    pub boolean_intent: &'static str,
    pub primitive_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfOperationDocumentProjection {
    pub display_name: String,
    pub source_revision: u64,
    pub layers: Vec<SdfOperationLayerProjection>,
    pub operations: Vec<SdfOperationEntryProjection>,
    pub issues: Vec<SdfOperationIssue>,
    pub lowered_operation_count: usize,
    pub touched_chunk_count: usize,
    pub can_commit: bool,
}

impl SdfOperationDocumentProjection {
    pub fn from_document(
        document: &SdfOperationDocument,
        candidate: &SdfOperationWindowCandidate,
    ) -> Self {
        Self {
            display_name: document.display_name.clone(),
            source_revision: document.source_revision(),
            layers: document
                .layers()
                .iter()
                .map(|layer| SdfOperationLayerProjection {
                    layer_id: layer.id,
                    display_name: layer.metadata.display_name.clone(),
                    enabled: layer.metadata.enabled,
                    operation_count: layer.operations.len(),
                    enabled_operation_count: layer.enabled_operation_count(),
                })
                .collect(),
            operations: document
                .layers()
                .iter()
                .flat_map(|layer| {
                    layer
                        .operations
                        .iter()
                        .map(move |operation| SdfOperationEntryProjection {
                            layer_id: layer.id,
                            operation_id: operation.id,
                            display_name: operation.display_name.clone(),
                            enabled: operation.enabled,
                            boolean_intent: boolean_intent_label(operation.primitive.boolean),
                            primitive_kind: primitive_kind_label(operation.primitive.kind),
                        })
                })
                .collect(),
            issues: candidate.ratification.issues.clone(),
            lowered_operation_count: candidate.operation_count(),
            touched_chunk_count: candidate.touched_chunks.len(),
            can_commit: candidate.can_commit(),
        }
    }
}

pub fn project_sdf_operation_document(
    document: &SdfOperationDocument,
    context: &crate::SdfOperationLoweringContext,
) -> SdfOperationDocumentProjection {
    let candidate = lower_sdf_operation_document(document, context);
    SdfOperationDocumentProjection::from_document(document, &candidate)
}

fn boolean_intent_label(intent: crate::SdfBooleanIntent) -> &'static str {
    match intent {
        crate::SdfBooleanIntent::Add => "add",
        crate::SdfBooleanIntent::Subtract => "subtract",
        crate::SdfBooleanIntent::Intersect => "intersect",
        crate::SdfBooleanIntent::SmoothAdd => "smooth_add",
        crate::SdfBooleanIntent::SmoothSubtract => "smooth_subtract",
        crate::SdfBooleanIntent::SmoothIntersect => "smooth_intersect",
    }
}

fn primitive_kind_label(kind: crate::SdfPrimitiveKind) -> &'static str {
    match kind {
        crate::SdfPrimitiveKind::Box => "box",
        crate::SdfPrimitiveKind::Sphere => "sphere",
        crate::SdfPrimitiveKind::Capsule => "capsule",
        crate::SdfPrimitiveKind::Cylinder => "cylinder",
        crate::SdfPrimitiveKind::Torus => "torus",
        crate::SdfPrimitiveKind::Plane => "plane",
    }
}

#[cfg(test)]
mod tests {
    use spatial::GridPartitionConfig;
    use world_ops::{CsgBooleanMode, Operation};

    use crate::{
        SceneQuat, SceneTransform, SceneVec3, SdfBooleanIntent, SdfLayerMoveDirection,
        SdfOperationCommandIntent, SdfOperationDocument, SdfOperationIssueCode,
        SdfOperationLoweringContext, SdfPrimitiveKind, SdfPrimitiveSpec,
        lower_sdf_operation_document,
    };

    fn sphere(boolean: SdfBooleanIntent) -> SdfPrimitiveSpec {
        SdfPrimitiveSpec::new(SdfPrimitiveKind::Sphere, boolean).with_transform(
            SceneTransform::new(
                SceneVec3::new(0.5, 0.5, 0.5),
                SceneQuat::identity(),
                SceneVec3::one(),
            ),
        )
    }

    #[test]
    fn sdf_operation_commands_add_reorder_disable_and_edit_document() {
        let mut document = SdfOperationDocument::new("test", "Test");
        let first = SdfOperationCommandIntent::AddLayer {
            stable_name: "first".to_string(),
            display_name: "First".to_string(),
        }
        .apply_to(&mut document)
        .expect("add layer");
        let second = document.add_layer("second", "Second");
        let crate::SdfOperationCommandOutcome::Layer(first) = first else {
            panic!("expected layer id");
        };

        SdfOperationCommandIntent::MoveLayer {
            layer_id: second,
            direction: SdfLayerMoveDirection::Up,
        }
        .apply_to(&mut document)
        .expect("move layer");
        assert_eq!(document.layers()[0].id, second);

        let outcome = SdfOperationCommandIntent::AddPrimitiveOperation {
            layer_id: first,
            display_name: "Sphere".to_string(),
            primitive: sphere(SdfBooleanIntent::Add),
            material_channel: 3,
        }
        .apply_to(&mut document)
        .expect("add operation");
        let crate::SdfOperationCommandOutcome::Operation(operation_id) = outcome else {
            panic!("expected operation id");
        };
        SdfOperationCommandIntent::SetOperationEnabled {
            operation_id,
            enabled: false,
        }
        .apply_to(&mut document)
        .expect("disable operation");
        assert!(!document.operation(operation_id).unwrap().enabled);
    }

    #[test]
    fn supported_sdf_operations_lower_to_deterministic_world_ops_records() {
        let mut document = SdfOperationDocument::with_default_layer("test", "Test");
        let layer_id = document.layers()[0].id;
        document
            .add_operation(layer_id, "Sphere", sphere(SdfBooleanIntent::Add), 7)
            .expect("add operation");
        let context = SdfOperationLoweringContext {
            partition: GridPartitionConfig {
                chunk_edge_meters: 1.0,
                fixed_point_scale: 8,
                ..GridPartitionConfig::default()
            },
            ..SdfOperationLoweringContext::default()
        };

        let first = lower_sdf_operation_document(&document, &context);
        let second = lower_sdf_operation_document(&document, &context);

        assert!(first.can_commit());
        assert_eq!(first, second);
        assert_eq!(first.operation_count(), 1);
        assert!(matches!(
            first.records[0].record.operation,
            Operation::CsgBrush(world_ops::CsgBrushOperation {
                mode: CsgBooleanMode::Add,
                material_channel: Some(7),
                ..
            })
        ));
        assert!(!first.touched_chunks.is_empty());
    }

    #[test]
    fn p1_boolean_intents_lower_to_world_ops_records() {
        let mut document = SdfOperationDocument::with_default_layer("test", "Test");
        let layer_id = document.layers()[0].id;
        for (name, primitive) in [
            ("Intersect", sphere(SdfBooleanIntent::Intersect)),
            (
                "Smooth Add",
                sphere(SdfBooleanIntent::SmoothAdd).with_smooth_radius_meters(1.0),
            ),
            (
                "Smooth Subtract",
                sphere(SdfBooleanIntent::SmoothSubtract).with_smooth_radius_meters(1.0),
            ),
            (
                "Smooth Intersect",
                sphere(SdfBooleanIntent::SmoothIntersect).with_smooth_radius_meters(1.0),
            ),
        ] {
            document
                .add_operation(layer_id, name, primitive, 0)
                .expect("add operation");
        }

        let candidate =
            lower_sdf_operation_document(&document, &SdfOperationLoweringContext::default());

        assert!(candidate.can_commit());
        assert_eq!(candidate.operation_count(), 4);
    }

    #[test]
    fn smooth_boolean_without_radius_blocks_commit() {
        let mut document = SdfOperationDocument::with_default_layer("test", "Test");
        let layer_id = document.layers()[0].id;
        document
            .add_operation(layer_id, "Smooth", sphere(SdfBooleanIntent::SmoothAdd), 0)
            .expect("add operation");

        let candidate =
            lower_sdf_operation_document(&document, &SdfOperationLoweringContext::default());

        assert!(!candidate.can_commit());
        assert_eq!(candidate.operation_count(), 0);
        assert!(
            candidate
                .ratification
                .issues
                .iter()
                .any(|issue| { issue.code == SdfOperationIssueCode::MissingSmoothRadius })
        );
    }
}
