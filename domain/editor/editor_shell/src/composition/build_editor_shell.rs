//! File: domain/editor/editor_shell/src/composition/build_editor_shell.rs
//! Purpose: Compose the first editor shell tree.

use crate::{UiTree, panel, split, vstack_with_policies};
use ui_layout::SizePolicy;
use ui_math::Axis;
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    BODY_CONSOLE_SPLIT_WIDGET_ID, BODY_ROOT_WIDGET_ID, CENTER_RIGHT_SPLIT_WIDGET_ID,
    EditorShellViewModel, LEFT_RIGHT_SPLIT_WIDGET_ID, ROOT_WIDGET_ID, build_console_panel,
    build_inspector_panel, build_outliner_panel, build_toolbar, build_viewport_panel,
};

pub fn build_editor_shell(view_model: &EditorShellViewModel, theme: &ThemeTokens) -> UiTree {
    let toolbar = build_toolbar(&view_model.toolbar, theme);
    let outliner = build_outliner_panel(&view_model.outliner, theme);
    let viewport = build_viewport_panel(&view_model.viewport, theme);
    let inspector = build_inspector_panel(&view_model.inspector, theme);
    let console = build_console_panel(&view_model.console, theme);

    let center_right = split(
        CENTER_RIGHT_SPLIT_WIDGET_ID,
        Axis::Horizontal,
        0.70,
        theme.spacing.sm,
        vec![viewport, inspector],
    );

    let body = split(
        LEFT_RIGHT_SPLIT_WIDGET_ID,
        Axis::Horizontal,
        0.22,
        theme.spacing.sm,
        vec![outliner, center_right],
    );

    let body_with_console = split(
        BODY_CONSOLE_SPLIT_WIDGET_ID,
        Axis::Vertical,
        0.78,
        theme.spacing.sm,
        vec![body, console],
    );

    let mut root_theme = theme.clone();
    root_theme.background_panel = if root_background_opaque_enabled() {
        theme.background
    } else {
        UiColor::new(theme.background.r, theme.background.g, theme.background.b, 0.0)
    };
    root_theme.border = UiColor::new(theme.border.r, theme.border.g, theme.border.b, 0.80);

    let root = panel(
        ROOT_WIDGET_ID,
        root_theme,
        vec![vstack_with_policies(
            BODY_ROOT_WIDGET_ID,
            theme.spacing.sm,
            vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
            vec![toolbar, body_with_console],
        )],
    );

    UiTree::new(root)
}

fn root_background_opaque_enabled() -> bool {
    std::env::var("RUNENWERK_EDITOR_VIEWPORT_ROOT_OPAQUE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}
