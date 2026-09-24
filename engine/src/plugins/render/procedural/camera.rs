use crate::plugins::render::GpuUniform;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProceduralCamera2dFixedAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProceduralCamera2dAspectPolicy {
    FillViewport {
        fixed_axis: ProceduralCamera2dFixedAxis,
    },
}

impl ProceduralCamera2dAspectPolicy {
    pub const fn fill_viewport(fixed_axis: ProceduralCamera2dFixedAxis) -> Self {
        Self::FillViewport { fixed_axis }
    }

    pub const fn fill_viewport_vertical() -> Self {
        Self::fill_viewport(ProceduralCamera2dFixedAxis::Vertical)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProceduralCamera2d {
    pub center: [f32; 2],
    pub fixed_axis_world_span: f32,
    pub aspect_policy: ProceduralCamera2dAspectPolicy,
}

impl ProceduralCamera2d {
    pub const fn fill_viewport(
        center: [f32; 2],
        fixed_axis: ProceduralCamera2dFixedAxis,
        fixed_axis_world_span: f32,
    ) -> Self {
        Self {
            center,
            fixed_axis_world_span,
            aspect_policy: ProceduralCamera2dAspectPolicy::fill_viewport(fixed_axis),
        }
    }

    pub const fn fill_viewport_vertical(center: [f32; 2], visible_world_height: f32) -> Self {
        Self::fill_viewport(
            center,
            ProceduralCamera2dFixedAxis::Vertical,
            visible_world_height,
        )
    }

    pub fn uniform_for_surface(
        self,
        surface_size: (u32, u32),
    ) -> Result<ProceduralCamera2dUniform, ProceduralCamera2dError> {
        let visible_world = self.visible_world_size(surface_size)?;
        let surface_width = surface_size.0 as f32;
        let surface_height = surface_size.1 as f32;
        let scale_x = 2.0 / visible_world[0];
        let scale_y = -2.0 / visible_world[1];

        Ok(ProceduralCamera2dUniform {
            world_to_clip: [
                scale_x,
                scale_y,
                -self.center[0] * scale_x,
                -self.center[1] * scale_y,
            ],
            viewport: [
                surface_width,
                surface_height,
                1.0 / surface_width,
                1.0 / surface_height,
            ],
            visible_world: [
                self.center[0],
                self.center[1],
                visible_world[0],
                visible_world[1],
            ],
        })
    }

    pub fn visible_world_size(
        self,
        surface_size: (u32, u32),
    ) -> Result<[f32; 2], ProceduralCamera2dError> {
        validate_surface(surface_size)?;
        validate_finite_positive("fixed_axis_world_span", self.fixed_axis_world_span)?;
        validate_finite("center.x", self.center[0])?;
        validate_finite("center.y", self.center[1])?;

        let aspect = surface_size.0 as f32 / surface_size.1 as f32;
        match self.aspect_policy {
            ProceduralCamera2dAspectPolicy::FillViewport {
                fixed_axis: ProceduralCamera2dFixedAxis::Vertical,
            } => Ok([
                self.fixed_axis_world_span * aspect,
                self.fixed_axis_world_span,
            ]),
            ProceduralCamera2dAspectPolicy::FillViewport {
                fixed_axis: ProceduralCamera2dFixedAxis::Horizontal,
            } => Ok([
                self.fixed_axis_world_span,
                self.fixed_axis_world_span / aspect,
            ]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, GpuUniform)]
pub struct ProceduralCamera2dUniform {
    pub world_to_clip: [f32; 4],
    pub viewport: [f32; 4],
    pub visible_world: [f32; 4],
}

impl ProceduralCamera2dUniform {
    pub fn clip_position(self, world_position: [f32; 2]) -> [f32; 2] {
        [
            world_position[0] * self.world_to_clip[0] + self.world_to_clip[2],
            world_position[1] * self.world_to_clip[1] + self.world_to_clip[3],
        ]
    }

    pub fn world_units_per_pixel(self) -> [f32; 2] {
        [
            self.visible_world[2] * self.viewport[2],
            self.visible_world[3] * self.viewport[3],
        ]
    }

    pub fn projected_pixels_per_world_unit(self) -> [f32; 2] {
        [
            self.viewport[0] / self.visible_world[2],
            self.viewport[1] / self.visible_world[3],
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProceduralSpriteSizing {
    WorldUnits {
        half_extents: [f32; 2],
    },
    Pixels {
        radius_px: f32,
        half_extents: [f32; 2],
    },
}

impl ProceduralSpriteSizing {
    pub const fn world_units(half_width: f32, half_height: f32) -> Self {
        Self::WorldUnits {
            half_extents: [half_width, half_height],
        }
    }

    pub const fn pixels(radius_px: f32, half_width: f32, half_height: f32) -> Self {
        Self::Pixels {
            radius_px,
            half_extents: [half_width, half_height],
        }
    }

    pub fn to_uniform_sprite(
        self,
        camera: ProceduralCamera2dUniform,
    ) -> Result<[f32; 4], ProceduralCamera2dError> {
        match self {
            Self::WorldUnits { half_extents } => {
                validate_half_extents(half_extents)?;
                Ok([half_extents[0], half_extents[1], 0.0, 0.0])
            }
            Self::Pixels {
                radius_px,
                half_extents,
            } => {
                validate_finite_positive("radius_px", radius_px)?;
                validate_half_extents(half_extents)?;
                let world_units_per_pixel = camera.world_units_per_pixel();
                Ok([
                    radius_px * half_extents[0] * world_units_per_pixel[0],
                    radius_px * half_extents[1] * world_units_per_pixel[1],
                    1.0,
                    0.0,
                ])
            }
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ProceduralCamera2dError {
    #[error("procedural camera surface width and height must be non-zero, got {width}x{height}")]
    EmptySurface { width: u32, height: u32 },
    #[error("procedural camera {field} must be finite, got {value}")]
    NonFinite { field: &'static str, value: f32 },
    #[error("procedural camera {field} must be positive, got {value}")]
    NonPositive { field: &'static str, value: f32 },
}

fn validate_surface(surface_size: (u32, u32)) -> Result<(), ProceduralCamera2dError> {
    if surface_size.0 == 0 || surface_size.1 == 0 {
        return Err(ProceduralCamera2dError::EmptySurface {
            width: surface_size.0,
            height: surface_size.1,
        });
    }
    Ok(())
}

fn validate_finite(field: &'static str, value: f32) -> Result<(), ProceduralCamera2dError> {
    if !value.is_finite() {
        return Err(ProceduralCamera2dError::NonFinite { field, value });
    }
    Ok(())
}

fn validate_finite_positive(
    field: &'static str,
    value: f32,
) -> Result<(), ProceduralCamera2dError> {
    validate_finite(field, value)?;
    if value <= 0.0 {
        return Err(ProceduralCamera2dError::NonPositive { field, value });
    }
    Ok(())
}

fn validate_half_extents(half_extents: [f32; 2]) -> Result<(), ProceduralCamera2dError> {
    validate_finite_positive("half_width", half_extents[0])?;
    validate_finite_positive("half_height", half_extents[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_fill_viewport(surface: (u32, u32), expected_visible_world: [f32; 2]) {
        let camera = ProceduralCamera2d::fill_viewport_vertical([0.5, 0.5], 1.0);
        let uniform = camera
            .uniform_for_surface(surface)
            .expect("surface should project");

        assert_eq!(
            [uniform.visible_world[2], uniform.visible_world[3]],
            expected_visible_world
        );
        let pixels_per_world = uniform.projected_pixels_per_world_unit();
        assert!(
            (pixels_per_world[0] - pixels_per_world[1]).abs() <= 0.0001,
            "x/y world scale should match for {surface:?}: {pixels_per_world:?}"
        );
        assert_eq!(uniform.clip_position([0.5, 0.5]), [0.0, 0.0]);
    }

    #[test]
    fn fill_viewport_vertical_projects_landscape_without_world_scale_skew() {
        assert_fill_viewport((1600, 900), [1600.0 / 900.0, 1.0]);
    }

    #[test]
    fn fill_viewport_vertical_projects_portrait_without_world_scale_skew() {
        assert_fill_viewport((900, 1600), [900.0 / 1600.0, 1.0]);
    }

    #[test]
    fn fill_viewport_vertical_projects_square_without_world_scale_skew() {
        assert_fill_viewport((1024, 1024), [1.0, 1.0]);
    }

    #[test]
    fn fill_viewport_vertical_projects_extreme_aspect_without_world_scale_skew() {
        assert_fill_viewport((3200, 360), [3200.0 / 360.0, 1.0]);
    }

    #[test]
    fn empty_surface_fails_explicitly() {
        let camera = ProceduralCamera2d::fill_viewport_vertical([0.5, 0.5], 1.0);
        assert_eq!(
            camera.uniform_for_surface((0, 720)),
            Err(ProceduralCamera2dError::EmptySurface {
                width: 0,
                height: 720,
            })
        );
    }

    #[test]
    fn pixel_sprite_sizing_converts_to_world_units() {
        let camera = ProceduralCamera2d::fill_viewport_vertical([0.5, 0.5], 1.0);
        let uniform = camera
            .uniform_for_surface((1600, 900))
            .expect("surface should project");
        let sprite = ProceduralSpriteSizing::pixels(9.0, 0.5, 1.0)
            .to_uniform_sprite(uniform)
            .expect("pixel sprite should convert");

        assert!((sprite[0] - 0.005).abs() <= 0.00001);
        assert!((sprite[1] - 0.010).abs() <= 0.00001);
        assert_eq!(sprite[2], 1.0);
    }
}
