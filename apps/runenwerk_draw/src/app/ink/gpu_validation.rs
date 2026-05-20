//! App-owned GPU validation and promotion/fallback state.

use std::collections::BTreeMap;

use drawing::{CanvasTileId, DrawingInkTileProduct, ProductQualityClass};

use crate::app::DrawingInkSurfaceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingInkGpuValidationStatus {
    Pending,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawingInkGpuValidationMetrics {
    pub max_channel_delta: u8,
    pub changed_pixel_count: u64,
    pub total_pixel_count: u64,
    pub changed_pixel_ratio: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingInkGpuValidationEntry {
    pub surface_kind: DrawingInkSurfaceKind,
    pub tile_id: CanvasTileId,
    pub quality_class: ProductQualityClass,
    pub descriptor_generation: u64,
    pub status: DrawingInkGpuValidationStatus,
    pub metrics: Option<DrawingInkGpuValidationMetrics>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct DrawingInkGpuValidationKey {
    surface_kind: DrawingInkSurfaceKind,
    tile_id: CanvasTileId,
    quality_class: ProductQualityClass,
}

impl DrawingInkGpuValidationKey {
    fn new(surface_kind: DrawingInkSurfaceKind, product: &DrawingInkTileProduct) -> Self {
        Self {
            surface_kind,
            tile_id: product.metadata.tile_id,
            quality_class: product.metadata.quality_class,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DrawingInkGpuValidationState {
    gpu_validations: BTreeMap<DrawingInkGpuValidationKey, DrawingInkGpuValidationEntry>,
}

impl DrawingInkGpuValidationState {
    pub(crate) fn gpu_validation_metadata(&self) -> Vec<DrawingInkGpuValidationEntry> {
        self.gpu_validations.values().cloned().collect()
    }

    pub(crate) fn gpu_validation_status(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) -> Option<DrawingInkGpuValidationStatus> {
        self.current_gpu_validation(surface_kind, product)
            .map(|entry| entry.status)
    }

    pub(crate) fn should_request_gpu_validation(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) -> bool {
        !matches!(
            self.gpu_validation_status(surface_kind, product),
            Some(DrawingInkGpuValidationStatus::Passed | DrawingInkGpuValidationStatus::Failed)
        )
    }

    pub(crate) fn should_request_gpu_target(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) -> bool {
        matches!(
            self.gpu_validation_status(surface_kind, product),
            Some(DrawingInkGpuValidationStatus::Pending | DrawingInkGpuValidationStatus::Passed)
        )
    }

    pub(crate) fn visible_surface_kind_for(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) -> DrawingInkSurfaceKind {
        match self.gpu_validation_status(surface_kind, product) {
            Some(DrawingInkGpuValidationStatus::Passed) => surface_kind.gpu_variant(),
            _ => surface_kind,
        }
    }

    pub(crate) fn record_gpu_validation_pending(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) {
        let key = DrawingInkGpuValidationKey::new(surface_kind, product);
        if self
            .gpu_validations
            .get(&key)
            .is_some_and(|entry| entry.descriptor_generation == product.descriptor_generation)
        {
            return;
        }
        self.gpu_validations.insert(
            key,
            DrawingInkGpuValidationEntry {
                surface_kind,
                tile_id: product.metadata.tile_id,
                quality_class: product.metadata.quality_class,
                descriptor_generation: product.descriptor_generation,
                status: DrawingInkGpuValidationStatus::Pending,
                metrics: None,
                reason: None,
            },
        );
    }

    pub(crate) fn record_gpu_validation_pass(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
        metrics: DrawingInkGpuValidationMetrics,
    ) -> (bool, String) {
        self.record_gpu_validation_result(
            surface_kind,
            product,
            DrawingInkGpuValidationStatus::Passed,
            Some(metrics),
            None,
        )
    }

    pub(crate) fn record_gpu_validation_failure(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
        reason: String,
    ) -> (bool, String) {
        self.record_gpu_validation_result(
            surface_kind,
            product,
            DrawingInkGpuValidationStatus::Failed,
            None,
            Some(reason),
        )
    }

    fn current_gpu_validation(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) -> Option<&DrawingInkGpuValidationEntry> {
        let key = DrawingInkGpuValidationKey::new(surface_kind, product);
        self.gpu_validations
            .get(&key)
            .filter(|entry| entry.descriptor_generation == product.descriptor_generation)
    }

    fn record_gpu_validation_result(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
        status: DrawingInkGpuValidationStatus,
        metrics: Option<DrawingInkGpuValidationMetrics>,
        reason: Option<String>,
    ) -> (bool, String) {
        let key = DrawingInkGpuValidationKey::new(surface_kind, product);
        let summary = reason.clone().unwrap_or_else(|| {
            format!(
                "gpu validation {:?} for tile L{}:{}:{} generation {}",
                status,
                product.metadata.tile_id.level.raw(),
                product.metadata.tile_id.x,
                product.metadata.tile_id.y,
                product.descriptor_generation
            )
        });
        self.gpu_validations.insert(
            key,
            DrawingInkGpuValidationEntry {
                surface_kind,
                tile_id: product.metadata.tile_id,
                quality_class: product.metadata.quality_class,
                descriptor_generation: product.descriptor_generation,
                status,
                metrics,
                reason,
            },
        );
        (status == DrawingInkGpuValidationStatus::Passed, summary)
    }
}
