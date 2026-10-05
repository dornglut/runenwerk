//! CPU-side viewport scene/material packet contracts.

use editor_core::EntityId;
use editor_scene::{
    SceneMaterialAssignmentState, SceneMaterialBindingDiagnostic,
    SceneMaterialBindingDiagnosticCode, SceneMaterialBindingDiagnosticSubject,
    SceneMaterialResolution, SceneMaterialSlotId, SceneModelMeshMaterialRegionSourceId,
    SceneModelMeshSourceId,
};
use engine::plugins::render::{
    GpuUniform, PreparedModelMeshMaterialRegionIdentity, PreparedModelMeshMaterialSelection,
    PreparedModelMeshMaterialSourceIdentity,
};
use scene::{LocalTransform, Vec3Value};

use crate::editor_runtime::{EditorPrimitive, EditorPrimitiveKind};

pub const EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES: usize = 64;
pub const EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS: usize = 16;

#[derive(Debug, Clone, Copy, GpuUniform)]
pub struct EditorViewportSceneProductUniform {
    pub surface: [f32; 4],
    pub viewport: [f32; 4],
    pub camera_position: [f32; 4],
    pub camera_forward: [f32; 4],
    pub camera_right: [f32; 4],
    pub camera_up: [f32; 4],
    pub object_transform: [f32; 4],
    pub primitive_params_a: [f32; 4],
    pub primitive_params_b: [f32; 4],
    pub primitive_flags: [u32; 4],
    pub primitive_slot_transforms: [[f32; 4]; EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES],
    pub primitive_slot_params_a: [[f32; 4]; EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES],
    pub primitive_slot_params_b: [[f32; 4]; EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES],
    pub primitive_slot_flags: [[u32; 4]; EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES],
    pub model_mesh_flags: [u32; 4],
    pub model_mesh_region_rects: [[f32; 4]; EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS],
    pub model_mesh_region_flags: [[u32; 4]; EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorViewportPrimitiveInstance {
    pub entity_id: EntityId,
    pub pick_slot: u32,
    pub translation: Vec3Value,
    pub scale: Vec3Value,
    pub primitive_kind: EditorPrimitiveKind,
    pub box_half_extents: Vec3Value,
    pub sphere_radius: f32,
    pub capsule_radius: f32,
    pub capsule_half_height: f32,
    pub material_slot_index: u32,
    pub selected: bool,
    pub hovered: bool,
}

impl EditorViewportPrimitiveInstance {
    pub fn from_transform_and_primitive(
        entity_id: EntityId,
        transform: LocalTransform,
        primitive: EditorPrimitive,
        selected: bool,
        hovered: bool,
    ) -> Self {
        let safe_scale = Vec3Value::new(
            transform.scale.x.abs().max(0.0001),
            transform.scale.y.abs().max(0.0001),
            transform.scale.z.abs().max(0.0001),
        );
        let radial_scale = safe_scale.x.max(safe_scale.z);
        let sphere_scale = safe_scale.x.max(safe_scale.y).max(safe_scale.z);
        Self {
            entity_id,
            pick_slot: 0,
            translation: transform.translation,
            scale: safe_scale,
            primitive_kind: primitive.kind(),
            box_half_extents: Vec3Value::new(
                primitive.box_half_extents.x.max(0.05) * safe_scale.x,
                primitive.box_half_extents.y.max(0.05) * safe_scale.y,
                primitive.box_half_extents.z.max(0.05) * safe_scale.z,
            ),
            sphere_radius: primitive.sphere_radius.max(0.05) * sphere_scale,
            capsule_radius: primitive.capsule_radius.max(0.05) * radial_scale,
            capsule_half_height: primitive.capsule_half_height.max(0.05) * safe_scale.y,
            material_slot_index: 0,
            selected,
            hovered,
        }
    }

    pub fn with_material_slot_index(mut self, material_slot_index: u32) -> Self {
        self.material_slot_index = material_slot_index;
        self
    }

    pub fn shader_slot_transform(self) -> [f32; 4] {
        [
            self.translation.x,
            self.translation.y,
            self.translation.z,
            0.0,
        ]
    }

    pub fn shader_slot_params_a(self) -> [f32; 4] {
        [
            self.box_half_extents.x.max(0.05),
            self.box_half_extents.y.max(0.05),
            self.box_half_extents.z.max(0.05),
            self.sphere_radius.max(0.05),
        ]
    }

    pub fn shader_slot_params_b(self) -> [f32; 4] {
        [
            self.capsule_radius.max(0.05),
            self.capsule_half_height.max(0.05),
            0.0,
            0.0,
        ]
    }

    pub fn shader_slot_flags(self) -> [u32; 4] {
        [
            self.primitive_kind.as_u32(),
            self.pick_slot,
            u32::from(self.selected),
            self.material_slot_index,
        ]
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EditorViewportSceneRenderPacket {
    primitives: Vec<EditorViewportPrimitiveInstance>,
    omitted_primitive_count: usize,
}

impl EditorViewportSceneRenderPacket {
    pub fn from_primitives(
        primitives: impl IntoIterator<Item = EditorViewportPrimitiveInstance>,
    ) -> Self {
        let mut primitives = primitives.into_iter().collect::<Vec<_>>();
        primitives.sort_by_key(|primitive| primitive.entity_id.0);
        let omitted_primitive_count = primitives
            .len()
            .saturating_sub(EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES);
        primitives.truncate(EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES);
        for (index, primitive) in primitives.iter_mut().enumerate() {
            primitive.pick_slot = (index + 1) as u32;
        }
        Self {
            primitives,
            omitted_primitive_count,
        }
    }

    pub fn primitives(&self) -> &[EditorViewportPrimitiveInstance] {
        &self.primitives
    }

    pub fn len(&self) -> usize {
        self.primitives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    pub fn omitted_primitive_count(&self) -> usize {
        self.omitted_primitive_count
    }

    pub fn has_overflow(&self) -> bool {
        self.omitted_primitive_count != 0
    }

    pub fn entity_for_pick_slot(&self, pick_slot: u32) -> Option<EntityId> {
        if pick_slot == 0 {
            return None;
        }

        self.primitives
            .iter()
            .find(|primitive| primitive.pick_slot == pick_slot)
            .map(|primitive| primitive.entity_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorViewportModelMeshMaterialSelection {
    pub material_region: SceneModelMeshMaterialRegionSourceId,
    pub requested_slot_id: SceneMaterialSlotId,
    pub resolved_slot_id: SceneMaterialSlotId,
    pub material_table_index: u32,
    pub used_default_fallback: bool,
    prepared_selection: PreparedModelMeshMaterialSelection,
}

impl EditorViewportModelMeshMaterialSelection {
    pub fn new(
        material_region: SceneModelMeshMaterialRegionSourceId,
        resolution: SceneMaterialResolution,
    ) -> Result<Self, String> {
        let prepared_selection =
            prepared_model_mesh_material_selection(&material_region, resolution)?;
        Ok(Self {
            material_region,
            requested_slot_id: resolution.requested_slot_id,
            resolved_slot_id: resolution.resolved_slot_id,
            material_table_index: resolution.material_table_index,
            used_default_fallback: resolution.used_default_fallback,
            prepared_selection,
        })
    }

    pub fn prepared_selection(&self) -> &PreparedModelMeshMaterialSelection {
        &self.prepared_selection
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EditorViewportModelMeshMaterialSelectionPacket {
    selections: Vec<EditorViewportModelMeshMaterialSelection>,
    diagnostics: Vec<SceneMaterialBindingDiagnostic>,
}

impl EditorViewportModelMeshMaterialSelectionPacket {
    pub fn from_model_mesh_regions(
        assignments: &SceneMaterialAssignmentState,
        material_regions: impl IntoIterator<Item = SceneModelMeshMaterialRegionSourceId>,
    ) -> Self {
        let mut selections = Vec::new();
        let mut diagnostics = Vec::new();
        for material_region in material_regions {
            let (resolution, mut region_diagnostics) =
                assignments.resolve_material_binding_for_model_mesh_region(&material_region);
            diagnostics.append(&mut region_diagnostics);
            match EditorViewportModelMeshMaterialSelection::new(material_region.clone(), resolution)
            {
                Ok(selection) => selections.push(selection),
                Err(message) => diagnostics.push(model_mesh_material_selection_diagnostic(
                    material_region,
                    message,
                )),
            }
        }
        selections.sort_by(|left, right| {
            left.material_region
                .cmp(&right.material_region)
                .then_with(|| left.material_table_index.cmp(&right.material_table_index))
        });
        Self {
            selections,
            diagnostics,
        }
    }

    pub fn from_blocking_diagnostics(
        diagnostics: impl IntoIterator<Item = SceneMaterialBindingDiagnostic>,
    ) -> Self {
        Self {
            selections: Vec::new(),
            diagnostics: diagnostics.into_iter().collect(),
        }
    }

    pub fn selections(&self) -> &[EditorViewportModelMeshMaterialSelection] {
        &self.selections
    }

    pub fn diagnostics(&self) -> &[SceneMaterialBindingDiagnostic] {
        &self.diagnostics
    }

    pub fn has_blocking_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub fn prepared_material_selections(&self) -> Vec<PreparedModelMeshMaterialSelection> {
        self.selections
            .iter()
            .map(|selection| selection.prepared_selection().clone())
            .collect()
    }
}

pub fn model_mesh_material_selection_diagnostic(
    material_region: SceneModelMeshMaterialRegionSourceId,
    message: impl Into<String>,
) -> SceneMaterialBindingDiagnostic {
    SceneMaterialBindingDiagnostic::new(
        SceneMaterialBindingDiagnosticCode::InvalidMaterialProduct,
        SceneMaterialBindingDiagnosticSubject::ModelMeshMaterialRegion(material_region),
        message,
    )
}

fn prepared_model_mesh_material_selection(
    material_region: &SceneModelMeshMaterialRegionSourceId,
    resolution: SceneMaterialResolution,
) -> Result<PreparedModelMeshMaterialSelection, String> {
    let source = prepared_model_mesh_source_identity(&material_region.model_mesh_source_id)?;
    let region = PreparedModelMeshMaterialRegionIdentity::new(
        source,
        material_region.material_region_id.key().to_string(),
    )
    .map_err(|error| error.to_string())?;
    PreparedModelMeshMaterialSelection::new(
        region,
        resolution.requested_slot_id.raw(),
        resolution.resolved_slot_id.raw(),
        resolution.material_table_index,
        resolution.used_default_fallback,
    )
    .map_err(|error| error.to_string())
}

fn prepared_model_mesh_source_identity(
    source: &SceneModelMeshSourceId,
) -> Result<PreparedModelMeshMaterialSourceIdentity, String> {
    let mut prepared =
        PreparedModelMeshMaterialSourceIdentity::new(source.asset_id.raw(), source.source_id.raw())
            .map_err(|error| error.to_string())?;
    if let Some(revision_id) = source.source_revision_id {
        prepared = prepared.with_source_revision_id(revision_id.raw());
    }
    if let Some(revision) = &source.source_revision {
        prepared = prepared.with_source_revision(revision.clone());
    }
    Ok(prepared)
}
