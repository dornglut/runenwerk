//! Editor-owned theme definition schemas.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ui_definition::UiDefinitionDiagnostic;
use ui_theme::{
    ThemeTokenDeclaration, ThemeTokenFamily, ThemeTokenGraph, ThemeTokenId, ThemeTokenLayer,
    ThemeTokenResolveRequest, ThemeTokenSourceId, ThemeTokenSourcePath, ThemeTokenValue,
    ThemeTokens, UiColor, resolve_theme_tokens,
};

pub const EDITOR_THEME_TARGET_PROFILE: &str = "editor.workbench";

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
    let graph = editor_theme_token_graph(definition)?;
    let report = resolve_theme_tokens(
        &graph,
        &ThemeTokenResolveRequest::activate(EDITOR_THEME_TARGET_PROFILE),
    );
    if report.has_errors() {
        return Err(EditorThemeFormationError::new(
            report
                .diagnostics
                .into_iter()
                .map(|diagnostic| {
                    UiDefinitionDiagnostic::error(diagnostic.code, diagnostic.message)
                })
                .collect(),
        ));
    }

    Ok(report.apply_to_theme_tokens(base))
}

pub fn editor_theme_token_graph(
    definition: &EditorThemeDefinition,
) -> Result<ThemeTokenGraph, EditorThemeFormationError> {
    let source = ThemeTokenSourceId::new(definition.id.clone());
    let target_profile = ThemeTokenSourceId::new(EDITOR_THEME_TARGET_PROFILE);
    let mut declarations = Vec::new();
    let mut diagnostics = Vec::new();

    for (token, value) in &definition.colors {
        match color_token_declaration(definition, token, value, &source, &target_profile) {
            Ok(declaration) => declarations.push(declaration),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    for (token, value) in &definition.spacing {
        match number_token_declaration(
            definition,
            token,
            *value,
            ThemeTokenFamily::Spacing,
            canonical_spacing_token_id,
            &source,
            &target_profile,
        ) {
            Ok(declaration) => declarations.push(declaration),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    for (token, value) in &definition.radius {
        match number_token_declaration(
            definition,
            token,
            *value,
            ThemeTokenFamily::Radius,
            canonical_radius_token_id,
            &source,
            &target_profile,
        ) {
            Ok(declaration) => declarations.push(declaration),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    for (token, value) in &definition.typography {
        match typography_token_declaration(definition, token, value, &source, &target_profile) {
            Ok(declaration) => declarations.push(declaration),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    if diagnostics.is_empty() {
        Ok(ThemeTokenGraph::new(declarations))
    } else {
        Err(EditorThemeFormationError::new(diagnostics))
    }
}

fn color_token_declaration(
    definition: &EditorThemeDefinition,
    token: &str,
    value: &str,
    source: &ThemeTokenSourceId,
    target_profile: &ThemeTokenSourceId,
) -> Result<ThemeTokenDeclaration, UiDefinitionDiagnostic> {
    let color = parse_hex_color(token, value)?;
    let id = canonical_color_token_id(token)?;
    Ok(token_declaration(
        definition,
        token,
        id,
        ThemeTokenFamily::Color,
        ThemeTokenValue::Color(color),
        source,
        target_profile,
    ))
}

fn number_token_declaration(
    definition: &EditorThemeDefinition,
    token: &str,
    value: f32,
    family: ThemeTokenFamily,
    token_id: fn(&str) -> Result<ThemeTokenId, UiDefinitionDiagnostic>,
    source: &ThemeTokenSourceId,
    target_profile: &ThemeTokenSourceId,
) -> Result<ThemeTokenDeclaration, UiDefinitionDiagnostic> {
    guard_non_negative_finite(
        invalid_value_code(family),
        family_name(family),
        token,
        value,
    )?;
    Ok(token_declaration(
        definition,
        token,
        token_id(token)?,
        family,
        ThemeTokenValue::Number(value),
        source,
        target_profile,
    ))
}

fn typography_token_declaration(
    definition: &EditorThemeDefinition,
    token: &str,
    value: &EditorTypographyTokenDefinition,
    source: &ThemeTokenSourceId,
    target_profile: &ThemeTokenSourceId,
) -> Result<ThemeTokenDeclaration, UiDefinitionDiagnostic> {
    if !value.size.is_finite() || value.size <= 0.0 {
        return Err(UiDefinitionDiagnostic::error(
            "editor.definition.theme.typography.invalid_size",
            format!(
                "theme typography token '{token}' has invalid size '{}'",
                value.size
            ),
        ));
    }
    Ok(token_declaration(
        definition,
        token,
        canonical_typography_token_id(token)?,
        ThemeTokenFamily::Typography,
        ThemeTokenValue::Number(value.size),
        source,
        target_profile,
    ))
}

fn token_declaration(
    definition: &EditorThemeDefinition,
    source_token: &str,
    id: ThemeTokenId,
    family: ThemeTokenFamily,
    value: ThemeTokenValue,
    source: &ThemeTokenSourceId,
    target_profile: &ThemeTokenSourceId,
) -> ThemeTokenDeclaration {
    ThemeTokenDeclaration::value(id, family, ThemeTokenLayer::Theme, value, source.clone())
        .for_target_profiles([target_profile.clone()])
        .with_source_path(ThemeTokenSourcePath::new(format!(
            "editor_definition.theme.{}.{}",
            definition.id, source_token
        )))
}

fn canonical_color_token_id(token: &str) -> Result<ThemeTokenId, UiDefinitionDiagnostic> {
    match token {
        "background" => Ok(ThemeTokenId::new("color.background")),
        "background_panel" | "surface" => Ok(ThemeTokenId::new("color.background_panel")),
        "foreground" => Ok(ThemeTokenId::new("color.foreground")),
        "foreground_muted" => Ok(ThemeTokenId::new("color.foreground_muted")),
        "accent" => Ok(ThemeTokenId::new("color.accent")),
        "border" => Ok(ThemeTokenId::new("color.border")),
        _ => Err(UiDefinitionDiagnostic::error(
            "editor.definition.theme.color.unknown_token",
            format!("unknown theme color token '{token}'"),
        )),
    }
}

fn canonical_spacing_token_id(token: &str) -> Result<ThemeTokenId, UiDefinitionDiagnostic> {
    match token {
        "xs" => Ok(ThemeTokenId::new("spacing.xs")),
        "sm" | "panel_gap" => Ok(ThemeTokenId::new("spacing.sm")),
        "md" => Ok(ThemeTokenId::new("spacing.md")),
        "lg" => Ok(ThemeTokenId::new("spacing.lg")),
        "xl" => Ok(ThemeTokenId::new("spacing.xl")),
        _ => Err(UiDefinitionDiagnostic::error(
            "editor.definition.theme.spacing.unknown_token",
            format!("unknown theme spacing token '{token}'"),
        )),
    }
}

fn canonical_radius_token_id(token: &str) -> Result<ThemeTokenId, UiDefinitionDiagnostic> {
    match token {
        "sm" | "control" => Ok(ThemeTokenId::new("radius.sm")),
        "md" => Ok(ThemeTokenId::new("radius.md")),
        "lg" => Ok(ThemeTokenId::new("radius.lg")),
        _ => Err(UiDefinitionDiagnostic::error(
            "editor.definition.theme.radius.unknown_token",
            format!("unknown theme radius token '{token}'"),
        )),
    }
}

fn canonical_typography_token_id(token: &str) -> Result<ThemeTokenId, UiDefinitionDiagnostic> {
    match token {
        "body" => Ok(ThemeTokenId::new("typography.body")),
        "body_small" => Ok(ThemeTokenId::new("typography.body_small")),
        "heading" => Ok(ThemeTokenId::new("typography.heading")),
        "monospace" => Ok(ThemeTokenId::new("typography.monospace")),
        _ => Err(UiDefinitionDiagnostic::error(
            "editor.definition.theme.typography.unknown_token",
            format!("unknown theme typography token '{token}'"),
        )),
    }
}

fn invalid_value_code(family: ThemeTokenFamily) -> &'static str {
    match family {
        ThemeTokenFamily::Spacing => "editor.definition.theme.spacing.invalid_value",
        ThemeTokenFamily::Radius => "editor.definition.theme.radius.invalid_value",
        _ => "editor.definition.theme.token.invalid_value",
    }
}

fn family_name(family: ThemeTokenFamily) -> &'static str {
    match family {
        ThemeTokenFamily::Spacing => "spacing",
        ThemeTokenFamily::Radius => "radius",
        ThemeTokenFamily::Typography => "typography",
        ThemeTokenFamily::Color => "color",
        ThemeTokenFamily::Opacity => "opacity",
        ThemeTokenFamily::Elevation => "elevation",
        ThemeTokenFamily::BorderWidth => "border_width",
        ThemeTokenFamily::Duration => "duration",
        ThemeTokenFamily::Easing => "easing",
    }
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
    fn theme_definition_exposes_generic_token_graph_provenance() {
        let definition = EditorThemeDefinition {
            id: "test.theme".to_string(),
            label: "Test Theme".to_string(),
            colors: BTreeMap::from([("accent".to_string(), "#3366ff".to_string())]),
            spacing: BTreeMap::from([("panel_gap".to_string(), 10.0)]),
            typography: BTreeMap::from([(
                "body".to_string(),
                EditorTypographyTokenDefinition {
                    font_family: "Inter".to_string(),
                    size: 13.0,
                    weight: 400,
                },
            )]),
            radius: BTreeMap::from([("control".to_string(), 6.0)]),
        };

        let graph = editor_theme_token_graph(&definition).expect("token graph should form");
        let report = resolve_theme_tokens(
            &graph,
            &ThemeTokenResolveRequest::activate(EDITOR_THEME_TARGET_PROFILE),
        );

        assert!(!report.has_errors(), "{:?}", report.diagnostics);
        let accent = report
            .tokens
            .get(&ThemeTokenId::new("color.accent"))
            .expect("accent token should resolve");
        assert_eq!(accent.winning_source.as_str(), "test.theme");
        assert_eq!(
            accent.value,
            ThemeTokenValue::Color(UiColor::new(0.2, 0.4, 1.0, 1.0))
        );
        assert!(report.tokens.contains_key(&ThemeTokenId::new("spacing.sm")));
        assert!(report.tokens.contains_key(&ThemeTokenId::new("radius.sm")));
        assert!(
            report
                .tokens
                .contains_key(&ThemeTokenId::new("typography.body"))
        );
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
