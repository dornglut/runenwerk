pub const REGION_COMPASS_MINIMUM_TOUCH_TARGET: f32 = 44.0;
pub const REGION_COMPASS_MAXIMUM_TEXT_SCALE: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegionCompassAccessibility {
    pub text_scale: f32,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub minimum_hit_size: f32,
}

impl RegionCompassAccessibility {
    pub fn validate(self) -> bool {
        self.text_scale.is_finite()
            && (1.0..=REGION_COMPASS_MAXIMUM_TEXT_SCALE).contains(&self.text_scale)
            && self.minimum_hit_size.is_finite()
            && self.minimum_hit_size >= REGION_COMPASS_MINIMUM_TOUCH_TARGET
    }

    pub const fn transition_duration_ms(self) -> u16 {
        if self.reduced_motion { 0 } else { 90 }
    }
}

impl Default for RegionCompassAccessibility {
    fn default() -> Self {
        Self {
            text_scale: 1.0,
            high_contrast: false,
            reduced_motion: false,
            minimum_hit_size: REGION_COMPASS_MINIMUM_TOUCH_TARGET,
        }
    }
}
