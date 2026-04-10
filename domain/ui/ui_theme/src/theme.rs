//! File: domain/ui/ui_theme/src/theme.rs
//! Purpose: Root theme token container.

use ui_text::{FontId, TextAlign, TextOverflow, TextStyle, TextWrap};

use crate::{RadiusScale, SpacingScale, TypographyScale, UiColor};

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeTokens {
    pub background: UiColor,
    pub background_panel: UiColor,
    pub foreground: UiColor,
    pub foreground_muted: UiColor,
    pub accent: UiColor,
    pub border: UiColor,
    pub border_width: f32,
    pub spacing: SpacingScale,
    pub radius: RadiusScale,
    pub typography: TypographyScale,
}

impl ThemeTokens {
    pub fn body_text_style(&self, font_id: FontId) -> TextStyle {
        TextStyle {
            font_id,
            font_size: self.typography.body,
            color: [
                self.foreground.r,
                self.foreground.g,
                self.foreground.b,
                self.foreground.a,
            ],
            line_height: Some((self.typography.body * 1.35).max(1.0)),
            align: TextAlign::Start,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        }
    }

    pub fn body_small_text_style(&self, font_id: FontId) -> TextStyle {
        TextStyle {
            font_id,
            font_size: self.typography.body_small,
            color: [
                self.foreground_muted.r,
                self.foreground_muted.g,
                self.foreground_muted.b,
                self.foreground_muted.a,
            ],
            line_height: Some((self.typography.body_small * 1.35).max(1.0)),
            align: TextAlign::Start,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        }
    }

    pub fn heading_text_style(&self, font_id: FontId) -> TextStyle {
        TextStyle {
            font_id,
            font_size: self.typography.heading,
            color: [
                self.foreground.r,
                self.foreground.g,
                self.foreground.b,
                self.foreground.a,
            ],
            line_height: Some((self.typography.heading * 1.25).max(1.0)),
            align: TextAlign::Start,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        }
    }

    pub fn monospace_text_style(&self, font_id: FontId) -> TextStyle {
        TextStyle {
            font_id,
            font_size: self.typography.monospace,
            color: [
                self.foreground.r,
                self.foreground.g,
                self.foreground.b,
                self.foreground.a,
            ],
            line_height: Some((self.typography.monospace * 1.35).max(1.0)),
            align: TextAlign::Start,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        }
    }

    pub fn scaled_by(&self, scale: f32) -> Self {
        let factor = scale.max(0.1);
        Self {
            background: self.background,
            background_panel: self.background_panel,
            foreground: self.foreground,
            foreground_muted: self.foreground_muted,
            accent: self.accent,
            border: self.border,
            border_width: (self.border_width * factor).clamp(0.5, 4.0),
            spacing: SpacingScale {
                xs: self.spacing.xs * factor,
                sm: self.spacing.sm * factor,
                md: self.spacing.md * factor,
                lg: self.spacing.lg * factor,
                xl: self.spacing.xl * factor,
            },
            radius: RadiusScale {
                sm: self.radius.sm * factor,
                md: self.radius.md * factor,
                lg: self.radius.lg * factor,
            },
            typography: TypographyScale {
                body: self.typography.body * factor,
                body_small: self.typography.body_small * factor,
                heading: self.typography.heading * factor,
                monospace: self.typography.monospace * factor,
            },
        }
    }
}

impl Default for ThemeTokens {
    fn default() -> Self {
        Self {
            background: UiColor::new(0.08, 0.09, 0.11, 1.0),
            background_panel: UiColor::new(0.12, 0.13, 0.16, 1.0),
            foreground: UiColor::new(0.92, 0.93, 0.95, 1.0),
            foreground_muted: UiColor::new(0.70, 0.73, 0.78, 1.0),
            accent: UiColor::new(0.38, 0.58, 0.95, 1.0),
            border: UiColor::new(0.24, 0.26, 0.32, 1.0),
            border_width: 1.0,
            spacing: SpacingScale {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 16.0,
                xl: 24.0,
            },
            radius: RadiusScale {
                sm: 4.0,
                md: 8.0,
                lg: 12.0,
            },
            typography: TypographyScale {
                body: 14.0,
                body_small: 12.0,
                heading: 18.0,
                monospace: 13.0,
            },
        }
    }
}
