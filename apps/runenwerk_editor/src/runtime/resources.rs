use engine::plugins::render::GpuUniform;
use glam::{Vec3, vec3};
use scene::Vec3Value;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_runtime::{EditorPrimitive, EditorPrimitiveKind};
use crate::shell::RunenwerkEditorShellState;

#[derive(ecs::Component, ecs::Resource)]
pub struct EditorHostResource {
    pub app: RunenwerkEditorApp,
    pub shell_state: RunenwerkEditorShellState,
    pub theme: ThemeTokens,
}

impl Default for EditorHostResource {
    fn default() -> Self {
        Self {
            app: RunenwerkEditorApp::new(),
            shell_state: RunenwerkEditorShellState::new(),
            theme: ThemeTokens::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, ecs::Component, ecs::Resource)]
pub struct EditorInputBridgeState {
    pub last_mouse_position: (f32, f32),
    pub last_logged_picking_revision: u64,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub struct EditorViewportSdfUniform {
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
}

#[derive(Debug, Clone, Copy, ecs::Component, ecs::Resource)]
pub struct EditorViewportRenderState {
    pub viewport_bounds_px: (f32, f32, f32, f32),
    pub has_primitive: bool,
    pub primitive_kind: EditorPrimitiveKind,
    pub primitive_translation: Vec3Value,
    pub box_half_extents: Vec3Value,
    pub sphere_radius: f32,
    pub capsule_radius: f32,
    pub capsule_half_height: f32,
}

impl Default for EditorViewportRenderState {
    fn default() -> Self {
        Self {
            viewport_bounds_px: (0.0, 0.0, 0.0, 0.0),
            has_primitive: false,
            primitive_kind: EditorPrimitiveKind::Box,
            primitive_translation: Vec3Value::zero(),
            box_half_extents: Vec3Value::new(0.5, 0.5, 0.5),
            sphere_radius: 0.6,
            capsule_radius: 0.35,
            capsule_half_height: 0.75,
        }
    }
}

impl EditorViewportRenderState {
    pub fn set_viewport_bounds(&mut self, bounds: (f32, f32, f32, f32)) {
        self.viewport_bounds_px = bounds;
    }

    pub fn set_primitive(&mut self, translation: Vec3Value, primitive: EditorPrimitive) {
        self.has_primitive = true;
        self.primitive_kind = primitive.kind();
        self.primitive_translation = translation;
        self.box_half_extents = primitive.box_half_extents;
        self.sphere_radius = primitive.sphere_radius;
        self.capsule_radius = primitive.capsule_radius;
        self.capsule_half_height = primitive.capsule_half_height;
    }

    pub fn clear_primitive(&mut self) {
        self.has_primitive = false;
    }

    pub fn compose_uniform(&self, surface: (u32, u32)) -> EditorViewportSdfUniform {
        let width = surface.0.max(1) as f32;
        let height = surface.1.max(1) as f32;
        let camera = editor_viewport_camera();

        EditorViewportSdfUniform {
            surface: [width, height, 1.0 / width, 1.0 / height],
            viewport: [
                self.viewport_bounds_px.0,
                self.viewport_bounds_px.1,
                self.viewport_bounds_px.2.max(0.0),
                self.viewport_bounds_px.3.max(0.0),
            ],
            camera_position: [
                camera.position.x,
                camera.position.y,
                camera.position.z,
                editor_viewport_camera_fov_y_radians(),
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
                if self.has_primitive { 1 } else { 0 },
                0,
                0,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EditorViewportCamera {
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

pub fn editor_viewport_camera() -> EditorViewportCamera {
    let position = vec3(4.0, 3.0, 4.0);
    let target = Vec3::ZERO;
    let world_up = Vec3::Y;
    let forward = (target - position).normalize_or_zero();
    let right = forward.cross(world_up).normalize_or_zero();
    let up = right.cross(forward).normalize_or_zero();

    EditorViewportCamera {
        position,
        forward,
        right,
        up,
    }
}

pub fn editor_viewport_camera_fov_y_radians() -> f32 {
    50.0_f32.to_radians()
}
