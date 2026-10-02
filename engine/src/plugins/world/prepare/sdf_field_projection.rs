use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use product::{ProductIdentity, ProductResidency, RenderProductSelection, RenderSelectedProduct};
use world_sdf::{
    FieldProductDescriptor, FieldProductKind, SDF_METRIC_BRICK_EDGE_SAMPLES, SDF_PAGE_EDGE_BRICKS,
    SdfChunkPayload, WorldSdfMetricError, WorldSdfMetricPayloadRef, WorldSdfMetricProductCandidate,
    WorldSdfPayloadRef, ratify_world_sdf_metric_product_candidate,
    validate_world_sdf_metric_payload,
};

use super::super::adapters::resources::{PartitionConfigResource, SdfChunkStoreResource};
use super::super::build::integration::{
    WorldRuntimeSdfMetricCapabilityCatalogResource, WorldRuntimeSdfProductCatalogResource,
};
use crate::plugins::render::features::world::{
    RenderSdfChunkResidencyEntry, RenderSdfResidencyResource, RenderSdfResidencyStatus,
};
use crate::plugins::render::frame::PreparedRenderProductSelectionResource;
use crate::runtime::WorldMut;
use runen_render::field_input::{RenderFieldSemanticInput, RenderFieldSemanticInputError};
use runen_render::space_time::RenderTemporalSupport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedWorldSdfFieldSource {
    product_id: ProductIdentity,
    product_generation: u64,
    payload_ref: WorldSdfPayloadRef,
    input: RenderFieldSemanticInput,
}

impl PreparedWorldSdfFieldSource {
    pub fn new(
        product_id: ProductIdentity,
        product_generation: u64,
        payload_ref: WorldSdfPayloadRef,
        input: RenderFieldSemanticInput,
    ) -> Self {
        Self {
            product_id,
            product_generation,
            payload_ref,
            input,
        }
    }

    pub const fn product_id(&self) -> ProductIdentity {
        self.product_id
    }

    pub const fn product_generation(&self) -> u64 {
        self.product_generation
    }

    pub const fn payload_ref(&self) -> WorldSdfPayloadRef {
        self.payload_ref
    }

    pub const fn input(&self) -> &RenderFieldSemanticInput {
        &self.input
    }
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct PreparedWorldSdfFieldSourceResource {
    sources: BTreeMap<ProductIdentity, PreparedWorldSdfFieldSource>,
}

impl PreparedWorldSdfFieldSourceResource {
    pub fn sources(&self) -> &BTreeMap<ProductIdentity, PreparedWorldSdfFieldSource> {
        &self.sources
    }

    pub fn source(&self, product_id: ProductIdentity) -> Option<&PreparedWorldSdfFieldSource> {
        self.sources.get(&product_id)
    }

    pub fn insert_source(&mut self, source: PreparedWorldSdfFieldSource) {
        self.sources.insert(source.product_id, source);
    }

    pub fn remove_source(
        &mut self,
        product_id: ProductIdentity,
    ) -> Option<PreparedWorldSdfFieldSource> {
        self.sources.remove(&product_id)
    }
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct WorldSdfFieldProjectionStateResource {
    published_product_ids: BTreeSet<ProductIdentity>,
}

impl WorldSdfFieldProjectionStateResource {
    pub fn published_product_ids(&self) -> &BTreeSet<ProductIdentity> {
        &self.published_product_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldSdfFieldProjectionError {
    Metric(WorldSdfMetricError),
    RenderField(RenderFieldSemanticInputError),
    MissingCanonicalSample { global_sample: [usize; 3] },
}

impl fmt::Display for WorldSdfFieldProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metric(error) => write!(formatter, "invalid metric World SDF source: {error}"),
            Self::RenderField(error) => {
                write!(formatter, "invalid projected renderer field input: {error}")
            }
            Self::MissingCanonicalSample { global_sample } => write!(
                formatter,
                "metric World SDF projection is missing canonical global sample {global_sample:?}"
            ),
        }
    }
}

impl Error for WorldSdfFieldProjectionError {}

impl From<WorldSdfMetricError> for WorldSdfFieldProjectionError {
    fn from(value: WorldSdfMetricError) -> Self {
        Self::Metric(value)
    }
}

impl From<RenderFieldSemanticInputError> for WorldSdfFieldProjectionError {
    fn from(value: RenderFieldSemanticInputError) -> Self {
        Self::RenderField(value)
    }
}

pub fn prepare_world_sdf_field_projection_system(mut world: WorldMut) {
    let catalog = match world.resource::<WorldRuntimeSdfProductCatalogResource>() {
        Ok(value) => value.clone(),
        Err(_) => {
            clear_published_world_sdf_field_sources(&mut world);
            return;
        }
    };
    let metric_capabilities =
        match world.resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>() {
            Ok(value) => value.clone(),
            Err(_) => {
                clear_published_world_sdf_field_sources(&mut world);
                return;
            }
        };
    let store = match world.resource::<SdfChunkStoreResource>() {
        Ok(value) => value.clone(),
        Err(_) => {
            clear_published_world_sdf_field_sources(&mut world);
            return;
        }
    };
    let partition = match world.resource::<PartitionConfigResource>() {
        Ok(value) => value.clone(),
        Err(_) => {
            clear_published_world_sdf_field_sources(&mut world);
            return;
        }
    };
    let selections = match world.resource::<PreparedRenderProductSelectionResource>() {
        Ok(value) => value.snapshot(),
        Err(_) => {
            clear_published_world_sdf_field_sources(&mut world);
            return;
        }
    };
    let residency = match world.resource::<RenderSdfResidencyResource>() {
        Ok(value) => value.clone(),
        Err(_) => {
            clear_published_world_sdf_field_sources(&mut world);
            return;
        }
    };

    let previously_published = world
        .resource::<WorldSdfFieldProjectionStateResource>()
        .map(|state| state.published_product_ids.clone())
        .unwrap_or_default();

    let mut prepared = BTreeMap::<ProductIdentity, PreparedWorldSdfFieldSource>::new();
    for (product_id, descriptor) in catalog.products() {
        let Some(source) = prepare_source(
            *product_id,
            descriptor,
            &metric_capabilities,
            &store,
            &partition,
            &selections,
            &residency,
        ) else {
            continue;
        };
        prepared.insert(*product_id, source);
    }

    let currently_published = prepared.keys().copied().collect::<BTreeSet<_>>();
    if let Ok(resource) = world.resource_mut::<PreparedWorldSdfFieldSourceResource>() {
        for product_id in previously_published.difference(&currently_published) {
            resource.remove_source(*product_id);
        }
        for source in prepared.into_values() {
            resource.insert_source(source);
        }
    } else {
        return;
    }

    if let Ok(state) = world.resource_mut::<WorldSdfFieldProjectionStateResource>() {
        state.published_product_ids = currently_published;
    }
}

fn prepare_source(
    product_id: ProductIdentity,
    descriptor: &FieldProductDescriptor,
    metric_capabilities: &WorldRuntimeSdfMetricCapabilityCatalogResource,
    store: &SdfChunkStoreResource,
    partition: &PartitionConfigResource,
    selections: &[RenderProductSelection],
    residency: &RenderSdfResidencyResource,
) -> Option<PreparedWorldSdfFieldSource> {
    if descriptor.kind != FieldProductKind::WorldSdfChunkPages || descriptor.payload_refs.len() != 1
    {
        return None;
    }
    let payload_ref = descriptor.payload_refs[0];
    let metric_payload_ref = *metric_capabilities.capability(product_id)?;
    if metric_payload_ref.payload_ref != payload_ref {
        return None;
    }
    let payload = store.chunks.get(&payload_ref.chunk_id)?;
    if WorldSdfPayloadRef::from(payload) != payload_ref {
        return None;
    }

    let ratification = ratify_world_sdf_metric_product_candidate(
        &WorldSdfMetricProductCandidate::new(descriptor.clone(), metric_payload_ref),
    );
    if ratification.has_blocking_issues() {
        return None;
    }

    let generation = descriptor.product_core().lineage.generation;
    let residency_entry = residency.entry(product_id)?;
    if !residency_matches_source(
        product_id,
        generation,
        payload_ref,
        payload,
        residency_entry,
        selections,
    ) {
        return None;
    }

    let input = project_metric_payload(&metric_payload_ref, payload, partition).ok()?;
    Some(PreparedWorldSdfFieldSource::new(
        product_id,
        generation,
        payload_ref,
        input,
    ))
}

fn residency_matches_source(
    product_id: ProductIdentity,
    generation: u64,
    payload_ref: WorldSdfPayloadRef,
    payload: &SdfChunkPayload,
    residency: &RenderSdfChunkResidencyEntry,
    selections: &[RenderProductSelection],
) -> bool {
    if residency.product_id != product_id
        || residency.product_generation != generation
        || residency.chunk_id != payload_ref.chunk_id
        || residency.chunk_revision != payload_ref.chunk_revision.0
        || residency.chunk_generation != payload.chunk_generation.0
        || residency.checksum != payload_ref.checksum
        || residency.source_residency != ProductResidency::Resident
        || residency.requested_residency != ProductResidency::Resident
        || !matches!(
            residency.status,
            RenderSdfResidencyStatus::Resident | RenderSdfResidencyStatus::Preserved
        )
        || !residency.query_policy.allows(
            residency.freshness,
            residency.source_residency,
            residency.authority_class,
        )
    {
        return false;
    }

    selections
        .iter()
        .flat_map(|selection| selection.selected_products.iter())
        .any(|selected| selected_matches_residency(selected, residency))
}

fn selected_matches_residency(
    selected: &RenderSelectedProduct,
    residency: &RenderSdfChunkResidencyEntry,
) -> bool {
    selected.product_id == residency.product_id
        && selected.generation == residency.product_generation
        && selected.scale_band == residency.scale_band
        && selected.freshness == residency.freshness
        && selected.residency == residency.source_residency
        && selected.authority_class == residency.authority_class
        && selected.query_policy == residency.query_policy
        && selected.query_policy.allows(
            selected.freshness,
            selected.residency,
            selected.authority_class,
        )
}

fn project_metric_payload(
    metric_payload_ref: &WorldSdfMetricPayloadRef,
    payload: &SdfChunkPayload,
    partition: &PartitionConfigResource,
) -> Result<RenderFieldSemanticInput, WorldSdfFieldProjectionError> {
    validate_world_sdf_metric_payload(metric_payload_ref, payload)?;

    let page = payload
        .page_table
        .values()
        .next()
        .expect("validated metric payload has exactly one page");
    let brick_sample_stride = SDF_METRIC_BRICK_EDGE_SAMPLES - 1;
    let global_edge = SDF_PAGE_EDGE_BRICKS * brick_sample_stride + 1;
    let global_sample_count = global_edge * global_edge * global_edge;
    let mut ranges = vec![None::<(i16, i16)>; global_sample_count];

    for brick_z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
        for brick_y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for brick_x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                let brick_coord = [brick_x, brick_y, brick_z];
                let brick = page
                    .bricks
                    .get(&brick_coord)
                    .expect("validated metric payload has every canonical brick");
                for sample_z in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                    for sample_y in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                        for sample_x in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                            let global = [
                                usize::from(brick_x) * brick_sample_stride + sample_x,
                                usize::from(brick_y) * brick_sample_stride + sample_y,
                                usize::from(brick_z) * brick_sample_stride + sample_z,
                            ];
                            let global_index =
                                dense_index(global_edge, global[0], global[1], global[2]);
                            let value = brick.samples.distances
                                [cube_sample_index(sample_x, sample_y, sample_z)];
                            match &mut ranges[global_index] {
                                Some((minimum, maximum)) => {
                                    *minimum = (*minimum).min(value);
                                    *maximum = (*maximum).max(value);
                                }
                                slot @ None => *slot = Some((value, value)),
                            }
                        }
                    }
                }
            }
        }
    }

    let encoding = metric_payload_ref.encoding;
    let mut max_duplicate_spread_units = 0_u32;
    let mut samples = Vec::with_capacity(global_sample_count);
    for z in 0..global_edge {
        for y in 0..global_edge {
            for x in 0..global_edge {
                let (minimum, maximum) = ranges[dense_index(global_edge, x, y, z)].ok_or(
                    WorldSdfFieldProjectionError::MissingCanonicalSample {
                        global_sample: [x, y, z],
                    },
                )?;
                let spread = i32::from(minimum).abs_diff(i32::from(maximum));
                max_duplicate_spread_units = max_duplicate_spread_units.max(spread);
                let midrange_encoded = (f64::from(minimum) + f64::from(maximum)) * 0.5;
                samples.push(encoding.decode_distance_meters(midrange_encoded));
            }
        }
    }

    let source_error_meters = encoding.max_absolute_error_meters();
    let canonicalization_error_meters =
        f64::from(max_duplicate_spread_units) * 0.5 / f64::from(encoding.distance_units_per_meter);
    let max_absolute_query_error_local_meters = source_error_meters + canonicalization_error_meters;
    let spacing = partition.chunk_edge_meters() / (global_edge - 1) as f64;
    let dimension =
        u32::try_from(global_edge).expect("revision-1 metric lattice dimension fits u32");
    let input = RenderFieldSemanticInput::dense(
        [0.0; 3],
        [spacing; 3],
        [dimension; 3],
        samples,
        max_absolute_query_error_local_meters,
        RenderTemporalSupport::unbounded(),
    )?;

    Ok(input)
}

fn clear_published_world_sdf_field_sources(world: &mut WorldMut) {
    let published = world
        .resource::<WorldSdfFieldProjectionStateResource>()
        .map(|state| state.published_product_ids.clone())
        .unwrap_or_default();
    if let Ok(resource) = world.resource_mut::<PreparedWorldSdfFieldSourceResource>() {
        for product_id in &published {
            resource.remove_source(*product_id);
        }
    }
    if let Ok(state) = world.resource_mut::<WorldSdfFieldProjectionStateResource>() {
        state.published_product_ids.clear();
    }
}

fn dense_index(edge: usize, x: usize, y: usize, z: usize) -> usize {
    z * edge * edge + y * edge + x
}

fn cube_sample_index(x: usize, y: usize, z: usize) -> usize {
    z * SDF_METRIC_BRICK_EDGE_SAMPLES * SDF_METRIC_BRICK_EDGE_SAMPLES
        + y * SDF_METRIC_BRICK_EDGE_SAMPLES
        + x
}

#[cfg(test)]
mod tests {
    use super::*;
    use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
    use world_ops::{ChunkGeneration, ChunkRevision};
    use world_sdf::{
        SDF_METRIC_BRICK_SAMPLE_COUNT, SdfBrickMetadata, SdfBrickRecord, SdfBrickSamples,
        SdfPageCoord3, SdfPageRecord, WorldSdfMetricEncoding, sample_world_sdf_metric_distance,
    };

    fn partition() -> PartitionConfigResource {
        PartitionConfigResource(
            GridPartitionConfig::try_new(4.0, [8, 8, 8])
                .expect("projection test partition should be valid"),
        )
    }

    fn boundary_spread_payload() -> (WorldSdfMetricPayloadRef, SdfChunkPayload) {
        let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
        let mut page = SdfPageRecord {
            page_generation: 0,
            bricks: BTreeMap::new(),
        };
        for z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                for x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                    page.bricks.insert(
                        [x, y, z],
                        SdfBrickRecord {
                            metadata: SdfBrickMetadata::default(),
                            samples: SdfBrickSamples {
                                distances: vec![0; SDF_METRIC_BRICK_SAMPLE_COUNT],
                            },
                        },
                    );
                }
            }
        }

        for brick_z in 0..=1_u8 {
            for brick_y in 0..=1_u8 {
                for brick_x in 0..=1_u8 {
                    let local = [
                        if brick_x == 0 { 2 } else { 0 },
                        if brick_y == 0 { 2 } else { 0 },
                        if brick_z == 0 { 2 } else { 0 },
                    ];
                    let value = match [brick_x, brick_y, brick_z] {
                        [0, 0, 0] => -4,
                        [1, 1, 1] => 4,
                        _ => 0,
                    };
                    page.bricks
                        .get_mut(&[brick_x, brick_y, brick_z])
                        .unwrap()
                        .samples
                        .distances[cube_sample_index(local[0], local[1], local[2])] = value;
                }
            }
        }

        let payload = SdfChunkPayload {
            chunk_id,
            chunk_revision: ChunkRevision(3),
            chunk_generation: ChunkGeneration(5),
            page_table: [(SdfPageCoord3::default(), page)].into_iter().collect(),
            hierarchy_revision: 0,
            checksum: 77,
        };
        let encoding =
            WorldSdfMetricEncoding::try_new(100, 4).expect("test metric encoding should be valid");
        let metric_ref =
            WorldSdfMetricPayloadRef::try_new(WorldSdfPayloadRef::from(&payload), encoding)
                .expect("test metric ref should be valid");
        (metric_ref, payload)
    }

    fn sample_projected(input: &RenderFieldSemanticInput, point: [f64; 3]) -> f64 {
        let origin = input.origin_local_meters();
        let spacing = input.sample_spacing_meters();
        let dimensions = input.dimensions();
        let axis = |axis: usize| {
            let coordinate = ((point[axis] - origin[axis]) / spacing[axis])
                .clamp(0.0, f64::from(dimensions[axis] - 1));
            let lower = (coordinate.floor() as u32).min(dimensions[axis] - 2);
            let upper = lower + 1;
            (
                lower as usize,
                upper as usize,
                coordinate - f64::from(lower),
            )
        };
        let (x0, x1, tx) = axis(0);
        let (y0, y1, ty) = axis(1);
        let (z0, z1, tz) = axis(2);
        let width = dimensions[0] as usize;
        let height = dimensions[1] as usize;
        let sample = |x: usize, y: usize, z: usize| {
            input
                .signed_distance_sample_meters(z * width * height + y * width + x)
                .unwrap()
        };
        let lerp = |left: f64, right: f64, t: f64| left + (right - left) * t;
        let c00 = lerp(sample(x0, y0, z0), sample(x1, y0, z0), tx);
        let c10 = lerp(sample(x0, y1, z0), sample(x1, y1, z0), tx);
        let c01 = lerp(sample(x0, y0, z1), sample(x1, y0, z1), tx);
        let c11 = lerp(sample(x0, y1, z1), sample(x1, y1, z1), tx);
        let c0 = lerp(c00, c10, ty);
        let c1 = lerp(c01, c11, ty);
        lerp(c0, c1, tz)
    }

    #[test]
    fn duplicate_midrange_projection_charges_exact_half_spread_error() {
        let (metric_ref, payload) = boundary_spread_payload();
        let projected =
            project_metric_payload(&metric_ref, &payload, &partition()).expect("valid projection");

        assert_eq!(projected.dimensions(), [9, 9, 9]);
        assert_eq!(projected.sample_spacing_meters(), [0.5; 3]);
        assert_eq!(projected.max_absolute_query_error_local_meters(), 0.08);
        let shared_index = dense_index(9, 2, 2, 2);
        assert_eq!(
            projected.signed_distance_sample_meters(shared_index),
            Some(0.0)
        );
    }

    #[test]
    fn regular_lattice_stays_within_half_spread_of_piecewise_source_query() {
        let (metric_ref, payload) = boundary_spread_payload();
        let partition = partition();
        let projected =
            project_metric_payload(&metric_ref, &payload, &partition).expect("valid projection");
        let canonicalization_error_meters = 0.04;

        for coordinate in [0.75, 0.9, 0.99, 1.0, 1.01, 1.1, 1.25] {
            let point = [coordinate; 3];
            let source = sample_world_sdf_metric_distance(&metric_ref, &payload, &partition, point)
                .expect("source query should be valid")
                .signed_distance_estimate_meters();
            let rendered = sample_projected(&projected, point);
            assert!(
                (rendered - source).abs() <= canonicalization_error_meters + 1.0e-12,
                "{point:?}: projected={rendered}, source={source}"
            );
        }
    }
}
