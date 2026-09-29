use crate::{
    SDF_BRICK_EDGE_SAMPLES, SDF_BRICK_SAMPLE_COUNT, SDF_PAGE_EDGE_BRICKS, SdfBrickRecord,
    SdfChunkPayload, SdfPageCoord3, WorldSdfPayloadRef,
};
use runen_spatial::{GridPartitionConfig, WorldPosition};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldSdfMetricSample {
    signed_distance_estimate_meters: f64,
    max_absolute_error_meters: f64,
}

impl WorldSdfMetricSample {
    pub const fn signed_distance_estimate_meters(self) -> f64 {
        self.signed_distance_estimate_meters
    }

    pub const fn max_absolute_error_meters(self) -> f64 {
        self.max_absolute_error_meters
    }

    pub fn safe_distance_lower_bound_meters(self) -> f64 {
        (self.signed_distance_estimate_meters.abs() - self.max_absolute_error_meters).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldSdfMetricQueryError {
    MissingMetricEncoding,
    UnsupportedMetricEncoding,
    PayloadReferenceMismatch,
    InvalidWorldPosition,
    PositionOutsidePayload,
    MissingPageData,
    MissingPage,
    MissingBrick,
    InvalidMetricSampleCount { actual: usize },
}

impl fmt::Display for WorldSdfMetricQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMetricEncoding => {
                write!(f, "world SDF payload reference has no metric-field capability")
            }
            Self::UnsupportedMetricEncoding => {
                write!(f, "world SDF metric-field encoding is unsupported or invalid")
            }
            Self::PayloadReferenceMismatch => {
                write!(f, "world SDF payload does not match the exact referenced chunk revision/checksum")
            }
            Self::InvalidWorldPosition => write!(f, "metric field query position is invalid"),
            Self::PositionOutsidePayload => {
                write!(f, "metric field query position lies outside the referenced chunk")
            }
            Self::MissingPageData => write!(f, "metric field payload contains no SDF page data"),
            Self::MissingPage => write!(f, "metric field query resolved to a missing SDF page"),
            Self::MissingBrick => write!(f, "metric field query resolved to a missing SDF brick"),
            Self::InvalidMetricSampleCount { actual } => write!(
                f,
                "metric SDF brick must contain exactly {SDF_BRICK_SAMPLE_COUNT} canonical corner samples, found {actual}"
            ),
        }
    }
}

impl Error for WorldSdfMetricQueryError {}

pub fn sample_world_sdf_metric_distance(
    payload_ref: &WorldSdfPayloadRef,
    payload: &SdfChunkPayload,
    partition: &GridPartitionConfig,
    world_position_meters: [f64; 3],
) -> Result<WorldSdfMetricSample, WorldSdfMetricQueryError> {
    let encoding = payload_ref
        .metric_encoding
        .ok_or(WorldSdfMetricQueryError::MissingMetricEncoding)?;
    if !encoding.is_supported() {
        return Err(WorldSdfMetricQueryError::UnsupportedMetricEncoding);
    }
    if payload_ref.chunk_id != payload.chunk_id
        || payload_ref.chunk_revision != payload.chunk_revision
        || payload_ref.checksum != payload.checksum
    {
        return Err(WorldSdfMetricQueryError::PayloadReferenceMismatch);
    }

    let world_position = WorldPosition::try_new(payload_ref.chunk_id.world_id, world_position_meters)
        .map_err(|_| WorldSdfMetricQueryError::InvalidWorldPosition)?;
    let query_chunk = partition
        .chunk_id_from_world_position(world_position)
        .map_err(|_| WorldSdfMetricQueryError::InvalidWorldPosition)?;
    if query_chunk != payload_ref.chunk_id {
        return Err(WorldSdfMetricQueryError::PositionOutsidePayload);
    }

    let local = chunk_local_position(partition, payload_ref.chunk_id, world_position_meters)?;
    let (page_coord, brick_coord, local_in_brick) =
        payload_brick_lookup(partition, payload, local)?;
    let page = payload
        .page_table
        .get(&page_coord)
        .ok_or(WorldSdfMetricQueryError::MissingPage)?;
    let brick = page
        .bricks
        .get(&brick_coord)
        .ok_or(WorldSdfMetricQueryError::MissingBrick)?;
    let encoded_distance = sample_metric_brick(brick, local_in_brick)?;

    Ok(WorldSdfMetricSample {
        signed_distance_estimate_meters: encoding.decode_distance_meters(encoded_distance),
        max_absolute_error_meters: encoding.max_absolute_error_meters(),
    })
}

fn chunk_local_position(
    partition: &GridPartitionConfig,
    chunk_id: runen_spatial::ChunkId,
    world_position_meters: [f64; 3],
) -> Result<[f64; 3], WorldSdfMetricQueryError> {
    let origin = partition
        .chunk_origin_world_position(chunk_id.world_id, chunk_id.coord)
        .map_err(|_| WorldSdfMetricQueryError::InvalidWorldPosition)?;
    let origin = origin.meters();
    Ok([
        world_position_meters[0] - origin[0],
        world_position_meters[1] - origin[1],
        world_position_meters[2] - origin[2],
    ])
}

fn payload_brick_lookup(
    partition: &GridPartitionConfig,
    payload: &SdfChunkPayload,
    local: [f64; 3],
) -> Result<(SdfPageCoord3, [u8; 3], [f64; 3]), WorldSdfMetricQueryError> {
    let (min_page, max_page) =
        payload_page_bounds(payload).ok_or(WorldSdfMetricQueryError::MissingPageData)?;
    let edge = partition.chunk_edge_meters();
    let local_clamped = [
        local[0].clamp(0.0, edge * (1.0 - 1.0e-12)),
        local[1].clamp(0.0, edge * (1.0 - 1.0e-12)),
        local[2].clamp(0.0, edge * (1.0 - 1.0e-12)),
    ];
    let page_span = [
        i32::from(max_page.x - min_page.x + 1).max(1),
        i32::from(max_page.y - min_page.y + 1).max(1),
        i32::from(max_page.z - min_page.z + 1).max(1),
    ];
    let (page_offset_x, brick_x, local_x) =
        quantize_payload_axis(local_clamped[0], edge, page_span[0]);
    let (page_offset_y, brick_y, local_y) =
        quantize_payload_axis(local_clamped[1], edge, page_span[1]);
    let (page_offset_z, brick_z, local_z) =
        quantize_payload_axis(local_clamped[2], edge, page_span[2]);
    Ok((
        SdfPageCoord3 {
            x: min_page.x + page_offset_x as i16,
            y: min_page.y + page_offset_y as i16,
            z: min_page.z + page_offset_z as i16,
        },
        [brick_x, brick_y, brick_z],
        [local_x, local_y, local_z],
    ))
}

fn quantize_payload_axis(local_axis: f64, edge: f64, page_span: i32) -> (i32, u8, f64) {
    let span = page_span.max(1);
    let page_coord_f = (local_axis / edge) * f64::from(span);
    let page_offset = page_coord_f
        .floor()
        .clamp(0.0, f64::from(span - 1)) as i32;
    let page_local = page_coord_f - f64::from(page_offset);
    let brick_coord_f = page_local * SDF_PAGE_EDGE_BRICKS as f64;
    let brick_index = brick_coord_f
        .floor()
        .clamp(0.0, (SDF_PAGE_EDGE_BRICKS - 1) as f64) as u8;
    let brick_local =
        (brick_coord_f - f64::from(brick_index)).clamp(0.0, 1.0 - 1.0e-12);
    (page_offset, brick_index, brick_local)
}

fn payload_page_bounds(payload: &SdfChunkPayload) -> Option<(SdfPageCoord3, SdfPageCoord3)> {
    let mut pages = payload.page_table.keys().copied();
    let first = pages.next()?;
    let mut min = first;
    let mut max = first;
    for page in pages {
        min.x = min.x.min(page.x);
        min.y = min.y.min(page.y);
        min.z = min.z.min(page.z);
        max.x = max.x.max(page.x);
        max.y = max.y.max(page.y);
        max.z = max.z.max(page.z);
    }
    Some((min, max))
}

fn sample_metric_brick(
    brick: &SdfBrickRecord,
    local_in_brick: [f64; 3],
) -> Result<f64, WorldSdfMetricQueryError> {
    if brick.samples.distances.len() != SDF_BRICK_SAMPLE_COUNT {
        return Err(WorldSdfMetricQueryError::InvalidMetricSampleCount {
            actual: brick.samples.distances.len(),
        });
    }
    debug_assert_eq!(SDF_BRICK_EDGE_SAMPLES, 2);

    let sample_at = |x: usize, y: usize, z: usize| -> f64 {
        f64::from(brick.samples.distances[cube_sample_index(x, y, z)])
    };
    let tx = local_in_brick[0].clamp(0.0, 1.0);
    let ty = local_in_brick[1].clamp(0.0, 1.0);
    let tz = local_in_brick[2].clamp(0.0, 1.0);

    let c00 = lerp(sample_at(0, 0, 0), sample_at(1, 0, 0), tx);
    let c10 = lerp(sample_at(0, 1, 0), sample_at(1, 1, 0), tx);
    let c01 = lerp(sample_at(0, 0, 1), sample_at(1, 0, 1), tx);
    let c11 = lerp(sample_at(0, 1, 1), sample_at(1, 1, 1), tx);
    let c0 = lerp(c00, c10, ty);
    let c1 = lerp(c01, c11, ty);
    Ok(lerp(c0, c1, tz))
}

fn cube_sample_index(x: usize, y: usize, z: usize) -> usize {
    z * SDF_BRICK_EDGE_SAMPLES * SDF_BRICK_EDGE_SAMPLES
        + y * SDF_BRICK_EDGE_SAMPLES
        + x
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SdfBrickMetadata, SdfBrickSamples, SdfPageRecord, WorldSdfMetricEncoding,
        WORLD_SDF_METRIC_SAMPLE_LAYOUT_REVISION,
    };
    use runen_spatial::{ChunkCoord3, ChunkId, WorldId};
    use std::collections::BTreeMap;
    use world_ops::{ChunkGeneration, ChunkRevision};

    const DISTANCE_UNITS_PER_METER: u32 = 10_000;

    fn partition() -> GridPartitionConfig {
        GridPartitionConfig::try_new(1.0, [8, 8, 8]).expect("test partition is valid")
    }

    fn chunk_id() -> ChunkId {
        ChunkId::new(WorldId::new(0), ChunkCoord3::default())
    }

    fn plane_payload() -> SdfChunkPayload {
        let mut page = SdfPageRecord {
            page_generation: 0,
            bricks: BTreeMap::new(),
        };
        let edge = SDF_PAGE_EDGE_BRICKS as f64;
        for brick_z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for brick_y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                for brick_x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                    let mut distances = Vec::with_capacity(SDF_BRICK_SAMPLE_COUNT);
                    for sample_z in 0..SDF_BRICK_EDGE_SAMPLES {
                        for sample_y in 0..SDF_BRICK_EDGE_SAMPLES {
                            for sample_x in 0..SDF_BRICK_EDGE_SAMPLES {
                                let point_x =
                                    (f64::from(brick_x) + sample_x as f64) / edge;
                                let distance = point_x - 0.5;
                                distances.push(
                                    (distance * f64::from(DISTANCE_UNITS_PER_METER)).round()
                                        as i16,
                                );
                            }
                        }
                    }
                    page.bricks.insert(
                        [brick_x, brick_y, brick_z],
                        SdfBrickRecord {
                            metadata: SdfBrickMetadata::default(),
                            samples: SdfBrickSamples { distances },
                        },
                    );
                }
            }
        }

        SdfChunkPayload {
            chunk_id: chunk_id(),
            chunk_revision: ChunkRevision(3),
            chunk_generation: ChunkGeneration(3),
            page_table: BTreeMap::from([(SdfPageCoord3::default(), page)]),
            hierarchy_revision: 3,
            checksum: 77,
        }
    }

    fn metric_ref(payload: &SdfChunkPayload) -> WorldSdfPayloadRef {
        WorldSdfPayloadRef::from(payload).with_metric_encoding(
            WorldSdfMetricEncoding::try_new(DISTANCE_UNITS_PER_METER, 1)
                .expect("positive metric scale is valid"),
        )
    }

    #[test]
    fn canonical_metric_topology_is_two_samples_per_edge() {
        assert_eq!(SDF_BRICK_EDGE_SAMPLES, 2);
        assert_eq!(SDF_BRICK_SAMPLE_COUNT, 8);
    }

    #[test]
    fn metric_plane_sampling_decodes_meters_and_trilinearly_interpolates() {
        let payload = plane_payload();
        let sample = sample_world_sdf_metric_distance(
            &metric_ref(&payload),
            &payload,
            &partition(),
            [0.375, 0.5, 0.5],
        )
        .expect("metric plane sample should succeed");

        assert!((sample.signed_distance_estimate_meters() + 0.125).abs() < 1.0e-12);
        assert!((sample.max_absolute_error_meters() - 0.0001).abs() < 1.0e-12);
        assert!((sample.safe_distance_lower_bound_meters() - 0.1249).abs() < 1.0e-12);
    }

    #[test]
    fn occupancy_only_payload_ref_cannot_satisfy_metric_query() {
        let payload = plane_payload();
        let error = sample_world_sdf_metric_distance(
            &WorldSdfPayloadRef::from(&payload),
            &payload,
            &partition(),
            [0.25, 0.5, 0.5],
        )
        .expect_err("occupancy-only ref must fail closed");
        assert_eq!(error, WorldSdfMetricQueryError::MissingMetricEncoding);
    }

    #[test]
    fn exact_payload_reference_mismatch_fails_closed() {
        let payload = plane_payload();
        let mut payload_ref = metric_ref(&payload);
        payload_ref.checksum = payload_ref.checksum.saturating_add(1);
        let error = sample_world_sdf_metric_distance(
            &payload_ref,
            &payload,
            &partition(),
            [0.25, 0.5, 0.5],
        )
        .expect_err("mismatched exact ref must fail");
        assert_eq!(error, WorldSdfMetricQueryError::PayloadReferenceMismatch);
    }

    #[test]
    fn unsupported_metric_layout_revision_fails_closed() {
        let payload = plane_payload();
        let mut payload_ref = metric_ref(&payload);
        payload_ref
            .metric_encoding
            .as_mut()
            .expect("metric encoding exists")
            .sample_layout_revision = WORLD_SDF_METRIC_SAMPLE_LAYOUT_REVISION + 1;
        let error = sample_world_sdf_metric_distance(
            &payload_ref,
            &payload,
            &partition(),
            [0.25, 0.5, 0.5],
        )
        .expect_err("unsupported layout must fail");
        assert_eq!(error, WorldSdfMetricQueryError::UnsupportedMetricEncoding);
    }

    #[test]
    fn another_perfect_cube_sample_count_is_not_an_implicit_metric_topology() {
        let mut payload = plane_payload();
        payload
            .page_table
            .get_mut(&SdfPageCoord3::default())
            .expect("page exists")
            .bricks
            .get_mut(&[0, 0, 0])
            .expect("brick exists")
            .samples
            .distances = vec![0; 27];

        let error = sample_world_sdf_metric_distance(
            &metric_ref(&payload),
            &payload,
            &partition(),
            [0.1, 0.1, 0.1],
        )
        .expect_err("27 values must not select an implicit 3x3x3 metric layout");
        assert_eq!(
            error,
            WorldSdfMetricQueryError::InvalidMetricSampleCount { actual: 27 }
        );
    }

    #[test]
    fn metric_payload_ref_roundtrip_preserves_encoding() {
        let payload = plane_payload();
        let payload_ref = metric_ref(&payload);
        let bytes = postcard::to_allocvec(&payload_ref).expect("serialize metric payload ref");
        let decoded =
            postcard::from_bytes::<WorldSdfPayloadRef>(&bytes).expect("deserialize metric ref");
        assert_eq!(decoded, payload_ref);
    }

    #[test]
    fn invalid_metric_scale_is_rejected_by_constructor() {
        assert!(WorldSdfMetricEncoding::try_new(0, 0).is_none());
    }
}
