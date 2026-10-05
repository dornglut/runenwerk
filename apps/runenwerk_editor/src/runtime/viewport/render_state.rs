//! File: apps/runenwerk_editor/src/runtime/viewport/render_state.rs
//! Purpose: Per-viewport runtime render-state ownership.

use std::collections::BTreeMap;

use editor_core::EntityId;
use editor_shell::ToolSurfaceInstanceId;
use editor_viewport::{
    ViewportCameraSettings, ViewportFieldVisualizerSettings, ViewportId, ViewportRuntimeSettings,
};
use engine::plugins::render::GpuParams;
use engine::runtime::ResMut;
use glam::{Vec3, vec3};
use scene::{LocalTransform, Vec3Value};
use ui_math::UiRect;
use ui_math::UiVector;

use crate::editor_runtime::{EditorPrimitive, EditorPrimitiveKind};
use crate::runtime::resources::{
    EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS, EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES,
    EditorViewportModelMeshMaterialSelection, EditorViewportModelMeshMaterialSelectionPacket,
    EditorViewportPrimitiveInstance, EditorViewportSceneProductUniform,
    EditorViewportSceneRenderPacket,
};

const VIEWPORT_BOUNDS_EPSILON: f32 = 0.25;
const BRANCH_TRACE_FLOAT_EPSILON: f32 = 0.0005;
const CAMERA_MIN_DISTANCE: f32 = 0.25;
const CAMERA_MAX_DISTANCE: f32 = 500.0;
const CAMERA_ORBIT_SENSITIVITY: f32 = 0.006;
const CAMERA_PAN_SENSITIVITY: f32 = 0.0015;
const CAMERA_ZOOM_SENSITIVITY: f32 = 0.08;
const CAMERA_MAX_PITCH_RADIANS: f32 = 1.553_343;
pub use editor_viewport::ViewportDebugStage as EditorViewportDebugStage;

#[derive(Debug, Clone, Copy)]
pub struct EditorViewportBranchTraceSnapshot {
    pub viewport_bounds_px: (f32, f32, f32, f32),
    pub viewport_valid: bool,
    pub shader_loaded: bool,
    pub debug_stage: EditorViewportDebugStage,
    pub has_primitive: bool,
    pub primitive_kind: EditorPrimitiveKind,
    pub primitive_translation: Vec3Value,
    pub surface: [f32; 4],
    pub viewport: [f32; 4],
    pub camera_position: [f32; 4],
    pub camera_forward: [f32; 4],
    pub camera_right: [f32; 4],
    pub camera_up: [f32; 4],
    pub primitive_params_a: [f32; 4],
    pub primitive_params_b: [f32; 4],
    pub primitive_flags: [u32; 4],
    pub model_mesh_flags: [u32; 4],
    pub first_model_mesh_region_flags: [u32; 4],
}

impl EditorViewportBranchTraceSnapshot {
    pub fn approx_eq(&self, other: &Self) -> bool {
        approx_bounds_eq(self.viewport_bounds_px, other.viewport_bounds_px)
            && self.viewport_valid == other.viewport_valid
            && self.shader_loaded == other.shader_loaded
            && self.debug_stage == other.debug_stage
            && self.has_primitive == other.has_primitive
            && self.primitive_kind == other.primitive_kind
            && approx_vec3(self.primitive_translation, other.primitive_translation)
            && approx_vec4(self.surface, other.surface)
            && approx_vec4(self.viewport, other.viewport)
            && approx_vec4(self.camera_position, other.camera_position)
            && approx_vec4(self.camera_forward, other.camera_forward)
            && approx_vec4(self.camera_right, other.camera_right)
            && approx_vec4(self.camera_up, other.camera_up)
            && approx_vec4(self.primitive_params_a, other.primitive_params_a)
            && approx_vec4(self.primitive_params_b, other.primitive_params_b)
            && self.primitive_flags == other.primitive_flags
            && self.model_mesh_flags == other.model_mesh_flags
            && self.first_model_mesh_region_flags == other.first_model_mesh_region_flags
    }

    pub fn summary_line(self) -> String {
        format!(
            "stage={}({}) valid={} shader_loaded={} has_primitive={} kind={:?} bounds=({:.1},{:.1},{:.1},{:.1}) viewport=({:.1},{:.1},{:.1},{:.1}) surface=({:.0},{:.0}) obj=({:.2},{:.2},{:.2}) params_a=({:.2},{:.2},{:.2},{:.2}) params_b=({:.2},{:.2},{:.2},{:.2}) flags={:?} model_mesh_flags={:?} model_mesh_first={:?} cam_pos=({:.2},{:.2},{:.2}) fov={:.3} cam_fwd=({:.3},{:.3},{:.3}) cam_right=({:.3},{:.3},{:.3}) cam_up=({:.3},{:.3},{:.3})",
            self.debug_stage.label(),
            self.debug_stage.as_u32(),
            self.viewport_valid,
            self.shader_loaded,
            self.has_primitive,
            self.primitive_kind,
            self.viewport_bounds_px.0,
            self.viewport_bounds_px.1,
            self.viewport_bounds_px.2,
            self.viewport_bounds_px.3,
            self.viewport[0],
            self.viewport[1],
            self.viewport[2],
            self.viewport[3],
            self.surface[0],
            self.surface[1],
            self.primitive_translation.x,
            self.primitive_translation.y,
            self.primitive_translation.z,
            self.primitive_params_a[0],
            self.primitive_params_a[1],
            self.primitive_params_a[2],
            self.primitive_params_a[3],
            self.primitive_params_b[0],
            self.primitive_params_b[1],
            self.primitive_params_b[2],
            self.primitive_params_b[3],
            self.primitive_flags,
            self.model_mesh_flags,
            self.first_model_mesh_region_flags,
            self.camera_position[0],
            self.camera_position[1],
            self.camera_position[2],
            self.camera_position[3],
            self.camera_forward[0],
            self.camera_forward[1],
            self.camera_forward[2],
            self.camera_right[0],
            self.camera_right[1],
            self.camera_right[2],
            self.camera_up[0],
            self.camera_up[1],
            self.camera_up[2],
        )
    }
}

#[derive(Debug, Clone, runen_ecs::Component, runen_ecs::Resource)]
pub struct EditorViewportRenderState {
    pub viewport_bounds_px: (f32, f32, f32, f32),
    pub effective_shell_scale: f32,
    pub viewport_valid: bool,
    pub shader_loaded: bool,
    pub debug_stage: EditorViewportDebugStage,
    pub root_background_opaque: bool,
    pub has_primitive: bool,
    pub primitive_kind: EditorPrimitiveKind,
    pub primitive_translation: Vec3Value,
    pub box_half_extents: Vec3Value,
    pub sphere_radius: f32,
    pub capsule_radius: f32,
    pub capsule_half_height: f32,
    pub scene_packet: EditorViewportSceneRenderPacket,
    pub model_mesh_material_selection_packet: EditorViewportModelMeshMaterialSelectionPacket,
    pub camera_settings: ViewportCameraSettings,
    pub camera: EditorViewportCamera,
    pub camera_fov_y_radians: f32,
    pub visibility_contradiction_active: bool,
    pub last_reported_viewport_bounds_px: Option<(f32, f32, f32, f32)>,
    pub last_reported_shell_scale: Option<f32>,
    pub last_reported_debug_state: Option<(EditorViewportDebugStage, bool, bool, bool, bool)>,
    pub last_reported_branch_trace: Option<EditorViewportBranchTraceSnapshot>,
}

impl Default for EditorViewportRenderState {
    fn default() -> Self {
        Self {
            viewport_bounds_px: (0.0, 0.0, 0.0, 0.0),
            effective_shell_scale: 1.0,
            viewport_valid: false,
            shader_loaded: false,
            debug_stage: EditorViewportDebugStage::Scene,
            root_background_opaque: false,
            has_primitive: false,
            primitive_kind: EditorPrimitiveKind::Box,
            primitive_translation: Vec3Value::zero(),
            box_half_extents: Vec3Value::new(0.5, 0.5, 0.5),
            sphere_radius: 0.6,
            capsule_radius: 0.35,
            capsule_half_height: 0.75,
            scene_packet: EditorViewportSceneRenderPacket::default(),
            model_mesh_material_selection_packet:
                EditorViewportModelMeshMaterialSelectionPacket::default(),
            camera_settings: ViewportCameraSettings::default(),
            camera: editor_viewport_camera(),
            camera_fov_y_radians: editor_viewport_camera_fov_y_radians(),
            visibility_contradiction_active: false,
            last_reported_viewport_bounds_px: None,
            last_reported_shell_scale: None,
            last_reported_debug_state: None,
            last_reported_branch_trace: None,
        }
    }
}

impl EditorViewportRenderState {
    pub fn set_viewport_bounds(&mut self, bounds: (f32, f32, f32, f32)) -> bool {
        let changed = !approx_bounds_eq(self.viewport_bounds_px, bounds);
        self.viewport_bounds_px = bounds;
        changed
    }

    pub fn set_effective_shell_scale(&mut self, scale: f32) -> bool {
        let changed = (self.effective_shell_scale - scale).abs() > f32::EPSILON;
        self.effective_shell_scale = scale;
        changed
    }

    pub fn set_debug_stage(&mut self, stage: EditorViewportDebugStage) -> bool {
        let changed = self.debug_stage != stage;
        self.debug_stage = stage;
        changed
    }

    pub fn set_root_background_opaque(&mut self, enabled: bool) -> bool {
        let changed = self.root_background_opaque != enabled;
        self.root_background_opaque = enabled;
        changed
    }

    pub fn from_viewport_settings(settings: ViewportRuntimeSettings) -> Self {
        let mut state = Self::default();
        state.apply_viewport_settings(settings);
        state
    }

    pub fn viewport_settings(
        &self,
        selected_primary_product_id: Option<editor_viewport::ExpressionProductId>,
        field_visualizer_settings: ViewportFieldVisualizerSettings,
    ) -> ViewportRuntimeSettings {
        ViewportRuntimeSettings {
            camera: self.camera_settings,
            debug_stage: self.debug_stage,
            root_background_opaque: self.root_background_opaque,
            selected_primary_product_id,
            field_visualizer_settings,
        }
    }

    pub fn apply_viewport_settings(&mut self, settings: ViewportRuntimeSettings) -> bool {
        let camera_changed = self.set_camera_settings(settings.camera);
        let debug_changed = self.set_debug_stage(settings.debug_stage);
        let root_changed = self.set_root_background_opaque(settings.root_background_opaque);
        camera_changed || debug_changed || root_changed
    }

    pub fn set_camera_settings(&mut self, settings: ViewportCameraSettings) -> bool {
        let settings = sanitized_camera_settings(settings);
        let changed = self.camera_settings != settings
            || (self.camera_fov_y_radians - settings.fov_y_radians).abs() > f32::EPSILON;
        self.camera_settings = settings;
        self.camera = editor_viewport_camera_from_settings(settings);
        self.camera_fov_y_radians = settings.fov_y_radians;
        changed
    }

    pub fn reset_camera(&mut self) -> bool {
        self.set_camera_settings(ViewportCameraSettings::default())
    }

    pub fn focus_camera_on(&mut self, orbit_target: [f32; 3]) -> bool {
        let mut settings = self.camera_settings;
        settings.orbit_target = orbit_target;
        self.set_camera_settings(settings)
    }

    pub fn orbit_camera(&mut self, delta: UiVector) -> bool {
        if delta == UiVector::ZERO {
            return false;
        }
        let mut settings = self.camera_settings;
        settings.yaw_radians += delta.x * CAMERA_ORBIT_SENSITIVITY;
        settings.pitch_radians = (settings.pitch_radians + delta.y * CAMERA_ORBIT_SENSITIVITY)
            .clamp(-CAMERA_MAX_PITCH_RADIANS, CAMERA_MAX_PITCH_RADIANS);
        self.set_camera_settings(settings)
    }

    pub fn pan_camera(&mut self, delta: UiVector) -> bool {
        if delta == UiVector::ZERO {
            return false;
        }
        let distance = self
            .camera_settings
            .distance
            .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
        let amount = distance * CAMERA_PAN_SENSITIVITY;
        let world_delta = (-self.camera.right * delta.x + self.camera.up * delta.y) * amount;
        let mut settings = self.camera_settings;
        settings.orbit_target[0] += world_delta.x;
        settings.orbit_target[1] += world_delta.y;
        settings.orbit_target[2] += world_delta.z;
        self.set_camera_settings(settings)
    }

    pub fn zoom_camera(&mut self, scroll_delta: f32) -> bool {
        if scroll_delta.abs() <= f32::EPSILON {
            return false;
        }
        let mut settings = self.camera_settings;
        settings.distance = (settings.distance * (-scroll_delta * CAMERA_ZOOM_SENSITIVITY).exp())
            .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
        self.set_camera_settings(settings)
    }

    pub fn should_report_bounds_change(&mut self) -> bool {
        let should_report = match self.last_reported_viewport_bounds_px {
            Some(last) => !approx_bounds_eq(last, self.viewport_bounds_px),
            None => true,
        };
        if should_report {
            self.last_reported_viewport_bounds_px = Some(self.viewport_bounds_px);
        }
        should_report
    }

    pub fn should_report_scale_change(&mut self) -> bool {
        let should_report = match self.last_reported_shell_scale {
            Some(last) => (last - self.effective_shell_scale).abs() > f32::EPSILON,
            None => true,
        };
        if should_report {
            self.last_reported_shell_scale = Some(self.effective_shell_scale);
        }
        should_report
    }

    pub fn set_primitive(&mut self, translation: Vec3Value, primitive: EditorPrimitive) {
        let transform = LocalTransform::from_translation(translation);
        let instance = EditorViewportPrimitiveInstance::from_transform_and_primitive(
            EntityId(0),
            transform,
            primitive,
            false,
            false,
        );
        self.set_scene_packet(EditorViewportSceneRenderPacket::from_primitives([instance]));
    }

    pub fn set_scene_packet(&mut self, packet: EditorViewportSceneRenderPacket) {
        self.has_primitive = !packet.is_empty();
        self.scene_packet = packet;
        self.sync_first_primitive_mirror();
    }

    pub fn set_model_mesh_material_selection_packet(
        &mut self,
        packet: EditorViewportModelMeshMaterialSelectionPacket,
    ) {
        self.model_mesh_material_selection_packet = packet;
    }

    pub fn clear_primitive(&mut self) {
        self.has_primitive = false;
        self.scene_packet = EditorViewportSceneRenderPacket::default();
        self.sync_first_primitive_mirror();
    }

    fn sync_first_primitive_mirror(&mut self) {
        let Some(first) = self.scene_packet.primitives().first().copied() else {
            self.primitive_kind = EditorPrimitiveKind::Box;
            self.primitive_translation = Vec3Value::zero();
            self.box_half_extents = Vec3Value::new(0.5, 0.5, 0.5);
            self.sphere_radius = 0.6;
            self.capsule_radius = 0.35;
            self.capsule_half_height = 0.75;
            return;
        };

        self.primitive_kind = first.primitive_kind;
        self.primitive_translation = first.translation;
        self.box_half_extents = first.box_half_extents;
        self.sphere_radius = first.sphere_radius;
        self.capsule_radius = first.capsule_radius;
        self.capsule_half_height = first.capsule_half_height;
    }

    pub fn update_visibility_diagnostics(&mut self, viewport_valid: bool, shader_loaded: bool) {
        self.viewport_valid = viewport_valid;
        self.shader_loaded = shader_loaded;
    }

    pub fn scene_should_be_invisible(&self) -> bool {
        self.debug_stage == EditorViewportDebugStage::Scene
            && (!self.viewport_valid || !self.shader_loaded || !self.has_primitive)
    }

    pub fn should_report_visibility_contradiction(&mut self, contradiction_active: bool) -> bool {
        let should_report = contradiction_active && !self.visibility_contradiction_active;
        self.visibility_contradiction_active = contradiction_active;
        should_report
    }

    pub fn should_report_debug_state_change(&mut self) -> bool {
        let next = (
            self.debug_stage,
            self.root_background_opaque,
            self.viewport_valid,
            self.shader_loaded,
            self.has_primitive,
        );
        let changed = self.last_reported_debug_state != Some(next);
        if changed {
            self.last_reported_debug_state = Some(next);
        }
        changed
    }

    pub fn branch_trace_snapshot(&self, surface: (u32, u32)) -> EditorViewportBranchTraceSnapshot {
        let uniform = self.compose_scene_product_uniform(surface);
        EditorViewportBranchTraceSnapshot {
            viewport_bounds_px: self.viewport_bounds_px,
            viewport_valid: self.viewport_valid,
            shader_loaded: self.shader_loaded,
            debug_stage: self.debug_stage,
            has_primitive: self.has_primitive,
            primitive_kind: self.primitive_kind,
            primitive_translation: self.primitive_translation,
            surface: uniform.surface,
            viewport: uniform.viewport,
            camera_position: uniform.camera_position,
            camera_forward: uniform.camera_forward,
            camera_right: uniform.camera_right,
            camera_up: uniform.camera_up,
            primitive_params_a: uniform.primitive_params_a,
            primitive_params_b: uniform.primitive_params_b,
            primitive_flags: uniform.primitive_flags,
            model_mesh_flags: uniform.model_mesh_flags,
            first_model_mesh_region_flags: uniform.model_mesh_region_flags[0],
        }
    }

    pub fn should_report_branch_trace_change(
        &mut self,
        snapshot: EditorViewportBranchTraceSnapshot,
    ) -> bool {
        let changed = match self.last_reported_branch_trace {
            Some(previous) => !previous.approx_eq(&snapshot),
            None => true,
        };
        if changed {
            self.last_reported_branch_trace = Some(snapshot);
        }
        changed
    }

    pub fn compose_scene_product_uniform(
        &self,
        surface: (u32, u32),
    ) -> EditorViewportSceneProductUniform {
        let width = surface.0.max(1) as f32;
        let height = surface.1.max(1) as f32;
        let camera = self.camera;
        let mut primitive_slot_transforms = [[0.0; 4]; EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES];
        let mut primitive_slot_params_a = [[0.0; 4]; EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES];
        let mut primitive_slot_params_b = [[0.0; 4]; EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES];
        let mut primitive_slot_flags = [[0; 4]; EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES];
        let mut model_mesh_region_rects =
            [[0.0; 4]; EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS];
        let mut model_mesh_region_flags = [[0; 4]; EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS];
        for (index, primitive) in self
            .scene_packet
            .primitives()
            .iter()
            .take(EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES)
            .enumerate()
        {
            primitive_slot_transforms[index] = primitive.shader_slot_transform();
            primitive_slot_params_a[index] = primitive.shader_slot_params_a();
            primitive_slot_params_b[index] = primitive.shader_slot_params_b();
            primitive_slot_flags[index] = primitive.shader_slot_flags();
        }
        let primitive_count = self
            .scene_packet
            .len()
            .min(EDITOR_VIEWPORT_MAX_PRIMITIVE_INSTANCES) as u32;
        let model_mesh_region_count = self
            .model_mesh_material_selection_packet
            .selections()
            .len()
            .min(EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS);
        let model_mesh_region_omitted_count = self
            .model_mesh_material_selection_packet
            .selections()
            .len()
            .saturating_sub(EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS);
        write_model_mesh_material_region_slots(
            self.model_mesh_material_selection_packet.selections(),
            &mut model_mesh_region_rects,
            &mut model_mesh_region_flags,
        );

        EditorViewportSceneProductUniform {
            surface: [width, height, 1.0 / width, 1.0 / height],
            viewport: [0.0, 0.0, width, height],
            camera_position: [
                camera.position.x,
                camera.position.y,
                camera.position.z,
                self.camera_fov_y_radians,
            ],
            camera_forward: [camera.forward.x, camera.forward.y, camera.forward.z, 0.0],
            camera_right: [camera.right.x, camera.right.y, camera.right.z, 0.0],
            camera_up: [camera.up.x, camera.up.y, camera.up.z, 0.0],
            object_transform: [
                self.primitive_translation.x,
                self.primitive_translation.y,
                self.primitive_translation.z,
                0.0,
            ],
            primitive_params_a: [
                self.box_half_extents.x.max(0.05),
                self.box_half_extents.y.max(0.05),
                self.box_half_extents.z.max(0.05),
                self.sphere_radius.max(0.05),
            ],
            primitive_params_b: [
                self.capsule_radius.max(0.05),
                self.capsule_half_height.max(0.05),
                0.0,
                0.0,
            ],
            primitive_flags: [
                self.primitive_kind.as_u32(),
                primitive_count,
                self.debug_stage.as_u32(),
                if self.root_background_opaque { 1 } else { 0 },
            ],
            primitive_slot_transforms,
            primitive_slot_params_a,
            primitive_slot_params_b,
            primitive_slot_flags,
            model_mesh_flags: [
                model_mesh_region_count as u32,
                model_mesh_region_omitted_count as u32,
                0,
                0,
            ],
            model_mesh_region_rects,
            model_mesh_region_flags,
        }
    }

    pub fn compose_scene_product_uniform_bytes(&self, surface: (u32, u32)) -> Vec<u8> {
        let raw = self.compose_scene_product_uniform(surface).to_gpu();
        engine::plugins::render::bytemuck::bytes_of(&raw).to_vec()
    }
}

fn write_model_mesh_material_region_slots(
    selections: &[EditorViewportModelMeshMaterialSelection],
    rects: &mut [[f32; 4]; EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS],
    flags: &mut [[u32; 4]; EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS],
) {
    let count = selections
        .len()
        .min(EDITOR_VIEWPORT_MAX_MODEL_MESH_MATERIAL_REGIONS);
    if count == 0 {
        return;
    }

    let count_f = count as f32;
    let half_width = (0.42 / count_f).clamp(0.055, 0.18);
    for (index, selection) in selections.iter().take(count).enumerate() {
        let center_x = (index as f32 + 0.5) / count_f;
        rects[index] = [center_x, 0.56, half_width, 0.22];
        flags[index] = [
            selection.material_table_index,
            (index + 1) as u32,
            u32::from(selection.used_default_fallback),
            0,
        ];
    }
}

fn approx_bounds_eq(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    (a.0 - b.0).abs() <= VIEWPORT_BOUNDS_EPSILON
        && (a.1 - b.1).abs() <= VIEWPORT_BOUNDS_EPSILON
        && (a.2 - b.2).abs() <= VIEWPORT_BOUNDS_EPSILON
        && (a.3 - b.3).abs() <= VIEWPORT_BOUNDS_EPSILON
}

fn approx_f32(a: f32, b: f32) -> bool {
    (a - b).abs() <= BRANCH_TRACE_FLOAT_EPSILON
}

fn approx_vec3(a: Vec3Value, b: Vec3Value) -> bool {
    approx_f32(a.x, b.x) && approx_f32(a.y, b.y) && approx_f32(a.z, b.z)
}

fn approx_vec4(a: [f32; 4], b: [f32; 4]) -> bool {
    approx_f32(a[0], b[0])
        && approx_f32(a[1], b[1])
        && approx_f32(a[2], b[2])
        && approx_f32(a[3], b[3])
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorViewportCamera {
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

pub fn editor_viewport_camera() -> EditorViewportCamera {
    editor_viewport_camera_from_settings(ViewportCameraSettings::default())
}

pub fn editor_viewport_camera_from_settings(
    settings: ViewportCameraSettings,
) -> EditorViewportCamera {
    let settings = sanitized_camera_settings(settings);
    let target = vec3(
        settings.orbit_target[0],
        settings.orbit_target[1],
        settings.orbit_target[2],
    );
    let (sin_yaw, cos_yaw) = settings.yaw_radians.sin_cos();
    let (sin_pitch, cos_pitch) = settings.pitch_radians.sin_cos();
    let offset = vec3(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw) * settings.distance;
    let position = target + offset;
    let world_up = Vec3::Y;
    let forward = (target - position).normalize_or_zero();
    let mut right = forward.cross(world_up).normalize_or_zero();
    if right.length_squared() <= f32::EPSILON {
        right = Vec3::X;
    }
    let up = right.cross(forward).normalize_or_zero();

    EditorViewportCamera {
        position,
        forward,
        right,
        up,
    }
}

pub fn editor_viewport_camera_fov_y_radians() -> f32 {
    ViewportCameraSettings::default().fov_y_radians
}

fn sanitized_camera_settings(settings: ViewportCameraSettings) -> ViewportCameraSettings {
    if !settings.is_valid() {
        return ViewportCameraSettings::default();
    }
    ViewportCameraSettings {
        orbit_target: settings.orbit_target,
        distance: settings
            .distance
            .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE),
        yaw_radians: settings.yaw_radians,
        pitch_radians: settings
            .pitch_radians
            .clamp(-CAMERA_MAX_PITCH_RADIANS, CAMERA_MAX_PITCH_RADIANS),
        fov_y_radians: settings
            .fov_y_radians
            .clamp(1.0_f32.to_radians(), 175.0_f32.to_radians()),
    }
}

#[derive(Debug, Clone)]
pub struct ViewportRenderStateEntry {
    pub viewport_id: ViewportId,
    pub tool_surface_id: Option<ToolSurfaceInstanceId>,
    pub bounds: UiRect,
    pub render_state: EditorViewportRenderState,
}

#[derive(Debug, Default, Clone, runen_ecs::Component, runen_ecs::Resource)]
pub struct ViewportRenderStateResource {
    states_by_viewport: BTreeMap<ViewportId, ViewportRenderStateEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportRenderStateCommand {
    SetCameraSettings {
        viewport_id: ViewportId,
        settings: ViewportCameraSettings,
    },
    OrbitCamera {
        viewport_id: ViewportId,
        delta: UiVector,
    },
    PanCamera {
        viewport_id: ViewportId,
        delta: UiVector,
    },
    ZoomCamera {
        viewport_id: ViewportId,
        scroll_delta: f32,
    },
    FocusCameraOn {
        viewport_id: ViewportId,
        orbit_target: [f32; 3],
    },
    ResetCamera {
        viewport_id: ViewportId,
    },
    SetDebugStage {
        viewport_id: ViewportId,
        debug_stage: EditorViewportDebugStage,
    },
    SetRootBackgroundOpaque {
        viewport_id: ViewportId,
        enabled: bool,
    },
}

#[derive(Debug, Default, Clone, runen_ecs::Component, runen_ecs::Resource)]
pub struct ViewportRenderStateCommandQueueResource {
    commands: Vec<ViewportRenderStateCommand>,
}

impl ViewportRenderStateResource {
    pub fn upsert_state(&mut self, state: ViewportRenderStateEntry) {
        self.states_by_viewport.insert(state.viewport_id, state);
    }

    pub fn state_for(&self, viewport_id: ViewportId) -> Option<&ViewportRenderStateEntry> {
        self.states_by_viewport.get(&viewport_id)
    }

    pub fn state_for_mut(
        &mut self,
        viewport_id: ViewportId,
    ) -> Option<&mut ViewportRenderStateEntry> {
        self.states_by_viewport.get_mut(&viewport_id)
    }

    pub fn viewport_ids(&self) -> impl Iterator<Item = ViewportId> + '_ {
        self.states_by_viewport.keys().copied()
    }

    pub fn entries(&self) -> impl Iterator<Item = &ViewportRenderStateEntry> {
        self.states_by_viewport.values()
    }

    pub fn retain_viewports(&mut self, mut keep: impl FnMut(ViewportId) -> bool) {
        self.states_by_viewport
            .retain(|viewport_id, _| keep(*viewport_id));
    }

    pub fn apply_command(&mut self, command: ViewportRenderStateCommand) -> bool {
        let viewport_id = match command {
            ViewportRenderStateCommand::SetCameraSettings { viewport_id, .. }
            | ViewportRenderStateCommand::OrbitCamera { viewport_id, .. }
            | ViewportRenderStateCommand::PanCamera { viewport_id, .. }
            | ViewportRenderStateCommand::ZoomCamera { viewport_id, .. }
            | ViewportRenderStateCommand::FocusCameraOn { viewport_id, .. }
            | ViewportRenderStateCommand::ResetCamera { viewport_id }
            | ViewportRenderStateCommand::SetDebugStage { viewport_id, .. }
            | ViewportRenderStateCommand::SetRootBackgroundOpaque { viewport_id, .. } => {
                viewport_id
            }
        };
        let Some(entry) = self.states_by_viewport.get_mut(&viewport_id) else {
            return false;
        };
        match command {
            ViewportRenderStateCommand::SetCameraSettings { settings, .. } => {
                entry.render_state.set_camera_settings(settings);
            }
            ViewportRenderStateCommand::OrbitCamera { delta, .. } => {
                entry.render_state.orbit_camera(delta);
            }
            ViewportRenderStateCommand::PanCamera { delta, .. } => {
                entry.render_state.pan_camera(delta);
            }
            ViewportRenderStateCommand::ZoomCamera { scroll_delta, .. } => {
                entry.render_state.zoom_camera(scroll_delta);
            }
            ViewportRenderStateCommand::FocusCameraOn { orbit_target, .. } => {
                entry.render_state.focus_camera_on(orbit_target);
            }
            ViewportRenderStateCommand::ResetCamera { .. } => {
                entry.render_state.reset_camera();
            }
            ViewportRenderStateCommand::SetDebugStage { debug_stage, .. } => {
                entry.render_state.set_debug_stage(debug_stage);
            }
            ViewportRenderStateCommand::SetRootBackgroundOpaque { enabled, .. } => {
                entry.render_state.set_root_background_opaque(enabled);
            }
        }
        true
    }

    pub fn apply_commands(
        &mut self,
        commands: impl IntoIterator<Item = ViewportRenderStateCommand>,
    ) -> usize {
        commands
            .into_iter()
            .filter(|command| self.apply_command(*command))
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.states_by_viewport.is_empty()
    }
}

impl ViewportRenderStateCommandQueueResource {
    pub fn push(&mut self, command: ViewportRenderStateCommand) {
        self.commands.push(command);
    }

    pub fn extend(&mut self, commands: impl IntoIterator<Item = ViewportRenderStateCommand>) {
        self.commands.extend(commands);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = ViewportRenderStateCommand> + '_ {
        self.commands.drain(..)
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

pub fn apply_viewport_render_state_commands_system(
    mut viewport_render_states: ResMut<ViewportRenderStateResource>,
    mut commands: ResMut<ViewportRenderStateCommandQueueResource>,
) {
    let commands = commands.drain().collect::<Vec<_>>();
    viewport_render_states.apply_commands(commands);
}

pub fn expression_dimensions_for_bounds(bounds: UiRect) -> editor_viewport::ExpressionDimensions {
    editor_viewport::ExpressionDimensions::new(
        bounds.width.max(1.0).round() as u32,
        bounds.height.max(1.0).round() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_state_tracks_distinct_bounds_per_viewport() {
        let mut states = ViewportRenderStateResource::default();
        let first = ViewportId(1_000_001);
        let second = ViewportId(1_000_002);

        states.upsert_state(ViewportRenderStateEntry {
            viewport_id: first,
            tool_surface_id: Some(ToolSurfaceInstanceId::try_from_raw(1).unwrap()),
            bounds: UiRect::new(0.0, 0.0, 320.0, 240.0),
            render_state: EditorViewportRenderState::default(),
        });
        states.upsert_state(ViewportRenderStateEntry {
            viewport_id: second,
            tool_surface_id: Some(ToolSurfaceInstanceId::try_from_raw(2).unwrap()),
            bounds: UiRect::new(320.0, 0.0, 480.0, 240.0),
            render_state: EditorViewportRenderState::default(),
        });

        assert_eq!(
            expression_dimensions_for_bounds(states.state_for(first).unwrap().bounds),
            editor_viewport::ExpressionDimensions::new(320, 240),
        );
        assert_eq!(
            expression_dimensions_for_bounds(states.state_for(second).unwrap().bounds),
            editor_viewport::ExpressionDimensions::new(480, 240),
        );
    }

    #[test]
    fn render_state_commands_apply_to_one_viewport() {
        let mut states = ViewportRenderStateResource::default();
        let first = ViewportId(2);
        let second = ViewportId(3);
        states.upsert_state(ViewportRenderStateEntry {
            viewport_id: first,
            tool_surface_id: Some(ToolSurfaceInstanceId::try_from_raw(1).unwrap()),
            bounds: UiRect::new(0.0, 0.0, 320.0, 240.0),
            render_state: EditorViewportRenderState::default(),
        });
        states.upsert_state(ViewportRenderStateEntry {
            viewport_id: second,
            tool_surface_id: Some(ToolSurfaceInstanceId::try_from_raw(2).unwrap()),
            bounds: UiRect::new(320.0, 0.0, 480.0, 240.0),
            render_state: EditorViewportRenderState::default(),
        });

        assert!(
            states.apply_command(ViewportRenderStateCommand::SetDebugStage {
                viewport_id: second,
                debug_stage: EditorViewportDebugStage::PrimitiveAvailability,
            })
        );
        assert!(
            states.apply_command(ViewportRenderStateCommand::SetRootBackgroundOpaque {
                viewport_id: second,
                enabled: true,
            })
        );

        assert_eq!(
            states.state_for(first).unwrap().render_state.debug_stage,
            EditorViewportDebugStage::Scene,
        );
        assert_eq!(
            states.state_for(second).unwrap().render_state.debug_stage,
            EditorViewportDebugStage::PrimitiveAvailability,
        );
        assert!(
            states
                .state_for(second)
                .unwrap()
                .render_state
                .root_background_opaque
        );
    }
}
