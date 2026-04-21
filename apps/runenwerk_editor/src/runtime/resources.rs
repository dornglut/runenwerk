use engine::plugins::render::GpuUniform;
use glam::{Vec3, vec3};
use scene::Vec3Value;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_runtime::{EditorPrimitive, EditorPrimitiveKind};
use crate::shell::RunenwerkEditorShellState;

const SHELL_READABILITY_BUMP: f32 = 1.15;
const SHELL_SCALE_MIN: f32 = 1.0;
const SHELL_SCALE_MAX: f32 = 3.0;
const VIEWPORT_BOUNDS_EPSILON: f32 = 0.25;
const BRANCH_TRACE_FLOAT_EPSILON: f32 = 0.0005;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorViewportDebugStage {
    Scene,
    ViewportCoverage,
    ViewportUvGradient,
    PrimitiveAvailability,
    PickingHitMiss,
}

impl EditorViewportDebugStage {
    pub fn as_u32(self) -> u32 {
        match self {
            Self::Scene => 0,
            Self::ViewportCoverage => 1,
            Self::ViewportUvGradient => 2,
            Self::PrimitiveAvailability => 3,
            Self::PickingHitMiss => 4,
        }
    }

    pub fn from_env_value(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "viewport_coverage" | "viewport-coverage" | "coverage" | "mask" => {
                Self::ViewportCoverage
            }
            "viewport_uv_gradient" | "viewport-uv-gradient" | "uv_gradient" | "gradient" => {
                Self::ViewportUvGradient
            }
            "primitive_availability"
            | "primitive-availability"
            | "primitive_gate"
            | "primitive-gate"
            | "primitivegate" => Self::PrimitiveAvailability,
            "picking_hit_miss" | "picking-hit-miss" | "hit_miss" | "hit-miss" | "hitmiss" => {
                Self::PickingHitMiss
            }
            "scene" | "" => Self::Scene,
            _ => Self::Scene,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Scene => "scene",
            Self::ViewportCoverage => "viewport_coverage",
            Self::ViewportUvGradient => "viewport_uv_gradient",
            Self::PrimitiveAvailability => "primitive_availability",
            Self::PickingHitMiss => "picking_hit_miss",
        }
    }
}

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

pub fn effective_shell_scale(scale_factor: f64) -> f32 {
    (scale_factor as f32).clamp(SHELL_SCALE_MIN, SHELL_SCALE_MAX) * SHELL_READABILITY_BUMP
}

pub fn scaled_shell_theme(theme: &ThemeTokens, scale_factor: f64) -> ThemeTokens {
    theme.scaled_by(effective_shell_scale(scale_factor))
}

#[derive(Debug, Default, Clone, Copy, ecs::Component, ecs::Resource)]
pub struct EditorInputBridgeState {
    pub last_mouse_position: (f32, f32),
    pub last_logged_picking_revision: u64,
}

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
}

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
    }

    pub fn summary_line(self) -> String {
        format!(
            "stage={}({}) valid={} shader_loaded={} has_primitive={} kind={:?} bounds=({:.1},{:.1},{:.1},{:.1}) viewport=({:.1},{:.1},{:.1},{:.1}) surface=({:.0},{:.0}) obj=({:.2},{:.2},{:.2}) params_a=({:.2},{:.2},{:.2},{:.2}) params_b=({:.2},{:.2},{:.2},{:.2}) flags={:?} cam_pos=({:.2},{:.2},{:.2}) fov={:.3} cam_fwd=({:.3},{:.3},{:.3}) cam_right=({:.3},{:.3},{:.3}) cam_up=({:.3},{:.3},{:.3})",
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

#[derive(Debug, Clone, Copy, ecs::Component, ecs::Resource)]
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
        self.has_primitive = true;
        self.primitive_kind = match primitive.kind() {
            EditorPrimitiveKind::Capsule => EditorPrimitiveKind::Box,
            kind => kind,
        };
        self.primitive_translation = translation;
        self.box_half_extents = primitive.box_half_extents;
        self.sphere_radius = primitive.sphere_radius;
        self.capsule_radius = primitive.capsule_radius;
        self.capsule_half_height = primitive.capsule_half_height;
    }

    pub fn clear_primitive(&mut self) {
        self.has_primitive = false;
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
        let camera = editor_viewport_camera();

        EditorViewportSceneProductUniform {
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
                self.debug_stage.as_u32(),
                if self.root_background_opaque { 1 } else { 0 },
            ],
        }
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
