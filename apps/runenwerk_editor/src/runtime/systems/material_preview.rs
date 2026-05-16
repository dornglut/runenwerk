//! File: apps/runenwerk_editor/src/runtime/systems/material_preview.rs
//! Purpose: Material Lab prepared renderer handoff.

use engine::plugins::render::PreparedMaterialFeatureResource;
use engine::runtime::{Res, ResMut};

use crate::material_lab::prepared_material_resource_for_preview;
use crate::runtime::resources::EditorHostResource;

pub fn prepare_material_preview_render_resource_system(
    host: Res<EditorHostResource>,
    mut material_feature: ResMut<PreparedMaterialFeatureResource>,
) {
    *material_feature =
        prepared_material_resource_for_preview(host.app.material_lab_runtime().active_preview());
}
