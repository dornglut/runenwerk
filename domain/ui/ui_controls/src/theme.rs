//! File: domain/ui/ui_controls/src/theme.rs
//! Crate: ui_controls

use serde::{Deserialize, Serialize};

use crate::package::ids::ControlKindId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlThemeTokenKind {
    Color,
    Spacing,
    Typography,
    Radius,
    Border,
    Opacity,
    Elevation,
}

impl ControlThemeTokenKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Spacing => "spacing",
            Self::Typography => "typography",
            Self::Radius => "radius",
            Self::Border => "border",
            Self::Opacity => "opacity",
            Self::Elevation => "elevation",
        }
    }
}
