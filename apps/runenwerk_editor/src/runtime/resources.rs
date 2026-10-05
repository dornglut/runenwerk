#[cfg(test)]
use editor_core::EntityId;
#[cfg(test)]
use editor_scene::{
    SceneMaterialAssignmentState, SceneMaterialSlotId, SceneModelMeshMaterialRegionSourceId,
    SceneModelMeshSourceId,
};
#[cfg(test)]
use editor_shell::{EDITOR_DESIGN_WORKSPACE_PROFILE_ID, MATERIAL_WORKSPACE_PROFILE_ID};
#[cfg(test)]
use glam::Vec3;
#[cfg(test)]
use scene::{LocalTransform, Vec3Value};
#[cfg(test)]
use ui_theme::ThemeTokens;

#[cfg(test)]
use crate::editor_runtime::{EditorPrimitive, EditorPrimitiveKind};
use crate::runtime::preview_process::PreviewProcessManager;

mod host;
mod input;
mod viewport_packet;

pub use crate::runtime::viewport::{
    EditorViewportBranchTraceSnapshot, EditorViewportCamera, EditorViewportDebugStage,
    EditorViewportRenderState, editor_viewport_camera, editor_viewport_camera_fov_y_radians,
    editor_viewport_camera_from_settings,
};
pub use host::{EditorHostResource, effective_shell_scale, scaled_shell_theme};
pub use input::{
    EditorCameraPointerButton, EditorInputBridgeState, EditorPointerOwner,
    EditorTargetViewportInteractionState,
};
pub use viewport_packet::{
    EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS, EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES,
    EditorViewportModelMeshMaterialSelection, EditorViewportModelMeshMaterialSelectionPacket,
    EditorViewportPrimitiveInstance, EditorViewportSceneProductUniform,
    EditorViewportSceneRenderPacket, model_mesh_material_selection_diagnostic,
};

#[derive(Default, runen_ecs::Resource)]
pub struct RuntimePreviewProcessResource {
    pub manager: PreviewProcessManager,
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{asset_id, asset_source_id, asset_source_revision_id};
    use editor_scene::{
        SceneMaterialPalette, SceneMaterialSlot, SceneMeshMaterialRegionId,
        SceneModelMeshMaterialSlotAssignment,
    };
    use ui_theme::UiColor;

    #[test]
    fn editor_host_resource_apply_theme_replaces_runtime_theme() {
        let mut host = EditorHostResource::default();
        let theme = ThemeTokens {
            accent: UiColor::new(0.2, 0.4, 1.0, 1.0),
            ..ThemeTokens::default()
        };

        host.apply_theme(theme.clone());

        assert_eq!(host.theme, theme);
    }

    #[test]
    fn material_lab_workbench_host_resource_bootstraps_material_profile() {
        let host = EditorHostResource::material_lab_workbench();

        assert_eq!(
            host.app.workbench_host().composition(),
            crate::shell::RunenwerkWorkbenchComposition::MaterialLab
        );
        assert_eq!(
            host.shell_state.active_workspace_profile_id(),
            MATERIAL_WORKSPACE_PROFILE_ID
        );
        assert_eq!(
            host.shell_state.open_workspace_profile_ids(),
            &[MATERIAL_WORKSPACE_PROFILE_ID]
        );
        assert!(matches!(
            host.app.runtime().session().active_document_descriptor(),
            Some(editor_core::DocumentDescriptor {
                kind: editor_core::DocumentKind::MaterialGraph,
                ..
            })
        ));
    }

    #[test]
    fn ui_designer_workbench_host_resource_bootstraps_editor_design_profile() {
        let host = EditorHostResource::ui_designer_workbench();

        assert_eq!(
            host.app.workbench_host().composition(),
            crate::shell::RunenwerkWorkbenchComposition::UiDesigner
        );
        assert_eq!(
            host.shell_state.active_workspace_profile_id(),
            EDITOR_DESIGN_WORKSPACE_PROFILE_ID
        );
        assert_eq!(
            host.shell_state.open_workspace_profile_ids(),
            &[EDITOR_DESIGN_WORKSPACE_PROFILE_ID]
        );
    }

    #[test]
    fn scene_product_uniform_uses_target_local_viewport_for_camera_aspect() {
        let mut state = EditorViewportRenderState::default();
        state.set_viewport_bounds((40.0, 50.0, 320.0, 180.0));

        let uniform = state.compose_scene_product_uniform((1280, 720));

        assert_eq!(uniform.viewport, [0.0, 0.0, 1280.0, 720.0]);
        assert_eq!(state.viewport_bounds_px, (40.0, 50.0, 320.0, 180.0));
    }

    #[test]
    fn scene_product_uniform_uses_viewport_owned_camera_state() {
        let state = EditorViewportRenderState {
            camera_fov_y_radians: 42.0_f32.to_radians(),
            ..Default::default()
        };

        let uniform = state.compose_scene_product_uniform((1280, 720));

        assert_eq!(uniform.viewport, [0.0, 0.0, 1280.0, 720.0]);
        assert_eq!(uniform.camera_position[3], 42.0_f32.to_radians());
    }

    #[test]
    fn scene_packet_sorts_instances_and_serializes_uniform_slots() {
        let mut sphere = EditorPrimitive::default();
        sphere.set_kind(EditorPrimitiveKind::Sphere);
        sphere.sphere_radius = 1.25;
        let box_primitive = EditorPrimitive {
            box_half_extents: Vec3Value::new(0.25, 0.5, 0.75),
            ..Default::default()
        };

        let packet = EditorViewportSceneRenderPacket::from_primitives([
            EditorViewportPrimitiveInstance::from_transform_and_primitive(
                editor_core::EntityId(20),
                LocalTransform::from_translation(Vec3Value::new(2.0, 0.0, 0.0)),
                sphere,
                false,
                true,
            ),
            EditorViewportPrimitiveInstance::from_transform_and_primitive(
                editor_core::EntityId(4),
                LocalTransform::from_translation(Vec3Value::new(-1.0, 0.0, 0.0)),
                box_primitive,
                true,
                false,
            ),
        ]);

        let mut state = EditorViewportRenderState::default();
        state.set_scene_packet(packet);
        let uniform = state.compose_scene_product_uniform((1280, 720));

        assert_eq!(uniform.primitive_flags[1], 2);
        assert_eq!(uniform.primitive_slot_flags[0], [0, 1, 1, 0]);
        assert_eq!(uniform.primitive_slot_flags[1], [1, 2, 0, 0]);
        assert_eq!(uniform.primitive_slot_transforms[0], [-1.0, 0.0, 0.0, 0.0]);
        assert_eq!(uniform.primitive_slot_params_a[0], [0.25, 0.5, 0.75, 0.6]);
        assert_eq!(uniform.primitive_slot_params_a[1][3], 1.25);
        assert_eq!(
            state.scene_packet.entity_for_pick_slot(1),
            Some(EntityId(4))
        );
        assert_eq!(
            state.scene_packet.entity_for_pick_slot(2),
            Some(EntityId(20))
        );
    }

    #[test]
    fn viewport_scene_packet_reports_uniform_slot_overflow() {
        let primitives = (0..EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES + 2).map(|index| {
            EditorViewportPrimitiveInstance::from_transform_and_primitive(
                editor_core::EntityId(index as u64),
                LocalTransform::from_translation(Vec3Value::new(index as f32, 0.0, 0.0)),
                EditorPrimitive::default(),
                false,
                false,
            )
        });

        let packet = EditorViewportSceneRenderPacket::from_primitives(primitives);
        let mut state = EditorViewportRenderState::default();
        state.set_scene_packet(packet);
        let uniform = state.compose_scene_product_uniform((1280, 720));

        assert_eq!(
            state.scene_packet.len(),
            EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES
        );
        assert!(state.scene_packet.has_overflow());
        assert_eq!(state.scene_packet.omitted_primitive_count(), 2);
        assert_eq!(
            uniform.primitive_flags[1],
            EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES as u32
        );
        assert_eq!(
            uniform.primitive_slot_flags[EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES - 1][1],
            EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES as u32
        );
    }

    #[test]
    fn viewport_scene_packet_pick_slots_preserve_full_entity_identity() {
        let large_entity = editor_core::EntityId(u64::from(u32::MAX) + 99);
        let instance = EditorViewportPrimitiveInstance::from_transform_and_primitive(
            large_entity,
            LocalTransform::default(),
            EditorPrimitive::default(),
            false,
            false,
        );

        let packet = EditorViewportSceneRenderPacket::from_primitives([instance]);
        let mut state = EditorViewportRenderState::default();
        state.set_scene_packet(packet);
        let uniform = state.compose_scene_product_uniform((1280, 720));

        assert_eq!(uniform.primitive_slot_flags[0][1], 1);
        assert_eq!(
            state.scene_packet.entity_for_pick_slot(1),
            Some(large_entity)
        );
        assert_eq!(state.scene_packet.entity_for_pick_slot(0), None);
    }

    #[test]
    fn viewport_primitive_slot_helpers_define_the_shader_packet_contract() {
        let mut primitive = EditorPrimitive::default();
        primitive.set_kind(EditorPrimitiveKind::Cylinder);
        primitive.capsule_radius = 0.45;
        primitive.capsule_half_height = 1.25;
        let instance = EditorViewportPrimitiveInstance::from_transform_and_primitive(
            editor_core::EntityId(42),
            LocalTransform::from_translation(Vec3Value::new(1.0, 2.0, 3.0)),
            primitive,
            true,
            true,
        );
        let packet = EditorViewportSceneRenderPacket::from_primitives([instance]);
        let instance = packet.primitives()[0];

        assert_eq!(instance.shader_slot_transform(), [1.0, 2.0, 3.0, 0.0]);
        assert_eq!(instance.shader_slot_params_b(), [0.45, 1.25, 0.0, 0.0]);
        assert_eq!(instance.shader_slot_flags(), [3, 1, 1, 0]);
    }

    #[test]
    fn sdf_material_slot_packet_uses_typed_u32_lane() {
        let instance = EditorViewportPrimitiveInstance::from_transform_and_primitive(
            editor_core::EntityId(42),
            LocalTransform::default(),
            EditorPrimitive::default(),
            false,
            false,
        )
        .with_material_slot_index(17);

        assert_eq!(instance.shader_slot_params_b()[2], 0.0);
        assert_eq!(instance.shader_slot_flags()[3], 17);
    }

    #[test]
    fn model_mesh_material_selection_packet_serializes_scene_product_uniform_regions() {
        let assigned_slot = SceneMaterialSlotId::new(2);
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(assigned_slot, "Imported Body").with_material_asset(asset_id(7)),
        ])
        .expect("valid palette");
        let material_region = SceneModelMeshMaterialRegionSourceId::new(
            SceneModelMeshSourceId::new(asset_id(42), asset_source_id(84))
                .with_source_revision_id(asset_source_revision_id(2))
                .with_source_revision("sha256:source"),
            SceneMeshMaterialRegionId::new("source_material_slot:0")
                .expect("source material slot key should be stable"),
        );
        let assignments = SceneMaterialAssignmentState::new_with_model_mesh_assignments(
            palette,
            [],
            [SceneModelMeshMaterialSlotAssignment::new(
                material_region.clone(),
                assigned_slot,
            )],
        )
        .expect("valid material assignment state");
        let packet = EditorViewportModelMeshMaterialSelectionPacket::from_model_mesh_regions(
            &assignments,
            [material_region],
        );

        let mut state = EditorViewportRenderState::default();
        state.set_model_mesh_material_selection_packet(packet);
        let uniform = state.compose_scene_product_uniform((1280, 720));

        assert_eq!(uniform.model_mesh_flags, [1, 0, 0, 0]);
        assert_eq!(uniform.model_mesh_region_flags[0], [1, 1, 0, 0]);
        assert_eq!(uniform.model_mesh_region_rects[0], [0.5, 0.56, 0.18, 0.22]);
        assert_eq!(uniform.model_mesh_region_flags[1], [0, 0, 0, 0]);
        assert_eq!(uniform.model_mesh_region_rects[1], [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            state
                .branch_trace_snapshot((1280, 720))
                .first_model_mesh_region_flags,
            [1, 1, 0, 0]
        );
    }

    #[test]
    fn model_mesh_preview_packet_uses_source_backed_material_regions() {
        let assigned_slot = SceneMaterialSlotId::new(2);
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(assigned_slot, "Imported Body").with_material_asset(asset_id(7)),
        ])
        .expect("valid palette");
        let material_region = SceneModelMeshMaterialRegionSourceId::new(
            SceneModelMeshSourceId::new(asset_id(42), asset_source_id(84))
                .with_source_revision_id(asset_source_revision_id(2))
                .with_source_revision("sha256:source"),
            SceneMeshMaterialRegionId::new("source_material_slot:0")
                .expect("source material slot key should be stable"),
        );
        let assignments = SceneMaterialAssignmentState::new_with_model_mesh_assignments(
            palette,
            [],
            [SceneModelMeshMaterialSlotAssignment::new(
                material_region.clone(),
                assigned_slot,
            )],
        )
        .expect("valid material assignment state");

        let packet = EditorViewportModelMeshMaterialSelectionPacket::from_model_mesh_regions(
            &assignments,
            [material_region],
        );

        assert!(packet.diagnostics().is_empty());
        assert_eq!(packet.selections().len(), 1);
        let selection = packet.selections().first().expect("selection should exist");
        assert_eq!(selection.material_table_index, 1);
        assert_eq!(selection.requested_slot_id, assigned_slot);
        assert_eq!(selection.resolved_slot_id, assigned_slot);
        let prepared = selection.prepared_selection();
        assert_eq!(prepared.surface.source.asset_id, asset_id(42).raw());
        assert_eq!(prepared.surface.source.source_id, asset_source_id(84).raw());
        assert_eq!(
            prepared.surface.source.source_revision_id,
            Some(asset_source_revision_id(2).raw())
        );
        assert_eq!(
            prepared.surface.source.source_revision.as_deref(),
            Some("sha256:source")
        );
        assert_eq!(prepared.surface.region_key, "source_material_slot:0");
        assert!(
            !prepared.surface.identity_key().contains("renderable_index"),
            "model/mesh preview packets must not use transient renderable identity"
        );
    }

    #[test]
    fn viewport_shaders_decode_scene_primitives_and_keep_grid_in_overlay() {
        let scene_shader =
            include_str!("../../../../assets/shaders/editor_viewport_scene_product.wgsl");
        let picking_shader =
            include_str!("../../../../assets/shaders/editor_viewport_picking_product.wgsl");
        let overlay_shader =
            include_str!("../../../../assets/shaders/editor_viewport_overlay_product.wgsl");
        for shader in [scene_shader, picking_shader] {
            assert_shader_supports_primitive(shader, EditorPrimitiveKind::Sphere);
            assert_shader_supports_primitive(shader, EditorPrimitiveKind::Capsule);
            assert_shader_supports_primitive(shader, EditorPrimitiveKind::Cylinder);
            assert_shader_supports_primitive(shader, EditorPrimitiveKind::Torus);
            assert_shader_supports_primitive(shader, EditorPrimitiveKind::Plane);
            assert!(shader.contains("return sdf_box("));
        }
        for shader in [scene_shader, picking_shader, overlay_shader] {
            assert!(
                shader.contains("model_mesh_region_flags : array<vec4<u32>, 16>"),
                "viewport product shaders must keep the model/mesh material-region uniform ABI aligned"
            );
        }
        assert!(
            scene_shader.contains("fn model_mesh_region_at(viewport_local: vec2<f32>)")
                && scene_shader.contains("fn shade_model_mesh_region"),
            "scene product shader must consume model/mesh material-region uniform slots"
        );
        assert!(
            !scene_shader.contains("sdf_ground_box")
                && !scene_shader.contains("grid_shade")
                && !scene_shader.contains("grid_color"),
            "scene product shader must render only packet-backed authored primitives"
        );
        assert!(
            scene_shader.contains("viewport_background"),
            "scene product misses must resolve to a deterministic background instead of retaining prior target pixels"
        );
        assert!(
            overlay_shader.contains("grid_color") && overlay_shader.contains("grid_overlay"),
            "viewport grid visuals must live in the overlay product shader"
        );
    }

    #[test]
    fn viewport_wgsl_shaders_parse_and_validate() {
        for (label, shader) in [
            (
                "scene",
                include_str!("../../../../assets/shaders/editor_viewport_scene_product.wgsl"),
            ),
            (
                "picking",
                include_str!("../../../../assets/shaders/editor_viewport_picking_product.wgsl"),
            ),
            (
                "overlay",
                include_str!("../../../../assets/shaders/editor_viewport_overlay_product.wgsl"),
            ),
        ] {
            let module = naga::front::wgsl::parse_str(shader)
                .unwrap_or_else(|error| panic!("{label} WGSL should parse: {error}"));
            naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::empty(),
            )
            .validate(&module)
            .unwrap_or_else(|error| panic!("{label} WGSL should validate: {error}"));
        }
    }

    fn assert_shader_supports_primitive(shader: &str, kind: EditorPrimitiveKind) {
        let branch = format!("primitive_kind == {}u", kind.as_u32());
        assert!(
            shader.contains(&branch),
            "shader must decode {:?} packet slots with branch {branch}",
            kind
        );
    }
}
