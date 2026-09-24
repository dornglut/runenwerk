//! File: domain/editor/editor_shell/src/composition/build_toolbar.rs
//! Purpose: Compatibility toolbar composition entrypoint.

use crate::{ToolbarViewModel, UiNode, build_defined_toolbar};
use ui_theme::ThemeTokens;

pub fn build_toolbar(view_model: &ToolbarViewModel, theme: &ThemeTokens) -> UiNode {
    build_defined_toolbar(view_model, theme).root
}
