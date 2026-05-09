//! File: domain/editor/editor_scene/src/sdf_authoring/document.rs
//! Purpose: Authored SDF operation document and layer state.

use std::collections::BTreeSet;

use crate::{SdfBrushLayerMetadata, SdfPrimitiveSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SdfOperationLayerId(pub u64);

impl SdfOperationLayerId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SdfOperationEntryId(pub u64);

impl SdfOperationEntryId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SdfOperationDocument {
    pub stable_name: String,
    pub display_name: String,
    pub source_revision: u64,
    layers: Vec<SdfOperationLayer>,
    next_layer_id: u64,
    next_operation_id: u64,
}

impl SdfOperationDocument {
    pub fn new(stable_name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            stable_name: stable_name.into(),
            display_name: display_name.into(),
            source_revision: 1,
            layers: Vec::new(),
            next_layer_id: 1,
            next_operation_id: 1,
        }
    }

    pub fn with_default_layer(
        stable_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        let mut document = Self::new(stable_name, display_name);
        document.add_layer("base", "Base");
        document
    }

    pub fn source_revision(&self) -> u64 {
        self.source_revision
    }

    pub fn layers(&self) -> &[SdfOperationLayer] {
        &self.layers
    }

    pub fn layer(&self, layer_id: SdfOperationLayerId) -> Option<&SdfOperationLayer> {
        self.layers.iter().find(|layer| layer.id == layer_id)
    }

    pub fn operation(&self, operation_id: SdfOperationEntryId) -> Option<&SdfOperationEntry> {
        self.layers
            .iter()
            .flat_map(|layer| layer.operations.iter())
            .find(|operation| operation.id == operation_id)
    }

    pub fn add_layer(
        &mut self,
        stable_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> SdfOperationLayerId {
        let id = self.allocate_layer_id();
        self.layers.push(SdfOperationLayer::new(
            id,
            SdfBrushLayerMetadata::new(stable_name, display_name),
        ));
        self.bump_revision();
        id
    }

    pub fn add_operation(
        &mut self,
        layer_id: SdfOperationLayerId,
        display_name: impl Into<String>,
        primitive: SdfPrimitiveSpec,
        material_channel: u16,
    ) -> Result<SdfOperationEntryId, SdfOperationDocumentError> {
        let id = self.allocate_operation_id();
        let source_revision = self.source_revision.saturating_add(1);
        let layer = self
            .layer_mut(layer_id)
            .ok_or(SdfOperationDocumentError::MissingLayer(layer_id))?;
        layer.operations.push(SdfOperationEntry::new(
            id,
            display_name,
            primitive,
            material_channel,
            source_revision,
        ));
        self.bump_revision();
        Ok(id)
    }

    pub fn set_layer_enabled(
        &mut self,
        layer_id: SdfOperationLayerId,
        enabled: bool,
    ) -> Result<(), SdfOperationDocumentError> {
        let layer = self
            .layer_mut(layer_id)
            .ok_or(SdfOperationDocumentError::MissingLayer(layer_id))?;
        layer.metadata.enabled = enabled;
        self.bump_revision();
        Ok(())
    }

    pub fn set_operation_enabled(
        &mut self,
        operation_id: SdfOperationEntryId,
        enabled: bool,
    ) -> Result<(), SdfOperationDocumentError> {
        let operation = self
            .operation_mut(operation_id)
            .ok_or(SdfOperationDocumentError::MissingOperation(operation_id))?;
        operation.enabled = enabled;
        self.bump_revision();
        Ok(())
    }

    pub fn update_operation_primitive(
        &mut self,
        operation_id: SdfOperationEntryId,
        primitive: SdfPrimitiveSpec,
    ) -> Result<(), SdfOperationDocumentError> {
        let next_revision = self.source_revision.saturating_add(1);
        let operation = self
            .operation_mut(operation_id)
            .ok_or(SdfOperationDocumentError::MissingOperation(operation_id))?;
        operation.primitive = primitive;
        operation.source_revision = next_revision;
        self.bump_revision();
        Ok(())
    }

    pub fn move_layer(
        &mut self,
        layer_id: SdfOperationLayerId,
        direction: SdfLayerMoveDirection,
    ) -> Result<(), SdfOperationDocumentError> {
        let index = self
            .layers
            .iter()
            .position(|layer| layer.id == layer_id)
            .ok_or(SdfOperationDocumentError::MissingLayer(layer_id))?;
        match direction {
            SdfLayerMoveDirection::Up if index > 0 => self.layers.swap(index, index - 1),
            SdfLayerMoveDirection::Down if index + 1 < self.layers.len() => {
                self.layers.swap(index, index + 1);
            }
            _ => {}
        }
        self.bump_revision();
        Ok(())
    }

    pub fn duplicate_layer_names(&self) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut duplicates = BTreeSet::new();
        for layer in &self.layers {
            if !seen.insert(layer.metadata.stable_name.clone()) {
                duplicates.insert(layer.metadata.stable_name.clone());
            }
        }
        duplicates
    }

    fn layer_mut(&mut self, layer_id: SdfOperationLayerId) -> Option<&mut SdfOperationLayer> {
        self.layers.iter_mut().find(|layer| layer.id == layer_id)
    }

    fn operation_mut(
        &mut self,
        operation_id: SdfOperationEntryId,
    ) -> Option<&mut SdfOperationEntry> {
        self.layers
            .iter_mut()
            .flat_map(|layer| layer.operations.iter_mut())
            .find(|operation| operation.id == operation_id)
    }

    fn allocate_layer_id(&mut self) -> SdfOperationLayerId {
        let id = SdfOperationLayerId(self.next_layer_id);
        self.next_layer_id = self.next_layer_id.saturating_add(1).max(1);
        id
    }

    fn allocate_operation_id(&mut self) -> SdfOperationEntryId {
        let id = SdfOperationEntryId(self.next_operation_id);
        self.next_operation_id = self.next_operation_id.saturating_add(1).max(1);
        id
    }

    fn bump_revision(&mut self) {
        self.source_revision = self.source_revision.saturating_add(1).max(1);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SdfOperationLayer {
    pub id: SdfOperationLayerId,
    pub metadata: SdfBrushLayerMetadata,
    pub operations: Vec<SdfOperationEntry>,
}

impl SdfOperationLayer {
    pub fn new(id: SdfOperationLayerId, metadata: SdfBrushLayerMetadata) -> Self {
        Self {
            id,
            metadata,
            operations: Vec::new(),
        }
    }

    pub fn enabled_operation_count(&self) -> usize {
        self.operations
            .iter()
            .filter(|operation| operation.enabled)
            .count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SdfOperationEntry {
    pub id: SdfOperationEntryId,
    pub display_name: String,
    pub primitive: SdfPrimitiveSpec,
    pub enabled: bool,
    pub material_channel: u16,
    pub deterministic_seed: u64,
    pub source_revision: u64,
}

impl SdfOperationEntry {
    pub fn new(
        id: SdfOperationEntryId,
        display_name: impl Into<String>,
        primitive: SdfPrimitiveSpec,
        material_channel: u16,
        source_revision: u64,
    ) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            primitive,
            enabled: true,
            material_channel,
            deterministic_seed: stable_operation_seed(id, source_revision),
            source_revision,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfLayerMoveDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfOperationDocumentError {
    MissingLayer(SdfOperationLayerId),
    MissingOperation(SdfOperationEntryId),
}

fn stable_operation_seed(id: SdfOperationEntryId, source_revision: u64) -> u64 {
    id.0.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(source_revision)
}
