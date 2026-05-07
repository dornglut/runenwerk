//! File: domain/ui/ui_runtime/src/input/hit_test.rs
//! Purpose: Hit testing against computed layout records.

use ui_math::UiPoint;

use crate::{ComputedLayout, ComputedLayoutMap, UiNode, UiNodeKind, UiTree, WidgetId};

pub fn hit_test_widget(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    point: UiPoint,
) -> Option<WidgetId> {
    hit_test_node(&tree.root, layouts, point, None)
}

fn hit_test_node(
    node: &UiNode,
    layouts: &ComputedLayoutMap,
    point: UiPoint,
    inherited_clip: Option<ui_math::UiRect>,
) -> Option<WidgetId> {
    let layout = layouts.get(&node.id)?;
    let clip = combine_clip(inherited_clip, clip_rect_for_node(node, layout))?;

    if !layout.bounds.contains(point) {
        return None;
    }
    if let Some(clip_rect) = clip
        && !clip_rect.contains(point)
    {
        return None;
    }

    for child in node.children.iter().rev() {
        if let Some(hit) = hit_test_node(child, layouts, point, clip) {
            return Some(hit);
        }
    }

    Some(node.id)
}

fn clip_rect_for_node(node: &UiNode, layout: &ComputedLayout) -> Option<ui_math::UiRect> {
    match &node.kind {
        UiNodeKind::Panel(_)
        | UiNodeKind::Popup(_)
        | UiNodeKind::Scroll(_)
        | UiNodeKind::TextInput(_)
        | UiNodeKind::NumericInput(_)
        | UiNodeKind::Tabs(_)
        | UiNodeKind::Select(_) => Some(layout.content_bounds),
        UiNodeKind::Button(_) => Some(layout.bounds),
        UiNodeKind::Table(_) | UiNodeKind::Tree(_) => Some(layout.bounds),
        UiNodeKind::Label(_)
        | UiNodeKind::Toggle(_)
        | UiNodeKind::Spacer(_)
        | UiNodeKind::Divider(_)
        | UiNodeKind::Image(_)
        | UiNodeKind::ViewportSurfaceEmbed(_)
        | UiNodeKind::Stack(_)
        | UiNodeKind::Split(_) => None,
    }
}

fn combine_clip(
    inherited: Option<ui_math::UiRect>,
    local: Option<ui_math::UiRect>,
) -> Option<Option<ui_math::UiRect>> {
    match (inherited, local) {
        (Some(a), Some(b)) => a.intersect(b).map(Some),
        (Some(a), None) => Some(Some(a)),
        (None, Some(b)) => Some(Some(b)),
        (None, None) => Some(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ButtonNode, UiRuntimeState, compute_tree_layout};
    use ui_math::{UiPoint, UiRect};
    use ui_text::FontId;
    use ui_theme::ThemeTokens;
    use ui_tree::{UiNodeKind, UiTree};

    #[test]
    fn button_hit_test_includes_padding_bounds() {
        let theme = ThemeTokens::default();
        let button_id = WidgetId(7);
        let tree = UiTree::new(UiNode::new(
            button_id,
            UiNodeKind::Button(ButtonNode::new(
                "File",
                theme.body_small_text_style(FontId(1)),
                theme,
            )),
        ));
        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(0.0, 0.0, 120.0, 32.0),
            &UiRuntimeState::default(),
        );
        let layout = layouts.get(&button_id).expect("button layout should exist");
        let point = UiPoint::new(
            layout.bounds.x + 1.0,
            layout.content_bounds.y + layout.content_bounds.height * 0.5,
        );

        assert!(layout.bounds.contains(point));
        assert!(!layout.content_bounds.contains(point));
        assert_eq!(hit_test_widget(&tree, &layouts, point), Some(button_id));
    }
}
