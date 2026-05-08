//! File: domain/editor/editor_viewport/src/camera.rs
//! Purpose: Viewport-owned camera, debug, and retained runtime settings.

use crate::ExpressionProductId;
use ui_math::{UiPoint, UiVector};

const DEFAULT_CAMERA_DISTANCE: f32 = 6.403_124_3;
const DEFAULT_CAMERA_YAW_RADIANS: f32 = std::f32::consts::FRAC_PI_4;
const DEFAULT_CAMERA_PITCH_RADIANS: f32 = 0.486_694_96;
const DEFAULT_CAMERA_FOV_Y_RADIANS: f32 = 0.872_664_63;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportProjection {
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorCameraState {
    pub projection: ViewportProjection,
    pub orbit_target: [f32; 3],
    pub distance: f32,
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub viewport_origin: UiPoint,
    pub viewport_size: ui_math::UiSize,
    pub pan_delta: UiVector,
}

impl Default for EditorCameraState {
    fn default() -> Self {
        Self {
            projection: ViewportProjection::Perspective,
            orbit_target: [0.0, 0.0, 0.0],
            distance: 5.0,
            yaw_radians: 0.0,
            pitch_radians: 0.0,
            viewport_origin: UiPoint::ZERO,
            viewport_size: ui_math::UiSize::ZERO,
            pan_delta: UiVector::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportDebugStage {
    Scene,
    ViewportCoverage,
    ViewportUvGradient,
    PrimitiveAvailability,
    PickingHitMiss,
}

impl ViewportDebugStage {
    pub const ALL: [Self; 5] = [
        Self::Scene,
        Self::ViewportCoverage,
        Self::ViewportUvGradient,
        Self::PrimitiveAvailability,
        Self::PickingHitMiss,
    ];

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

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Scene => "Scene",
            Self::ViewportCoverage => "Coverage",
            Self::ViewportUvGradient => "UV",
            Self::PrimitiveAvailability => "Primitive",
            Self::PickingHitMiss => "Picking",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportCameraSettings {
    pub orbit_target: [f32; 3],
    pub distance: f32,
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub fov_y_radians: f32,
}

impl ViewportCameraSettings {
    pub fn is_valid(self) -> bool {
        self.orbit_target.iter().all(|value| value.is_finite())
            && self.distance.is_finite()
            && self.distance > 0.0
            && self.yaw_radians.is_finite()
            && self.pitch_radians.is_finite()
            && self.fov_y_radians.is_finite()
            && self.fov_y_radians > 0.0
            && self.fov_y_radians < std::f32::consts::PI
    }
}

impl Default for ViewportCameraSettings {
    fn default() -> Self {
        Self {
            orbit_target: [0.0, 0.0, 0.0],
            distance: DEFAULT_CAMERA_DISTANCE,
            yaw_radians: DEFAULT_CAMERA_YAW_RADIANS,
            pitch_radians: DEFAULT_CAMERA_PITCH_RADIANS,
            fov_y_radians: DEFAULT_CAMERA_FOV_Y_RADIANS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportRuntimeSettings {
    pub camera: ViewportCameraSettings,
    pub debug_stage: ViewportDebugStage,
    pub root_background_opaque: bool,
    pub selected_primary_product_id: Option<ExpressionProductId>,
}

impl Default for ViewportRuntimeSettings {
    fn default() -> Self {
        Self {
            camera: ViewportCameraSettings::default(),
            debug_stage: ViewportDebugStage::Scene,
            root_background_opaque: false,
            selected_primary_product_id: None,
        }
    }
}
