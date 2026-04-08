//! File: domain/editor/editor_shell/src/composition/build_editor_shell.rs
//! Purpose: Compose the first editor shell tree.

use crate::{UiTree, panel, split, vstack};
use ui_math::Axis;
use ui_theme::ThemeTokens;

use crate::{
    BODY_ROOT_WIDGET_ID, CENTER_RIGHT_SPLIT_WIDGET_ID, EditorShellViewModel,
    LEFT_RIGHT_SPLIT_WIDGET_ID, ROOT_WIDGET_ID, build_inspector_panel, build_outliner_panel,
    build_toolbar, build_viewport_panel,
};

pub fn build_editor_shell(view_model: &EditorShellViewModel, theme: &ThemeTokens) -> UiTree {
    let toolbar = build_toolbar(&view_model.toolbar, theme);
    let outliner = build_outliner_panel(&view_model.outliner, theme);
    let viewport = build_viewport_panel(&view_model.viewport, theme);
    let inspector = build_inspector_panel(&view_model.inspector, theme);

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

    let root = panel(
        ROOT_WIDGET_ID,
        theme.clone(),
        vec![vstack(
            BODY_ROOT_WIDGET_ID,
            theme.spacing.sm,
            vec![toolbar, body],
        )],
    );

    UiTree::new(root)
}
