//! Editor-owned theme definition schemas.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ui_definition::UiDefinitionDiagnostic;
use ui_theme::{ThemeTokens, UiColor};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorThemeDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub colors: BTreeMap<String, String>,
    #[serde(default)]
    pub spacing: BTreeMap<String, f32>,
    #[serde(default)]
    pub typography: BTreeMap<String, EditorTypographyTokenDefinition>,
    #[serde(default)]
    pub radius: BTreeMap<String, f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorTypographyTokenDefinition {
    pub font_family: String,
    pub size: f32,
    pub weight: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorThemeFormationError {
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
}

impl EditorThemeFormationError {
    pub fn new(diagnostics: Vec<UiDefinitionDiagnostic>) -> Self {
        Self { diagnostics }
    }
}

impl std::fmt::Display for EditorThemeFormationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "editor theme formation failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl std::error::Error for EditorThemeFormationError {}

pub fn form_theme_tokens(
    definition: &EditorThemeDefinition,
    base: &ThemeTokens,
) -> Result<ThemeTokens, EditorThemeFormationError> {
    let mut formed = base.clone();
    let mut diagnostics = Vec::new();

    for (token, value) in &definition.colors {
        match apply_color_token(&mut formed, token, value) {
            Ok(()) => {}
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    for (token, value) in &definition.spacing {
        match apply_spacing_token(&mut formed, token, *value) {
            Ok(()) => {}
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    for (token, value) in &definition.radius {
        match apply_radius_token(&mut formed, token, *value) {
            Ok(()) => {}
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    for (token, value) in &definition.typography {
        match apply_typography_token(&mut formed, token, value) {
            Ok(()) => {}
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    if diagnostics.is_empty() {
        Ok(formed)
    } else {
        Err(EditorThemeFormationError::new(diagnostics))
    }
}

fn apply_color_token(
    theme: &mut ThemeTokens,
    token: &str,
    value: &str,
) -> Result<(), UiDefinitionDiagnostic> {
    let color = parse_hex_color(token, value)?;
    match token {
        "background" => theme.background = color,
        "background_panel" | "surface" => theme.background_panel = color,
        "foreground" => theme.foreground = color,
        "foreground_muted" => theme.foreground_muted = color,
        "accent" => theme.accent = color,
        "border" => theme.border = color,
        _ => {
            return Err(UiDefinitionDiagnostic::error(
                "editor.definition.theme.color.unknown_token",
                format!("unknown theme color token '{token}'"),
            ));
        }
    }
    Ok(())
}

fn apply_spacing_token(
    theme: &mut ThemeTokens,
    token: &str,
    value: f32,
) -> Result<(), UiDefinitionDiagnostic> {
    guard_non_negative_finite(
        "editor.definition.theme.spacing.invalid_value",
        "spacing",
        token,
        value,
    )?;
    match token {
        "xs" => theme.spacing.xs = value,
        "sm" | "panel_gap" => theme.spacing.sm = value,
        "md" => theme.spacing.md = value,
        "lg" => theme.spacing.lg = value,
        "xl" => theme.spacing.xl = value,
        _ => {
            return Err(UiDefinitionDiagnostic::error(
                "editor.definition.theme.spacing.unknown_token",
                format!("unknown theme spacing token '{token}'"),
            ));
        }
    }
    Ok(())
}

fn apply_radius_token(
    theme: &mut ThemeTokens,
    token: &str,
    value: f32,
) -> Result<(), UiDefinitionDiagnostic> {
    guard_non_negative_finite(
        "editor.definition.theme.radius.invalid_value",
        "radius",
        token,
        value,
    )?;
    match token {
        "sm" | "control" => theme.radius.sm = value,
        "md" => theme.radius.md = value,
        "lg" => theme.radius.lg = value,
        _ => {
            return Err(UiDefinitionDiagnostic::error(
                "editor.definition.theme.radius.unknown_token",
                format!("unknown theme radius token '{token}'"),
            ));
        }
    }
    Ok(())
}

fn apply_typography_token(
    theme: &mut ThemeTokens,
    token: &str,
    value: &EditorTypographyTokenDefinition,
) -> Result<(), UiDefinitionDiagnostic> {
    if !value.size.is_finite() || value.size <= 0.0 {
        return Err(UiDefinitionDiagnostic::error(
            "editor.definition.theme.typography.invalid_size",
            format!(
                "theme typography token '{token}' has invalid size '{}'",
                value.size
            ),
        ));
    }
    match token {
        "body" => theme.typography.body = value.size,
        "body_small" => theme.typography.body_small = value.size,
        "heading" => theme.typography.heading = value.size,
        "monospace" => theme.typography.monospace = value.size,
        _ => {
            return Err(UiDefinitionDiagnostic::error(
                "editor.definition.theme.typography.unknown_token",
                format!("unknown theme typography token '{token}'"),
            ));
        }
    }
    Ok(())
}

fn guard_non_negative_finite(
    code: &'static str,
    family: &'static str,
    token: &str,
    value: f32,
) -> Result<(), UiDefinitionDiagnostic> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(UiDefinitionDiagnostic::error(
            code,
            format!("theme {family} token '{token}' has invalid value '{value}'"),
        ))
    }
}

fn parse_hex_color(token: &str, value: &str) -> Result<UiColor, UiDefinitionDiagnostic> {
    let raw = value.trim();
    let hex = raw.strip_prefix('#').ok_or_else(|| {
        UiDefinitionDiagnostic::error(
            "editor.definition.theme.color.invalid_format",
            format!("theme color token '{token}' must use #RRGGBB or #RRGGBBAA"),
        )
    })?;
    if hex.len() != 6 && hex.len() != 8 {
        return Err(UiDefinitionDiagnostic::error(
            "editor.definition.theme.color.invalid_format",
            format!("theme color token '{token}' must use #RRGGBB or #RRGGBBAA"),
        ));
    }
    let r = parse_hex_pair(token, hex, 0)?;
    let g = parse_hex_pair(token, hex, 2)?;
    let b = parse_hex_pair(token, hex, 4)?;
    let a = if hex.len() == 8 {
        parse_hex_pair(token, hex, 6)?
    } else {
        255
    };
    Ok(UiColor::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

fn parse_hex_pair(token: &str, hex: &str, offset: usize) -> Result<u8, UiDefinitionDiagnostic> {
    u8::from_str_radix(&hex[offset..offset + 2], 16).map_err(|_| {
        UiDefinitionDiagnostic::error(
            "editor.definition.theme.color.invalid_hex",
            format!("theme color token '{token}' contains non-hex digits"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_definition_applies_accent_color_to_theme_tokens() {
        let base = ThemeTokens::default();
        let definition = EditorThemeDefinition {
            id: "test.theme".to_string(),
            label: "Test Theme".to_string(),
            colors: BTreeMap::from([("accent".to_string(), "#3366ff".to_string())]),
            spacing: BTreeMap::from([("panel_gap".to_string(), 10.0)]),
            typography: BTreeMap::new(),
            radius: BTreeMap::from([("control".to_string(), 6.0)]),
        };

        let formed = form_theme_tokens(&definition, &base).expect("theme should form");

        assert_eq!(formed.accent, UiColor::new(0.2, 0.4, 1.0, 1.0));
        assert_eq!(formed.spacing.sm, 10.0);
        assert_eq!(formed.radius.sm, 6.0);
        assert_eq!(formed.background, base.background);
    }

    #[test]
    fn theme_definition_rejects_malformed_color_token() {
        let definition = EditorThemeDefinition {
            id: "test.theme".to_string(),
            label: "Test Theme".to_string(),
            colors: BTreeMap::from([("accent".to_string(), "3366ff".to_string())]),
            spacing: BTreeMap::new(),
            typography: BTreeMap::new(),
            radius: BTreeMap::new(),
        };

        let error = form_theme_tokens(&definition, &ThemeTokens::default())
            .expect_err("malformed color should be rejected");

        assert_eq!(error.diagnostics.len(), 1);
        assert_eq!(
            error.diagnostics[0].code,
            "editor.definition.theme.color.invalid_format"
        );
    }
}
