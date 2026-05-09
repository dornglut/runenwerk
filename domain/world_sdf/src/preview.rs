use foundation_ratification::{RatificationIssue, RatificationReport};
use serde::{Deserialize, Serialize};
use spatial::ChunkId;

use crate::{
    FieldProductCandidate, FieldProductDescriptor, FieldProductIssueCode, FieldProductKind,
    FieldProductSubject, ratify_field_product_candidate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldPreviewGrid {
    pub chunk_id: ChunkId,
    pub dimensions: [u16; 3],
}

impl FieldPreviewGrid {
    pub const fn new(chunk_id: ChunkId, dimensions: [u16; 3]) -> Self {
        Self {
            chunk_id,
            dimensions,
        }
    }

    pub fn expected_sample_count(&self) -> usize {
        self.dimensions
            .iter()
            .map(|dimension| usize::from((*dimension).max(1)))
            .product()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldPreviewPayload {
    ScalarDistance {
        grid: FieldPreviewGrid,
        samples: Vec<i16>,
    },
    VectorGradient {
        grid: FieldPreviewGrid,
        samples: Vec<[i16; 3]>,
    },
    OccupancySupport {
        grid: FieldPreviewGrid,
        samples: Vec<u8>,
    },
    MaterialChannel {
        grid: FieldPreviewGrid,
        samples: Vec<u16>,
    },
}

impl FieldPreviewPayload {
    pub const fn kind(&self) -> FieldProductKind {
        match self {
            Self::ScalarDistance { .. } => FieldProductKind::ScalarDistance,
            Self::VectorGradient { .. } => FieldProductKind::VectorGradient,
            Self::OccupancySupport { .. } => FieldProductKind::OccupancySupport,
            Self::MaterialChannel { .. } => FieldProductKind::MaterialChannel,
        }
    }

    pub const fn grid(&self) -> &FieldPreviewGrid {
        match self {
            Self::ScalarDistance { grid, .. }
            | Self::VectorGradient { grid, .. }
            | Self::OccupancySupport { grid, .. }
            | Self::MaterialChannel { grid, .. } => grid,
        }
    }

    pub fn sample_count(&self) -> usize {
        match self {
            Self::ScalarDistance { samples, .. } => samples.len(),
            Self::VectorGradient { samples, .. } => samples.len(),
            Self::OccupancySupport { samples, .. } => samples.len(),
            Self::MaterialChannel { samples, .. } => samples.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldPreviewProduct {
    pub descriptor: FieldProductDescriptor,
    pub payload: FieldPreviewPayload,
}

impl FieldPreviewProduct {
    pub const fn new(descriptor: FieldProductDescriptor, payload: FieldPreviewPayload) -> Self {
        Self {
            descriptor,
            payload,
        }
    }
}

pub fn ratify_field_preview_product(
    product: &FieldPreviewProduct,
) -> RatificationReport<FieldProductIssueCode, FieldProductSubject> {
    let mut report =
        ratify_field_product_candidate(&FieldProductCandidate::new(product.descriptor.clone()));
    let subject = FieldProductSubject::Product(product.descriptor.product_id.0);

    if product.descriptor.kind != product.payload.kind() {
        report.push(RatificationIssue::error(
            FieldProductIssueCode::PreviewKindMismatch,
            subject.clone(),
            "field preview payload kind must match descriptor kind",
        ));
    }
    if product.payload.grid().dimensions.contains(&0) {
        report.push(RatificationIssue::error(
            FieldProductIssueCode::PreviewInvalidGridDimensions,
            subject.clone(),
            "field preview grid dimensions must be non-zero",
        ));
    }
    if product.payload.sample_count() != product.payload.grid().expected_sample_count() {
        report.push(RatificationIssue::error(
            FieldProductIssueCode::PreviewSampleCountMismatch,
            subject.clone(),
            "field preview payload sample count must match grid dimensions",
        ));
    }
    if !product
        .descriptor
        .scope
        .chunk_ids
        .contains(&product.payload.grid().chunk_id)
    {
        report.push(RatificationIssue::error(
            FieldProductIssueCode::PreviewChunkOutsideScope,
            subject,
            "field preview payload chunk must be listed in descriptor scope",
        ));
    }

    report
}

#[cfg(test)]
mod tests {
    use spatial::{ChunkCoord3, ChunkId, WorldId};

    use super::*;
    use crate::{FieldProductId, FieldProductLineage, FieldProductScope};

    fn descriptor(chunk_id: ChunkId, kind: FieldProductKind) -> FieldProductDescriptor {
        FieldProductDescriptor::new(
            FieldProductId(1),
            kind,
            FieldProductScope::from_chunks([chunk_id]),
            FieldProductLineage::new(7, "world_sdf.preview.test"),
        )
    }

    #[test]
    fn preview_product_ratifier_accepts_descriptor_payload_match() {
        let chunk_id = ChunkId::new(WorldId(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
        let grid = FieldPreviewGrid::new(chunk_id, [2, 2, 2]);
        let product = FieldPreviewProduct::new(
            descriptor(chunk_id, FieldProductKind::ScalarDistance),
            FieldPreviewPayload::ScalarDistance {
                grid,
                samples: vec![0; 8],
            },
        );

        assert!(!ratify_field_preview_product(&product).has_blocking_issues());
    }

    #[test]
    fn preview_product_ratifier_rejects_kind_and_sample_mismatch() {
        let chunk_id = ChunkId::new(WorldId(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
        let grid = FieldPreviewGrid::new(chunk_id, [2, 2, 2]);
        let product = FieldPreviewProduct::new(
            descriptor(chunk_id, FieldProductKind::VectorGradient),
            FieldPreviewPayload::ScalarDistance {
                grid,
                samples: vec![0; 7],
            },
        );
        let report = ratify_field_preview_product(&product);

        assert!(report.has_blocking_issues());
        assert!(
            report
                .iter()
                .any(|issue| { issue.code() == &FieldProductIssueCode::PreviewKindMismatch })
        );
        assert!(
            report.iter().any(|issue| {
                issue.code() == &FieldProductIssueCode::PreviewSampleCountMismatch
            })
        );
    }
}
