use crate::{
    SDF_METRIC_BRICK_EDGE_SAMPLES, SDF_METRIC_BRICK_SAMPLE_COUNT, SDF_PAGE_EDGE_BRICKS,
    SdfBrickRecord, SdfChunkPayload, SdfPageCoord3, WorldSdfMetricPayloadRef,
};
use runen_spatial::{GridPartitionConfig, WorldPosition};
use std::error::Error;
use std::fmt;

const SDF_METRIC_PAGE_BRICK_COUNT: usize =
    SDF_PAGE_EDGE_BRICKS * SDF_PAGE_EDGE_BRICKS * SDF_PAGE_EDGE_BRICKS;

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
pub enum WorldSdfMetricError {
    UnsupportedMetricEncoding,
    PayloadReferenceMismatch,
    InvalidWorldPosition,
    PositionOutsidePayload,
    InvalidMetricPageCount { actual: usize },
    InvalidMetricPageCoord { actual: SdfPageCoord3 },
    InvalidMetricBrickCount { actual: usize },
    MissingMetricBrick { brick_coord: [u8; 3] },
    InvalidMetricSampleCount { actual: usize },
}

impl fmt::Display for WorldSdfMetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMetricEncoding => {
                write!(
                    f,
                    "world SDF metric-field encoding is unsupported or invalid"
                )
            }
            Self::PayloadReferenceMismatch => {
                write!(
                    f,
                    "world SDF payload does not match the exact referenced chunk revision/checksum"
                )
            }
            Self::InvalidWorldPosition => write!(f, "metric field query position is invalid"),
            Self::PositionOutsidePayload => {
                write!(
                    f,
                    "metric field query position lies outside the referenced chunk"
                )
            }
            Self::InvalidMetricPageCount { actual } => write!(
                f,
                "metric SDF layout revision 1 requires exactly one page, found {actual}"
            ),
            Self::InvalidMetricPageCoord { actual } => write!(
                f,
                "metric SDF layout revision 1 requires canonical page coordinate [0, 0, 0], found [{}, {}, {}]",
                actual.x, actual.y, actual.z
            ),
            Self::InvalidMetricBrickCount { actual } => write!(
                f,
                "metric SDF page requires exactly {SDF_METRIC_PAGE_BRICK_COUNT} bricks, found {actual}"
            ),
            Self::MissingMetricBrick { brick_coord } => write!(
                f,
                "metric SDF page is missing canonical brick coordinate {brick_coord:?}"
            ),
            Self::InvalidMetricSampleCount { actual } => write!(
                f,
                "metric SDF brick must contain exactly {SDF_METRIC_BRICK_SAMPLE_COUNT} canonical samples, found {actual}"
            ),
        }
    }
}

impl Error for WorldSdfMetricError {}

pub fn validate_world_sdf_metric_payload(
    metric_payload_ref: &WorldSdfMetricPayloadRef,
    payload: &SdfChunkPayload,
) -> Result<(), WorldSdfMetricError> {
    validate_metric_reference(metric_payload_ref, payload)?;

    if payload.page_table.len() != 1 {
        return Err(WorldSdfMetricError::InvalidMetricPageCount {
            actual: payload.page_table.len(),
        });
    }

    let (page_coord, page) = payload
        .page_table
        .iter()
        .next()
        .expect("one-page metric payload was checked above");
    if *page_coord != SdfPageCoord3::default() {
        return Err(WorldSdfMetricError::InvalidMetricPageCoord {
            actual: *page_coord,
        });
    }
    if page.bricks.len() != SDF_METRIC_PAGE_BRICK_COUNT {
        return Err(WorldSdfMetricError::InvalidMetricBrickCount {
            actual: page.bricks.len(),
        });
    }

    for z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
        for y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                let brick_coord = [x, y, z];
                let brick = page
                    .bricks
                    .get(&brick_coord)
                    .ok_or(WorldSdfMetricError::MissingMetricBrick { brick_coord })?;
                validate_metric_brick(brick)?;
            }
        }
    }

    Ok(())
}

pub fn sample_world_sdf_metric_distance(
    metric_payload_ref: &WorldSdfMetricPayloadRef,
    payload: &SdfChunkPayload,
    partition: &GridPartitionConfig,
    world_position_meters: [f64; 3],
) -> Result<WorldSdfMetricSample, WorldSdfMetricError> {
    validate_world_sdf_metric_payload(metric_payload_ref, payload)?;
    let payload_ref = metric_payload_ref.payload_ref;
    let encoding = metric_payload_ref.encoding;

    let world_position = WorldPosition::try_new(payload_ref.chunk_id.world_id, world_position_meters)
        .map_err(|_| WorldSdfMetricError::InvalidWorldPosition)?;
    let query_chunk = partition
        .chunk_id_from_world_position(world_position)
        .map_err(|_| WorldSdfMetricError::InvalidWorldPosition)?;
    if query_chunk != payload_ref.chunk_id {
        return Err(WorldSdfMetricError::PositionOutsidePayload);
    }

    let local = chunk_local_position(partition, payload_ref.chunk_id, world_position_meters)?;
    let (brick_coord, local_in_brick) = metric_brick_lookup(partition, local);
    let page = payload
        .page_table
        .values()
        .next()
        .expect("one-page metric payload was validated above");
    let brick = page
        .bricks
        .get(&brick_coord)
        .expect("complete metric brick coverage was validated above");
    let encoded_distance = sample_metric_brick(brick, local_in_brick)?;

    Ok(WorldSdfMetricSample {
        signed_distance_estimate_meters: encoding.decode_distance_meters(encoded_distance),
        max_absolute_error_meters: encoding.max_absolute_error_meters(),
    })
}

fn validate_metric_reference(
    metric_payload_ref: &WorldSdfMetricPayloadRef,
    payload: &SdfChunkPayload,
) -> Result<(), WorldSdfMetricError> {
    if !metric_payload_ref.encoding.is_supported() {
        return Err(WorldSdfMetricError::UnsupportedMetricEncoding);
    }
    let payload_ref = metric_payload_ref.payload_ref;
    if payload_ref.chunk_id != payload.chunk_id
        || payload_ref.chunk_revision != payload.chunk_revision
        || payload_ref.checksum != payload.checksum
    {
        return Err(WorldSdfMetricError::PayloadReferenceMismatch);
    }
    Ok(())
}

fn validate_metric_brick(brick: &SdfBrickRecord) -> Result<(), WorldSdfMetricError> {
    if brick.samples.distances.len() != SDF_METRIC_BRICK_SAMPLE_COUNT {
        return Err(WorldSdfMetricError::InvalidMetricSampleCount {
            actual: brick.samples.distances.len(),
        });
    }
    Ok(())
}

fn chunk_local_position(
    partition: &GridPartitionConfig,
    chunk_id: runen_spatial::ChunkId,
    world_position_meters: [f64; 3],
) -> Result<[f64; 3], WorldSdfMetricError> {
    let origin = partition
        .chunk_origin_world_position(chunk_id.world_id, chunk_id.coord)
        .map_err(|_| WorldSdfMetricError::InvalidWorldPosition)?;
    let origin = origin.meters();
    Ok([
        world_position_meters[0] - origin[0],
        world_position_meters[1] - origin[1],
        world_position_meters[2] - origin[2],
    ])
}

fn metric_brick_lookup(partition: &GridPartitionConfig, local: [f64; 3]) -> ([u8; 3], [f64; 3]) {
    let edge = partition.chunk_edge_meters();
    let local_clamped = [
        local[0].clamp(0.0, edge * (1.0 - 1.0e-12)),
        local[1].clamp(0.0, edge * (1.0 - 1.0e-12)),
        local[2].clamp(0.0, edge * (1.0 - 1.0e-12)),
    ];
    let (brick_x, local_x) = quantize_metric_axis(local_clamped[0], edge);
    let (brick_y, local_y) = quantize_metric_axis(local_clamped[1], edge);
    let (brick_z, local_z) = quantize_metric_axis(local_clamped[2], edge);
    ([brick_x, brick_y, brick_z], [local_x, local_y, local_z])
}

fn quantize_metric_axis(local_axis: f64, edge: f64) -> (u8, f64) {
    let brick_coord_f = (local_axis / edge) * SDF_PAGE_EDGE_BRICKS as f64;
    let brick_index = brick_coord_f
        .floor()
        .clamp(0.0, (SDF_PAGE_EDGE_BRICKS - 1) as f64) as u8;
    let brick_local = (brick_coord_f - f64::from(brick_index)).clamp(0.0, 1.0 - 1.0e-12);
    (brick_index, brick_local)
}

fn sample_metric_brick(
    brick: &SdfBrickRecord,
    local_in_brick: [f64; 3],
) -> Result<f64, WorldSdfMetricError> {
    validate_metric_brick(brick)?;
    debug_assert_eq!(SDF_METRIC_BRICK_EDGE_SAMPLES, 3);

    let sample_at = |x: usize, y: usize, z: usize| -> f64 {
        f64::from(brick.samples.distances[cube_sample_index(x, y, z)])
    };
    let (x0, x1, tx) = metric_sample_axis(local_in_brick[0]);
    let (y0, y1, ty) = metric_sample_axis(local_in_brick[1]);
    let (z0, z1, tz) = metric_sample_axis(local_in_brick[2]);

    let c00 = lerp(sample_at(x0, y0, z0), sample_at(x1, y0, z0), tx);
    let c10 = lerp(sample_at(x0, y1, z0), sample_at(x1, y1, z0), tx);
    let c01 = lerp(sample_at(x0, y0, z1), sample_at(x1, y0, z1), tx);
    let c11 = lerp(sample_at(x0, y1, z1), sample_at(x1, y1, z1), tx);
    let c0 = lerp(c00, c10, ty);
    let c1 = lerp(c01, c11, ty);
    Ok(lerp(c0, c1, tz))
}

fn metric_sample_axis(local_axis: f64) -> (usize, usize, f64) {
    let last = SDF_METRIC_BRICK_EDGE_SAMPLES - 1;
    let scaled = local_axis.clamp(0.0, 1.0) * last as f64;
    let lower = (scaled.floor() as usize).min(last - 1);
    let upper = lower + 1;
    let t = scaled - lower as f64;
    (lower, upper, t)
}

fn cube_sample_index(x: usize, y: usize, z: usize) -> usize {
    z * SDF_METRIC_BRICK_EDGE_SAMPLES * SDF_METRIC_BRICK_EDGE_SAMPLES
        + y * SDF_METRIC_BRICK_EDGE_SAMPLES
        + x
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SdfBrickMetadata, SdfBrickSamples, SdfPageCoord3, SdfPageRecord,
        WORLD_SDF_METRIC_LAYOUT_REVISION, WorldSdfMetricEncoding,
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

    fn affine_field_payload() -> SdfChunkPayload {
        let mut page = SdfPageRecord {
            page_generation: 0,
            bricks: BTreeMap::new(),
        };
        let edge = SDF_PAGE_EDGE_BRICKS as f64;
        for brick_z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for brick_y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                for brick_x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                    let mut distances = Vec::with_capacity(SDF_METRIC_BRICK_SAMPLE_COUNT);
                    for sample_z in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                        for sample_y in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                            for sample_x in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                                let sample_denominator =
                                    (SDF_METRIC_BRICK_EDGE_SAMPLES - 1) as f64;
                                let point_x =
                                    (f64::from(brick_x) + sample_x as f64 / sample_denominator)
                                        / edge;
                                let point_y =
                                    (f64::from(brick_y) + sample_y as f64 / sample_denominator)
                                        / edge;
                                let point_z =
                                    (f64::from(brick_z) + sample_z as f64 / sample_denominator)
                                        / edge;
                                let distance =
                                    0.25 * point_x + 0.5 * point_y + 0.75 * point_z - 0.5;
                                distances
                                    .push((distance * f64::from(DISTANCE_UNITS_PER_METER)).round()
                                        as i16);
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

    fn metric_ref(payload: &SdfChunkPayload) -> WorldSdfMetricPayloadRef {
        WorldSdfMetricPayloadRef::try_new(
            crate::WorldSdfPayloadRef::from(payload),
            WorldSdfMetricEncoding::try_new(DISTANCE_UNITS_PER_METER, 1)
                .expect("positive metric scale is valid"),
        )
        .expect("supported metric capability should be valid")
    }

    #[test]
    fn canonical_metric_topology_aligns_half_brick_sample_planes() {
        assert_eq!(SDF_METRIC_BRICK_EDGE_SAMPLES, 3);
        assert_eq!(SDF_METRIC_BRICK_SAMPLE_COUNT, 27);
        assert_eq!(SDF_METRIC_PAGE_BRICK_COUNT, 64);

        let (lower, upper, t) = metric_sample_axis(0.5);
        assert_eq!((lower, upper), (1, 2));
        assert_eq!(t, 0.0);
    }

    #[test]
    fn metric_affine_field_sampling_respects_xyz_corner_order_and_decodes_meters() {
        let payload = affine_field_payload();
        validate_world_sdf_metric_payload(&metric_ref(&payload), &payload)
            .expect("complete plane payload should satisfy metric layout revision 1");
        let sample = sample_world_sdf_metric_distance(
            &metric_ref(&payload),
            &payload,
            &partition(),
            [0.375, 0.5, 0.625],
        )
        .expect("metric affine-field sample should succeed");

        assert!((sample.signed_distance_estimate_meters() - 0.3125).abs() < 1.0e-12);
        assert!((sample.max_absolute_error_meters() - 0.0001).abs() < 1.0e-12);
        assert!((sample.safe_distance_lower_bound_meters() - 0.3124).abs() < 1.0e-12);
    }

    #[test]
    fn exact_payload_reference_mismatch_fails_closed() {
        let payload = affine_field_payload();
        let mut payload_ref = metric_ref(&payload);
        payload_ref.payload_ref.checksum = payload_ref.payload_ref.checksum.saturating_add(1);
        let error = validate_world_sdf_metric_payload(&payload_ref, &payload)
            .expect_err("mismatched exact ref must fail");
        assert_eq!(error, WorldSdfMetricError::PayloadReferenceMismatch);
    }

    #[test]
    fn unsupported_metric_layout_revision_fails_closed() {
        let payload = affine_field_payload();
        let mut payload_ref = metric_ref(&payload);
        payload_ref.encoding.layout_revision = WORLD_SDF_METRIC_LAYOUT_REVISION + 1;
        let error = validate_world_sdf_metric_payload(&payload_ref, &payload)
            .expect_err("unsupported layout must fail");
        assert_eq!(error, WorldSdfMetricError::UnsupportedMetricEncoding);
    }

    #[test]
    fn metric_layout_revision_one_rejects_multiple_pages() {
        let mut payload = affine_field_payload();
        let page = payload
            .page_table
            .values()
            .next()
            .expect("affine field page exists")
            .clone();
        payload
            .page_table
            .insert(SdfPageCoord3 { x: 1, y: 0, z: 0 }, page);

        let error = validate_world_sdf_metric_payload(&metric_ref(&payload), &payload)
            .expect_err("revision one must remain one-page");
        assert_eq!(
            error,
            WorldSdfMetricError::InvalidMetricPageCount { actual: 2 }
        );
    }

    #[test]
    fn metric_layout_revision_one_requires_canonical_origin_page() {
        let mut payload = affine_field_payload();
        let page = payload
            .page_table
            .remove(&SdfPageCoord3::default())
            .expect("origin page exists");
        let noncanonical = SdfPageCoord3 { x: 1, y: 0, z: 0 };
        payload.page_table.insert(noncanonical, page);

        let error = validate_world_sdf_metric_payload(&metric_ref(&payload), &payload)
            .expect_err("revision one must not reinterpret a noncanonical page coordinate");
        assert_eq!(
            error,
            WorldSdfMetricError::InvalidMetricPageCoord {
                actual: noncanonical
            }
        );
    }

    #[test]
    fn metric_layout_revision_one_requires_complete_brick_coverage() {
        let mut payload = affine_field_payload();
        payload
            .page_table
            .get_mut(&SdfPageCoord3::default())
            .expect("page exists")
            .bricks
            .remove(&[3, 3, 3]);

        let error = validate_world_sdf_metric_payload(&metric_ref(&payload), &payload)
            .expect_err("revision one must cover all canonical brick coordinates");
        assert_eq!(
            error,
            WorldSdfMetricError::InvalidMetricBrickCount { actual: 63 }
        );
    }

    #[test]
    fn malformed_unqueried_brick_invalidates_metric_payload() {
        let mut payload = affine_field_payload();
        payload
            .page_table
            .get_mut(&SdfPageCoord3::default())
            .expect("page exists")
            .bricks
            .get_mut(&[3, 3, 3])
            .expect("brick exists")
            .samples
            .distances = vec![0; 8];

        let error = sample_world_sdf_metric_distance(
            &metric_ref(&payload),
            &payload,
            &partition(),
            [0.1, 0.1, 0.1],
        )
        .expect_err("one malformed brick must invalidate the whole metric payload");
        assert_eq!(
            error,
            WorldSdfMetricError::InvalidMetricSampleCount { actual: 8 }
        );
    }

    #[test]
    fn other_perfect_cube_topologies_do_not_select_metric_layout_revision_one() {
        for sample_count in [8_usize, 64] {
            let mut payload = affine_field_payload();
            payload
                .page_table
                .get_mut(&SdfPageCoord3::default())
                .expect("page exists")
                .bricks
                .get_mut(&[0, 0, 0])
                .expect("brick exists")
                .samples
                .distances = vec![0; sample_count];

            let error = validate_world_sdf_metric_payload(&metric_ref(&payload), &payload)
                .expect_err("another perfect-cube topology must not select metric layout revision one");
            assert_eq!(
                error,
                WorldSdfMetricError::InvalidMetricSampleCount {
                    actual: sample_count
                }
            );
        }
    }

    #[test]
    fn metric_payload_ref_roundtrip_preserves_encoding() {
        let payload = affine_field_payload();
        let payload_ref = metric_ref(&payload);
        let bytes = postcard::to_allocvec(&payload_ref).expect("serialize metric payload ref");
        let decoded = postcard::from_bytes::<WorldSdfMetricPayloadRef>(&bytes)
            .expect("deserialize metric capability");
        assert_eq!(decoded, payload_ref);
    }

    #[test]
    fn invalid_metric_scale_is_rejected_by_constructor() {
        assert!(WorldSdfMetricEncoding::try_new(0, 0).is_none());

        let payload = affine_field_payload();
        let unsupported = WorldSdfMetricEncoding {
            layout_revision: WORLD_SDF_METRIC_LAYOUT_REVISION + 1,
            distance_units_per_meter: DISTANCE_UNITS_PER_METER,
            max_absolute_error_units: 1,
        };
        assert!(
            WorldSdfMetricPayloadRef::try_new(crate::WorldSdfPayloadRef::from(&payload), unsupported)
                .is_none()
        );
    }
}
