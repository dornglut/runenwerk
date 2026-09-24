//! File: domain/editor/editor_viewport/src/expression/product.rs
//! Purpose: Typed viewport expression product contracts.

use std::num::NonZeroU32;

use editor_core::RealityVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExpressionProductId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionProductKind {
    SceneColor2D,
    PickingIds2D,
    Overlay2D,
    Depth2D,
    Diagnostics2D,
    ScalarField2D,
    VectorField2D,
    Atlas2D,
    VolumeSlice2D,
    BrickmapDebug2D,
    HistoryColor2D,
    MaterialPreview2D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionSourceRealityClass {
    ObservedScene,
    DerivedPicking,
    DerivedOverlay,
    Diagnostics,
    DerivedField,
    DerivedAsset,
    DerivedVolume,
    DerivedHistory,
    DerivedMaterial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionFreshness {
    Current,
    PotentiallyStale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExpressionDimensions {
    pub width: u32,
    pub height: u32,
}

impl ExpressionDimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionFormat {
    Rgba8Unorm,
    R32Uint,
    Depth32Float,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionPresentationHints {
    pub srgb: bool,
    pub premultiplied_alpha: bool,
    pub y_flipped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionChannelLayerSliceMetadata {
    pub channel_label: Option<String>,
    pub layer_label: Option<String>,
    pub slice_label: Option<String>,
    pub slice_count: Option<NonZeroU32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionProductDescriptor {
    pub id: ExpressionProductId,
    pub kind: ExpressionProductKind,
    pub dimensions: ExpressionDimensions,
    pub format: ExpressionFormat,
    pub producer_label: String,
    pub source_reality_class: ExpressionSourceRealityClass,
    pub source_version: RealityVersion,
    pub freshness: ExpressionFreshness,
    pub presentation_hints: ExpressionPresentationHints,
    pub channel_layer_slice: Option<ExpressionChannelLayerSliceMetadata>,
}

impl ExpressionProductDescriptor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ExpressionProductId,
        kind: ExpressionProductKind,
        dimensions: ExpressionDimensions,
        format: ExpressionFormat,
        producer_label: impl Into<String>,
        source_reality_class: ExpressionSourceRealityClass,
        source_version: RealityVersion,
        freshness: ExpressionFreshness,
        presentation_hints: ExpressionPresentationHints,
        channel_layer_slice: Option<ExpressionChannelLayerSliceMetadata>,
    ) -> Self {
        Self {
            id,
            kind,
            dimensions,
            format,
            producer_label: producer_label.into(),
            source_reality_class,
            source_version,
            freshness,
            presentation_hints,
            channel_layer_slice,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_product_kind_subset_is_present() {
        let required = [
            ExpressionProductKind::SceneColor2D,
            ExpressionProductKind::PickingIds2D,
            ExpressionProductKind::Overlay2D,
        ];

        assert_eq!(required.len(), 3);
    }

    #[test]
    fn product_kind_catalog_includes_field_asset_volume_and_history_products() {
        let future_products = [
            ExpressionProductKind::ScalarField2D,
            ExpressionProductKind::VectorField2D,
            ExpressionProductKind::Atlas2D,
            ExpressionProductKind::VolumeSlice2D,
            ExpressionProductKind::BrickmapDebug2D,
            ExpressionProductKind::HistoryColor2D,
            ExpressionProductKind::MaterialPreview2D,
        ];

        assert_eq!(future_products.len(), 7);
    }

    #[test]
    fn material_preview_descriptor_uses_derived_material_reality_class() {
        let descriptor = ExpressionProductDescriptor::new(
            ExpressionProductId(42),
            ExpressionProductKind::MaterialPreview2D,
            ExpressionDimensions::new(512, 512),
            ExpressionFormat::Rgba8Unorm,
            "material.preview",
            ExpressionSourceRealityClass::DerivedMaterial,
            RealityVersion(3),
            ExpressionFreshness::Current,
            ExpressionPresentationHints {
                srgb: true,
                premultiplied_alpha: false,
                y_flipped: false,
            },
            None,
        );

        assert_eq!(descriptor.kind, ExpressionProductKind::MaterialPreview2D);
        assert_eq!(
            descriptor.source_reality_class,
            ExpressionSourceRealityClass::DerivedMaterial
        );
    }

    #[test]
    fn descriptor_keeps_required_identity_fields() {
        let descriptor = ExpressionProductDescriptor::new(
            ExpressionProductId(7),
            ExpressionProductKind::SceneColor2D,
            ExpressionDimensions::new(1280, 720),
            ExpressionFormat::Rgba8Unorm,
            "scene.producer",
            ExpressionSourceRealityClass::ObservedScene,
            RealityVersion(13),
            ExpressionFreshness::Current,
            ExpressionPresentationHints {
                srgb: true,
                premultiplied_alpha: false,
                y_flipped: false,
            },
            None,
        );

        assert_eq!(descriptor.id, ExpressionProductId(7));
        assert_eq!(descriptor.kind, ExpressionProductKind::SceneColor2D);
        assert_eq!(descriptor.dimensions, ExpressionDimensions::new(1280, 720));
        assert_eq!(descriptor.source_version, RealityVersion(13));
    }
}
