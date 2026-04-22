//! File: domain/ui/ui_runtime/src/widgets/scroll.rs
//! Purpose: Vertical scroll-container widget constructor.

use crate::{ScrollNode, UiNode, UiNodeKind, WidgetId};
use ui_theme::ThemeTokens;

pub fn vscroll(id: WidgetId, theme: ThemeTokens, children: Vec<UiNode>) -> UiNode {
    UiNode::with_children(id, UiNodeKind::Scroll(ScrollNode::new(theme)), children)
}
