//! File: domain/editor/editor_shell/src/ux_lab/design_system.rs
//! Purpose: Editor UX Lab design-system token and recipe evidence contracts.

use std::collections::BTreeSet;

use ui_definition::{
    AuthoredId, UiNodeDefinition, UiRecipeAccessibilityDefinition, UiRecipeAccessibilityLabel,
    UiRecipeAccessibilityRole, UiRecipeDeclaration, UiRecipeExpansionReport,
    UiRecipeExpansionRequest, UiRecipeId, UiRecipeKind, UiRecipeLayoutBehavior, UiRecipeLibrary,
    UiRecipeStateVariantId, UiRecipeTargetProfileId, UiRecipeTokenRequirement, UiValueBinding,
    expand_ui_recipe,
};
use ui_theme::{ThemeTokenFamily, ThemeTokenId};

pub const EDITOR_UX_TARGET_PROFILE: &str = "editor.workbench";
pub const EDITOR_UX_DESIGN_SYSTEM_PACKAGE_ID: &str = "editor.product.design_system";
pub const EDITOR_UX_PRIMARY_BUTTON_RECIPE_ID: &str = "editor.pattern.primary_button";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorUxDesignSystemEvidence {
    pub target_profile: UiRecipeTargetProfileId,
    pub recipe_id: UiRecipeId,
    pub token_ids: BTreeSet<ThemeTokenId>,
    pub state_variants: BTreeSet<UiRecipeStateVariantId>,
}

impl EditorUxDesignSystemEvidence {
    pub fn new(
        target_profile: UiRecipeTargetProfileId,
        recipe_id: UiRecipeId,
        token_ids: impl IntoIterator<Item = ThemeTokenId>,
        state_variants: impl IntoIterator<Item = UiRecipeStateVariantId>,
    ) -> Self {
        Self {
            target_profile,
            recipe_id,
            token_ids: token_ids.into_iter().collect(),
            state_variants: state_variants.into_iter().collect(),
        }
    }
}

pub fn editor_design_system_recipe_library() -> UiRecipeLibrary {
    UiRecipeLibrary {
        declarations: vec![primary_button_recipe()],
    }
}

pub fn primary_button_recipe() -> UiRecipeDeclaration {
    UiRecipeDeclaration {
        id: id(EDITOR_UX_PRIMARY_BUTTON_RECIPE_ID),
        kind: UiRecipeKind::Widget,
        label: "Editor Primary Button".to_string(),
        category: "editor.product.action".to_string(),
        source_package: id(EDITOR_UX_DESIGN_SYSTEM_PACKAGE_ID),
        source_location: None,
        target_profiles: ids([EDITOR_UX_TARGET_PROFILE]),
        template: UiNodeDefinition::Button {
            id: id("primary-button"),
            label: UiValueBinding::static_text("Primary action"),
            route: None,
            availability: None,
            selected: None,
        },
        slots: Vec::new(),
        token_requirements: vec![
            required_token(ThemeTokenFamily::Color, "color.accent"),
            optional_token(ThemeTokenFamily::Color, "color.foreground"),
            required_token(ThemeTokenFamily::Radius, "radius.sm"),
            required_token(ThemeTokenFamily::Spacing, "spacing.sm"),
            required_token(ThemeTokenFamily::Typography, "typography.body"),
        ],
        state_variants: primary_button_state_variants(),
        accessibility: Some(UiRecipeAccessibilityDefinition {
            role: UiRecipeAccessibilityRole::Button,
            label: UiRecipeAccessibilityLabel::Static("Primary action".to_string()),
            required_semantics: ["activate".to_string(), "focusable".to_string()]
                .into_iter()
                .collect(),
        }),
        layout: UiRecipeLayoutBehavior {
            min_width: Some(72.0),
            min_height: Some(28.0),
            resizable: true,
        },
        focus_navigation: ui_definition::UiRecipeFocusNavigation {
            focusable: true,
            tab_order: Some(0),
            directional: true,
        },
        preview_only: false,
    }
}

pub fn primary_button_design_system_evidence() -> EditorUxDesignSystemEvidence {
    EditorUxDesignSystemEvidence::new(
        id(EDITOR_UX_TARGET_PROFILE),
        id(EDITOR_UX_PRIMARY_BUTTON_RECIPE_ID),
        [
            token("color.accent"),
            token("color.foreground"),
            token("radius.sm"),
            token("spacing.sm"),
            token("typography.body"),
        ],
        primary_button_state_variants(),
    )
}

pub fn expand_primary_button_recipe_contract() -> UiRecipeExpansionReport {
    let library = editor_design_system_recipe_library();
    expand_ui_recipe(
        &library,
        &UiRecipeExpansionRequest::activate(
            EDITOR_UX_PRIMARY_BUTTON_RECIPE_ID,
            EDITOR_UX_TARGET_PROFILE,
        ),
    )
}

fn required_token(family: ThemeTokenFamily, token_id: &str) -> UiRecipeTokenRequirement {
    UiRecipeTokenRequirement {
        family,
        token: Some(token(token_id)),
        required: true,
    }
}

fn optional_token(family: ThemeTokenFamily, token_id: &str) -> UiRecipeTokenRequirement {
    UiRecipeTokenRequirement {
        family,
        token: Some(token(token_id)),
        required: false,
    }
}

fn primary_button_state_variants() -> BTreeSet<UiRecipeStateVariantId> {
    ids([
        "default",
        "focused",
        "disabled",
        "warning",
        "error",
        "overflow",
        "long-label",
        "density.comfortable",
        "density.compact",
        "accessibility.high-contrast",
        "motion.reduced",
    ])
}

fn id(value: &str) -> AuthoredId {
    AuthoredId::from(value)
}

fn ids(values: impl IntoIterator<Item = &'static str>) -> BTreeSet<AuthoredId> {
    values.into_iter().map(AuthoredId::from).collect()
}

fn token(value: &str) -> ThemeTokenId {
    ThemeTokenId::new(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_button_recipe_expands_with_token_and_state_contracts() {
        let recipe = primary_button_recipe();

        assert_eq!(recipe.token_requirements.len(), 5);
        assert!(recipe.state_variants.contains(&id("focused")));
        assert!(
            recipe
                .state_variants
                .contains(&id("accessibility.high-contrast"))
        );
        let report = expand_primary_button_recipe_contract();
        assert!(!report.has_errors(), "{:?}", report.diagnostics);
        assert!(report.root.is_some());
    }
}
