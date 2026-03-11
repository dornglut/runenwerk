use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterpolationConfig {
    pub min_delay_ms: f32,
    pub max_delay_ms: f32,
    pub small_error_distance: f32,
    pub medium_error_distance: f32,
    pub large_error_distance: f32,
    pub hard_snap_distance: f32,
    pub small_error_blend_seconds: f32,
    pub medium_error_blend_seconds: f32,
}

impl Default for InterpolationConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 55.0,
            max_delay_ms: 180.0,
            small_error_distance: 0.12,
            medium_error_distance: 0.55,
            large_error_distance: 1.2,
            hard_snap_distance: 3.0,
            small_error_blend_seconds: 0.12,
            medium_error_blend_seconds: 0.06,
        }
    }
}
