//! File: domain/editor/editor_scene/src/model/material.rs
//! Purpose: Source-backed scene material palette and primitive slot contracts.

use std::collections::BTreeMap;

use asset::{AssetId, AssetSourceId, AssetSourceRevisionId};
use editor_core::EntityId;

pub const DEFAULT_SCENE_MATERIAL_SLOT_ID: SceneMaterialSlotId = SceneMaterialSlotId(1);
pub const MAX_PORTABLE_SCENE_MATERIAL_SLOTS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneMaterialSlotId(pub u64);

impl SceneMaterialSlotId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneMaterialPaletteEntryId(pub u64);

impl SceneMaterialPaletteEntryId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SdfPrimitiveSourceId(pub EntityId);

impl SdfPrimitiveSourceId {
    pub const fn new(entity_id: EntityId) -> Self {
        Self(entity_id)
    }

    pub const fn entity_id(self) -> EntityId {
        self.0
    }
}

impl From<EntityId> for SdfPrimitiveSourceId {
    fn from(value: EntityId) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneModelMeshSourceId {
    pub asset_id: AssetId,
    pub source_id: AssetSourceId,
    pub source_revision_id: Option<AssetSourceRevisionId>,
    pub source_revision: Option<String>,
}

impl SceneModelMeshSourceId {
    pub fn new(asset_id: AssetId, source_id: AssetSourceId) -> Self {
        Self {
            asset_id,
            source_id,
            source_revision_id: None,
            source_revision: None,
        }
    }

    pub fn with_source_revision_id(mut self, revision_id: AssetSourceRevisionId) -> Self {
        self.source_revision_id = Some(revision_id);
        self
    }

    pub fn with_source_revision(mut self, revision: impl Into<String>) -> Self {
        self.source_revision = Some(revision.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneMeshMaterialRegionId(String);

impl SceneMeshMaterialRegionId {
    pub fn new(region_key: impl Into<String>) -> Result<Self, String> {
        let region_key = region_key.into();
        let trimmed = region_key.trim();
        if trimmed.is_empty() {
            return Err("scene mesh material region id must not be empty".to_string());
        }
        if trimmed != region_key {
            return Err("scene mesh material region id must already be normalized".to_string());
        }
        if is_transient_mesh_material_region_key(trimmed) {
            return Err(format!(
                "scene mesh material region id {trimmed:?} uses transient runtime identity"
            ));
        }
        Ok(Self(region_key))
    }

    pub fn key(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneModelMeshMaterialRegionSourceId {
    pub model_mesh_source_id: SceneModelMeshSourceId,
    pub material_region_id: SceneMeshMaterialRegionId,
}

impl SceneModelMeshMaterialRegionSourceId {
    pub fn new(
        model_mesh_source_id: SceneModelMeshSourceId,
        material_region_id: SceneMeshMaterialRegionId,
    ) -> Self {
        Self {
            model_mesh_source_id,
            material_region_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialSourceRef {
    pub asset_id: AssetId,
    pub source_id: AssetSourceId,
    pub source_revision_id: Option<AssetSourceRevisionId>,
    pub source_revision: Option<String>,
}

impl SceneMaterialSourceRef {
    pub fn new(asset_id: AssetId, source_id: AssetSourceId) -> Self {
        Self {
            asset_id,
            source_id,
            source_revision_id: None,
            source_revision: None,
        }
    }

    pub fn with_source_revision_id(mut self, revision_id: AssetSourceRevisionId) -> Self {
        self.source_revision_id = Some(revision_id);
        self
    }

    pub fn with_source_revision(mut self, revision: impl Into<String>) -> Self {
        self.source_revision = Some(revision.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialSlot {
    pub slot_id: SceneMaterialSlotId,
    pub palette_entry_id: SceneMaterialPaletteEntryId,
    pub display_name: String,
    pub source_ref: Option<SceneMaterialSourceRef>,
    pub material_asset_id: Option<AssetId>,
    pub is_default: bool,
}

impl SceneMaterialSlot {
    pub fn new(slot_id: SceneMaterialSlotId, display_name: impl Into<String>) -> Self {
        Self {
            slot_id,
            palette_entry_id: SceneMaterialPaletteEntryId(slot_id.raw()),
            display_name: display_name.into(),
            source_ref: None,
            material_asset_id: None,
            is_default: false,
        }
    }

    pub fn default_generated() -> Self {
        Self::new(DEFAULT_SCENE_MATERIAL_SLOT_ID, "Default Material").with_default(true)
    }

    pub fn with_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }

    pub fn with_palette_entry_id(mut self, entry_id: SceneMaterialPaletteEntryId) -> Self {
        self.palette_entry_id = entry_id;
        self
    }

    pub fn with_source_ref(mut self, source_ref: SceneMaterialSourceRef) -> Self {
        self.source_ref = Some(source_ref);
        self
    }

    pub fn with_material_asset(mut self, asset_id: AssetId) -> Self {
        self.material_asset_id = Some(asset_id);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialPalette {
    pub slots: Vec<SceneMaterialSlot>,
}

impl Default for SceneMaterialPalette {
    fn default() -> Self {
        Self {
            slots: vec![SceneMaterialSlot::default_generated()],
        }
    }
}

impl SceneMaterialPalette {
    pub fn new(slots: impl IntoIterator<Item = SceneMaterialSlot>) -> Result<Self, String> {
        let slots = slots.into_iter().collect::<Vec<_>>();
        if slots.is_empty() {
            return Err("scene material palette requires a default material slot".to_string());
        }
        if slots.len() > MAX_PORTABLE_SCENE_MATERIAL_SLOTS {
            return Err(format!(
                "scene material palette has {} slots, portable limit is {}",
                slots.len(),
                MAX_PORTABLE_SCENE_MATERIAL_SLOTS
            ));
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut entry_ids = std::collections::BTreeSet::new();
        let mut default_count = 0;
        for slot in &slots {
            if slot.slot_id.raw() == 0 {
                return Err("scene material slot id must be nonzero".to_string());
            }
            if !ids.insert(slot.slot_id) {
                return Err(format!(
                    "duplicate scene material slot {}",
                    slot.slot_id.raw()
                ));
            }
            if slot.palette_entry_id.raw() == 0 {
                return Err("scene material palette entry id must be nonzero".to_string());
            }
            if !entry_ids.insert(slot.palette_entry_id) {
                return Err(format!(
                    "duplicate scene material palette entry {}",
                    slot.palette_entry_id.raw()
                ));
            }
            if slot.is_default {
                default_count += 1;
            }
        }
        if default_count != 1 {
            return Err("scene material palette requires exactly one default slot".to_string());
        }
        Ok(Self { slots })
    }

    pub fn default_slot(&self) -> &SceneMaterialSlot {
        self.slots
            .iter()
            .find(|slot| slot.is_default)
            .expect("palette constructor enforces a default slot")
    }

    pub fn contains_slot(&self, slot_id: SceneMaterialSlotId) -> bool {
        self.slots.iter().any(|slot| slot.slot_id == slot_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveMaterialSlotAssignment {
    pub entity_id: EntityId,
    pub slot_id: SceneMaterialSlotId,
}

impl PrimitiveMaterialSlotAssignment {
    pub const fn new(entity_id: EntityId, slot_id: SceneMaterialSlotId) -> Self {
        Self { entity_id, slot_id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SdfPrimitiveMaterialSlotAssignment {
    pub primitive: SdfPrimitiveSourceId,
    pub slot_id: SceneMaterialSlotId,
}

impl SdfPrimitiveMaterialSlotAssignment {
    pub const fn new(primitive: SdfPrimitiveSourceId, slot_id: SceneMaterialSlotId) -> Self {
        Self { primitive, slot_id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneModelMeshMaterialSlotAssignment {
    pub material_region: SceneModelMeshMaterialRegionSourceId,
    pub slot_id: SceneMaterialSlotId,
}

impl SceneModelMeshMaterialSlotAssignment {
    pub fn new(
        material_region: SceneModelMeshMaterialRegionSourceId,
        slot_id: SceneMaterialSlotId,
    ) -> Self {
        Self {
            material_region,
            slot_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneMaterialBindingDiagnosticCode {
    MissingAssignedSlot,
    MissingMaterialSource,
    InvalidMaterialProduct,
    PreparationFailedPreservedPriorValid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneMaterialBindingDiagnosticSubject {
    SdfPrimitive(SdfPrimitiveSourceId),
    ModelMeshMaterialRegion(SceneModelMeshMaterialRegionSourceId),
    MaterialSlot(SceneMaterialSlotId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialBindingDiagnostic {
    pub code: SceneMaterialBindingDiagnosticCode,
    pub subject: SceneMaterialBindingDiagnosticSubject,
    pub message: String,
}

impl SceneMaterialBindingDiagnostic {
    pub fn new(
        code: SceneMaterialBindingDiagnosticCode,
        subject: SceneMaterialBindingDiagnosticSubject,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            subject,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneMaterialResolution {
    pub requested_slot_id: SceneMaterialSlotId,
    pub resolved_slot_id: SceneMaterialSlotId,
    pub material_table_index: u32,
    pub used_default_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialAssignmentState {
    pub palette: SceneMaterialPalette,
    sdf_primitive_slots: BTreeMap<SdfPrimitiveSourceId, SceneMaterialSlotId>,
    model_mesh_region_slots: BTreeMap<SceneModelMeshMaterialRegionSourceId, SceneMaterialSlotId>,
    source_revision: u64,
}

impl Default for SceneMaterialAssignmentState {
    fn default() -> Self {
        Self {
            palette: SceneMaterialPalette::default(),
            sdf_primitive_slots: BTreeMap::new(),
            model_mesh_region_slots: BTreeMap::new(),
            source_revision: 1,
        }
    }
}

impl SceneMaterialAssignmentState {
    pub fn new(
        palette: SceneMaterialPalette,
        assignments: impl IntoIterator<Item = SdfPrimitiveMaterialSlotAssignment>,
    ) -> Result<Self, String> {
        let mut state = Self {
            palette,
            sdf_primitive_slots: BTreeMap::new(),
            model_mesh_region_slots: BTreeMap::new(),
            source_revision: 1,
        };
        for assignment in assignments {
            state.assign_sdf_primitive_material_slot(assignment.primitive, assignment.slot_id)?;
        }
        Ok(state)
    }

    pub fn new_with_model_mesh_assignments(
        palette: SceneMaterialPalette,
        sdf_assignments: impl IntoIterator<Item = SdfPrimitiveMaterialSlotAssignment>,
        model_mesh_assignments: impl IntoIterator<Item = SceneModelMeshMaterialSlotAssignment>,
    ) -> Result<Self, String> {
        let mut state = Self::new(palette, sdf_assignments)?;
        for assignment in model_mesh_assignments {
            state
                .assign_model_mesh_material_slot(assignment.material_region, assignment.slot_id)?;
        }
        Ok(state)
    }

    pub fn palette(&self) -> &SceneMaterialPalette {
        &self.palette
    }

    pub fn source_revision(&self) -> u64 {
        self.source_revision
    }

    pub fn with_source_revision(mut self, source_revision: u64) -> Self {
        self.source_revision = source_revision.max(1);
        self
    }

    pub fn assignments(&self) -> impl Iterator<Item = SdfPrimitiveMaterialSlotAssignment> + '_ {
        self.sdf_primitive_slots.iter().map(|(primitive, slot_id)| {
            SdfPrimitiveMaterialSlotAssignment::new(*primitive, *slot_id)
        })
    }

    pub fn model_mesh_assignments(
        &self,
    ) -> impl Iterator<Item = SceneModelMeshMaterialSlotAssignment> + '_ {
        self.model_mesh_region_slots
            .iter()
            .map(|(material_region, slot_id)| {
                SceneModelMeshMaterialSlotAssignment::new(material_region.clone(), *slot_id)
            })
    }

    pub fn assign_sdf_primitive_material_slot(
        &mut self,
        primitive: SdfPrimitiveSourceId,
        slot_id: SceneMaterialSlotId,
    ) -> Result<(), String> {
        if !self.palette.contains_slot(slot_id) {
            return Err(format!(
                "scene material assignment references unknown slot {}",
                slot_id.raw()
            ));
        }
        if self.sdf_primitive_slots.get(&primitive).copied() == Some(slot_id) {
            return Ok(());
        }
        self.sdf_primitive_slots.insert(primitive, slot_id);
        self.bump_revision();
        Ok(())
    }

    pub fn assign_model_mesh_material_slot(
        &mut self,
        material_region: SceneModelMeshMaterialRegionSourceId,
        slot_id: SceneMaterialSlotId,
    ) -> Result<(), String> {
        if !self.palette.contains_slot(slot_id) {
            return Err(format!(
                "scene model/mesh material assignment references unknown slot {}",
                slot_id.raw()
            ));
        }
        if self.model_mesh_region_slots.get(&material_region).copied() == Some(slot_id) {
            return Ok(());
        }
        self.model_mesh_region_slots
            .insert(material_region, slot_id);
        self.bump_revision();
        Ok(())
    }

    pub fn resolve_material_slot_for_sdf_primitive(
        &self,
        primitive: SdfPrimitiveSourceId,
    ) -> SceneMaterialResolution {
        self.resolve_material_binding_for_sdf_scene_packet(primitive)
            .0
    }

    pub fn resolve_material_slot_for_model_mesh_region(
        &self,
        material_region: &SceneModelMeshMaterialRegionSourceId,
    ) -> SceneMaterialResolution {
        self.resolve_material_binding_for_model_mesh_region(material_region)
            .0
    }

    pub fn resolve_material_binding_for_sdf_scene_packet(
        &self,
        primitive: SdfPrimitiveSourceId,
    ) -> (SceneMaterialResolution, Vec<SceneMaterialBindingDiagnostic>) {
        let requested_slot_id = self
            .sdf_primitive_slots
            .get(&primitive)
            .copied()
            .unwrap_or_else(|| self.palette.default_slot().slot_id);
        let default_slot_id = self.palette.default_slot().slot_id;
        let (resolved_slot_id, used_default_fallback, mut diagnostics) = if self
            .palette
            .contains_slot(requested_slot_id)
        {
            (requested_slot_id, false, Vec::new())
        } else {
            (
                default_slot_id,
                true,
                vec![SceneMaterialBindingDiagnostic::new(
                    SceneMaterialBindingDiagnosticCode::MissingAssignedSlot,
                    SceneMaterialBindingDiagnosticSubject::SdfPrimitive(primitive),
                    format!(
                        "SDF primitive {} references missing material slot {}; using default slot {}",
                        primitive.entity_id().0,
                        requested_slot_id.raw(),
                        default_slot_id.raw()
                    ),
                )],
            )
        };
        let material_table_index = self.material_table_index_for_slot(resolved_slot_id);
        let Some(slot) = self
            .palette
            .slots
            .iter()
            .find(|slot| slot.slot_id == resolved_slot_id)
        else {
            return (
                SceneMaterialResolution {
                    requested_slot_id,
                    resolved_slot_id: default_slot_id,
                    material_table_index: 0,
                    used_default_fallback: true,
                },
                diagnostics,
            );
        };
        if slot.source_ref.is_none() && slot.material_asset_id.is_none() && !slot.is_default {
            diagnostics.push(SceneMaterialBindingDiagnostic::new(
                SceneMaterialBindingDiagnosticCode::MissingMaterialSource,
                SceneMaterialBindingDiagnosticSubject::MaterialSlot(slot.slot_id),
                format!(
                    "scene material slot {} has no stable material source reference",
                    slot.slot_id.raw()
                ),
            ));
        }
        (
            SceneMaterialResolution {
                requested_slot_id,
                resolved_slot_id,
                material_table_index,
                used_default_fallback,
            },
            diagnostics,
        )
    }

    pub fn resolve_material_binding_for_model_mesh_region(
        &self,
        material_region: &SceneModelMeshMaterialRegionSourceId,
    ) -> (SceneMaterialResolution, Vec<SceneMaterialBindingDiagnostic>) {
        let requested_slot_id = self
            .model_mesh_region_slots
            .get(material_region)
            .copied()
            .unwrap_or_else(|| self.palette.default_slot().slot_id);
        let default_slot_id = self.palette.default_slot().slot_id;
        let (resolved_slot_id, used_default_fallback, mut diagnostics) = if self
            .palette
            .contains_slot(requested_slot_id)
        {
            (requested_slot_id, false, Vec::new())
        } else {
            (
                default_slot_id,
                true,
                vec![SceneMaterialBindingDiagnostic::new(
                    SceneMaterialBindingDiagnosticCode::MissingAssignedSlot,
                    SceneMaterialBindingDiagnosticSubject::ModelMeshMaterialRegion(
                        material_region.clone(),
                    ),
                    format!(
                        "model/mesh material region {}:{}:{} references missing material slot {}; using default slot {}",
                        material_region.model_mesh_source_id.asset_id.raw(),
                        material_region.model_mesh_source_id.source_id.raw(),
                        material_region.material_region_id.key(),
                        requested_slot_id.raw(),
                        default_slot_id.raw()
                    ),
                )],
            )
        };
        let material_table_index = self.material_table_index_for_slot(resolved_slot_id);
        let Some(slot) = self
            .palette
            .slots
            .iter()
            .find(|slot| slot.slot_id == resolved_slot_id)
        else {
            return (
                SceneMaterialResolution {
                    requested_slot_id,
                    resolved_slot_id: default_slot_id,
                    material_table_index: 0,
                    used_default_fallback: true,
                },
                diagnostics,
            );
        };
        if slot.source_ref.is_none() && slot.material_asset_id.is_none() && !slot.is_default {
            diagnostics.push(SceneMaterialBindingDiagnostic::new(
                SceneMaterialBindingDiagnosticCode::MissingMaterialSource,
                SceneMaterialBindingDiagnosticSubject::MaterialSlot(slot.slot_id),
                format!(
                    "scene material slot {} has no stable material source reference",
                    slot.slot_id.raw()
                ),
            ));
        }
        (
            SceneMaterialResolution {
                requested_slot_id,
                resolved_slot_id,
                material_table_index,
                used_default_fallback,
            },
            diagnostics,
        )
    }

    pub fn material_table_index_for_slot(&self, slot_id: SceneMaterialSlotId) -> u32 {
        self.palette
            .slots
            .iter()
            .position(|slot| slot.slot_id == slot_id)
            .or_else(|| self.palette.slots.iter().position(|slot| slot.is_default))
            .unwrap_or(0) as u32
    }

    pub fn material_table_identity(&self) -> String {
        let mut identity = format!("scene-material-table:v1:revision={}", self.source_revision);
        for (index, slot) in self.palette.slots.iter().enumerate() {
            identity.push_str(&format!(
                "|slot={index}:slot_id={}:entry={}:default={}:display={}",
                slot.slot_id.raw(),
                slot.palette_entry_id.raw(),
                slot.is_default,
                slot.display_name
            ));
            if let Some(source_ref) = &slot.source_ref {
                identity.push_str(&format!(
                    ":source_asset={}:source_id={}:source_revision_id={}:source_revision={}",
                    source_ref.asset_id.raw(),
                    source_ref.source_id.raw(),
                    source_ref
                        .source_revision_id
                        .map(|revision| revision.raw().to_string())
                        .unwrap_or_default(),
                    source_ref.source_revision.as_deref().unwrap_or_default()
                ));
            }
            if let Some(material_asset_id) = slot.material_asset_id {
                identity.push_str(&format!(":material_asset={}", material_asset_id.raw()));
            }
        }
        for assignment in self.assignments() {
            identity.push_str(&format!(
                "|sdf_primitive={}:slot={}",
                assignment.primitive.entity_id().0,
                assignment.slot_id.raw()
            ));
        }
        for assignment in self.model_mesh_assignments() {
            let source = &assignment.material_region.model_mesh_source_id;
            identity.push_str(&format!(
                "|model_mesh=asset:{}:source:{}:source_revision_id={}:source_revision={}:region={}:slot={}",
                source.asset_id.raw(),
                source.source_id.raw(),
                source
                    .source_revision_id
                    .map(|revision| revision.raw().to_string())
                    .unwrap_or_default(),
                source.source_revision.as_deref().unwrap_or_default(),
                assignment.material_region.material_region_id.key(),
                assignment.slot_id.raw()
            ));
        }
        identity
    }

    fn bump_revision(&mut self) {
        self.source_revision = self.source_revision.saturating_add(1).max(1);
    }
}

fn is_transient_mesh_material_region_key(region_key: &str) -> bool {
    let normalized = region_key.to_ascii_lowercase();
    [
        "entity:",
        "ecs:",
        "renderable:",
        "renderable_index:",
        "renderer:",
        "draw:",
        "residency:",
        "palette:",
        "display:",
        "ui:",
        "artifact:",
        "generated:",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{asset_id, asset_source_id, asset_source_revision_id};

    #[test]
    fn palette_requires_exactly_one_default_slot() {
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Rock")
                .with_material_asset(asset_id(7)),
        ])
        .expect("valid palette");

        assert_eq!(
            palette.default_slot().slot_id,
            DEFAULT_SCENE_MATERIAL_SLOT_ID
        );
        assert!(palette.contains_slot(SceneMaterialSlotId::new(2)));
    }

    #[test]
    fn palette_rejects_duplicate_slot_ids() {
        let error = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(DEFAULT_SCENE_MATERIAL_SLOT_ID, "Duplicate"),
        ])
        .expect_err("duplicate slot id should fail");

        assert!(error.contains("duplicate"));
    }

    #[test]
    fn palette_rejects_duplicate_palette_entry_ids() {
        let error = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Duplicate entry")
                .with_palette_entry_id(SceneMaterialPaletteEntryId::new(1)),
        ])
        .expect_err("duplicate palette entry id should fail");

        assert!(error.contains("palette entry"));
    }

    #[test]
    fn sdf_assignment_state_resolves_explicit_and_default_slots() {
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Rock")
                .with_material_asset(asset_id(7)),
        ])
        .expect("valid palette");
        let mut assignments = SceneMaterialAssignmentState::new(palette, []).expect("state");
        let assigned = SdfPrimitiveSourceId::new(EntityId(10));
        let unassigned = SdfPrimitiveSourceId::new(EntityId(11));

        assignments
            .assign_sdf_primitive_material_slot(assigned, SceneMaterialSlotId::new(2))
            .expect("assign material");

        let assigned_resolution = assignments.resolve_material_slot_for_sdf_primitive(assigned);
        let default_resolution = assignments.resolve_material_slot_for_sdf_primitive(unassigned);
        assert_eq!(
            assigned_resolution.resolved_slot_id,
            SceneMaterialSlotId::new(2)
        );
        assert_eq!(assigned_resolution.material_table_index, 1);
        assert_eq!(
            default_resolution.resolved_slot_id,
            DEFAULT_SCENE_MATERIAL_SLOT_ID
        );
        assert_eq!(default_resolution.material_table_index, 0);
    }

    #[test]
    fn sdf_assignment_state_revision_changes_only_when_assignment_changes() {
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Rock")
                .with_material_asset(asset_id(7)),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(3), "Moss")
                .with_material_asset(asset_id(8)),
        ])
        .expect("valid palette");
        let mut assignments = SceneMaterialAssignmentState::new(palette, []).expect("state");
        let primitive = SdfPrimitiveSourceId::new(EntityId(10));
        let initial_revision = assignments.source_revision();

        assignments
            .assign_sdf_primitive_material_slot(primitive, SceneMaterialSlotId::new(2))
            .expect("assign material");
        let assigned_revision = assignments.source_revision();
        assignments
            .assign_sdf_primitive_material_slot(primitive, SceneMaterialSlotId::new(2))
            .expect("same assignment should be a no-op");
        assert_eq!(assignments.source_revision(), assigned_revision);

        assignments
            .assign_sdf_primitive_material_slot(primitive, SceneMaterialSlotId::new(3))
            .expect("changed assignment should update revision");
        assert!(assigned_revision > initial_revision);
        assert!(assignments.source_revision() > assigned_revision);
    }

    #[test]
    fn model_mesh_material_assignment_survives_save_reload() {
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Imported body")
                .with_material_asset(asset_id(7)),
        ])
        .expect("valid palette");
        let material_region = model_mesh_region("body");
        let mut saved_state =
            SceneMaterialAssignmentState::new_with_model_mesh_assignments(palette, [], [])
                .expect("state");

        saved_state
            .assign_model_mesh_material_slot(material_region.clone(), SceneMaterialSlotId::new(2))
            .expect("assign material");

        let saved_palette = saved_state.palette.clone();
        let saved_model_assignments = saved_state.model_mesh_assignments().collect::<Vec<_>>();
        let reloaded_state = SceneMaterialAssignmentState::new_with_model_mesh_assignments(
            saved_palette,
            [],
            saved_model_assignments,
        )
        .expect("reload");

        let resolution =
            reloaded_state.resolve_material_slot_for_model_mesh_region(&material_region);
        assert_eq!(resolution.resolved_slot_id, SceneMaterialSlotId::new(2));
        assert_eq!(resolution.material_table_index, 1);
    }

    #[test]
    fn model_mesh_material_table_identity_changes_when_assignment_changes() {
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Body red")
                .with_material_asset(asset_id(7)),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(3), "Body blue")
                .with_material_asset(asset_id(8)),
        ])
        .expect("valid palette");
        let material_region = model_mesh_region("body");
        let mut state =
            SceneMaterialAssignmentState::new_with_model_mesh_assignments(palette, [], [])
                .expect("state");

        state
            .assign_model_mesh_material_slot(material_region.clone(), SceneMaterialSlotId::new(2))
            .expect("assign material");
        let red_identity = state.material_table_identity();
        state
            .assign_model_mesh_material_slot(material_region.clone(), SceneMaterialSlotId::new(3))
            .expect("reassign material");
        let blue_identity = state.material_table_identity();

        assert_ne!(red_identity, blue_identity);
        assert!(blue_identity.contains("model_mesh=asset:42:source:84"));
        assert!(blue_identity.contains("region=body"));
        assert!(blue_identity.contains("slot=3"));
    }

    #[test]
    fn model_mesh_material_region_rejects_transient_identity() {
        let error = SceneMeshMaterialRegionId::new("renderable_index:7")
            .expect_err("transient renderer identity should fail");

        assert!(error.contains("transient runtime identity"));
    }

    #[test]
    fn scene_material_slot_does_not_store_generated_artifact_truth() {
        let slot = SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Rock")
            .with_material_asset(asset_id(7));
        let state = SceneMaterialAssignmentState::new(
            SceneMaterialPalette::new([SceneMaterialSlot::default_generated(), slot])
                .expect("valid palette"),
            [],
        )
        .expect("state");

        let identity = state.material_table_identity();
        for forbidden in [
            "formed_artifact",
            "shader_artifact",
            "material_cache",
            "shader_cache",
            "prior_valid",
        ] {
            assert!(
                !identity.contains(forbidden),
                "authored scene material identity must not contain derived {forbidden} truth"
            );
        }
    }

    #[test]
    fn sdf_assignment_state_reports_broken_slot_and_uses_default() {
        let mut state = SceneMaterialAssignmentState::default();
        let primitive = SdfPrimitiveSourceId::new(EntityId(12));
        state
            .sdf_primitive_slots
            .insert(primitive, SceneMaterialSlotId::new(99));

        let (resolution, diagnostics) =
            state.resolve_material_binding_for_sdf_scene_packet(primitive);

        assert_eq!(resolution.requested_slot_id, SceneMaterialSlotId::new(99));
        assert_eq!(resolution.resolved_slot_id, DEFAULT_SCENE_MATERIAL_SLOT_ID);
        assert!(resolution.used_default_fallback);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SceneMaterialBindingDiagnosticCode::MissingAssignedSlot
        }));
    }

    #[test]
    fn sdf_primitive_material_assignment_survives_save_reload() {
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(SceneMaterialSlotId::new(2), "Rock")
                .with_material_asset(asset_id(7)),
        ])
        .expect("valid palette");
        let mut saved_state = SceneMaterialAssignmentState::new(palette, []).expect("state");
        let primitive = SdfPrimitiveSourceId::new(EntityId(77));
        saved_state
            .assign_sdf_primitive_material_slot(primitive, SceneMaterialSlotId::new(2))
            .expect("assign material");

        let saved_palette = saved_state.palette.clone();
        let saved_assignments = saved_state.assignments().collect::<Vec<_>>();
        let reloaded_state =
            SceneMaterialAssignmentState::new(saved_palette, saved_assignments).expect("reload");

        let resolution = reloaded_state.resolve_material_slot_for_sdf_primitive(primitive);
        assert_eq!(resolution.resolved_slot_id, SceneMaterialSlotId::new(2));
        assert_eq!(resolution.material_table_index, 1);
    }

    fn model_mesh_region(region_key: &str) -> SceneModelMeshMaterialRegionSourceId {
        SceneModelMeshMaterialRegionSourceId::new(
            SceneModelMeshSourceId::new(asset_id(42), asset_source_id(84))
                .with_source_revision_id(asset_source_revision_id(2)),
            SceneMeshMaterialRegionId::new(region_key).expect("stable region key"),
        )
    }
}
