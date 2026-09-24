//! File: domain/editor/editor_shell/src/ux_lab/product_patterns.rs
//! Purpose: Editor UX Lab product-pattern evidence contracts.

use crate::{
    NumericInputConfig, TableColumn, TableRow, TreeRow, UiNode, UiNodeKind, VisibleWidgetState,
    WidgetId, button, hstack, label, numeric_input, panel, product_surface, select, split, table,
    tabs, text_input, toggle, tree, vscroll,
};
use ui_math::{Axis, UiSize};
use ui_render_data::ProductSurfaceTextureBindingSource;
use ui_text::FontId;
use ui_theme::ThemeTokens;

pub const SHELL_PRODUCT_PATTERNS_SCENARIO_ID: &str = "editor.shell.product_patterns.certified";
pub const SHELL_PRODUCT_PATTERNS_TARGET_PROFILE: &str = "editor.shell.patterns";

pub const SHELL_PATTERNS_ROOT_WIDGET_ID: WidgetId = WidgetId(80_000);
pub const SHELL_PATTERNS_TOOLBAR_SAVE_WIDGET_ID: WidgetId = WidgetId(80_010);
pub const SHELL_PATTERNS_STATUS_TOGGLE_WIDGET_ID: WidgetId = WidgetId(80_011);
pub const SHELL_PATTERNS_TAB_WIDGET_ID: WidgetId = WidgetId(80_012);
pub const SHELL_PATTERNS_INSPECTOR_TEXT_WIDGET_ID: WidgetId = WidgetId(80_020);
pub const SHELL_PATTERNS_INSPECTOR_NUMERIC_WIDGET_ID: WidgetId = WidgetId(80_021);
pub const SHELL_PATTERNS_INSPECTOR_SELECT_WIDGET_ID: WidgetId = WidgetId(80_022);
pub const SHELL_PATTERNS_INSPECTOR_READONLY_WIDGET_ID: WidgetId = WidgetId(80_023);
pub const SHELL_PATTERNS_PALETTE_ITEM_WIDGET_ID: WidgetId = WidgetId(80_030);
pub const SHELL_PATTERNS_DIAGNOSTIC_WIDGET_ID: WidgetId = WidgetId(80_040);
pub const SHELL_PATTERNS_PREVIEW_WIDGET_ID: WidgetId = WidgetId(80_050);
pub const SHELL_PATTERNS_TABLE_WIDGET_ID: WidgetId = WidgetId(80_060);
pub const SHELL_PATTERNS_TREE_WIDGET_ID: WidgetId = WidgetId(80_070);
pub const SHELL_PATTERNS_DOCK_SPLIT_WIDGET_ID: WidgetId = WidgetId(80_080);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorUxProductPatternEvidence {
    pub target_profile: &'static str,
    pub pattern_kinds: Vec<&'static str>,
    pub state_kinds: Vec<&'static str>,
    pub interaction_kinds: Vec<&'static str>,
    pub route_kinds: Vec<&'static str>,
    pub native_evidence_checks: Vec<&'static str>,
}

pub fn shell_product_pattern_evidence() -> EditorUxProductPatternEvidence {
    EditorUxProductPatternEvidence {
        target_profile: SHELL_PRODUCT_PATTERNS_TARGET_PROFILE,
        pattern_kinds: vec![
            "inspector",
            "palette",
            "diagnostics",
            "preview",
            "table",
            "tree",
            "tab",
            "toolbar",
            "status",
            "split",
            "dock",
        ],
        state_kinds: vec![
            "default",
            "focused",
            "selected",
            "disabled",
            "readonly",
            "loading",
            "empty",
            "warning",
            "error",
            "degraded",
            "long_text",
            "dense",
            "overflow",
            "keyboard_focus",
        ],
        interaction_kinds: vec![
            "toolbar_activate",
            "status_toggle",
            "tab_select",
            "inspector_edit",
            "palette_select",
            "diagnostic_navigation",
            "preview_focus",
            "table_row_select",
            "tree_row_select",
            "split_dock_focus",
            "keyboard_traversal",
        ],
        route_kinds: vec![
            "save_project",
            "toggle_diagnostics",
            "select_tab",
            "set_inspector_text",
            "set_inspector_number",
            "select_palette_item",
            "navigate_diagnostic",
            "focus_preview",
            "select_table_row",
            "select_tree_row",
            "dock_split_route",
        ],
        native_evidence_checks: vec![
            "product_pattern_report",
            "native_or_platform_impossible_capture",
            "focus_traversal",
            "accessibility",
            "diagnostics_snapshot",
            "timing_report",
        ],
    }
}

pub fn shell_product_pattern_required_states() -> [VisibleWidgetState; 8] {
    [
        VisibleWidgetState::Default,
        VisibleWidgetState::Focused,
        VisibleWidgetState::Selected,
        VisibleWidgetState::Disabled,
        VisibleWidgetState::Checked,
        VisibleWidgetState::Error,
        VisibleWidgetState::Empty,
        VisibleWidgetState::Overflow,
    ]
}

pub fn shell_product_pattern_fixture_root() -> UiNode {
    let theme = ThemeTokens::default();
    let text_style = theme.body_small_text_style(FontId(1));
    let title_style = theme.body_text_style(FontId(1));

    let mut readonly = text_input(
        SHELL_PATTERNS_INSPECTOR_READONLY_WIDGET_ID,
        "readonly transform inherited from prefab source",
        "Readonly",
        text_style.clone(),
        theme.clone(),
    );
    if let UiNodeKind::TextInput(input) = &mut readonly.kind {
        input.editable = false;
    }

    let mut loading = button(
        WidgetId(80_041),
        "Diagnostics loading",
        text_style.clone(),
        theme.clone(),
    );
    if let UiNodeKind::Button(button) = &mut loading.kind {
        button.enabled = false;
    }

    panel(
        SHELL_PATTERNS_ROOT_WIDGET_ID,
        theme.clone(),
        vec![
            label(
                WidgetId(80_001),
                "Editor Shell Product Patterns",
                title_style.clone(),
            ),
            hstack(
                WidgetId(80_002),
                theme.spacing.sm,
                vec![
                    button(
                        SHELL_PATTERNS_TOOLBAR_SAVE_WIDGET_ID,
                        "Save Project",
                        text_style.clone(),
                        theme.clone(),
                    ),
                    toggle(
                        SHELL_PATTERNS_STATUS_TOGGLE_WIDGET_ID,
                        "Diagnostics",
                        true,
                        text_style.clone(),
                        theme.clone(),
                    ),
                    tabs(
                        SHELL_PATTERNS_TAB_WIDGET_ID,
                        ["Scene", "Assets", "Diagnostics"],
                        2,
                        text_style.clone(),
                        theme.clone(),
                    ),
                ],
            ),
            split(
                SHELL_PATTERNS_DOCK_SPLIT_WIDGET_ID,
                Axis::Horizontal,
                0.42,
                theme.spacing.md,
                vec![
                    vscroll(
                        WidgetId(80_003),
                        theme.clone(),
                        vec![panel(
                            WidgetId(80_004),
                            theme.clone(),
                            vec![
                                label(WidgetId(80_005), "Inspector", title_style.clone()),
                                text_input(
                                    SHELL_PATTERNS_INSPECTOR_TEXT_WIDGET_ID,
                                    "Hero Player With Very Long Localized Display Name",
                                    "Display name",
                                    text_style.clone(),
                                    theme.clone(),
                                ),
                                numeric_input(
                                    SHELL_PATTERNS_INSPECTOR_NUMERIC_WIDGET_ID,
                                    NumericInputConfig::new(42.0, 0.5, Some(0.0), Some(100.0), 1),
                                    text_style.clone(),
                                    theme.clone(),
                                ),
                                select(
                                    SHELL_PATTERNS_INSPECTOR_SELECT_WIDGET_ID,
                                    ["Product", "Fallback", "Diagnostic"],
                                    Some(0),
                                    "Readiness",
                                    text_style.clone(),
                                    theme.clone(),
                                ),
                                readonly,
                                label(WidgetId(80_006), "Palette", title_style.clone()),
                                button(
                                    SHELL_PATTERNS_PALETTE_ITEM_WIDGET_ID,
                                    "Add product card pattern",
                                    text_style.clone(),
                                    theme.clone(),
                                ),
                                label(WidgetId(80_007), "Diagnostics", title_style.clone()),
                                button(
                                    SHELL_PATTERNS_DIAGNOSTIC_WIDGET_ID,
                                    "Open overflow warning",
                                    text_style.clone(),
                                    theme.clone(),
                                ),
                                loading,
                            ],
                        )],
                    ),
                    panel(
                        WidgetId(80_008),
                        theme.clone(),
                        vec![
                            product_surface(
                                SHELL_PATTERNS_PREVIEW_WIDGET_ID,
                                ProductSurfaceTextureBindingSource::dynamic_texture(
                                    "editor.product_pattern.preview",
                                    "shell-patterns",
                                ),
                                UiSize::new(192.0, 128.0),
                            ),
                            table(
                                SHELL_PATTERNS_TABLE_WIDGET_ID,
                                [
                                    TableColumn::new("Surface", 120.0),
                                    TableColumn::new("State", 96.0),
                                ],
                                [
                                    TableRow::new(["Inspector", "Focused"]),
                                    TableRow::new(["Diagnostics", "Warning"]),
                                    TableRow::new(["Preview", "Degraded"]),
                                ],
                                text_style.clone(),
                                title_style.clone(),
                                theme.clone(),
                            ),
                            tree(
                                SHELL_PATTERNS_TREE_WIDGET_ID,
                                [
                                    TreeRow::new("Dock Root", 0, true),
                                    TreeRow::new("Left Inspector", 1, false),
                                    TreeRow::new("Right Preview", 1, false),
                                ],
                                text_style,
                                theme,
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_product_pattern_evidence_names_all_pm006_pattern_families() {
        let evidence = shell_product_pattern_evidence();

        for pattern in [
            "inspector",
            "palette",
            "diagnostics",
            "preview",
            "table",
            "tree",
            "tab",
            "toolbar",
            "status",
            "split",
            "dock",
        ] {
            assert!(
                evidence.pattern_kinds.contains(&pattern),
                "missing pattern kind {pattern}"
            );
        }
        assert!(evidence.state_kinds.contains(&"degraded"));
        assert!(evidence.route_kinds.contains(&"dock_split_route"));
        assert!(
            evidence
                .native_evidence_checks
                .contains(&"product_pattern_report")
        );
    }
}
