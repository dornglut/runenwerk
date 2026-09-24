//! File: domain/drawing/src/brush/dynamics.rs
//! Purpose: Explicit brush dynamics data.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DynamicsCurve {
    pub enabled: bool,
    pub minimum_scale: f32,
    pub gamma: f32,
}

impl DynamicsCurve {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            minimum_scale: 1.0,
            gamma: 1.0,
        }
    }

    pub const fn pressure(minimum_scale: f32, gamma: f32) -> Self {
        Self {
            enabled: true,
            minimum_scale,
            gamma,
        }
    }

    pub fn is_valid(self) -> bool {
        self.minimum_scale.is_finite()
            && self.gamma.is_finite()
            && (0.0..=1.0).contains(&self.minimum_scale)
            && self.gamma > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushDynamics {
    pub pressure_to_size: DynamicsCurve,
    pub pressure_to_opacity: DynamicsCurve,
}

impl BrushDynamics {
    pub const fn none() -> Self {
        Self {
            pressure_to_size: DynamicsCurve::disabled(),
            pressure_to_opacity: DynamicsCurve::disabled(),
        }
    }
}
