//! Generic UI theme token graph and deterministic resolution.

use crate::{ThemeTokens, UiColor};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ThemeTokenId(String);

impl ThemeTokenId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ThemeTokenId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for ThemeTokenId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ThemeTokenSourceId(String);

impl ThemeTokenSourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ThemeTokenSourceId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for ThemeTokenSourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ThemeTokenSourcePath(String);

impl ThemeTokenSourcePath {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ThemeTokenSourcePath {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for ThemeTokenSourcePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub type ThemeTargetProfileId = ThemeTokenSourceId;
pub type ThemeModeId = ThemeTokenSourceId;
pub type ThemeStateVariantId = ThemeTokenSourceId;
pub type ThemePlatformId = ThemeTokenSourceId;
pub type ThemeAccessibilityModeId = ThemeTokenSourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ThemeTokenLayer {
    Primitive,
    Semantic,
    Component,
    State,
    Mode,
    Theme,
    Skin,
    Platform,
    Accessibility,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ThemeTokenFamily {
    Color,
    Spacing,
    Radius,
    Typography,
    Opacity,
    Elevation,
    BorderWidth,
    Duration,
    Easing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThemeTokenValue {
    Color(UiColor),
    Number(f32),
    Text(String),
}

impl ThemeTokenValue {
    pub fn family(&self) -> ThemeTokenFamily {
        match self {
            Self::Color(_) => ThemeTokenFamily::Color,
            Self::Number(_) => ThemeTokenFamily::Spacing,
            Self::Text(_) => ThemeTokenFamily::Easing,
        }
    }

    fn is_valid_for_family(&self, family: ThemeTokenFamily) -> bool {
        match (family, self) {
            (ThemeTokenFamily::Color, Self::Color(value)) => [value.r, value.g, value.b, value.a]
                .into_iter()
                .all(|component| component.is_finite() && (0.0..=1.0).contains(&component)),
            (
                ThemeTokenFamily::Spacing
                | ThemeTokenFamily::Radius
                | ThemeTokenFamily::Typography
                | ThemeTokenFamily::Opacity
                | ThemeTokenFamily::Elevation
                | ThemeTokenFamily::BorderWidth
                | ThemeTokenFamily::Duration,
                Self::Number(value),
            ) => value.is_finite() && *value >= 0.0,
            (ThemeTokenFamily::Easing, Self::Text(value)) => !value.trim().is_empty(),
            _ => false,
        }
    }

    fn validation_error_code(&self, family: ThemeTokenFamily) -> Option<&'static str> {
        let kind_matches = match (family, self) {
            (ThemeTokenFamily::Color, Self::Color(_)) => true,
            (
                ThemeTokenFamily::Spacing
                | ThemeTokenFamily::Radius
                | ThemeTokenFamily::Typography
                | ThemeTokenFamily::Opacity
                | ThemeTokenFamily::Elevation
                | ThemeTokenFamily::BorderWidth
                | ThemeTokenFamily::Duration,
                Self::Number(_),
            ) => true,
            (ThemeTokenFamily::Easing, Self::Text(_)) => true,
            _ => false,
        };

        if !kind_matches {
            Some("ui.theme.token.family_mismatch")
        } else if !self.is_valid_for_family(family) {
            Some("ui.theme.token.malformed_value")
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThemeTokenValueSource {
    Value(ThemeTokenValue),
    Alias(ThemeTokenId),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ThemeTokenSelector {
    pub component: Option<ThemeTokenSourceId>,
    pub state: Option<ThemeStateVariantId>,
    pub mode: Option<ThemeModeId>,
    pub platform: Option<ThemePlatformId>,
    pub accessibility: Option<ThemeAccessibilityModeId>,
}

impl ThemeTokenSelector {
    pub fn matches(&self, request: &ThemeTokenResolveRequest) -> bool {
        selector_matches(&self.component, &request.component)
            && selector_matches(&self.state, &request.state)
            && selector_matches(&self.mode, &request.mode)
            && selector_matches(&self.platform, &request.platform)
            && selector_matches(&self.accessibility, &request.accessibility)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeTokenDeclaration {
    pub id: ThemeTokenId,
    pub family: ThemeTokenFamily,
    pub layer: ThemeTokenLayer,
    pub value: ThemeTokenValueSource,
    pub source: ThemeTokenSourceId,
    pub source_path: Option<ThemeTokenSourcePath>,
    pub target_profiles: Vec<ThemeTargetProfileId>,
    pub selector: ThemeTokenSelector,
    pub preview_only: bool,
}

impl ThemeTokenDeclaration {
    pub fn value(
        id: impl Into<ThemeTokenId>,
        family: ThemeTokenFamily,
        layer: ThemeTokenLayer,
        value: ThemeTokenValue,
        source: impl Into<ThemeTokenSourceId>,
    ) -> Self {
        Self {
            id: id.into(),
            family,
            layer,
            value: ThemeTokenValueSource::Value(value),
            source: source.into(),
            source_path: None,
            target_profiles: Vec::new(),
            selector: ThemeTokenSelector::default(),
            preview_only: false,
        }
    }

    pub fn alias(
        id: impl Into<ThemeTokenId>,
        family: ThemeTokenFamily,
        layer: ThemeTokenLayer,
        alias: impl Into<ThemeTokenId>,
        source: impl Into<ThemeTokenSourceId>,
    ) -> Self {
        Self {
            id: id.into(),
            family,
            layer,
            value: ThemeTokenValueSource::Alias(alias.into()),
            source: source.into(),
            source_path: None,
            target_profiles: Vec::new(),
            selector: ThemeTokenSelector::default(),
            preview_only: false,
        }
    }

    pub fn for_target_profiles(
        mut self,
        profiles: impl IntoIterator<Item = ThemeTargetProfileId>,
    ) -> Self {
        self.target_profiles = profiles.into_iter().collect();
        self
    }

    pub fn with_selector(mut self, selector: ThemeTokenSelector) -> Self {
        self.selector = selector;
        self
    }

    pub fn with_source_path(mut self, source_path: impl Into<ThemeTokenSourcePath>) -> Self {
        self.source_path = Some(source_path.into());
        self
    }

    pub fn preview_only(mut self) -> Self {
        self.preview_only = true;
        self
    }

    fn supports_profile(&self, profile: &ThemeTargetProfileId) -> bool {
        self.target_profiles.is_empty() || self.target_profiles.contains(profile)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThemeTokenGraph {
    pub declarations: Vec<ThemeTokenDeclaration>,
}

impl ThemeTokenGraph {
    pub fn new(declarations: Vec<ThemeTokenDeclaration>) -> Self {
        Self { declarations }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeTokenActivationMode {
    Preview,
    Activate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeTokenResolveRequest {
    pub target_profile: ThemeTargetProfileId,
    pub component: Option<ThemeTokenSourceId>,
    pub state: Option<ThemeStateVariantId>,
    pub mode: Option<ThemeModeId>,
    pub platform: Option<ThemePlatformId>,
    pub accessibility: Option<ThemeAccessibilityModeId>,
    pub activation: ThemeTokenActivationMode,
}

impl ThemeTokenResolveRequest {
    pub fn activate(target_profile: impl Into<ThemeTargetProfileId>) -> Self {
        Self {
            target_profile: target_profile.into(),
            component: None,
            state: None,
            mode: None,
            platform: None,
            accessibility: None,
            activation: ThemeTokenActivationMode::Activate,
        }
    }

    pub fn preview(target_profile: impl Into<ThemeTargetProfileId>) -> Self {
        Self {
            activation: ThemeTokenActivationMode::Preview,
            ..Self::activate(target_profile)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeTokenDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeTokenActivationImpact {
    None,
    BlocksActivation,
    PreviewOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeTokenDiagnostic {
    pub severity: ThemeTokenDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub token: Option<ThemeTokenId>,
    pub target_profile: ThemeTargetProfileId,
    pub owning_domain: &'static str,
    pub source: Option<ThemeTokenSourceId>,
    pub source_path: Option<ThemeTokenSourcePath>,
    pub alias_path: Vec<ThemeTokenId>,
    pub winning_source: Option<ThemeTokenSourceId>,
    pub losing_sources: Vec<ThemeTokenSourceId>,
    pub activation_impact: ThemeTokenActivationImpact,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedThemeToken {
    pub id: ThemeTokenId,
    pub family: ThemeTokenFamily,
    pub value: ThemeTokenValue,
    pub winning_source: ThemeTokenSourceId,
    pub losing_sources: Vec<ThemeTokenSourceId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeTokenResolutionReport {
    pub tokens: BTreeMap<ThemeTokenId, ResolvedThemeToken>,
    pub diagnostics: Vec<ThemeTokenDiagnostic>,
}

impl ThemeTokenResolutionReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == ThemeTokenDiagnosticSeverity::Error)
    }

    pub fn apply_to_theme_tokens(&self, base: &ThemeTokens) -> ThemeTokens {
        let mut formed = base.clone();
        for token in self.tokens.values() {
            apply_known_token(&mut formed, token);
        }
        formed
    }
}

pub fn resolve_theme_tokens(
    graph: &ThemeTokenGraph,
    request: &ThemeTokenResolveRequest,
) -> ThemeTokenResolutionReport {
    let mut diagnostics = Vec::new();
    let mut active = BTreeMap::<ThemeTokenId, ActiveThemeToken>::new();
    let mut active_selectors = BTreeMap::<
        (ThemeTokenId, ThemeTokenLayer, ThemeTokenSelector),
        ThemeTokenDeclaration,
    >::new();
    let mut declarations = graph.declarations.clone();
    declarations.sort_by(|left, right| {
        left.layer
            .cmp(&right.layer)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.source.cmp(&right.source))
    });

    for declaration in declarations {
        if (declaration.preview_only || declaration.layer == ThemeTokenLayer::Preview)
            && request.activation == ThemeTokenActivationMode::Activate
        {
            let diagnostic = error_for_declaration(
                "ui.theme.preview_only_activation",
                format!(
                    "preview-only token '{}' cannot be activated",
                    declaration.id
                ),
                &declaration,
                request,
                "convert the preview override into a deterministic theme package before activation",
            )
            .with_activation_impact(ThemeTokenActivationImpact::PreviewOnly);
            diagnostics.push(diagnostic);
            continue;
        }
        if !declaration.supports_profile(&request.target_profile) {
            diagnostics.push(error_for_declaration(
                "ui.theme.target_profile.unsupported",
                format!(
                    "token '{}' does not support target profile '{}'",
                    declaration.id, request.target_profile
                ),
                &declaration,
                request,
                "add an explicit compatible token declaration for the requested target profile",
            ));
            continue;
        }
        if let Some(diagnostic) = validate_selector_compatibility(&declaration, request) {
            diagnostics.push(diagnostic);
            continue;
        }
        if !declaration.selector.matches(request) {
            continue;
        }
        if declaration.layer == ThemeTokenLayer::Accessibility
            && active
                .get(&declaration.id)
                .is_some_and(|token| token.declaration.layer == ThemeTokenLayer::Accessibility)
        {
            diagnostics.push(error_for_declaration(
                "ui.theme.accessibility.conflict",
                format!(
                    "multiple accessibility overrides resolve token '{}'",
                    declaration.id
                ),
                &declaration,
                request,
                "make accessibility overrides mutually exclusive or assign a deterministic layer",
            ));
            continue;
        }
        if let ThemeTokenValueSource::Value(value) = &declaration.value {
            if let Some(code) = value.validation_error_code(declaration.family) {
                diagnostics.push(error_for_declaration(
                    code,
                    format!(
                        "token '{}' value does not match family {:?}",
                        declaration.id, declaration.family
                    ),
                    &declaration,
                    request,
                    "use a token value compatible with the declared family",
                ));
                continue;
            }
        }
        let selector_key = (
            declaration.id.clone(),
            declaration.layer,
            declaration.selector.clone(),
        );
        if let Some(previous) = active_selectors.insert(selector_key, declaration.clone()) {
            diagnostics.push(
                error_for_declaration(
                    "ui.theme.selector.duplicate",
                    format!(
                        "token '{}' has duplicate selectors at layer {:?}",
                        declaration.id, declaration.layer
                    ),
                    &declaration,
                    request,
                    "make duplicate selectors mutually exclusive or move one declaration to a higher precedence layer",
                )
                .with_sources(Some(previous.source), vec![declaration.source.clone()]),
            );
            continue;
        }
        let losing_sources = active
            .remove(&declaration.id)
            .map(|previous| {
                let mut sources = previous.losing_sources;
                sources.push(previous.declaration.source);
                sources
            })
            .unwrap_or_default();

        active.insert(
            declaration.id.clone(),
            ActiveThemeToken {
                losing_sources,
                declaration,
            },
        );
    }

    let mut resolved = BTreeMap::new();
    for id in active.keys() {
        let mut chain = Vec::new();
        if let Some(token) = resolve_token(id, &active, request, &mut chain, &mut diagnostics) {
            resolved.insert(id.clone(), token);
        }
    }

    ThemeTokenResolutionReport {
        tokens: resolved,
        diagnostics,
    }
}

fn resolve_token(
    id: &ThemeTokenId,
    active: &BTreeMap<ThemeTokenId, ActiveThemeToken>,
    request: &ThemeTokenResolveRequest,
    chain: &mut Vec<ThemeTokenId>,
    diagnostics: &mut Vec<ThemeTokenDiagnostic>,
) -> Option<ResolvedThemeToken> {
    if chain.contains(id) {
        let active_token = active.get(id);
        let alias_path = chain
            .iter()
            .cloned()
            .chain(std::iter::once(id.clone()))
            .collect();
        let diagnostic = match active_token {
            Some(active_token) => error_for_declaration(
                "ui.theme.alias.cycle",
                format!("theme token alias cycle includes '{id}'"),
                &active_token.declaration,
                request,
                "break the alias cycle with a concrete token value",
            ),
            None => error(
                "ui.theme.alias.cycle",
                format!("theme token alias cycle includes '{id}'"),
                Some(id.clone()),
                request,
                None,
                "break the alias cycle with a concrete token value",
            ),
        };
        diagnostics.push(diagnostic.with_alias_path(alias_path));
        return None;
    }
    let active_token = active.get(id)?;
    match &active_token.declaration.value {
        ThemeTokenValueSource::Value(value) => {
            if let Some(code) = value.validation_error_code(active_token.declaration.family) {
                diagnostics.push(error_for_declaration(
                    code,
                    format!(
                        "token '{}' value does not match family {:?}",
                        id, active_token.declaration.family
                    ),
                    &active_token.declaration,
                    request,
                    "use a token value compatible with the declared family",
                ));
                return None;
            }
            Some(ResolvedThemeToken {
                id: id.clone(),
                family: active_token.declaration.family,
                value: value.clone(),
                winning_source: active_token.declaration.source.clone(),
                losing_sources: active_token.losing_sources.clone(),
            })
        }
        ThemeTokenValueSource::Alias(alias) => {
            chain.push(id.clone());
            let resolved = match resolve_token(alias, active, request, chain, diagnostics) {
                Some(resolved) => resolved,
                None => {
                    if !active.contains_key(alias) {
                        diagnostics.push(error_for_declaration(
                            "ui.theme.alias.missing",
                            format!("token '{}' aliases missing token '{}'", id, alias),
                            &active_token.declaration,
                            request,
                            "declare the alias target or replace the alias with a concrete value",
                        ).with_alias_path(vec![id.clone(), alias.clone()]));
                    }
                    chain.pop();
                    return None;
                }
            };
            chain.pop();
            if resolved.family != active_token.declaration.family {
                diagnostics.push(
                    error_for_declaration(
                        "ui.theme.alias.family_mismatch",
                        format!(
                            "token '{}' family {:?} does not match alias '{}' family {:?}",
                            id, active_token.declaration.family, alias, resolved.family
                        ),
                        &active_token.declaration,
                        request,
                        "alias only to a token in the same family",
                    )
                    .with_alias_path(vec![id.clone(), alias.clone()]),
                );
                return None;
            }
            Some(ResolvedThemeToken {
                id: id.clone(),
                family: active_token.declaration.family,
                value: resolved.value,
                winning_source: active_token.declaration.source.clone(),
                losing_sources: {
                    let mut sources = active_token.losing_sources.clone();
                    sources.push(resolved.winning_source);
                    sources.extend(resolved.losing_sources);
                    sources
                },
            })
        }
    }
}

fn apply_known_token(theme: &mut ThemeTokens, token: &ResolvedThemeToken) {
    match (token.id.as_str(), &token.value) {
        ("color.background", ThemeTokenValue::Color(value)) => theme.background = *value,
        ("color.background_panel", ThemeTokenValue::Color(value)) => {
            theme.background_panel = *value
        }
        ("color.foreground", ThemeTokenValue::Color(value)) => theme.foreground = *value,
        ("color.foreground_muted", ThemeTokenValue::Color(value)) => {
            theme.foreground_muted = *value
        }
        ("color.accent", ThemeTokenValue::Color(value)) => theme.accent = *value,
        ("color.border", ThemeTokenValue::Color(value)) => theme.border = *value,
        ("spacing.xs", ThemeTokenValue::Number(value)) => theme.spacing.xs = *value,
        ("spacing.sm", ThemeTokenValue::Number(value)) => theme.spacing.sm = *value,
        ("spacing.md", ThemeTokenValue::Number(value)) => theme.spacing.md = *value,
        ("spacing.lg", ThemeTokenValue::Number(value)) => theme.spacing.lg = *value,
        ("spacing.xl", ThemeTokenValue::Number(value)) => theme.spacing.xl = *value,
        ("radius.sm", ThemeTokenValue::Number(value)) => theme.radius.sm = *value,
        ("radius.md", ThemeTokenValue::Number(value)) => theme.radius.md = *value,
        ("radius.lg", ThemeTokenValue::Number(value)) => theme.radius.lg = *value,
        ("typography.body", ThemeTokenValue::Number(value)) => theme.typography.body = *value,
        ("typography.body_small", ThemeTokenValue::Number(value)) => {
            theme.typography.body_small = *value
        }
        ("typography.heading", ThemeTokenValue::Number(value)) => theme.typography.heading = *value,
        ("typography.monospace", ThemeTokenValue::Number(value)) => {
            theme.typography.monospace = *value
        }
        ("border.width", ThemeTokenValue::Number(value)) => theme.border_width = *value,
        _ => {}
    }
}

fn selector_matches<T: PartialEq>(selector: &Option<T>, request: &Option<T>) -> bool {
    selector.is_none() || selector == request
}

fn validate_selector_compatibility(
    declaration: &ThemeTokenDeclaration,
    request: &ThemeTokenResolveRequest,
) -> Option<ThemeTokenDiagnostic> {
    let selector = &declaration.selector;
    let has_context_selector = selector.state.is_some()
        || selector.mode.is_some()
        || selector.platform.is_some()
        || selector.accessibility.is_some();
    let code = match declaration.layer {
        ThemeTokenLayer::Primitive | ThemeTokenLayer::Semantic => (selector.component.is_some()
            || has_context_selector)
            .then_some("ui.theme.selector.incompatible"),
        ThemeTokenLayer::Component => (selector.component.is_none() || has_context_selector)
            .then_some("ui.theme.selector.incompatible"),
        ThemeTokenLayer::State => (selector.state.is_none() || selector.mode.is_some())
            .then_some("ui.theme.selector.incompatible"),
        ThemeTokenLayer::Mode => (selector.mode.is_none() || selector.state.is_some())
            .then_some("ui.theme.selector.incompatible"),
        ThemeTokenLayer::Theme | ThemeTokenLayer::Skin => {
            has_context_selector.then_some("ui.theme.selector.incompatible")
        }
        ThemeTokenLayer::Platform => {
            (selector.platform.is_none()).then_some("ui.theme.selector.incompatible")
        }
        ThemeTokenLayer::Accessibility => {
            (selector.accessibility.is_none()).then_some("ui.theme.selector.incompatible")
        }
        ThemeTokenLayer::Preview => None,
    };

    code.map(|code| {
        error_for_declaration(
            code,
            format!(
                "token '{}' has selector fields incompatible with layer {:?}",
                declaration.id, declaration.layer
            ),
            declaration,
            request,
            "move selectors to the matching precedence layer or split the token declaration",
        )
    })
}

fn error_for_declaration(
    code: impl Into<String>,
    message: impl Into<String>,
    declaration: &ThemeTokenDeclaration,
    request: &ThemeTokenResolveRequest,
    suggested_fix: impl Into<String>,
) -> ThemeTokenDiagnostic {
    error(
        code,
        message,
        Some(declaration.id.clone()),
        request,
        Some(declaration.source.clone()),
        suggested_fix,
    )
    .with_source_path(declaration.source_path.clone())
}

fn error(
    code: impl Into<String>,
    message: impl Into<String>,
    token: Option<ThemeTokenId>,
    request: &ThemeTokenResolveRequest,
    source: Option<ThemeTokenSourceId>,
    suggested_fix: impl Into<String>,
) -> ThemeTokenDiagnostic {
    ThemeTokenDiagnostic {
        severity: ThemeTokenDiagnosticSeverity::Error,
        code: code.into(),
        message: message.into(),
        token,
        target_profile: request.target_profile.clone(),
        owning_domain: "domain/ui/ui_theme",
        source,
        source_path: None,
        alias_path: Vec::new(),
        winning_source: None,
        losing_sources: Vec::new(),
        activation_impact: ThemeTokenActivationImpact::BlocksActivation,
        suggested_fix: suggested_fix.into(),
    }
}

impl ThemeTokenDiagnostic {
    fn with_source_path(mut self, source_path: Option<ThemeTokenSourcePath>) -> Self {
        self.source_path = source_path;
        self
    }

    fn with_alias_path(mut self, alias_path: Vec<ThemeTokenId>) -> Self {
        self.alias_path = alias_path;
        self
    }

    fn with_sources(
        mut self,
        winning_source: Option<ThemeTokenSourceId>,
        losing_sources: Vec<ThemeTokenSourceId>,
    ) -> Self {
        self.winning_source = winning_source;
        self.losing_sources = losing_sources;
        self
    }

    fn with_activation_impact(mut self, activation_impact: ThemeTokenActivationImpact) -> Self {
        self.activation_impact = activation_impact;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveThemeToken {
    declaration: ThemeTokenDeclaration,
    losing_sources: Vec<ThemeTokenSourceId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_resolution_applies_deterministic_layer_order() {
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Primitive,
                ThemeTokenValue::Number(4.0),
                "base",
            ),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::State,
                ThemeTokenValue::Number(6.0),
                "hover",
            )
            .with_selector(ThemeTokenSelector {
                state: Some("hover".into()),
                ..ThemeTokenSelector::default()
            }),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Mode,
                ThemeTokenValue::Number(8.0),
                "compact",
            )
            .with_selector(ThemeTokenSelector {
                mode: Some("compact".into()),
                ..ThemeTokenSelector::default()
            }),
        ]);
        let mut request = ThemeTokenResolveRequest::activate("editor.workbench");
        request.state = Some("hover".into());
        request.mode = Some("compact".into());

        let report = resolve_theme_tokens(&graph, &request);

        assert!(!report.has_errors(), "{:?}", report.diagnostics);
        let resolved = report
            .tokens
            .get(&ThemeTokenId::from("spacing.sm"))
            .unwrap();
        assert_eq!(resolved.value, ThemeTokenValue::Number(8.0));
        assert_eq!(resolved.winning_source.as_str(), "compact");
        assert_eq!(
            resolved
                .losing_sources
                .iter()
                .map(ThemeTokenSourceId::as_str)
                .collect::<Vec<_>>(),
            ["base", "hover"]
        );
    }

    #[test]
    fn token_resolution_applies_full_precedence_order() {
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Primitive,
                ThemeTokenValue::Number(2.0),
                "primitive",
            ),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Component,
                ThemeTokenValue::Number(4.0),
                "button",
            )
            .with_selector(ThemeTokenSelector {
                component: Some("button".into()),
                ..ThemeTokenSelector::default()
            }),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::State,
                ThemeTokenValue::Number(6.0),
                "hover",
            )
            .with_selector(ThemeTokenSelector {
                state: Some("hover".into()),
                ..ThemeTokenSelector::default()
            }),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Mode,
                ThemeTokenValue::Number(8.0),
                "compact",
            )
            .with_selector(ThemeTokenSelector {
                mode: Some("compact".into()),
                ..ThemeTokenSelector::default()
            }),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Theme,
                ThemeTokenValue::Number(10.0),
                "theme",
            ),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Skin,
                ThemeTokenValue::Number(12.0),
                "skin",
            ),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Platform,
                ThemeTokenValue::Number(14.0),
                "desktop",
            )
            .with_selector(ThemeTokenSelector {
                platform: Some("desktop".into()),
                ..ThemeTokenSelector::default()
            }),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Accessibility,
                ThemeTokenValue::Number(16.0),
                "high-contrast",
            )
            .with_selector(ThemeTokenSelector {
                accessibility: Some("high_contrast".into()),
                ..ThemeTokenSelector::default()
            }),
        ]);
        let mut request = ThemeTokenResolveRequest::activate("editor.workbench");
        request.component = Some("button".into());
        request.state = Some("hover".into());
        request.mode = Some("compact".into());
        request.platform = Some("desktop".into());
        request.accessibility = Some("high_contrast".into());

        let report = resolve_theme_tokens(&graph, &request);

        assert!(!report.has_errors(), "{:?}", report.diagnostics);
        let resolved = report
            .tokens
            .get(&ThemeTokenId::from("spacing.sm"))
            .unwrap();
        assert_eq!(resolved.value, ThemeTokenValue::Number(16.0));
        assert_eq!(resolved.winning_source.as_str(), "high-contrast");
        assert_eq!(
            resolved
                .losing_sources
                .iter()
                .map(ThemeTokenSourceId::as_str)
                .collect::<Vec<_>>(),
            [
                "primitive",
                "button",
                "hover",
                "compact",
                "theme",
                "skin",
                "desktop"
            ]
        );
    }

    #[test]
    fn token_resolution_rejects_alias_cycles() {
        let graph = ThemeTokenGraph::new(vec![
            ThemeTokenDeclaration::alias(
                "color.a",
                ThemeTokenFamily::Color,
                ThemeTokenLayer::Semantic,
                "color.b",
                "base",
            ),
            ThemeTokenDeclaration::alias(
                "color.b",
                ThemeTokenFamily::Color,
                ThemeTokenLayer::Semantic,
                "color.a",
                "base",
            ),
        ]);

        let report = resolve_theme_tokens(
            &graph,
            &ThemeTokenResolveRequest::activate("editor.workbench"),
        );

        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "ui.theme.alias.cycle"),
            "{:?}",
            report.diagnostics
        );
    }

    #[test]
    fn token_resolution_rejects_missing_aliases() {
        let graph = ThemeTokenGraph::new(vec![ThemeTokenDeclaration::alias(
            "color.accent",
            ThemeTokenFamily::Color,
            ThemeTokenLayer::Semantic,
            "color.brand",
            "base",
        )]);

        let report = resolve_theme_tokens(
            &graph,
            &ThemeTokenResolveRequest::activate("editor.workbench"),
        );

        assert!(report.has_errors());
        assert_eq!(report.diagnostics[0].code, "ui.theme.alias.missing");
    }

    #[test]
    fn token_resolution_rejects_alias_family_mismatches() {
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Primitive,
                ThemeTokenValue::Number(4.0),
                "base",
            ),
            ThemeTokenDeclaration::alias(
                "color.accent",
                ThemeTokenFamily::Color,
                ThemeTokenLayer::Semantic,
                "spacing.sm",
                "semantic",
            ),
        ]);

        let report = resolve_theme_tokens(
            &graph,
            &ThemeTokenResolveRequest::activate("editor.workbench"),
        );

        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "ui.theme.alias.family_mismatch"),
            "{:?}",
            report.diagnostics
        );
    }

    #[test]
    fn token_resolution_rejects_malformed_values_with_source_path() {
        let graph = ThemeTokenGraph::new(vec![
            ThemeTokenDeclaration::value(
                "color.accent",
                ThemeTokenFamily::Color,
                ThemeTokenLayer::Primitive,
                ThemeTokenValue::Color(UiColor::new(1.2, 0.0, 0.0, 1.0)),
                "base",
            )
            .with_source_path("themes/base.ron:12"),
        ]);

        let report = resolve_theme_tokens(
            &graph,
            &ThemeTokenResolveRequest::activate("editor.workbench"),
        );

        assert!(report.has_errors());
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, "ui.theme.token.malformed_value");
        assert_eq!(
            diagnostic
                .source_path
                .as_ref()
                .map(ThemeTokenSourcePath::as_str),
            Some("themes/base.ron:12")
        );
    }

    #[test]
    fn token_resolution_rejects_duplicate_selectors() {
        let selector = ThemeTokenSelector {
            mode: Some("compact".into()),
            ..ThemeTokenSelector::default()
        };
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Mode,
                ThemeTokenValue::Number(6.0),
                "compact-a",
            )
            .with_selector(selector.clone()),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Mode,
                ThemeTokenValue::Number(8.0),
                "compact-b",
            )
            .with_selector(selector),
        ]);
        let mut request = ThemeTokenResolveRequest::activate("editor.workbench");
        request.mode = Some("compact".into());

        let report = resolve_theme_tokens(&graph, &request);

        assert!(report.has_errors());
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, "ui.theme.selector.duplicate");
        assert_eq!(
            diagnostic
                .winning_source
                .as_ref()
                .map(ThemeTokenSourceId::as_str),
            Some("compact-a")
        );
        assert_eq!(
            diagnostic
                .losing_sources
                .iter()
                .map(ThemeTokenSourceId::as_str)
                .collect::<Vec<_>>(),
            ["compact-b"]
        );
    }

    #[test]
    fn component_selectors_match_requested_component() {
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Component,
                ThemeTokenValue::Number(6.0),
                "button",
            )
            .with_selector(ThemeTokenSelector {
                component: Some("button".into()),
                ..ThemeTokenSelector::default()
            }),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Component,
                ThemeTokenValue::Number(10.0),
                "panel",
            )
            .with_selector(ThemeTokenSelector {
                component: Some("panel".into()),
                ..ThemeTokenSelector::default()
            }),
        ]);
        let mut request = ThemeTokenResolveRequest::activate("editor.workbench");
        request.component = Some("button".into());

        let report = resolve_theme_tokens(&graph, &request);

        assert!(!report.has_errors(), "{:?}", report.diagnostics);
        assert_eq!(
            report
                .tokens
                .get(&ThemeTokenId::from("spacing.sm"))
                .map(|token| &token.value),
            Some(&ThemeTokenValue::Number(6.0))
        );
    }

    #[test]
    fn token_resolution_rejects_incompatible_state_mode_selectors() {
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "spacing.sm",
                ThemeTokenLayer::State,
                ThemeTokenValue::Number(6.0),
                "hover-compact",
            )
            .with_selector(ThemeTokenSelector {
                state: Some("hover".into()),
                mode: Some("compact".into()),
                ..ThemeTokenSelector::default()
            }),
        ]);
        let mut request = ThemeTokenResolveRequest::activate("editor.workbench");
        request.state = Some("hover".into());
        request.mode = Some("compact".into());

        let report = resolve_theme_tokens(&graph, &request);

        assert!(report.has_errors());
        assert_eq!(report.diagnostics[0].code, "ui.theme.selector.incompatible");
    }

    #[test]
    fn state_variants_preview_across_target_profiles() {
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "spacing.sm",
                ThemeTokenLayer::Primitive,
                ThemeTokenValue::Number(4.0),
                "base",
            )
            .for_target_profiles(["editor.workbench".into(), "game.runtime".into()]),
            token_value(
                "spacing.sm",
                ThemeTokenLayer::State,
                ThemeTokenValue::Number(6.0),
                "hover",
            )
            .for_target_profiles(["editor.workbench".into(), "game.runtime".into()])
            .with_selector(ThemeTokenSelector {
                state: Some("hover".into()),
                ..ThemeTokenSelector::default()
            }),
        ]);
        let mut editor_request = ThemeTokenResolveRequest::preview("editor.workbench");
        editor_request.state = Some("hover".into());
        let mut runtime_request = ThemeTokenResolveRequest::preview("game.runtime");
        runtime_request.state = Some("hover".into());

        let editor_report = resolve_theme_tokens(&graph, &editor_request);
        let runtime_report = resolve_theme_tokens(&graph, &runtime_request);

        assert!(
            !editor_report.has_errors(),
            "{:?}",
            editor_report.diagnostics
        );
        assert!(
            !runtime_report.has_errors(),
            "{:?}",
            runtime_report.diagnostics
        );
        assert_eq!(
            editor_report
                .tokens
                .get(&ThemeTokenId::from("spacing.sm"))
                .map(|token| &token.value),
            Some(&ThemeTokenValue::Number(6.0))
        );
        assert_eq!(
            runtime_report
                .tokens
                .get(&ThemeTokenId::from("spacing.sm"))
                .map(|token| &token.value),
            Some(&ThemeTokenValue::Number(6.0))
        );
    }

    #[test]
    fn token_resolution_rejects_unsupported_target_profiles() {
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "color.accent",
                ThemeTokenLayer::Primitive,
                ThemeTokenValue::Color(UiColor::new(0.0, 0.2, 1.0, 1.0)),
                "base",
            )
            .for_target_profiles(["editor.workbench".into()]),
        ]);

        let report =
            resolve_theme_tokens(&graph, &ThemeTokenResolveRequest::activate("game.runtime"));

        assert!(report.has_errors());
        assert_eq!(
            report.diagnostics[0].code,
            "ui.theme.target_profile.unsupported"
        );
    }

    #[test]
    fn token_resolution_rejects_accessibility_conflicts() {
        let selector = ThemeTokenSelector {
            accessibility: Some("high_contrast".into()),
            ..ThemeTokenSelector::default()
        };
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "color.accent",
                ThemeTokenLayer::Accessibility,
                ThemeTokenValue::Color(UiColor::new(1.0, 1.0, 0.0, 1.0)),
                "a11y-a",
            )
            .with_selector(selector.clone()),
            token_value(
                "color.accent",
                ThemeTokenLayer::Accessibility,
                ThemeTokenValue::Color(UiColor::new(0.0, 1.0, 1.0, 1.0)),
                "a11y-b",
            )
            .with_selector(selector),
        ]);
        let mut request = ThemeTokenResolveRequest::activate("editor.workbench");
        request.accessibility = Some("high_contrast".into());

        let report = resolve_theme_tokens(&graph, &request);

        assert!(report.has_errors());
        assert_eq!(
            report.diagnostics[0].code,
            "ui.theme.accessibility.conflict"
        );
    }

    #[test]
    fn token_resolution_rejects_preview_only_activation() {
        let graph = ThemeTokenGraph::new(vec![
            token_value(
                "color.accent",
                ThemeTokenLayer::Preview,
                ThemeTokenValue::Color(UiColor::new(1.0, 0.0, 0.0, 1.0)),
                "preview",
            )
            .preview_only(),
        ]);

        let report = resolve_theme_tokens(
            &graph,
            &ThemeTokenResolveRequest::activate("editor.workbench"),
        );

        assert!(report.has_errors());
        assert_eq!(
            report.diagnostics[0].code,
            "ui.theme.preview_only_activation"
        );
    }

    #[test]
    fn resolved_tokens_can_update_theme_token_packet() {
        let graph = ThemeTokenGraph::new(vec![token_value(
            "color.accent",
            ThemeTokenLayer::Primitive,
            ThemeTokenValue::Color(UiColor::new(0.1, 0.2, 0.3, 1.0)),
            "base",
        )]);

        let report = resolve_theme_tokens(
            &graph,
            &ThemeTokenResolveRequest::activate("editor.workbench"),
        );
        let formed = report.apply_to_theme_tokens(&ThemeTokens::default());

        assert_eq!(formed.accent, UiColor::new(0.1, 0.2, 0.3, 1.0));
    }

    fn token_value(
        id: &str,
        layer: ThemeTokenLayer,
        value: ThemeTokenValue,
        source: &str,
    ) -> ThemeTokenDeclaration {
        ThemeTokenDeclaration::value(id, value.family(), layer, value, source)
    }
}
