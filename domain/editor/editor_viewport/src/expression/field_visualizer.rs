//! File: domain/editor/editor_viewport/src/expression/field_visualizer.rs
//! Purpose: Viewport-owned field visualizer presentation settings.

use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ViewportFieldVisualizerComponent {
    #[default]
    Auto,
    X,
    Y,
    Z,
    W,
    Magnitude,
}

impl ViewportFieldVisualizerComponent {
    pub const ALL: [Self; 6] = [
        Self::Auto,
        Self::X,
        Self::Y,
        Self::Z,
        Self::W,
        Self::Magnitude,
    ];

    pub const fn display_label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::W => "W",
            Self::Magnitude => "Magnitude",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ViewportFieldVisualizerColorRamp {
    #[default]
    Grayscale,
    Heat,
    DivergingSigned,
}

impl ViewportFieldVisualizerColorRamp {
    pub const ALL: [Self; 3] = [Self::Grayscale, Self::Heat, Self::DivergingSigned];

    pub const fn display_label(self) -> &'static str {
        match self {
            Self::Grayscale => "Grayscale",
            Self::Heat => "Heat",
            Self::DivergingSigned => "Diverging Signed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ViewportFieldVisualizerDebugMode {
    #[default]
    Values,
    Availability,
    Freshness,
}

impl ViewportFieldVisualizerDebugMode {
    pub const ALL: [Self; 3] = [Self::Values, Self::Availability, Self::Freshness];

    pub const fn display_label(self) -> &'static str {
        match self {
            Self::Values => "Values",
            Self::Availability => "Availability",
            Self::Freshness => "Freshness",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ViewportFieldVisualizerSettings {
    pub component: ViewportFieldVisualizerComponent,
    pub slice_index: u32,
    pub color_ramp: ViewportFieldVisualizerColorRamp,
    pub debug_mode: ViewportFieldVisualizerDebugMode,
}

impl ViewportFieldVisualizerSettings {
    pub const fn with_component(mut self, component: ViewportFieldVisualizerComponent) -> Self {
        self.component = component;
        self
    }

    pub const fn with_slice_index(mut self, slice_index: u32) -> Self {
        self.slice_index = slice_index;
        self
    }

    pub const fn with_color_ramp(mut self, color_ramp: ViewportFieldVisualizerColorRamp) -> Self {
        self.color_ramp = color_ramp;
        self
    }

    pub const fn with_debug_mode(mut self, debug_mode: ViewportFieldVisualizerDebugMode) -> Self {
        self.debug_mode = debug_mode;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewportFieldVisualizerSettingsPatch {
    SetComponent(ViewportFieldVisualizerComponent),
    SetSliceIndex(u32),
    StepSliceIndex(i32),
    SetColorRamp(ViewportFieldVisualizerColorRamp),
    SetDebugMode(ViewportFieldVisualizerDebugMode),
}

impl ViewportFieldVisualizerSettingsPatch {
    pub fn apply_to(
        self,
        current: ViewportFieldVisualizerSettings,
        slice_count: Option<NonZeroU32>,
    ) -> ViewportFieldVisualizerSettings {
        let next = match self {
            Self::SetComponent(component) => current.with_component(component),
            Self::SetSliceIndex(slice_index) => current.with_slice_index(slice_index),
            Self::StepSliceIndex(delta) => current.with_slice_index(stepped_slice_index(
                current.slice_index,
                delta,
                slice_count,
            )),
            Self::SetColorRamp(color_ramp) => current.with_color_ramp(color_ramp),
            Self::SetDebugMode(debug_mode) => current.with_debug_mode(debug_mode),
        };
        next.with_slice_index(clamped_slice_index(next.slice_index, slice_count))
    }
}

fn stepped_slice_index(current: u32, delta: i32, slice_count: Option<NonZeroU32>) -> u32 {
    let next = if delta.is_negative() {
        current.saturating_sub(delta.unsigned_abs())
    } else {
        current.saturating_add(delta as u32)
    };
    clamped_slice_index(next, slice_count)
}

fn clamped_slice_index(slice_index: u32, slice_count: Option<NonZeroU32>) -> u32 {
    slice_count
        .map(|count| slice_index.min(count.get().saturating_sub(1)))
        .unwrap_or(slice_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_visualizer_settings_default_to_v1_contract() {
        let settings = ViewportFieldVisualizerSettings::default();

        assert_eq!(settings.component, ViewportFieldVisualizerComponent::Auto);
        assert_eq!(settings.slice_index, 0);
        assert_eq!(
            settings.color_ramp,
            ViewportFieldVisualizerColorRamp::Grayscale
        );
        assert_eq!(
            settings.debug_mode,
            ViewportFieldVisualizerDebugMode::Values
        );
    }

    #[test]
    fn field_visualizer_patch_merges_one_setting_without_resetting_others() {
        let current = ViewportFieldVisualizerSettings::default()
            .with_component(ViewportFieldVisualizerComponent::Magnitude)
            .with_slice_index(3)
            .with_color_ramp(ViewportFieldVisualizerColorRamp::Grayscale)
            .with_debug_mode(ViewportFieldVisualizerDebugMode::Values);

        let next = ViewportFieldVisualizerSettingsPatch::SetColorRamp(
            ViewportFieldVisualizerColorRamp::DivergingSigned,
        )
        .apply_to(current, None);

        assert_eq!(next.component, ViewportFieldVisualizerComponent::Magnitude);
        assert_eq!(next.slice_index, 3);
        assert_eq!(
            next.color_ramp,
            ViewportFieldVisualizerColorRamp::DivergingSigned
        );
        assert_eq!(next.debug_mode, ViewportFieldVisualizerDebugMode::Values);
    }

    #[test]
    fn field_visualizer_patch_clamps_slice_steps_to_metadata_bounds() {
        let current = ViewportFieldVisualizerSettings::default().with_slice_index(1);
        let slice_count = NonZeroU32::new(2);

        let stepped_up =
            ViewportFieldVisualizerSettingsPatch::StepSliceIndex(1).apply_to(current, slice_count);
        let stepped_down =
            ViewportFieldVisualizerSettingsPatch::StepSliceIndex(-4).apply_to(current, slice_count);
        let set_far =
            ViewportFieldVisualizerSettingsPatch::SetSliceIndex(99).apply_to(current, slice_count);

        assert_eq!(stepped_up.slice_index, 1);
        assert_eq!(stepped_down.slice_index, 0);
        assert_eq!(set_far.slice_index, 1);
    }

    #[test]
    fn field_visualizer_patch_leaves_unbounded_slice_metadata_open() {
        let current = ViewportFieldVisualizerSettings::default().with_slice_index(1);

        let next = ViewportFieldVisualizerSettingsPatch::StepSliceIndex(5).apply_to(current, None);

        assert_eq!(next.slice_index, 6);
    }
}
