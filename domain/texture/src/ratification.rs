//! File: domain/texture/src/ratification.rs
//! Purpose: Domain ratification for texture descriptors and preview requests.

use ratification::{RatificationIssue, RatificationReport};

use crate::{TextureDescriptor, TextureDimension, TexturePreviewDescriptor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureIssueCode {
    EmptyLabel,
    InvalidExtent,
    Texture2dDepthMustBeOne,
    Texture3dDepthMustExceedOne,
    MipCountZero,
    Ktx2DescriptorHashMissing,
    Ktx2ArtifactRevisionMissing,
    Ktx2LayerCountZero,
    Ktx2FaceCountInvalid,
    Ktx2LevelCountMismatch,
    Ktx2ByteLengthZero,
    Ktx2LevelByteLengthZero,
    Ktx2UnsupportedTranscode,
    PreviewProductMismatch,
    PreviewMipOutOfRange,
    PreviewSliceOutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureIssueSubject {
    Descriptor,
    Product,
    Preview,
}

pub type TextureRatificationReport = RatificationReport<TextureIssueCode, TextureIssueSubject>;

pub fn ratify_texture_descriptor(descriptor: &TextureDescriptor) -> TextureRatificationReport {
    let mut report = TextureRatificationReport::new();

    if descriptor.label.trim().is_empty() {
        report.push(RatificationIssue::error(
            TextureIssueCode::EmptyLabel,
            TextureIssueSubject::Descriptor,
            "texture descriptor label must not be empty",
        ));
    }
    if !descriptor.extent.is_non_zero() {
        report.push(RatificationIssue::error(
            TextureIssueCode::InvalidExtent,
            TextureIssueSubject::Descriptor,
            "texture extent dimensions must be non-zero",
        ));
    }
    match descriptor.dimension {
        TextureDimension::Texture2D if descriptor.extent.depth != 1 => {
            report.push(RatificationIssue::error(
                TextureIssueCode::Texture2dDepthMustBeOne,
                TextureIssueSubject::Descriptor,
                "Texture2D descriptors must use depth 1",
            ));
        }
        TextureDimension::Texture3DVolume if descriptor.extent.depth <= 1 => {
            report.push(RatificationIssue::error(
                TextureIssueCode::Texture3dDepthMustExceedOne,
                TextureIssueSubject::Descriptor,
                "Texture3D descriptors must use depth greater than 1",
            ));
        }
        _ => {}
    }
    if descriptor.mip_count == 0 {
        report.push(RatificationIssue::error(
            TextureIssueCode::MipCountZero,
            TextureIssueSubject::Descriptor,
            "texture descriptors must expose at least one mip level",
        ));
    }
    let ktx2 = descriptor.ktx2_metadata();
    if ktx2.descriptor_hash.trim().is_empty() {
        report.push(RatificationIssue::error(
            TextureIssueCode::Ktx2DescriptorHashMissing,
            TextureIssueSubject::Descriptor,
            "KTX2 texture descriptors must carry a descriptor hash",
        ));
    }
    if ktx2.artifact_revision.trim().is_empty() {
        report.push(RatificationIssue::error(
            TextureIssueCode::Ktx2ArtifactRevisionMissing,
            TextureIssueSubject::Descriptor,
            "KTX2 texture descriptors must carry an artifact revision",
        ));
    }
    if ktx2.layer_count == 0 {
        report.push(RatificationIssue::error(
            TextureIssueCode::Ktx2LayerCountZero,
            TextureIssueSubject::Descriptor,
            "KTX2 texture descriptors must expose at least one layer",
        ));
    }
    if ktx2.face_count != 1 {
        report.push(RatificationIssue::error(
            TextureIssueCode::Ktx2FaceCountInvalid,
            TextureIssueSubject::Descriptor,
            "material KTX2 texture descriptors must be non-cubemap textures",
        ));
    }
    if ktx2.level_count != descriptor.mip_count {
        report.push(RatificationIssue::error(
            TextureIssueCode::Ktx2LevelCountMismatch,
            TextureIssueSubject::Descriptor,
            "KTX2 level count must match the texture descriptor mip count",
        ));
    }
    if matches!(
        ktx2.transcode_status,
        crate::TextureTranscodeStatus::Unsupported
    ) {
        report.push(RatificationIssue::error(
            TextureIssueCode::Ktx2UnsupportedTranscode,
            TextureIssueSubject::Descriptor,
            "KTX2 texture descriptor marks the artifact as unsupported for runtime residency",
        ));
    }
    if ktx2.byte_length.is_some_and(|length| length == 0) {
        report.push(RatificationIssue::error(
            TextureIssueCode::Ktx2ByteLengthZero,
            TextureIssueSubject::Descriptor,
            "KTX2 artifact byte length must be non-zero when present",
        ));
    }
    if ktx2.level_byte_lengths.contains(&0) {
        report.push(RatificationIssue::error(
            TextureIssueCode::Ktx2LevelByteLengthZero,
            TextureIssueSubject::Descriptor,
            "KTX2 level byte lengths must be non-zero",
        ));
    }

    report
}

pub fn ratify_texture_preview(
    descriptor: &TextureDescriptor,
    preview: &TexturePreviewDescriptor,
) -> TextureRatificationReport {
    let mut report = ratify_texture_descriptor(descriptor);

    if preview.product_id != descriptor.product_id {
        report.push(RatificationIssue::error(
            TextureIssueCode::PreviewProductMismatch,
            TextureIssueSubject::Preview,
            "texture preview product id does not match descriptor product id",
        ));
    }
    if preview.mip_level >= descriptor.mip_count {
        report.push(RatificationIssue::error(
            TextureIssueCode::PreviewMipOutOfRange,
            TextureIssueSubject::Preview,
            "texture preview mip level is outside descriptor mip range",
        ));
    }
    if preview.slice_index >= descriptor.extent.depth {
        report.push(RatificationIssue::error(
            TextureIssueCode::PreviewSliceOutOfRange,
            TextureIssueSubject::Preview,
            "texture preview slice index is outside descriptor depth range",
        ));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TextureExtent, TextureProductId};

    #[test]
    fn texture2d_rejects_volume_depth() {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(1),
            "albedo",
            TextureDimension::Texture2D,
            TextureExtent::new(128, 128, 4),
        );

        let report = ratify_texture_descriptor(&descriptor);

        assert!(report.has_blocking_issues());
        assert_eq!(
            *report.issues()[0].code(),
            TextureIssueCode::Texture2dDepthMustBeOne
        );
    }

    #[test]
    fn texture3d_preview_rejects_out_of_range_slice() {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(2),
            "volume",
            TextureDimension::Texture3DVolume,
            TextureExtent::new(32, 32, 8),
        );
        let preview = TexturePreviewDescriptor::new(TextureProductId::new(2)).with_slice_index(8);

        let report = ratify_texture_preview(&descriptor, &preview);

        assert!(report.has_blocking_issues());
        assert_eq!(
            *report.issues()[0].code(),
            TextureIssueCode::PreviewSliceOutOfRange
        );
    }

    #[test]
    fn ktx2_metadata_rejects_unsupported_transcode() {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(3),
            "unsupported",
            TextureDimension::Texture2D,
            TextureExtent::new(16, 16, 1),
        )
        .with_ktx2_metadata(
            crate::Ktx2TextureMetadata::new(crate::TexturePixelFormat::Bc7Unorm, 1, "hash", "1")
                .with_transcode_status(crate::TextureTranscodeStatus::Unsupported),
        );

        let report = ratify_texture_descriptor(&descriptor);

        assert!(report.has_blocking_issues());
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| *issue.code() == TextureIssueCode::Ktx2UnsupportedTranscode)
        );
    }

    #[test]
    fn ktx2_descriptor_hash_is_stable_and_non_empty() {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(4),
            "albedo",
            TextureDimension::Texture2D,
            TextureExtent::new(16, 16, 1),
        );

        assert!(!descriptor.descriptor_hash().is_empty());
        assert!(!ratify_texture_descriptor(&descriptor).has_blocking_issues());
    }
}
