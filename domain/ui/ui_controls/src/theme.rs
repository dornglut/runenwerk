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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlThemeTokenRole {
    Base,
    Accent,
    Surface,
    Text,
    Border,
    FocusRing,
    Overlay,
    Feedback,
}

impl ControlThemeTokenRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Accent => "accent",
            Self::Surface => "surface",
            Self::Text => "text",
            Self::Border => "border",
            Self::FocusRing => "focus-ring",
            Self::Overlay => "overlay",
            Self::Feedback => "feedback",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlThemeTokenRequirement {
    pub token_id: String,
    pub kind: ControlThemeTokenKind,
    pub role: ControlThemeTokenRole,
    #[serde(default = "default_required")]
    pub required: bool,
    #[serde(default)]
    pub fallback_token_id: Option<String>,
    #[serde(default)]
    pub notes: String,
}

impl ControlThemeTokenRequirement {
    pub fn new(
        token_id: impl Into<String>,
        kind: ControlThemeTokenKind,
        role: ControlThemeTokenRole,
    ) -> Self {
        Self {
            token_id: token_id.into(),
            kind,
            role,
            required: true,
            fallback_token_id: None,
            notes: String::new(),
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_fallback(mut self, fallback_token_id: impl Into<String>) -> Self {
        self.fallback_token_id = Some(fallback_token_id.into());
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlVisualState {
    Normal,
    Hover,
    Pressed,
    Focused,
    Selected,
    Disabled,
    Error,
    Warning,
    Info,
    Active,
    Loading,
    ReadOnly,
}

impl ControlVisualState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Hover => "hover",
            Self::Pressed => "pressed",
            Self::Focused => "focused",
            Self::Selected => "selected",
            Self::Disabled => "disabled",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Active => "active",
            Self::Loading => "loading",
            Self::ReadOnly => "read-only",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlVisualStateRequirement {
    pub state: ControlVisualState,
    #[serde(default = "default_required")]
    pub required: bool,
    #[serde(default)]
    pub notes: String,
}

impl ControlVisualStateRequirement {
    pub fn new(state: ControlVisualState) -> Self {
        Self {
            state,
            required: true,
            notes: String::new(),
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }
}

fn default_required() -> bool {
    true
}
