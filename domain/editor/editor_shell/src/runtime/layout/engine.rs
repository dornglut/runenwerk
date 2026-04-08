//! File: domain/ui/ui_runtime/src/layout/engine.rs
//! Purpose: Retained-tree layout computation for ui_runtime.

use ui_layout::{
    CrossAxisAlignment, LayoutConstraints, MainAxisAlignment, SizePolicy, SplitLayout, StackItem,
    StackLayout,
};
use ui_math::{Axis, UiRect, UiSize};

use crate::{
    ButtonNode, ComputedLayout, ComputedLayoutMap, LabelNode, PanelNode, SplitNode, StackNode,
    UiNode, UiNodeKind, UiTree,
};

pub fn compute_tree_layout(tree: &UiTree, bounds: UiRect) -> ComputedLayoutMap {
    let mut out = ComputedLayoutMap::new();
    layout_node(&tree.root, bounds, &mut out);
    out
}

fn layout_node(node: &UiNode, bounds: UiRect, out: &mut ComputedLayoutMap) -> UiSize {
    match &node.kind {
        UiNodeKind::Panel(panel) => layout_panel(node, panel, bounds, out),
        UiNodeKind::Label(label) => layout_label(node, label, bounds, out),
        UiNodeKind::Button(button) => layout_button(node, button, bounds, out),
        UiNodeKind::Stack(stack) => layout_stack(node, stack, bounds, out),
        UiNodeKind::Split(split) => layout_split(node, split, bounds, out),
    }
}

fn layout_panel(
    node: &UiNode,
    panel: &PanelNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let content_bounds = bounds.inset(panel.padding);

    let mut content_size = UiSize::ZERO;
    for child in &node.children {
        let child_size = layout_node(child, content_bounds, out);
        content_size.width = content_size.width.max(child_size.width);
        content_size.height = content_size.height.max(child_size.height);
    }

    let measured_size = UiSize::new(
        (content_size.width + panel.padding.horizontal()).max(panel.min_size.width),
        (content_size.height + panel.padding.vertical()).max(panel.min_size.height),
    );

    out.insert(
        node.id,
        ComputedLayout::new(bounds, content_bounds, measured_size),
    );

    measured_size
}

fn layout_label(
    node: &UiNode,
    label: &LabelNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let line_height = label
        .text_style
        .line_height_or_default(label.text_style.font_size * 1.2);

    let estimated_width =
        (label.text.chars().count() as f32 * label.text_style.font_size * 0.6).max(0.0);

    let size = label
        .constraints
        .constrain(UiSize::new(estimated_width, line_height));

    let layout_bounds = UiRect::new(bounds.x, bounds.y, size.width, size.height);

    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, layout_bounds, size),
    );

    size
}

fn layout_button(
    node: &UiNode,
    button: &ButtonNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let line_height = button
        .text_style
        .line_height_or_default(button.text_style.font_size * 1.2);

    let text_width =
        (button.label.chars().count() as f32 * button.text_style.font_size * 0.6).max(0.0);

    let measured_size = UiSize::new(
        (text_width + button.padding.horizontal()).max(button.min_size.width),
        (line_height + button.padding.vertical()).max(button.min_size.height),
    );

    let layout_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        measured_size.width.min(bounds.width.max(0.0)),
        measured_size.height.min(bounds.height.max(0.0)),
    );

    let content_bounds = layout_bounds.inset(button.padding);

    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, content_bounds, measured_size),
    );

    measured_size
}

fn layout_stack(
    node: &UiNode,
    stack: &StackNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let content_bounds = bounds.inset(stack.padding);

    let mut child_items = Vec::with_capacity(node.children.len());
    for (index, child) in node.children.iter().enumerate() {
        let measured = measure_node(child);
        let policy = stack
            .child_main_policies
            .get(index)
            .copied()
            .unwrap_or(SizePolicy::Auto);

        child_items.push(StackItem {
            size: measured,
            main_policy: policy,
        });
    }

    let layout = match stack.axis {
        Axis::Vertical => StackLayout::vertical(stack.gap),
        Axis::Horizontal => StackLayout::horizontal(stack.gap),
    }
    .with_main_align(MainAxisAlignment::Start)
    .with_cross_align(CrossAxisAlignment::Stretch);

    let arranged = layout.arrange(content_bounds, &child_items);

    for (child, child_bounds) in node.children.iter().zip(arranged.into_iter()) {
        layout_node(child, child_bounds, out);
    }

    let measured_content = layout.measure(
        &child_items,
        LayoutConstraints::loose(content_bounds.size()),
    );

    let measured_size = UiSize::new(
        measured_content.width + stack.padding.horizontal(),
        measured_content.height + stack.padding.vertical(),
    );

    out.insert(
        node.id,
        ComputedLayout::new(bounds, content_bounds, measured_size),
    );

    measured_size
}

fn layout_split(
    node: &UiNode,
    split: &SplitNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let layout = SplitLayout::new(split.axis, split.ratio, split.gap);

    let measured_size = bounds.size();

    match node.children.as_slice() {
        [left, right] => {
            let (left_bounds, right_bounds) = layout.arrange(bounds);
            layout_node(left, left_bounds, out);
            layout_node(right, right_bounds, out);
        }
        [only] => {
            layout_node(only, bounds, out);
        }
        _ => {}
    }

    out.insert(node.id, ComputedLayout::new(bounds, bounds, measured_size));

    measured_size
}

fn measure_node(node: &UiNode) -> UiSize {
    match &node.kind {
        UiNodeKind::Panel(panel) => {
            let mut child_size = UiSize::ZERO;
            for child in &node.children {
                let measured = measure_node(child);
                child_size.width = child_size.width.max(measured.width);
                child_size.height = child_size.height.max(measured.height);
            }

            UiSize::new(
                (child_size.width + panel.padding.horizontal()).max(panel.min_size.width),
                (child_size.height + panel.padding.vertical()).max(panel.min_size.height),
            )
        }
        UiNodeKind::Label(label) => {
            let line_height = label
                .text_style
                .line_height_or_default(label.text_style.font_size * 1.2);

            let estimated_width =
                (label.text.chars().count() as f32 * label.text_style.font_size * 0.6).max(0.0);

            label
                .constraints
                .constrain(UiSize::new(estimated_width, line_height))
        }
        UiNodeKind::Button(button) => {
            let line_height = button
                .text_style
                .line_height_or_default(button.text_style.font_size * 1.2);

            let text_width =
                (button.label.chars().count() as f32 * button.text_style.font_size * 0.6).max(0.0);

            UiSize::new(
                (text_width + button.padding.horizontal()).max(button.min_size.width),
                (line_height + button.padding.vertical()).max(button.min_size.height),
            )
        }
        UiNodeKind::Stack(stack) => {
            let mut items = Vec::with_capacity(node.children.len());
            for (index, child) in node.children.iter().enumerate() {
                let size = measure_node(child);
                let policy = stack
                    .child_main_policies
                    .get(index)
                    .copied()
                    .unwrap_or(SizePolicy::Auto);

                items.push(StackItem {
                    size,
                    main_policy: policy,
                });
            }

            let layout = match stack.axis {
                Axis::Vertical => StackLayout::vertical(stack.gap),
                Axis::Horizontal => StackLayout::horizontal(stack.gap),
            }
            .with_main_align(MainAxisAlignment::Start)
            .with_cross_align(CrossAxisAlignment::Stretch);

            let measured = layout.measure(
                &items,
                LayoutConstraints::loose(UiSize::new(f32::MAX, f32::MAX)),
            );

            UiSize::new(
                measured.width + stack.padding.horizontal(),
                measured.height + stack.padding.vertical(),
            )
        }
        UiNodeKind::Split(_) => UiSize::ZERO,
    }
}
