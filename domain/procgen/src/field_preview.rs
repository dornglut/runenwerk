//! File: domain/procgen/src/field_preview.rs
//! Purpose: Deterministic CPU field-preview formation for procgen documents.

use spatial::ChunkId;
use world_ops::QuantizedAabb;
use world_sdf::{
    FieldPreviewGrid, FieldPreviewPayload, FieldPreviewProduct, FieldProductDescriptor,
    FieldProductId, FieldProductKind, FieldProductLineage, FieldProductScope,
    ratify_field_preview_product,
};

use crate::{
    ProcgenDocument, ProcgenExplanationEntry, ProcgenNodeCatalog, ProcgenNodeKind,
    ProcgenNodeParameters, ProcgenRatificationReport, ProcgenWriteTarget, ProcgenWriteTargetKind,
    determinism::{
        determinism_key_for_document, parameter_hash_for_document, stable_nonzero_hash64,
    },
    lower_procgen_to_world_ops, ratify_procgen_document,
};

pub const DEFAULT_PROCGEN_FIELD_PREVIEW_MAX_AXIS: u16 = 32;
pub const DEFAULT_PROCGEN_FIELD_PREVIEW_MAX_SAMPLES: usize = 32 * 32 * 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcgenFieldPreviewPolicy {
    pub max_axis: u16,
    pub max_samples: usize,
}

impl Default for ProcgenFieldPreviewPolicy {
    fn default() -> Self {
        Self {
            max_axis: DEFAULT_PROCGEN_FIELD_PREVIEW_MAX_AXIS,
            max_samples: DEFAULT_PROCGEN_FIELD_PREVIEW_MAX_SAMPLES,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcgenFieldPreviewDiagnosticCode {
    RatificationRejected,
    MissingDensityTarget,
    MissingMaterialTarget,
    MissingHeightNoiseNode,
    MissingMaterialRuleNode,
    MissingMaterialChannel,
    InvalidGridExtent,
    PreviewProductRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenFieldPreviewDiagnostic {
    pub code: ProcgenFieldPreviewDiagnosticCode,
    pub message: String,
}

impl ProcgenFieldPreviewDiagnostic {
    pub fn new(code: ProcgenFieldPreviewDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenFieldPreviewFormation {
    pub report: ProcgenRatificationReport,
    pub determinism_key: Option<String>,
    pub products: Vec<FieldPreviewProduct>,
    pub diagnostics: Vec<ProcgenFieldPreviewDiagnostic>,
    pub explanations: Vec<ProcgenExplanationEntry>,
}

impl ProcgenFieldPreviewFormation {
    pub fn is_accepted(&self) -> bool {
        !self.report.has_blocking_issues() && self.diagnostics.is_empty()
    }
}

pub fn form_procgen_field_preview_products(
    document: &ProcgenDocument,
    catalog: &ProcgenNodeCatalog,
    policy: ProcgenFieldPreviewPolicy,
) -> ProcgenFieldPreviewFormation {
    let report = ratify_procgen_document(document, catalog);
    if report.has_blocking_issues() {
        return ProcgenFieldPreviewFormation {
            report,
            determinism_key: None,
            products: Vec::new(),
            diagnostics: vec![ProcgenFieldPreviewDiagnostic::new(
                ProcgenFieldPreviewDiagnosticCode::RatificationRejected,
                "procgen document has blocking ratification issues",
            )],
            explanations: Vec::new(),
        };
    }

    let lowering = lower_procgen_to_world_ops(document, catalog);
    if lowering.report.has_blocking_issues() {
        return ProcgenFieldPreviewFormation {
            report: lowering.report,
            determinism_key: None,
            products: Vec::new(),
            diagnostics: vec![ProcgenFieldPreviewDiagnostic::new(
                ProcgenFieldPreviewDiagnosticCode::RatificationRejected,
                "procgen document cannot lower to a world operation window",
            )],
            explanations: Vec::new(),
        };
    }

    let determinism_key = determinism_key_for_document(document).0;
    let Some(density_target) = first_sorted_target(document, ProcgenWriteTargetKind::DensityField)
    else {
        return rejected_formation(
            report,
            determinism_key,
            ProcgenFieldPreviewDiagnosticCode::MissingDensityTarget,
            "procgen field preview requires a density write target",
        );
    };
    let Some(material_target) =
        first_sorted_target(document, ProcgenWriteTargetKind::MaterialChannel)
    else {
        return rejected_formation(
            report,
            determinism_key,
            ProcgenFieldPreviewDiagnosticCode::MissingMaterialTarget,
            "procgen field preview requires a material-channel write target",
        );
    };
    let Some(height_node) = first_node_parameter(document, ProcgenNodeKind::HeightNoise) else {
        return rejected_formation(
            report,
            determinism_key,
            ProcgenFieldPreviewDiagnosticCode::MissingHeightNoiseNode,
            "procgen field preview requires a height-noise node",
        );
    };
    let Some(material_node) = first_node_parameter(document, ProcgenNodeKind::MaterialRule) else {
        return rejected_formation(
            report,
            determinism_key,
            ProcgenFieldPreviewDiagnosticCode::MissingMaterialRuleNode,
            "procgen field preview requires a material-rule node",
        );
    };
    let Some(material_channel) = material_node
        .material_channel
        .or(material_target.material_channel)
    else {
        return rejected_formation(
            report,
            determinism_key,
            ProcgenFieldPreviewDiagnosticCode::MissingMaterialChannel,
            "procgen field preview requires a material channel",
        );
    };

    let Some(dimensions) = grid_dimensions(density_target.bounds_q, policy) else {
        return rejected_formation(
            report,
            determinism_key,
            ProcgenFieldPreviewDiagnosticCode::InvalidGridExtent,
            format!(
                "procgen field preview bounds must be non-zero, each axis <= {}, and samples <= {}",
                policy.max_axis, policy.max_samples
            ),
        );
    };

    let mut chunk_ids = document.scope.chunk_ids.clone();
    chunk_ids.sort();
    let mut diagnostics = Vec::new();
    let mut products = Vec::new();
    for chunk_id in chunk_ids {
        let context = PreviewSamplingContext {
            document,
            bounds_q: density_target.bounds_q,
            height_node,
            material_node,
            material_channel,
            determinism_key: determinism_key.as_str(),
        };
        let samples = sample_chunk(context, chunk_id, dimensions);
        let grid = FieldPreviewGrid::new(chunk_id, dimensions);
        for (kind, payload) in [
            (
                FieldProductKind::ScalarDistance,
                FieldPreviewPayload::ScalarDistance {
                    grid,
                    samples: samples.scalar_distances.clone(),
                },
            ),
            (
                FieldProductKind::MaterialChannel,
                FieldPreviewPayload::MaterialChannel {
                    grid,
                    samples: samples.material_channels.clone(),
                },
            ),
        ] {
            let descriptor = preview_descriptor(document, chunk_id, kind, determinism_key.as_str());
            let product = FieldPreviewProduct::new(descriptor, payload);
            let product_report = ratify_field_preview_product(&product);
            if product_report.has_blocking_issues() {
                diagnostics.extend(product_report.iter().map(|issue| {
                    ProcgenFieldPreviewDiagnostic::new(
                        ProcgenFieldPreviewDiagnosticCode::PreviewProductRejected,
                        format!("{:?}: {}", issue.code(), issue.message()),
                    )
                }));
                continue;
            }
            products.push(product);
        }
    }

    let explanations = products
        .iter()
        .map(|product| {
            ProcgenExplanationEntry::new(
                format!("field_preview:{}", product.descriptor.product_id.0),
                format!(
                    "formed deterministic {:?} preview over grid {:?}",
                    product.descriptor.kind,
                    product.payload.grid().dimensions
                ),
            )
        })
        .collect();

    ProcgenFieldPreviewFormation {
        report,
        determinism_key: Some(determinism_key),
        products,
        diagnostics,
        explanations,
    }
}

#[derive(Debug, Clone, Default)]
struct ProcgenFieldPreviewSamples {
    scalar_distances: Vec<i16>,
    material_channels: Vec<u16>,
}

#[derive(Debug, Clone, Copy)]
struct PreviewSamplingContext<'a> {
    document: &'a ProcgenDocument,
    bounds_q: QuantizedAabb,
    height_node: &'a ProcgenNodeParameters,
    material_node: &'a ProcgenNodeParameters,
    material_channel: u16,
    determinism_key: &'a str,
}

fn sample_chunk(
    context: PreviewSamplingContext<'_>,
    chunk_id: ChunkId,
    dimensions: [u16; 3],
) -> ProcgenFieldPreviewSamples {
    let mut samples = ProcgenFieldPreviewSamples::default();
    let scale = context.document.lowering_policy.fixed_point_scale.max(1);
    let y_extent = (context.bounds_q.max.y - context.bounds_q.min.y).max(1) as f32;
    let amplitude = (context.height_node.weight.abs().max(1) as f32 / 16.0).clamp(0.0625, 1.0);
    let material_mask = 1u16 << context.material_channel;

    for z in 0..dimensions[2] {
        for y in 0..dimensions[1] {
            for x in 0..dimensions[0] {
                let nx = normalized_sample(x, dimensions[0]);
                let ny = normalized_sample(y, dimensions[1]);
                let nz = normalized_sample(z, dimensions[2]);
                let height = terrain_height(context, chunk_id, nx, nz, amplitude);
                let y_position = context.bounds_q.min.y as f32 + ny * y_extent;
                let terrain_position = context.bounds_q.min.y as f32 + height * y_extent;
                let distance = y_position - terrain_position;
                samples
                    .scalar_distances
                    .push(quantize_distance(distance, scale));
                samples
                    .material_channels
                    .push(if distance <= 0.0 { material_mask } else { 0 });
            }
        }
    }

    samples
}

fn terrain_height(
    context: PreviewSamplingContext<'_>,
    chunk_id: ChunkId,
    nx: f32,
    nz: f32,
    amplitude: f32,
) -> f32 {
    let height_noise = bilerp(
        corner_noise(
            context.document,
            chunk_id,
            0,
            0,
            context.height_node,
            context.determinism_key,
        ),
        corner_noise(
            context.document,
            chunk_id,
            1,
            0,
            context.height_node,
            context.determinism_key,
        ),
        corner_noise(
            context.document,
            chunk_id,
            0,
            1,
            context.height_node,
            context.determinism_key,
        ),
        corner_noise(
            context.document,
            chunk_id,
            1,
            1,
            context.height_node,
            context.determinism_key,
        ),
        smoothstep(nx),
        smoothstep(nz),
    );
    let material_bias = (node_noise(
        context.document,
        context.material_node,
        context.determinism_key,
    ) - 0.5)
        * 0.2;
    (0.5 + (height_noise - 0.5) * amplitude + material_bias).clamp(0.05, 0.95)
}

fn corner_noise(
    document: &ProcgenDocument,
    chunk_id: ChunkId,
    x: i32,
    z: i32,
    node: &ProcgenNodeParameters,
    determinism_key: &str,
) -> f32 {
    let chunk = format!(
        "{}:{}:{}",
        chunk_id.coord.x + x,
        chunk_id.coord.y,
        chunk_id.coord.z + z
    );
    hash_unit([
        "procgen.field_preview.corner",
        determinism_key,
        document.world_seed.as_str(),
        document.generator_version.as_str(),
        document.source_revision.as_str(),
        node.seed_salt.as_str(),
        chunk.as_str(),
    ])
}

fn node_noise(
    document: &ProcgenDocument,
    node: &ProcgenNodeParameters,
    determinism_key: &str,
) -> f32 {
    let parameter_hash = parameter_hash_for_document(document);
    hash_unit([
        "procgen.field_preview.node",
        determinism_key,
        document.world_seed.as_str(),
        node.seed_salt.as_str(),
        parameter_hash.as_str(),
    ])
}

fn preview_descriptor(
    document: &ProcgenDocument,
    chunk_id: ChunkId,
    kind: FieldProductKind,
    determinism_key: &str,
) -> FieldProductDescriptor {
    let source_revision = stable_nonzero_hash64([
        "procgen.field_preview.source_revision",
        document.source_revision.as_str(),
        determinism_key,
    ]);
    let producer = format!(
        "procgen.generator.{}.field_preview",
        document.generator_id.raw()
    );
    let mut descriptor = FieldProductDescriptor::new(
        FieldProductId(stable_nonzero_hash64([
            "procgen.field_preview.product",
            determinism_key,
            kind.product_kind_name(),
            chunk_label(chunk_id).as_str(),
        ])),
        kind,
        FieldProductScope::from_chunks([chunk_id]),
        FieldProductLineage::new(source_revision, producer),
    );
    descriptor.scale_band = "preview".to_string();
    descriptor.rebuild_policy = "rebuild_on_procgen_determinism_key_change".to_string();
    descriptor
}

fn grid_dimensions(bounds_q: QuantizedAabb, policy: ProcgenFieldPreviewPolicy) -> Option<[u16; 3]> {
    let dimensions = [
        checked_extent(bounds_q.min.x, bounds_q.max.x, policy.max_axis)?,
        checked_extent(bounds_q.min.y, bounds_q.max.y, policy.max_axis)?,
        checked_extent(bounds_q.min.z, bounds_q.max.z, policy.max_axis)?,
    ];
    let samples = dimensions
        .iter()
        .map(|dimension| usize::from(*dimension))
        .product::<usize>();
    (samples <= policy.max_samples).then_some(dimensions)
}

fn checked_extent(min: i32, max: i32, max_axis: u16) -> Option<u16> {
    let extent = max.checked_sub(min)?;
    if extent <= 0 || extent > i32::from(max_axis) {
        return None;
    }
    u16::try_from(extent).ok()
}

fn first_sorted_target(
    document: &ProcgenDocument,
    kind: ProcgenWriteTargetKind,
) -> Option<&ProcgenWriteTarget> {
    document
        .write_targets
        .iter()
        .filter(|target| target.kind == kind)
        .min_by_key(|target| {
            (
                target.target_id.as_str(),
                target.bounds_q.min.x,
                target.bounds_q.min.y,
                target.bounds_q.min.z,
                target.bounds_q.max.x,
                target.bounds_q.max.y,
                target.bounds_q.max.z,
            )
        })
}

fn first_node_parameter(
    document: &ProcgenDocument,
    kind: ProcgenNodeKind,
) -> Option<&ProcgenNodeParameters> {
    document
        .node_parameters
        .iter()
        .filter(|parameter| parameter.kind == kind)
        .min_by_key(|parameter| parameter.node_id)
}

fn rejected_formation(
    report: ProcgenRatificationReport,
    determinism_key: String,
    code: ProcgenFieldPreviewDiagnosticCode,
    message: impl Into<String>,
) -> ProcgenFieldPreviewFormation {
    ProcgenFieldPreviewFormation {
        report,
        determinism_key: Some(determinism_key),
        products: Vec::new(),
        diagnostics: vec![ProcgenFieldPreviewDiagnostic::new(code, message)],
        explanations: Vec::new(),
    }
}

fn normalized_sample(index: u16, dimension: u16) -> f32 {
    (f32::from(index) + 0.5) / f32::from(dimension.max(1))
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn bilerp(a: f32, b: f32, c: f32, d: f32, tx: f32, tz: f32) -> f32 {
    let x0 = a + (b - a) * tx;
    let x1 = c + (d - c) * tx;
    x0 + (x1 - x0) * tz
}

fn hash_unit(parts: impl IntoIterator<Item = impl AsRef<str>>) -> f32 {
    (stable_nonzero_hash64(parts) as f64 / u64::MAX as f64) as f32
}

fn quantize_distance(distance: f32, fixed_point_scale: i32) -> i16 {
    (distance * fixed_point_scale as f32)
        .round()
        .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16
}

fn chunk_label(chunk_id: ChunkId) -> String {
    format!(
        "world:{}:chunk:{}:{}:{}",
        chunk_id.world_id.0, chunk_id.coord.x, chunk_id.coord.y, chunk_id.coord.z
    )
}

#[cfg(test)]
mod tests {
    use product::{ProductIdentity, ratify_product_publication};
    use spatial::{ChunkCoord3, ChunkId};
    use world_ops::QuantizedVec3;
    use world_sdf::{FieldProductKind, ratify_field_preview_product};

    use super::*;
    use crate::{
        ProcgenInputProduct, ProcgenNodeCatalog,
        products::build_procgen_formed_preview_publication_outcome,
        test_fixtures::{bounds, valid_document},
    };

    #[test]
    fn valid_document_forms_scalar_and_material_preview_products() {
        let document = valid_document();
        let formation = form_procgen_field_preview_products(
            &document,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );

        assert!(formation.is_accepted());
        assert_eq!(formation.products.len(), 2);
        assert!(
            formation
                .products
                .iter()
                .any(|product| product.descriptor.kind == FieldProductKind::ScalarDistance)
        );
        assert!(
            formation
                .products
                .iter()
                .any(|product| product.descriptor.kind == FieldProductKind::MaterialChannel)
        );
        for product in &formation.products {
            assert!(!ratify_field_preview_product(product).has_blocking_issues());
            assert_eq!(product.payload.grid().dimensions, [16, 16, 16]);
            assert_eq!(product.payload.sample_count(), 16 * 16 * 16);
        }
    }

    #[test]
    fn formed_preview_publication_passes_product_ratifier() {
        let document = valid_document();
        let formation = form_procgen_field_preview_products(
            &document,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );
        let outcome = build_procgen_formed_preview_publication_outcome(
            &document,
            &ProcgenNodeCatalog::first_slice(),
            &formation.products,
            1,
        )
        .expect("formed preview publication should build");

        assert!(!ratify_product_publication(&outcome).has_blocking_issues());
    }

    #[test]
    fn identical_inputs_produce_identical_preview_products_and_explanations() {
        let first = valid_document();
        let second = valid_document();

        let first_formation = form_procgen_field_preview_products(
            &first,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );
        let second_formation = form_procgen_field_preview_products(
            &second,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );

        assert_eq!(first_formation.products, second_formation.products);
        assert_eq!(first_formation.explanations, second_formation.explanations);
    }

    #[test]
    fn deterministic_input_changes_alter_product_ids_or_samples() {
        let first = valid_document();
        let mut second = valid_document();
        second.input_products.clear();
        second = second.with_input_product(ProcgenInputProduct::new(ProductIdentity::new(77), 13));
        second.refresh_cache_lineage();

        let first_formation = form_procgen_field_preview_products(
            &first,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );
        let second_formation = form_procgen_field_preview_products(
            &second,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );

        assert_ne!(first_formation.products, second_formation.products);
    }

    #[test]
    fn oversized_bounds_reject_without_products() {
        let mut document = valid_document();
        for target in &mut document.write_targets {
            target.bounds_q.max = QuantizedVec3 {
                x: 33,
                y: 16,
                z: 16,
            };
        }
        document.refresh_cache_lineage();

        let formation = form_procgen_field_preview_products(
            &document,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );

        assert!(formation.products.is_empty());
        assert!(formation.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ProcgenFieldPreviewDiagnosticCode::InvalidGridExtent
        }));
    }

    #[test]
    fn missing_material_channel_rejects_without_products() {
        let mut document = valid_document();
        for parameter in &mut document.node_parameters {
            if parameter.kind == ProcgenNodeKind::MaterialRule {
                parameter.material_channel = None;
            }
        }
        for target in &mut document.write_targets {
            if target.kind == ProcgenWriteTargetKind::MaterialChannel {
                target.material_channel = None;
            }
        }
        document.refresh_cache_lineage();

        let formation = form_procgen_field_preview_products(
            &document,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );

        assert!(formation.products.is_empty());
        assert!(!formation.report.is_accepted());
    }

    #[test]
    fn changed_scope_changes_preview_product_identity() {
        let first = valid_document();
        let mut second = valid_document();
        let world_id = second.scope.world_id;
        second.scope.chunk_ids = vec![ChunkId::new(world_id, ChunkCoord3 { x: 1, y: 0, z: 0 })];
        second.refresh_cache_lineage();

        let first_formation = form_procgen_field_preview_products(
            &first,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );
        let second_formation = form_procgen_field_preview_products(
            &second,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );

        assert_ne!(
            first_formation.products[0].descriptor.product_id,
            second_formation.products[0].descriptor.product_id
        );
    }

    #[test]
    fn exact_bounds_fixture_still_matches_policy() {
        assert_eq!(
            grid_dimensions(bounds(), ProcgenFieldPreviewPolicy::default()),
            Some([16, 16, 16])
        );
    }
}
