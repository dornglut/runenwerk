use crate::{
    FieldProductConsumerClass, FieldProductDescriptor, FieldProductFreshness, FieldProductKind,
    RegionSdfSummary, SDF_METRIC_BRICK_EDGE_SAMPLES, SDF_METRIC_BRICK_SAMPLE_COUNT,
    SDF_PAGE_EDGE_BRICKS, SdfBrickMetadata, SdfChunkPayload, SdfPageCoord3,
    WORLD_SDF_OPEN_TOP_INTERIOR_SOURCE_SCHEMA_REVISION, WorldSdfMetricEncoding,
    WorldSdfMetricPayloadRef, WorldSdfOpenTopRectInteriorSource, WorldSdfPayloadRef,
    validate_world_sdf_metric_payload,
};
use runen_spatial::GridPartitionConfig;
// The exact formation contract exposes its existing typed World revisions.
pub use world_ops::{ChunkGeneration, ChunkRevision};

/// First version of the Arena one-page source formation; not a general cave certificate.
pub const WORLD_SDF_OPEN_TOP_EXACT_FORMATION_REVISION: u16 = 1;
const UNITS_PER_METER: u32 = 1024;
const MAX_ERROR_UNITS: u32 = 444;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldSdfExactFormationError {
    UnsupportedSourceOrEncoding,
    InvalidProductOrRevision,
    WrongPayloadIdentity,
    InvalidPartition,
    InvalidMetricLayout,
    InvalidPageOrHierarchy,
    InvalidSampleOrMetadata { brick: [u8; 3] },
    InvalidRegionSummary,
    UnrepresentableSample,
}
impl std::fmt::Display for WorldSdfExactFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorldSDF exact source formation rejected: {self:?}")
    }
}
impl std::error::Error for WorldSdfExactFormationError {}

/// Only the independent full-content verifier can construct this evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldSdfVerifiedExactSource {
    source: WorldSdfOpenTopRectInteriorSource,
    formation_revision: u16,
    partition: GridPartitionConfig,
    metric_ref: WorldSdfMetricPayloadRef,
    source_revision: u64,
    chunk_generation: ChunkGeneration,
    region_summary: RegionSdfSummary,
}
impl WorldSdfVerifiedExactSource {
    pub fn source(&self) -> WorldSdfOpenTopRectInteriorSource {
        self.source
    }
    pub fn formation_revision(&self) -> u16 {
        self.formation_revision
    }
    pub fn partition(&self) -> &GridPartitionConfig {
        &self.partition
    }
    pub fn metric_ref(&self) -> WorldSdfMetricPayloadRef {
        self.metric_ref
    }
    pub fn source_revision(&self) -> u64 {
        self.source_revision
    }
    pub fn chunk_generation(&self) -> ChunkGeneration {
        self.chunk_generation
    }
    pub fn region_summary(&self) -> RegionSdfSummary {
        self.region_summary
    }

    /// Public World store/partition mutation makes acquired-read revalidation mandatory.
    pub fn revalidate(
        &self,
        descriptor: &FieldProductDescriptor,
        payload: &SdfChunkPayload,
        summary: RegionSdfSummary,
        partition: &GridPartitionConfig,
    ) -> Result<(), WorldSdfExactFormationError> {
        if partition != &self.partition {
            return Err(WorldSdfExactFormationError::InvalidPartition);
        }
        let current = verify_world_sdf_open_top_exact_formation(
            self.source,
            partition,
            descriptor,
            self.metric_ref,
            payload,
            summary,
        )?;
        if current != *self {
            return Err(WorldSdfExactFormationError::WrongPayloadIdentity);
        }
        Ok(())
    }
}

/// Independent verifier: all 1,728 sampled distances, all 512 occupancy octants,
/// all metadata and the region summary. Never calls the Arena producer.
pub fn verify_world_sdf_open_top_exact_formation(
    source: WorldSdfOpenTopRectInteriorSource,
    partition: &GridPartitionConfig,
    descriptor: &FieldProductDescriptor,
    metric_ref: WorldSdfMetricPayloadRef,
    payload: &SdfChunkPayload,
    region_summary: RegionSdfSummary,
) -> Result<WorldSdfVerifiedExactSource, WorldSdfExactFormationError> {
    use WorldSdfExactFormationError as E;
    if source.schema_revision() != WORLD_SDF_OPEN_TOP_INTERIOR_SOURCE_SCHEMA_REVISION
        || metric_ref.encoding
            != WorldSdfMetricEncoding::try_new(UNITS_PER_METER, MAX_ERROR_UNITS)
                .expect("accepted exact formation encoding is valid")
    {
        return Err(E::UnsupportedSourceOrEncoding);
    }
    if descriptor.kind != FieldProductKind::WorldSdfChunkPages
        || descriptor.freshness != FieldProductFreshness::Current
        || !matches!(
            descriptor.consumer_class,
            FieldProductConsumerClass::RuntimeRead | FieldProductConsumerClass::CollisionQuery
        )
        || descriptor.lineage.source_revision == 0
        || descriptor.payload_refs.len() != 1
        || descriptor.scope.chunk_ids.len() != 1
        || !descriptor.scope.region_ids.is_empty()
        || !descriptor.scope.chunk_ids.contains(&payload.chunk_id)
    {
        return Err(E::InvalidProductOrRevision);
    }
    if descriptor.payload_refs[0] != WorldSdfPayloadRef::from(payload)
        || metric_ref.payload_ref != descriptor.payload_refs[0]
    {
        return Err(E::WrongPayloadIdentity);
    }
    validate_world_sdf_metric_payload(&metric_ref, payload).map_err(|_| E::InvalidMetricLayout)?;
    let origin = partition
        .chunk_origin_world_position(payload.chunk_id.world_id, payload.chunk_id.coord)
        .map_err(|_| E::InvalidPartition)?
        .meters();
    let edge = partition.chunk_edge_meters();
    if !edge.is_finite() || edge <= 0.0 || origin.iter().any(|v| !v.is_finite()) {
        return Err(E::InvalidPartition);
    }
    let page = payload
        .page_table
        .get(&SdfPageCoord3::default())
        .ok_or(E::InvalidMetricLayout)?;
    if page.page_generation != 0 || payload.hierarchy_revision != 0 {
        return Err(E::InvalidPageOrHierarchy);
    }
    let brick_edge = edge / SDF_PAGE_EDGE_BRICKS as f64;
    let mut min = i16::MAX;
    let mut max = i16::MIN;
    let mut occupied = false;
    let mut surface = false;
    for z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
        for y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                let coord = [x, y, z];
                let brick = page.bricks.get(&coord).ok_or(E::InvalidMetricLayout)?;
                if brick.samples.distances.len() != SDF_METRIC_BRICK_SAMPLE_COUNT {
                    return Err(E::InvalidMetricLayout);
                }
                let mut bmin = i16::MAX;
                let mut bmax = i16::MIN;
                for sz in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                    for sy in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                        for sx in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                            let pos = [
                                origin[0] + (f64::from(x) + sx as f64 / 2.0) * brick_edge,
                                origin[1] + (f64::from(y) + sy as f64 / 2.0) * brick_edge,
                                origin[2] + (f64::from(z) + sz as f64 / 2.0) * brick_edge,
                            ];
                            let v = (source.signed_distance_meters(pos)
                                * f64::from(UNITS_PER_METER))
                            .round();
                            if !v.is_finite() || v < f64::from(i16::MIN) || v > f64::from(i16::MAX)
                            {
                                return Err(E::UnrepresentableSample);
                            }
                            let expected = v as i16;
                            let idx =
                                sz * SDF_METRIC_BRICK_EDGE_SAMPLES * SDF_METRIC_BRICK_EDGE_SAMPLES
                                    + sy * SDF_METRIC_BRICK_EDGE_SAMPLES
                                    + sx;
                            if brick.samples.distances[idx] != expected {
                                return Err(E::InvalidSampleOrMetadata { brick: coord });
                            }
                            bmin = bmin.min(expected);
                            bmax = bmax.max(expected);
                        }
                    }
                }
                let mut mask = 0_u8;
                for oz in 0..2_u8 {
                    for oy in 0..2_u8 {
                        for ox in 0..2_u8 {
                            let pos = [
                                origin[0]
                                    + (f64::from(x) + (f64::from(ox) + 0.5) / 2.0) * brick_edge,
                                origin[1]
                                    + (f64::from(y) + (f64::from(oy) + 0.5) / 2.0) * brick_edge,
                                origin[2]
                                    + (f64::from(z) + (f64::from(oz) + 0.5) / 2.0) * brick_edge,
                            ];
                            if source.signed_distance_meters(pos) < 0.0 {
                                mask |= 1_u8 << (ox | (oy << 1) | (oz << 2));
                            }
                        }
                    }
                }
                let band = bmin <= 0 && bmax >= 0;
                if brick.metadata
                    != (SdfBrickMetadata {
                        min_distance: bmin,
                        max_distance: bmax,
                        occupancy_mask: mask,
                        material_channel_mask: u16::from(mask != 0),
                        surface_band_present: band,
                        ..SdfBrickMetadata::default()
                    })
                {
                    return Err(E::InvalidSampleOrMetadata { brick: coord });
                }
                min = min.min(bmin);
                max = max.max(bmax);
                occupied |= mask != 0;
                surface |= band;
            }
        }
    }
    if region_summary
        != (RegionSdfSummary {
            min_distance: min,
            max_distance: max,
            occupied_chunk_count: u32::from(occupied),
            surface_chunk_count: u32::from(surface),
        })
    {
        return Err(E::InvalidRegionSummary);
    }
    Ok(WorldSdfVerifiedExactSource {
        source,
        formation_revision: WORLD_SDF_OPEN_TOP_EXACT_FORMATION_REVISION,
        partition: partition.clone(),
        metric_ref,
        source_revision: descriptor.lineage.source_revision,
        chunk_generation: payload.chunk_generation,
        region_summary,
    })
}

/// Producer-owned pairing of a full-precision immutable source with the
/// *actual* bytes formed from that value. A caller cannot attach a second
/// source to pre-existing bytes, even when the quantized bytes are identical.
/// This is a narrow open-top formation contract, not a general SDF registry.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldSdfOpenTopExactFormation {
    source: WorldSdfOpenTopRectInteriorSource,
    partition: GridPartitionConfig,
    payload: SdfChunkPayload,
    descriptor: FieldProductDescriptor,
    region_summary: RegionSdfSummary,
    metric_ref: WorldSdfMetricPayloadRef,
}

impl WorldSdfOpenTopExactFormation {
    pub fn source(&self) -> WorldSdfOpenTopRectInteriorSource {
        self.source
    }
    pub fn partition(&self) -> &GridPartitionConfig {
        &self.partition
    }
    pub fn payload(&self) -> &SdfChunkPayload {
        &self.payload
    }
    pub fn descriptor(&self) -> &FieldProductDescriptor {
        &self.descriptor
    }
    pub fn region_summary(&self) -> RegionSdfSummary {
        self.region_summary
    }
    pub fn metric_ref(&self) -> WorldSdfMetricPayloadRef {
        self.metric_ref
    }
    pub fn into_payload(self) -> SdfChunkPayload {
        self.payload
    }
}

/// The only way to mint an exact open-top source formation. It constructs
/// bytes and provenance together, rather than certifying an independently
/// supplied source after quantization has erased distinctions.
#[allow(clippy::too_many_arguments)]
pub fn form_world_sdf_open_top_exact_source(
    source: WorldSdfOpenTopRectInteriorSource,
    partition: GridPartitionConfig,
    chunk_id: runen_spatial::ChunkId,
    chunk_revision: ChunkRevision,
    chunk_generation: ChunkGeneration,
    checksum: u64,
    product_id: crate::FieldProductId,
    lineage: crate::FieldProductLineage,
) -> Result<WorldSdfOpenTopExactFormation, WorldSdfExactFormationError> {
    use crate::{SdfBrickRecord, SdfBrickSamples, SdfPageRecord};
    use WorldSdfExactFormationError as E;
    let origin = partition
        .chunk_origin_world_position(chunk_id.world_id, chunk_id.coord)
        .map_err(|_| E::InvalidPartition)?
        .meters();
    let edge = partition.chunk_edge_meters();
    if !edge.is_finite() || edge <= 0.0 || origin.iter().any(|v| !v.is_finite()) {
        return Err(E::InvalidPartition);
    }
    let brick_edge = edge / SDF_PAGE_EDGE_BRICKS as f64;
    let mut page = SdfPageRecord {
        page_generation: 0,
        bricks: Default::default(),
    };
    let mut min = i16::MAX;
    let mut max = i16::MIN;
    let mut occupied = false;
    let mut surface = false;
    for z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
        for y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                let mut distances = Vec::with_capacity(SDF_METRIC_BRICK_SAMPLE_COUNT);
                for sz in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                    for sy in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                        for sx in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                            let position = [
                                origin[0] + (f64::from(x) + sx as f64 / 2.0) * brick_edge,
                                origin[1] + (f64::from(y) + sy as f64 / 2.0) * brick_edge,
                                origin[2] + (f64::from(z) + sz as f64 / 2.0) * brick_edge,
                            ];
                            let encoded = (source.signed_distance_meters(position)
                                * f64::from(UNITS_PER_METER))
                            .round();
                            if !encoded.is_finite()
                                || encoded < f64::from(i16::MIN)
                                || encoded > f64::from(i16::MAX)
                            {
                                return Err(E::UnrepresentableSample);
                            }
                            distances.push(encoded as i16);
                        }
                    }
                }
                let bmin = *distances.iter().min().ok_or(E::InvalidMetricLayout)?;
                let bmax = *distances.iter().max().ok_or(E::InvalidMetricLayout)?;
                let mut mask = 0_u8;
                for oz in 0..2_u8 {
                    for oy in 0..2_u8 {
                        for ox in 0..2_u8 {
                            let position = [
                                origin[0]
                                    + (f64::from(x) + (f64::from(ox) + 0.5) / 2.0) * brick_edge,
                                origin[1]
                                    + (f64::from(y) + (f64::from(oy) + 0.5) / 2.0) * brick_edge,
                                origin[2]
                                    + (f64::from(z) + (f64::from(oz) + 0.5) / 2.0) * brick_edge,
                            ];
                            if source.signed_distance_meters(position) < 0.0 {
                                mask |= 1 << (ox | (oy << 1) | (oz << 2));
                            }
                        }
                    }
                }
                let band = bmin <= 0 && bmax >= 0;
                page.bricks.insert(
                    [x, y, z],
                    SdfBrickRecord {
                        metadata: SdfBrickMetadata {
                            min_distance: bmin,
                            max_distance: bmax,
                            occupancy_mask: mask,
                            material_channel_mask: u16::from(mask != 0),
                            surface_band_present: band,
                            ..SdfBrickMetadata::default()
                        },
                        samples: SdfBrickSamples { distances },
                    },
                );
                min = min.min(bmin);
                max = max.max(bmax);
                occupied |= mask != 0;
                surface |= band;
            }
        }
    }
    let payload = SdfChunkPayload {
        chunk_id,
        chunk_revision,
        chunk_generation,
        page_table: std::collections::BTreeMap::from([(SdfPageCoord3::default(), page)]),
        hierarchy_revision: 0,
        checksum,
    };
    let mut descriptor = FieldProductDescriptor::new(
        product_id,
        FieldProductKind::WorldSdfChunkPages,
        crate::FieldProductScope::from_chunks([chunk_id]),
        lineage,
    );
    descriptor.consumer_class = FieldProductConsumerClass::RuntimeRead;
    descriptor
        .payload_refs
        .push(WorldSdfPayloadRef::from(&payload));
    let metric_ref = WorldSdfMetricPayloadRef::try_new(
        WorldSdfPayloadRef::from(&payload),
        WorldSdfMetricEncoding::try_new(UNITS_PER_METER, MAX_ERROR_UNITS)
            .expect("canonical encoding valid"),
    )
    .ok_or(E::InvalidMetricLayout)?;
    Ok(WorldSdfOpenTopExactFormation {
        source,
        partition,
        payload,
        descriptor,
        region_summary: RegionSdfSummary {
            min_distance: min,
            max_distance: max,
            occupied_chunk_count: u32::from(occupied),
            surface_chunk_count: u32::from(surface),
        },
        metric_ref,
    })
}
