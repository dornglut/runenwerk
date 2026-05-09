//! File: domain/editor/editor_scene/src/sdf_authoring/preview.rs
//! Purpose: Deterministic CPU field-preview formation for authored SDF operations.

use glam::Vec3;
use world_sdf::{
    FieldPreviewGrid, FieldPreviewPayload, FieldPreviewProduct, FieldProductDescriptor,
    FieldProductId, FieldProductKind, FieldProductLineage, FieldProductScope,
    ratify_field_preview_product,
};

use crate::{
    SceneQuat, SceneTransform, SdfBooleanIntent, SdfOperationDocument, SdfOperationEntry,
    SdfOperationLoweringContext, SdfPreviewDiagnostic, SdfPrimitiveKind,
    lower_sdf_operation_document,
};

pub const DEFAULT_SDF_FIELD_PREVIEW_GRID_EDGE: u16 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SdfFieldPreviewRequest {
    pub grid_edge: u16,
    pub product_id_seed: u64,
}

impl Default for SdfFieldPreviewRequest {
    fn default() -> Self {
        Self {
            grid_edge: DEFAULT_SDF_FIELD_PREVIEW_GRID_EDGE,
            product_id_seed: 1,
        }
    }
}

impl SdfFieldPreviewRequest {
    pub fn grid_edge(self) -> u16 {
        self.grid_edge.max(1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SdfFieldPreviewFormation {
    pub source_revision: u64,
    pub products: Vec<FieldPreviewProduct>,
    pub diagnostics: Vec<SdfPreviewDiagnostic>,
}

impl SdfFieldPreviewFormation {
    pub fn product_count(&self) -> usize {
        self.products.len()
    }
}

pub fn form_sdf_field_preview_products(
    document: &SdfOperationDocument,
    context: &SdfOperationLoweringContext,
    request: SdfFieldPreviewRequest,
) -> SdfFieldPreviewFormation {
    let candidate = lower_sdf_operation_document(document, context);
    let mut diagnostics = candidate
        .ratification
        .issues
        .iter()
        .filter(|issue| issue.severity == crate::SdfOperationIssueSeverity::Error)
        .map(|issue| {
            SdfPreviewDiagnostic::new(
                "editor_scene.sdf_preview.ratification_error",
                format!("{:?}: {}", issue.code, issue.message),
            )
        })
        .collect::<Vec<_>>();

    if !candidate.can_commit() {
        return SdfFieldPreviewFormation {
            source_revision: document.source_revision(),
            products: Vec::new(),
            diagnostics,
        };
    }

    let operations = enabled_operations(document);
    let mut products = Vec::new();
    for (chunk_index, chunk_id) in candidate.touched_chunks.iter().copied().enumerate() {
        let samples = sample_chunk(document, context, chunk_id, request.grid_edge());
        for (kind_index, kind) in [
            FieldProductKind::ScalarDistance,
            FieldProductKind::VectorGradient,
            FieldProductKind::OccupancySupport,
            FieldProductKind::MaterialChannel,
        ]
        .into_iter()
        .enumerate()
        {
            let product_id = request
                .product_id_seed
                .saturating_add((chunk_index as u64).saturating_mul(4))
                .saturating_add(kind_index as u64)
                .saturating_add(1);
            let descriptor = preview_descriptor(
                FieldProductId(product_id.max(1)),
                kind,
                chunk_id,
                document.source_revision(),
                request.grid_edge(),
            );
            let grid = FieldPreviewGrid::new(
                chunk_id,
                [
                    request.grid_edge(),
                    request.grid_edge(),
                    request.grid_edge(),
                ],
            );
            let payload = match kind {
                FieldProductKind::ScalarDistance => FieldPreviewPayload::ScalarDistance {
                    grid,
                    samples: samples.distances.clone(),
                },
                FieldProductKind::VectorGradient => FieldPreviewPayload::VectorGradient {
                    grid,
                    samples: samples.gradients.clone(),
                },
                FieldProductKind::OccupancySupport => FieldPreviewPayload::OccupancySupport {
                    grid,
                    samples: samples.occupancy.clone(),
                },
                FieldProductKind::MaterialChannel => FieldPreviewPayload::MaterialChannel {
                    grid,
                    samples: samples.material_channels.clone(),
                },
                FieldProductKind::WorldSdfChunkPages | FieldProductKind::BrickmapDebug => {
                    continue;
                }
            };
            let product = FieldPreviewProduct::new(descriptor, payload);
            let report = ratify_field_preview_product(&product);
            if report.has_blocking_issues() {
                diagnostics.extend(report.iter().map(|issue| {
                    SdfPreviewDiagnostic::new(
                        "editor_scene.sdf_preview.product_rejected",
                        format!("{:?}: {}", issue.code(), issue.message()),
                    )
                }));
                continue;
            }
            products.push(product);
        }
    }

    if operations.is_empty() && diagnostics.is_empty() {
        diagnostics.push(SdfPreviewDiagnostic::new(
            "editor_scene.sdf_preview.empty_window",
            "no enabled SDF operations are available for field-preview formation",
        ));
    }

    SdfFieldPreviewFormation {
        source_revision: document.source_revision(),
        products,
        diagnostics,
    }
}

#[derive(Debug, Clone, Default)]
struct ChunkSamples {
    distances: Vec<i16>,
    gradients: Vec<[i16; 3]>,
    occupancy: Vec<u8>,
    material_channels: Vec<u16>,
}

#[derive(Debug, Clone, Copy)]
struct EvaluatedPoint {
    distance: f32,
    material_channel_mask: u16,
}

fn sample_chunk(
    document: &SdfOperationDocument,
    context: &SdfOperationLoweringContext,
    chunk_id: spatial::ChunkId,
    grid_edge: u16,
) -> ChunkSamples {
    let mut samples = ChunkSamples::default();
    let operations = enabled_operations(document);
    let edge = context.partition.chunk_edge_meters.max(1.0);
    let grid_edge_f = f32::from(grid_edge.max(1));
    let step = edge / grid_edge_f;
    let base = Vec3::new(
        chunk_id.coord.x as f32 * edge,
        chunk_id.coord.y as f32 * edge,
        chunk_id.coord.z as f32 * edge,
    );
    let gradient_epsilon = step.max(0.001);

    for z in 0..grid_edge {
        for y in 0..grid_edge {
            for x in 0..grid_edge {
                let point = base
                    + Vec3::new(
                        (f32::from(x) + 0.5) * step,
                        (f32::from(y) + 0.5) * step,
                        (f32::from(z) + 0.5) * step,
                    );
                let evaluated = evaluate_operations(&operations, point);
                let distance = finite_distance_or_max(evaluated.distance);
                let gradient = estimate_preview_gradient(&operations, point, gradient_epsilon);
                samples
                    .distances
                    .push(quantize_distance(distance, context.fixed_point_scale()));
                samples.gradients.push(quantize_normal(gradient));
                samples.occupancy.push(u8::from(distance <= 0.0));
                samples
                    .material_channels
                    .push(evaluated.material_channel_mask);
            }
        }
    }
    samples
}

fn preview_descriptor(
    product_id: FieldProductId,
    kind: FieldProductKind,
    chunk_id: spatial::ChunkId,
    source_revision: u64,
    grid_edge: u16,
) -> FieldProductDescriptor {
    let mut descriptor = FieldProductDescriptor::new(
        product_id,
        kind,
        FieldProductScope::from_chunks([chunk_id]),
        FieldProductLineage::new(source_revision, "editor_scene.sdf_authoring.cpu_preview"),
    );
    descriptor.scale_band = format!("p1_cpu_{}x{}x{}", grid_edge, grid_edge, grid_edge);
    descriptor.rebuild_policy = "rebuild_on_sdf_source_revision_or_dirty_chunk".to_string();
    descriptor
}

fn enabled_operations(document: &SdfOperationDocument) -> Vec<&SdfOperationEntry> {
    document
        .layers()
        .iter()
        .filter(|layer| layer.metadata.enabled)
        .flat_map(|layer| layer.operations.iter())
        .filter(|operation| operation.enabled)
        .collect()
}

fn evaluate_operations(operations: &[&SdfOperationEntry], point: Vec3) -> EvaluatedPoint {
    let mut distance = f32::INFINITY;
    let mut material_channel_mask = 0_u16;
    for operation in operations {
        let primitive_distance = primitive_distance(&operation.primitive, point);
        let before = distance;
        distance = match operation.primitive.boolean {
            SdfBooleanIntent::Add => before.min(primitive_distance),
            SdfBooleanIntent::Subtract => before.max(-primitive_distance),
            SdfBooleanIntent::Intersect => before.max(primitive_distance),
            SdfBooleanIntent::SmoothAdd => smooth_union(
                before,
                primitive_distance,
                operation.primitive.smooth_radius_meters.unwrap_or(0.0),
            ),
            SdfBooleanIntent::SmoothSubtract => smooth_subtract(
                before,
                primitive_distance,
                operation.primitive.smooth_radius_meters.unwrap_or(0.0),
            ),
            SdfBooleanIntent::SmoothIntersect => smooth_intersect(
                before,
                primitive_distance,
                operation.primitive.smooth_radius_meters.unwrap_or(0.0),
            ),
        };

        if matches!(
            operation.primitive.boolean,
            SdfBooleanIntent::Add
                | SdfBooleanIntent::Intersect
                | SdfBooleanIntent::SmoothAdd
                | SdfBooleanIntent::SmoothIntersect
        ) && distance <= 0.0
            && primitive_distance <= operation.primitive.smooth_radius_meters.unwrap_or(0.0)
        {
            material_channel_mask |= material_channel_bit(operation.material_channel);
        }
        if matches!(
            operation.primitive.boolean,
            SdfBooleanIntent::Subtract | SdfBooleanIntent::SmoothSubtract
        ) && primitive_distance <= 0.0
        {
            material_channel_mask = 0;
        }
    }

    EvaluatedPoint {
        distance,
        material_channel_mask,
    }
}

fn primitive_distance(spec: &crate::SdfPrimitiveSpec, point: Vec3) -> f32 {
    let transform = spec.transform;
    let center = Vec3::new(
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
    );
    let local = point - center;
    match spec.kind {
        SdfPrimitiveKind::Sphere => local.length() - max_abs_scale(transform),
        SdfPrimitiveKind::Box => {
            let half_extents = Vec3::new(
                transform.scale.x.abs().max(0.001),
                transform.scale.y.abs().max(0.001),
                transform.scale.z.abs().max(0.001),
            );
            let q = local.abs() - half_extents;
            q.max(Vec3::ZERO).length() + q.x.max(q.y).max(q.z).min(0.0)
        }
        SdfPrimitiveKind::Capsule => {
            let half_height = transform.scale.y.abs().max(0.001);
            let radius = transform
                .scale
                .x
                .abs()
                .max(transform.scale.z.abs())
                .max(0.001);
            let start = center - Vec3::Y * half_height;
            let end = center + Vec3::Y * half_height;
            let pa = point - start;
            let ba = end - start;
            let h = (pa.dot(ba) / ba.length_squared().max(f32::EPSILON)).clamp(0.0, 1.0);
            (point - (start + ba * h)).length() - radius
        }
        SdfPrimitiveKind::Cylinder => {
            let radius = transform
                .scale
                .x
                .abs()
                .max(transform.scale.z.abs())
                .max(0.001);
            let half_height = transform.scale.y.abs().max(0.001);
            let d = Vec3::new(local.x, 0.0, local.z).length() - radius;
            let y = local.y.abs() - half_height;
            Vec3::new(d.max(0.0), y.max(0.0), 0.0).length() + d.max(y).min(0.0)
        }
        SdfPrimitiveKind::Torus => {
            let major = transform
                .scale
                .x
                .abs()
                .max(transform.scale.z.abs())
                .max(0.001);
            let minor = transform.scale.y.abs().max(0.001);
            let q = Vec3::new(
                Vec3::new(local.x, 0.0, local.z).length() - major,
                local.y,
                0.0,
            );
            Vec3::new(q.x, q.y, 0.0).length() - minor
        }
        SdfPrimitiveKind::Plane => {
            let normal = rotated_axis_y(transform.rotation);
            normal.dot(point - center)
        }
    }
}

fn estimate_preview_gradient(operations: &[&SdfOperationEntry], point: Vec3, epsilon: f32) -> Vec3 {
    let dx = Vec3::X * epsilon;
    let dy = Vec3::Y * epsilon;
    let dz = Vec3::Z * epsilon;
    let gradient = Vec3::new(
        evaluate_operations(operations, point + dx).distance
            - evaluate_operations(operations, point - dx).distance,
        evaluate_operations(operations, point + dy).distance
            - evaluate_operations(operations, point - dy).distance,
        evaluate_operations(operations, point + dz).distance
            - evaluate_operations(operations, point - dz).distance,
    ) / (2.0 * epsilon);
    if gradient.is_finite() && gradient.length_squared() > f32::EPSILON {
        gradient.normalize()
    } else {
        Vec3::Y
    }
}

fn smooth_union(a: f32, b: f32, k: f32) -> f32 {
    if !a.is_finite() {
        return b;
    }
    let k = k.max(0.0);
    if k <= f32::EPSILON {
        return a.min(b);
    }
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

fn smooth_subtract(a: f32, b: f32, k: f32) -> f32 {
    if !a.is_finite() {
        return a;
    }
    let k = k.max(0.0);
    if k <= f32::EPSILON {
        return a.max(-b);
    }
    let h = (0.5 - 0.5 * (b + a) / k).clamp(0.0, 1.0);
    a + (-b - a) * h + k * h * (1.0 - h)
}

fn smooth_intersect(a: f32, b: f32, k: f32) -> f32 {
    if !a.is_finite() {
        return a;
    }
    let k = k.max(0.0);
    if k <= f32::EPSILON {
        return a.max(b);
    }
    let h = (0.5 - 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h + k * h * (1.0 - h)
}

fn quantize_distance(distance: f32, fixed_point_scale: i32) -> i16 {
    (distance * fixed_point_scale.max(1) as f32)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

fn quantize_normal(normal: Vec3) -> [i16; 3] {
    [
        quantize_normal_component(normal.x),
        quantize_normal_component(normal.y),
        quantize_normal_component(normal.z),
    ]
}

fn quantize_normal_component(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16
}

fn finite_distance_or_max(distance: f32) -> f32 {
    if distance.is_finite() {
        distance
    } else {
        i16::MAX as f32
    }
}

fn max_abs_scale(transform: SceneTransform) -> f32 {
    transform
        .scale
        .x
        .abs()
        .max(transform.scale.y.abs())
        .max(transform.scale.z.abs())
        .max(0.001)
}

fn material_channel_bit(material_channel: u16) -> u16 {
    1_u16
        .checked_shl((material_channel as u32).min(15))
        .unwrap_or(1)
}

fn rotated_axis_y(rotation: SceneQuat) -> Vec3 {
    let qv = Vec3::new(rotation.x, rotation.y, rotation.z);
    let v = Vec3::Y;
    let t = 2.0 * qv.cross(v);
    let normal = v + rotation.w * t + qv.cross(t);
    if normal.length_squared() <= f32::EPSILON {
        Vec3::Y
    } else {
        normal.normalize()
    }
}

#[cfg(test)]
mod tests {
    use spatial::GridPartitionConfig;
    use world_sdf::FieldProductKind;

    use super::*;
    use crate::{SceneTransform, SceneVec3, SdfPrimitiveSpec};

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
    fn field_preview_forms_four_product_kinds_per_touched_chunk() {
        let mut document = SdfOperationDocument::with_default_layer("test", "Test");
        let layer_id = document.layers()[0].id;
        document
            .add_operation(layer_id, "Sphere", sphere(SdfBooleanIntent::Add), 2)
            .expect("add operation");
        let context = SdfOperationLoweringContext {
            partition: GridPartitionConfig {
                chunk_edge_meters: 1.0,
                fixed_point_scale: 8,
                ..GridPartitionConfig::default()
            },
            ..SdfOperationLoweringContext::default()
        };

        let formation = form_sdf_field_preview_products(
            &document,
            &context,
            SdfFieldPreviewRequest {
                grid_edge: 4,
                product_id_seed: 10,
            },
        );

        assert!(formation.diagnostics.is_empty());
        assert!(!formation.products.is_empty());
        assert_eq!(formation.products.len() % 4, 0);
        assert!(
            formation
                .products
                .iter()
                .any(|product| product.descriptor.kind == FieldProductKind::ScalarDistance)
        );
        assert!(formation.products.iter().all(|product| {
            product.payload.sample_count() == product.payload.grid().expected_sample_count()
        }));
    }

    #[test]
    fn smooth_preview_without_radius_fails_closed() {
        let mut document = SdfOperationDocument::with_default_layer("test", "Test");
        let layer_id = document.layers()[0].id;
        document
            .add_operation(layer_id, "Smooth", sphere(SdfBooleanIntent::SmoothAdd), 0)
            .expect("add operation");

        let formation = form_sdf_field_preview_products(
            &document,
            &SdfOperationLoweringContext::default(),
            SdfFieldPreviewRequest::default(),
        );

        assert!(formation.products.is_empty());
        assert!(!formation.diagnostics.is_empty());
    }
}
