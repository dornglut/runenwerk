//! File: domain/ui/ui_runtime/src/layout/engine.rs
//! Purpose: Retained-tree layout computation for ui_runtime.

use ui_layout::{
    CrossAxisAlignment, LayoutConstraints, MainAxisAlignment, SizePolicy, SplitLayout, StackItem,
    StackLayout,
};
use ui_math::{Axis, UiRect, UiSize};

use crate::{
    ButtonNode, ComputedLayout, ComputedLayoutMap, DividerNode, GraphCanvasNode, ImageNode,
    LabelNode, NumericInputNode, OverlayAdornmentNode, PanelNode, PopupAlign, PopupFlipPolicy,
    PopupNode, PopupPlacement, PopupSide, ProductSurfaceNode, RadialMenuAnchor, RadialMenuNode,
    ScrollNode, SelectNode, SpacerNode, SplitNode, StackNode, TableNode, TabsNode, TextInputNode,
    ToggleNode, TreeNode, UiNode, UiNodeKind, UiRuntimeState, UiTree, ViewportSurfaceEmbedNode,
};

pub fn compute_tree_layout(
    tree: &UiTree,
    bounds: UiRect,
    state: &UiRuntimeState,
) -> ComputedLayoutMap {
    let mut out = ComputedLayoutMap::new();
    layout_node(&tree.root, bounds, state, &mut out);
    out
}

fn layout_node(
    node: &UiNode,
    bounds: UiRect,
    state: &UiRuntimeState,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    match &node.kind {
        UiNodeKind::Panel(panel) => layout_panel(node, panel, bounds, state, out),
        UiNodeKind::Popup(popup) => layout_popup(node, popup, bounds, state, out),
        UiNodeKind::RadialMenu(radial) => layout_radial_menu(node, radial, bounds, state, out),
        UiNodeKind::OverlayAdornment(adornment) => {
            layout_overlay_adornment(node, adornment, bounds, state, out)
        }
        UiNodeKind::Label(label) => layout_label(node, label, bounds, out),
        UiNodeKind::Button(button) => layout_button(node, button, bounds, out),
        UiNodeKind::TextInput(text_input) => layout_text_input(node, text_input, bounds, out),
        UiNodeKind::Toggle(toggle) => layout_toggle(node, toggle, bounds, out),
        UiNodeKind::NumericInput(numeric) => layout_numeric_input(node, numeric, bounds, out),
        UiNodeKind::Tabs(tabs) => layout_tabs(node, tabs, bounds, out),
        UiNodeKind::Select(select) => layout_select(node, select, bounds, out),
        UiNodeKind::Table(table) => layout_table(node, table, bounds, out),
        UiNodeKind::Tree(tree) => layout_tree(node, tree, bounds, out),
        UiNodeKind::Spacer(spacer) => layout_spacer(node, spacer, bounds, out),
        UiNodeKind::Divider(divider) => layout_divider(node, divider, bounds, out),
        UiNodeKind::Image(image) => layout_image(node, image, bounds, out),
        UiNodeKind::ProductSurface(surface) => layout_product_surface(node, surface, bounds, out),
        UiNodeKind::GraphCanvas(graph_canvas) => {
            layout_graph_canvas(node, graph_canvas, bounds, out)
        }
        UiNodeKind::ViewportSurfaceEmbed(embed) => {
            layout_viewport_surface_embed(node, embed, bounds, out)
        }
        UiNodeKind::Scroll(scroll) => layout_scroll(node, scroll, bounds, state, out),
        UiNodeKind::Stack(stack) => layout_stack(node, stack, bounds, state, out),
        UiNodeKind::Split(split) => layout_split(node, split, bounds, state, out),
    }
}

fn layout_panel(
    node: &UiNode,
    panel: &PanelNode,
    bounds: UiRect,
    state: &UiRuntimeState,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let content_bounds = bounds.inset(panel.padding);

    let normal_children = node
        .children
        .iter()
        .filter(|child| !is_popup_node(child))
        .collect::<Vec<_>>();

    let content_size = if normal_children.is_empty() {
        UiSize::ZERO
    } else if normal_children.len() == 1 {
        layout_node(normal_children[0], content_bounds, state, out)
    } else {
        let child_items = normal_children
            .iter()
            .map(|child| StackItem::auto(measure_node(child)))
            .collect::<Vec<_>>();

        let layout = StackLayout::vertical(panel.gap)
            .with_main_align(MainAxisAlignment::Start)
            .with_cross_align(CrossAxisAlignment::Stretch);

        let arranged = layout.arrange(content_bounds, &child_items);
        for (child, child_bounds) in normal_children.iter().zip(arranged) {
            layout_node(child, child_bounds, state, out);
        }

        layout.measure(
            &child_items,
            LayoutConstraints::loose(content_bounds.size()),
        )
    };

    let measured_size = UiSize::new(
        (content_size.width + panel.padding.horizontal()).max(panel.min_size.width),
        (content_size.height + panel.padding.vertical()).max(panel.min_size.height),
    );

    out.insert(
        node.id,
        ComputedLayout::new(bounds, content_bounds, measured_size),
    );

    for popup in node.children.iter().filter(|child| is_popup_node(child)) {
        layout_node(popup, content_bounds, state, out);
    }

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

    let constrained_size = label
        .constraints
        .constrain(UiSize::new(estimated_width, line_height));
    let size = UiSize::new(
        constrained_size.width.min(bounds.width.max(0.0)),
        constrained_size.height.min(bounds.height.max(0.0)),
    );

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

    let layout_width = if button.fill_width {
        bounds.width.max(0.0)
    } else {
        measured_size.width.min(bounds.width.max(0.0))
    };
    let layout_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        layout_width,
        measured_size.height.min(bounds.height.max(0.0)),
    );

    let content_bounds = layout_bounds.inset(button.padding);

    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, content_bounds, measured_size),
    );

    measured_size
}

fn layout_text_input(
    node: &UiNode,
    text_input: &TextInputNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let line_height = text_input
        .text_style
        .line_height_or_default(text_input.text_style.font_size * 1.2);
    let display_text = if text_input.value.is_empty() {
        &text_input.placeholder
    } else {
        &text_input.value
    };
    let text_width =
        (display_text.chars().count() as f32 * text_input.text_style.font_size * 0.6).max(0.0);
    let measured_size = UiSize::new(
        (text_width + text_input.padding.horizontal()).max(text_input.min_size.width),
        (line_height + text_input.padding.vertical()).max(text_input.min_size.height),
    );
    let layout_width = if text_input.fill_width {
        bounds.width.max(0.0)
    } else {
        measured_size.width.min(bounds.width.max(0.0))
    };
    let layout_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        layout_width,
        measured_size.height.min(bounds.height.max(0.0)),
    );
    let content_bounds = layout_bounds.inset(text_input.padding);
    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, content_bounds, measured_size),
    );
    measured_size
}

fn layout_toggle(
    node: &UiNode,
    toggle: &ToggleNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let line_height = toggle
        .text_style
        .line_height_or_default(toggle.text_style.font_size * 1.2);
    let label_width =
        (toggle.label.chars().count() as f32 * toggle.text_style.font_size * 0.6).max(0.0);
    let measured_size = UiSize::new(
        (label_width + toggle.padding.horizontal() + line_height).max(toggle.min_size.width),
        (line_height + toggle.padding.vertical()).max(toggle.min_size.height),
    );
    let layout_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        measured_size.width.min(bounds.width.max(0.0)),
        measured_size.height.min(bounds.height.max(0.0)),
    );
    let content_bounds = layout_bounds.inset(toggle.padding);
    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, content_bounds, measured_size),
    );
    measured_size
}

fn layout_numeric_input(
    node: &UiNode,
    numeric: &NumericInputNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let line_height = numeric
        .text_style
        .line_height_or_default(numeric.text_style.font_size * 1.2);
    let value_text = format!("{:.*}", usize::from(numeric.precision), numeric.value);
    let text_width =
        (value_text.chars().count() as f32 * numeric.text_style.font_size * 0.6).max(0.0);
    let measured_size = UiSize::new(
        (text_width + numeric.padding.horizontal() + line_height).max(numeric.min_size.width),
        (line_height + numeric.padding.vertical()).max(numeric.min_size.height),
    );
    let layout_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        measured_size.width.min(bounds.width.max(0.0)),
        measured_size.height.min(bounds.height.max(0.0)),
    );
    let content_bounds = layout_bounds.inset(numeric.padding);
    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, content_bounds, measured_size),
    );
    measured_size
}

fn layout_tabs(
    node: &UiNode,
    tabs: &TabsNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let line_height = tabs
        .text_style
        .line_height_or_default(tabs.text_style.font_size * 1.2);
    let width = tabs
        .labels
        .iter()
        .map(|label| {
            label.chars().count() as f32 * tabs.text_style.font_size * 0.6
                + tabs.padding.horizontal()
        })
        .sum::<f32>()
        .max(tabs.min_size.width);
    let measured_size = UiSize::new(
        width,
        (line_height + tabs.padding.vertical()).max(tabs.min_size.height),
    );
    let layout_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        measured_size.width.min(bounds.width.max(0.0)),
        measured_size.height.min(bounds.height.max(0.0)),
    );
    let content_bounds = layout_bounds.inset(tabs.padding);
    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, content_bounds, measured_size),
    );
    measured_size
}

fn layout_viewport_surface_embed(
    node: &UiNode,
    embed: &ViewportSurfaceEmbedNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let measured_size = UiSize::new(
        bounds.width.max(embed.min_size.width),
        bounds.height.max(embed.min_size.height),
    );
    out.insert(node.id, ComputedLayout::new(bounds, bounds, measured_size));
    measured_size
}

fn layout_select(
    node: &UiNode,
    select: &SelectNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let line_height = select
        .text_style
        .line_height_or_default(select.text_style.font_size * 1.2);
    let display_text = select
        .selected_index
        .and_then(|index| select.options.get(index))
        .unwrap_or(&select.placeholder);
    let text_width =
        (display_text.chars().count() as f32 * select.text_style.font_size * 0.6).max(0.0);
    let measured_size = UiSize::new(
        (text_width + select.padding.horizontal() + line_height).max(select.min_size.width),
        (line_height + select.padding.vertical()).max(select.min_size.height),
    );
    let layout_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        measured_size.width.min(bounds.width.max(0.0)),
        measured_size.height.min(bounds.height.max(0.0)),
    );
    let content_bounds = layout_bounds.inset(select.padding);
    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, content_bounds, measured_size),
    );
    measured_size
}

fn layout_table(
    node: &UiNode,
    table: &TableNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let measured_size = table_size(table);
    let layout_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        measured_size.width.min(bounds.width.max(0.0)),
        measured_size.height.min(bounds.height.max(0.0)),
    );
    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, layout_bounds, measured_size),
    );
    measured_size
}

fn layout_tree(
    node: &UiNode,
    tree: &TreeNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let measured_size = tree_size(tree);
    let layout_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        measured_size.width.min(bounds.width.max(0.0)),
        measured_size.height.min(bounds.height.max(0.0)),
    );
    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, layout_bounds, measured_size),
    );
    measured_size
}

fn layout_spacer(
    node: &UiNode,
    spacer: &SpacerNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let measured_size = UiSize::new(
        bounds.width.max(spacer.min_size.width),
        bounds.height.max(spacer.min_size.height),
    );
    out.insert(node.id, ComputedLayout::new(bounds, bounds, measured_size));
    measured_size
}

fn layout_divider(
    node: &UiNode,
    divider: &DividerNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let measured_size = divider_size_for_bounds(divider, bounds);
    let layout_bounds = match divider.axis {
        Axis::Horizontal => UiRect::new(
            bounds.x,
            bounds.y + (bounds.height - measured_size.height).max(0.0) * 0.5,
            measured_size.width,
            measured_size.height,
        ),
        Axis::Vertical => UiRect::new(
            bounds.x + (bounds.width - measured_size.width).max(0.0) * 0.5,
            bounds.y + (bounds.height - measured_size.height).max(0.0) * 0.5,
            measured_size.width,
            measured_size.height,
        ),
    };
    out.insert(
        node.id,
        ComputedLayout::new(layout_bounds, layout_bounds, measured_size),
    );
    measured_size
}

fn layout_image(
    node: &UiNode,
    image: &ImageNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let measured_size = UiSize::new(
        bounds.width.max(image.min_size.width),
        bounds.height.max(image.min_size.height),
    );
    out.insert(node.id, ComputedLayout::new(bounds, bounds, measured_size));
    measured_size
}

fn layout_product_surface(
    node: &UiNode,
    surface: &ProductSurfaceNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let measured_size = UiSize::new(
        bounds.width.max(surface.min_size.width),
        bounds.height.max(surface.min_size.height),
    );
    out.insert(node.id, ComputedLayout::new(bounds, bounds, measured_size));
    measured_size
}

fn layout_graph_canvas(
    node: &UiNode,
    graph_canvas: &GraphCanvasNode,
    bounds: UiRect,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let measured_size = UiSize::new(
        bounds.width.max(graph_canvas.min_size.width),
        bounds.height.max(graph_canvas.min_size.height),
    );
    out.insert(node.id, ComputedLayout::new(bounds, bounds, measured_size));
    measured_size
}

fn layout_scroll(
    node: &UiNode,
    scroll: &ScrollNode,
    bounds: UiRect,
    state: &UiRuntimeState,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let base_content_bounds = UiRect::new(
        bounds.x,
        bounds.y,
        bounds.width.max(0.0),
        bounds.height.max(0.0),
    );
    let measured_content = node
        .children
        .first()
        .map(measure_node)
        .unwrap_or(UiSize::ZERO);
    let content_bounds = base_content_bounds;

    if let Some(child) = node.children.first() {
        let content_width = if scroll.axes.contains(Axis::Horizontal) {
            measured_content.width.max(content_bounds.width)
        } else {
            content_bounds.width
        };
        let content_height = if scroll.axes.contains(Axis::Vertical) {
            measured_content.height.max(content_bounds.height)
        } else {
            content_bounds.height
        };
        let max_x = (content_width - content_bounds.width).max(0.0);
        let max_y = (content_height - content_bounds.height).max(0.0);
        let offset_x = if scroll.axes.contains(Axis::Horizontal) {
            state
                .scroll_offset_for_axis(node.id, Axis::Horizontal)
                .clamp(0.0, max_x)
        } else {
            0.0
        };
        let offset_y = if scroll.axes.contains(Axis::Vertical) {
            state
                .scroll_offset_for_axis(node.id, Axis::Vertical)
                .clamp(0.0, max_y)
        } else {
            0.0
        };
        let child_bounds = UiRect::new(
            content_bounds.x - offset_x,
            content_bounds.y - offset_y,
            content_width,
            content_height,
        );
        layout_node(child, child_bounds, state, out);
    }

    // Preserve the child's unconstrained content extent in measured_size so ancestor
    // scroll containers can still detect overflow through nested scroll nodes.
    let measured_size = UiSize::new(
        measured_content.width.max(content_bounds.width),
        measured_content.height.max(content_bounds.height),
    );
    out.insert(
        node.id,
        ComputedLayout::new(bounds, content_bounds, measured_size),
    );
    measured_size
}

fn layout_stack(
    node: &UiNode,
    stack: &StackNode,
    bounds: UiRect,
    state: &UiRuntimeState,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let content_bounds = bounds.inset(stack.padding);
    let normal_children = node
        .children
        .iter()
        .filter(|child| !is_popup_node(child))
        .collect::<Vec<_>>();

    let mut child_items = Vec::with_capacity(normal_children.len());
    for (index, child) in normal_children.iter().enumerate() {
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

    for (child, child_bounds) in normal_children.iter().zip(arranged) {
        layout_node(child, child_bounds, state, out);
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

    for popup in node.children.iter().filter(|child| is_popup_node(child)) {
        layout_node(popup, content_bounds, state, out);
    }

    measured_size
}

fn layout_popup(
    node: &UiNode,
    popup: &PopupNode,
    bounds: UiRect,
    state: &UiRuntimeState,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let Some(anchor_layout) = out.get(&popup.anchor) else {
        out.insert(node.id, ComputedLayout::new(bounds, bounds, UiSize::ZERO));
        return UiSize::ZERO;
    };

    let content_size = measure_popup_content(node, popup);
    let measured_size = UiSize::new(
        (content_size.width + popup.padding.horizontal()).max(popup.min_size.width),
        (content_size.height + popup.padding.vertical()).max(popup.min_size.height),
    );
    let anchor = anchor_layout.bounds;
    let (target_x, target_y, popup_width, popup_height) = match popup.placement {
        PopupPlacement::Outside {
            preferred_side,
            align,
            flip_policy,
        } => placed_outside_popup_bounds(
            anchor,
            bounds,
            measured_size,
            popup.offset,
            preferred_side,
            align,
            flip_policy,
        ),
        _ => {
            let popup_width = measured_size.width.min(bounds.width.max(0.0));
            let popup_height = measured_size.height.min(bounds.height.max(0.0));
            let (target_x, target_y) = match popup.placement {
                PopupPlacement::BottomStart => (anchor.x, anchor.y + anchor.height + popup.offset),
                PopupPlacement::RightStart => (anchor.x + anchor.width + popup.offset, anchor.y),
                PopupPlacement::InsideTopEnd => (
                    anchor.x + anchor.width - popup_width - popup.offset,
                    anchor.y + popup.offset,
                ),
                PopupPlacement::InsideBottomStart => (
                    anchor.x + popup.offset,
                    anchor.y + anchor.height - popup_height - popup.offset,
                ),
                PopupPlacement::TopStart => (anchor.x + popup.offset, anchor.y + popup.offset),
                PopupPlacement::InsideLeft => (anchor.x + popup.offset, anchor.y),
                PopupPlacement::InsideRight => (
                    anchor.x + anchor.width - popup_width - popup.offset,
                    anchor.y,
                ),
                PopupPlacement::InsideTop => (anchor.x, anchor.y + popup.offset),
                PopupPlacement::InsideBottom => (
                    anchor.x,
                    anchor.y + anchor.height - popup_height - popup.offset,
                ),
                PopupPlacement::Outside { .. } => unreachable!(),
            };
            (target_x, target_y, popup_width, popup_height)
        }
    };
    let x = target_x.clamp(bounds.x, bounds.x + (bounds.width - popup_width).max(0.0));
    let y = target_y.clamp(bounds.y, bounds.y + (bounds.height - popup_height).max(0.0));
    let popup_bounds = UiRect::new(x, y, popup_width, popup_height);
    let content_bounds = popup_bounds.inset(popup.padding);

    let normal_children = node
        .children
        .iter()
        .filter(|child| !is_popup_node(child))
        .collect::<Vec<_>>();
    let stretch_single_scroll_child = normal_children.len() == 1
        && matches!(normal_children[0].kind, UiNodeKind::Scroll(_))
        && measured_size.height > popup_bounds.height + f32::EPSILON;
    let child_items = normal_children
        .iter()
        .map(|child| {
            if stretch_single_scroll_child {
                StackItem::flex(measure_node(child), 1.0)
            } else {
                StackItem::auto(measure_node(child))
            }
        })
        .collect::<Vec<_>>();
    let arranged = StackLayout::vertical(popup.gap)
        .with_main_align(MainAxisAlignment::Start)
        .with_cross_align(CrossAxisAlignment::Stretch)
        .arrange(content_bounds, &child_items);
    for (child, child_bounds) in normal_children.into_iter().zip(arranged) {
        layout_node(child, child_bounds, state, out);
    }

    out.insert(
        node.id,
        ComputedLayout::new(popup_bounds, content_bounds, measured_size),
    );

    measured_size
}

fn placed_outside_popup_bounds(
    anchor: UiRect,
    bounds: UiRect,
    measured_size: UiSize,
    offset: f32,
    preferred_side: PopupSide,
    align: PopupAlign,
    flip_policy: PopupFlipPolicy,
) -> (f32, f32, f32, f32) {
    let side = outside_popup_side(
        anchor,
        bounds,
        measured_size,
        offset,
        preferred_side,
        flip_policy,
    );
    let available_main = outside_popup_available_main(anchor, bounds, offset, side);
    let popup_width = match side {
        PopupSide::Left | PopupSide::Right => measured_size
            .width
            .min(available_main)
            .min(bounds.width.max(0.0)),
        PopupSide::Top | PopupSide::Bottom => measured_size.width.min(bounds.width.max(0.0)),
    };
    let popup_height = match side {
        PopupSide::Top | PopupSide::Bottom => measured_size
            .height
            .min(available_main)
            .min(bounds.height.max(0.0)),
        PopupSide::Left | PopupSide::Right => measured_size.height.min(bounds.height.max(0.0)),
    };

    let target_x = match side {
        PopupSide::Bottom | PopupSide::Top => {
            aligned_popup_cross_position(anchor.x, anchor.width, popup_width, align)
        }
        PopupSide::Left => anchor.x - popup_width - offset,
        PopupSide::Right => anchor.x + anchor.width + offset,
    };
    let target_y = match side {
        PopupSide::Bottom => anchor.y + anchor.height + offset,
        PopupSide::Top => anchor.y - popup_height - offset,
        PopupSide::Left | PopupSide::Right => {
            aligned_popup_cross_position(anchor.y, anchor.height, popup_height, align)
        }
    };

    (target_x, target_y, popup_width, popup_height)
}

fn outside_popup_side(
    anchor: UiRect,
    bounds: UiRect,
    measured_size: UiSize,
    offset: f32,
    preferred_side: PopupSide,
    flip_policy: PopupFlipPolicy,
) -> PopupSide {
    if matches!(flip_policy, PopupFlipPolicy::None) {
        return preferred_side;
    }

    let preferred_available = outside_popup_available_main(anchor, bounds, offset, preferred_side);
    let preferred_required = outside_popup_required_main(measured_size, preferred_side);
    if preferred_available >= preferred_required {
        return preferred_side;
    }

    let opposite = opposite_popup_side(preferred_side);
    let opposite_available = outside_popup_available_main(anchor, bounds, offset, opposite);
    if opposite_available > preferred_available {
        opposite
    } else {
        preferred_side
    }
}

fn outside_popup_available_main(
    anchor: UiRect,
    bounds: UiRect,
    offset: f32,
    side: PopupSide,
) -> f32 {
    match side {
        PopupSide::Top => (anchor.y - bounds.y - offset).max(0.0),
        PopupSide::Bottom => {
            (bounds.y + bounds.height - (anchor.y + anchor.height) - offset).max(0.0)
        }
        PopupSide::Left => (anchor.x - bounds.x - offset).max(0.0),
        PopupSide::Right => (bounds.x + bounds.width - (anchor.x + anchor.width) - offset).max(0.0),
    }
}

fn outside_popup_required_main(measured_size: UiSize, side: PopupSide) -> f32 {
    match side {
        PopupSide::Top | PopupSide::Bottom => measured_size.height,
        PopupSide::Left | PopupSide::Right => measured_size.width,
    }
}

fn opposite_popup_side(side: PopupSide) -> PopupSide {
    match side {
        PopupSide::Top => PopupSide::Bottom,
        PopupSide::Bottom => PopupSide::Top,
        PopupSide::Left => PopupSide::Right,
        PopupSide::Right => PopupSide::Left,
    }
}

fn aligned_popup_cross_position(
    anchor_cross_position: f32,
    anchor_cross_size: f32,
    popup_cross_size: f32,
    align: PopupAlign,
) -> f32 {
    match align {
        PopupAlign::Start => anchor_cross_position,
        PopupAlign::Center => anchor_cross_position + (anchor_cross_size - popup_cross_size) * 0.5,
        PopupAlign::End => anchor_cross_position + anchor_cross_size - popup_cross_size,
    }
}

fn layout_overlay_adornment(
    node: &UiNode,
    adornment: &OverlayAdornmentNode,
    bounds: UiRect,
    state: &UiRuntimeState,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let Some(anchor_layout) = out.get(&adornment.anchor) else {
        out.insert(node.id, ComputedLayout::new(bounds, bounds, UiSize::ZERO));
        return UiSize::ZERO;
    };

    let content_size = measure_overlay_adornment_content(node);
    let measured_size = UiSize::new(
        content_size.width.max(adornment.min_size.width),
        content_size.height.max(adornment.min_size.height),
    );
    let anchor = anchor_layout.bounds;
    let (target_x, target_y, adornment_width, adornment_height) = match adornment.placement {
        PopupPlacement::InsideLeft => (
            anchor.x + adornment.offset,
            anchor.y,
            measured_size
                .width
                .min(anchor.width)
                .min(bounds.width.max(0.0)),
            anchor.height.min(bounds.height.max(0.0)),
        ),
        PopupPlacement::InsideRight => (
            anchor.x + anchor.width - measured_size.width - adornment.offset,
            anchor.y,
            measured_size
                .width
                .min(anchor.width)
                .min(bounds.width.max(0.0)),
            anchor.height.min(bounds.height.max(0.0)),
        ),
        PopupPlacement::InsideTop => (
            anchor.x,
            anchor.y + adornment.offset,
            anchor.width.min(bounds.width.max(0.0)),
            measured_size
                .height
                .min(anchor.height)
                .min(bounds.height.max(0.0)),
        ),
        PopupPlacement::InsideBottom => (
            anchor.x,
            anchor.y + anchor.height - measured_size.height - adornment.offset,
            anchor.width.min(bounds.width.max(0.0)),
            measured_size
                .height
                .min(anchor.height)
                .min(bounds.height.max(0.0)),
        ),
        _ => {
            let adornment_width = measured_size.width.min(bounds.width.max(0.0));
            let adornment_height = measured_size.height.min(bounds.height.max(0.0));
            let (target_x, target_y) = match adornment.placement {
                PopupPlacement::BottomStart => {
                    (anchor.x, anchor.y + anchor.height + adornment.offset)
                }
                PopupPlacement::RightStart => {
                    (anchor.x + anchor.width + adornment.offset, anchor.y)
                }
                PopupPlacement::InsideTopEnd => (
                    anchor.x + anchor.width - adornment_width - adornment.offset,
                    anchor.y + adornment.offset,
                ),
                PopupPlacement::InsideBottomStart => (
                    anchor.x + adornment.offset,
                    anchor.y + anchor.height - adornment_height - adornment.offset,
                ),
                PopupPlacement::TopStart => {
                    (anchor.x + adornment.offset, anchor.y + adornment.offset)
                }
                PopupPlacement::InsideLeft
                | PopupPlacement::InsideRight
                | PopupPlacement::InsideTop
                | PopupPlacement::InsideBottom
                | PopupPlacement::Outside { .. } => unreachable!(),
            };
            (target_x, target_y, adornment_width, adornment_height)
        }
    };
    let x = target_x.clamp(
        bounds.x,
        bounds.x + (bounds.width - adornment_width).max(0.0),
    );
    let y = target_y.clamp(
        bounds.y,
        bounds.y + (bounds.height - adornment_height).max(0.0),
    );
    let adornment_bounds = UiRect::new(x, y, adornment_width, adornment_height);
    let child_items = node
        .children
        .iter()
        .filter(|child| !is_popup_node(child))
        .map(|child| StackItem::auto(measure_node(child)))
        .collect::<Vec<_>>();
    let children = node
        .children
        .iter()
        .filter(|child| !is_popup_node(child))
        .collect::<Vec<_>>();
    if adornment.stretch_child && children.len() == 1 {
        layout_node(children[0], adornment_bounds, state, out);
    } else {
        let arranged = StackLayout::vertical(0.0)
            .with_main_align(MainAxisAlignment::Start)
            .with_cross_align(CrossAxisAlignment::Stretch)
            .arrange(adornment_bounds, &child_items);
        for (child, child_bounds) in children.into_iter().zip(arranged) {
            layout_node(child, child_bounds, state, out);
        }
    }

    out.insert(
        node.id,
        ComputedLayout::new(adornment_bounds, adornment_bounds, measured_size),
    );
    UiSize::ZERO
}

fn layout_radial_menu(
    node: &UiNode,
    radial: &RadialMenuNode,
    bounds: UiRect,
    state: &UiRuntimeState,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let outer_radius = radial.outer_radius.max(radial.inner_radius).max(1.0);
    let menu_size = UiSize::new(outer_radius * 2.0, outer_radius * 2.0);
    let Some(anchor_center) = radial_anchor_center(radial, out) else {
        out.insert(node.id, ComputedLayout::new(bounds, bounds, UiSize::ZERO));
        return UiSize::ZERO;
    };
    let x = (anchor_center.x - outer_radius).clamp(
        bounds.x,
        bounds.x + (bounds.width - menu_size.width).max(0.0),
    );
    let y = (anchor_center.y - outer_radius).clamp(
        bounds.y,
        bounds.y + (bounds.height - menu_size.height).max(0.0),
    );
    let menu_bounds = UiRect::new(x, y, menu_size.width, menu_size.height);
    let center_x = menu_bounds.x + menu_bounds.width * 0.5;
    let center_y = menu_bounds.y + menu_bounds.height * 0.5;
    let radius = ((radial.inner_radius + radial.outer_radius) * 0.5).max(0.0);
    let children = node
        .children
        .iter()
        .filter(|child| !is_popup_node(child))
        .collect::<Vec<_>>();
    let count = children.len().max(1) as f32;
    for (index, child) in children.into_iter().enumerate() {
        let angle = radial.start_angle_radians + index as f32 * std::f32::consts::TAU / count;
        let child_size = measure_node(child);
        let width = child_size.width.max(radial.item_size.width);
        let height = child_size.height.max(radial.item_size.height);
        let child_x = (center_x + angle.cos() * radius - width * 0.5).clamp(
            menu_bounds.x,
            menu_bounds.x + (menu_bounds.width - width).max(0.0),
        );
        let child_y = (center_y + angle.sin() * radius - height * 0.5).clamp(
            menu_bounds.y,
            menu_bounds.y + (menu_bounds.height - height).max(0.0),
        );
        layout_node(
            child,
            UiRect::new(child_x, child_y, width, height),
            state,
            out,
        );
    }

    out.insert(
        node.id,
        ComputedLayout::new(menu_bounds, menu_bounds, menu_size),
    );
    UiSize::ZERO
}

fn radial_anchor_center(
    radial: &RadialMenuNode,
    layouts: &ComputedLayoutMap,
) -> Option<ui_math::UiPoint> {
    match radial.anchor {
        RadialMenuAnchor::Widget(anchor) => {
            let anchor_layout = layouts.get(&anchor)?;
            Some(ui_math::UiPoint::new(
                anchor_layout.bounds.x + anchor_layout.bounds.width * 0.5,
                anchor_layout.bounds.y + anchor_layout.bounds.height * 0.5,
            ))
        }
        RadialMenuAnchor::Point(point) => Some(point),
    }
}

fn layout_split(
    node: &UiNode,
    split: &SplitNode,
    bounds: UiRect,
    state: &UiRuntimeState,
    out: &mut ComputedLayoutMap,
) -> UiSize {
    let layout = SplitLayout::new(split.axis, split.ratio, split.gap);

    let measured_size = bounds.size();

    match node.children.as_slice() {
        [left, right] => {
            let (left_bounds, right_bounds) = layout.arrange(bounds);
            layout_node(left, left_bounds, state, out);
            layout_node(right, right_bounds, state, out);
        }
        [left, right, handle] => {
            let (left_bounds, right_bounds) = layout.arrange(bounds);
            layout_node(left, left_bounds, state, out);
            layout_node(right, right_bounds, state, out);
            let handle_size = measure_node(handle);
            let handle_bounds = arrange_split_handle_bounds(
                split.axis,
                bounds,
                left_bounds,
                right_bounds,
                handle_size,
            );
            layout_node(handle, handle_bounds, state, out);
        }
        [only] => {
            layout_node(only, bounds, state, out);
        }
        _ => {}
    }

    out.insert(node.id, ComputedLayout::new(bounds, bounds, measured_size));

    measured_size
}

fn arrange_split_handle_bounds(
    axis: Axis,
    split_bounds: UiRect,
    first_bounds: UiRect,
    second_bounds: UiRect,
    handle_size: UiSize,
) -> UiRect {
    let handle_width = handle_size.width.min(split_bounds.width.max(0.0));
    let handle_height = handle_size.height.min(split_bounds.height.max(0.0));
    match axis {
        Axis::Horizontal => {
            let boundary = (first_bounds.x + first_bounds.width + second_bounds.x) * 0.5;
            let x = (boundary - handle_width * 0.5).clamp(
                split_bounds.x,
                split_bounds.x + (split_bounds.width - handle_width).max(0.0),
            );
            let y = (split_bounds.y + (split_bounds.height - handle_height) * 0.5).clamp(
                split_bounds.y,
                split_bounds.y + (split_bounds.height - handle_height).max(0.0),
            );
            UiRect::new(x, y, handle_width, handle_height)
        }
        Axis::Vertical => {
            let boundary = (first_bounds.y + first_bounds.height + second_bounds.y) * 0.5;
            let x = (split_bounds.x + (split_bounds.width - handle_width) * 0.5).clamp(
                split_bounds.x,
                split_bounds.x + (split_bounds.width - handle_width).max(0.0),
            );
            let y = (boundary - handle_height * 0.5).clamp(
                split_bounds.y,
                split_bounds.y + (split_bounds.height - handle_height).max(0.0),
            );
            UiRect::new(x, y, handle_width, handle_height)
        }
    }
}

fn measure_node(node: &UiNode) -> UiSize {
    match &node.kind {
        UiNodeKind::Panel(panel) => {
            let mut child_size = UiSize::ZERO;
            for child in node.children.iter().filter(|child| !is_popup_node(child)) {
                let measured = measure_node(child);
                child_size.width = child_size.width.max(measured.width);
                child_size.height = child_size.height.max(measured.height);
            }

            UiSize::new(
                (child_size.width + panel.padding.horizontal()).max(panel.min_size.width),
                (child_size.height + panel.padding.vertical()).max(panel.min_size.height),
            )
        }
        UiNodeKind::Popup(_) | UiNodeKind::RadialMenu(_) | UiNodeKind::OverlayAdornment(_) => {
            UiSize::ZERO
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
        UiNodeKind::TextInput(text_input) => {
            let line_height = text_input
                .text_style
                .line_height_or_default(text_input.text_style.font_size * 1.2);
            let display_text = if text_input.value.is_empty() {
                &text_input.placeholder
            } else {
                &text_input.value
            };
            let text_width =
                (display_text.chars().count() as f32 * text_input.text_style.font_size * 0.6)
                    .max(0.0);
            UiSize::new(
                (text_width + text_input.padding.horizontal()).max(text_input.min_size.width),
                (line_height + text_input.padding.vertical()).max(text_input.min_size.height),
            )
        }
        UiNodeKind::Toggle(toggle) => {
            let line_height = toggle
                .text_style
                .line_height_or_default(toggle.text_style.font_size * 1.2);
            let label_width =
                (toggle.label.chars().count() as f32 * toggle.text_style.font_size * 0.6).max(0.0);
            UiSize::new(
                (label_width + toggle.padding.horizontal() + line_height)
                    .max(toggle.min_size.width),
                (line_height + toggle.padding.vertical()).max(toggle.min_size.height),
            )
        }
        UiNodeKind::NumericInput(numeric) => {
            let line_height = numeric
                .text_style
                .line_height_or_default(numeric.text_style.font_size * 1.2);
            let value_text = format!("{:.*}", usize::from(numeric.precision), numeric.value);
            let text_width =
                (value_text.chars().count() as f32 * numeric.text_style.font_size * 0.6).max(0.0);
            UiSize::new(
                (text_width + numeric.padding.horizontal() + line_height)
                    .max(numeric.min_size.width),
                (line_height + numeric.padding.vertical()).max(numeric.min_size.height),
            )
        }
        UiNodeKind::Tabs(tabs) => {
            let line_height = tabs
                .text_style
                .line_height_or_default(tabs.text_style.font_size * 1.2);
            let width = tabs
                .labels
                .iter()
                .map(|label| {
                    label.chars().count() as f32 * tabs.text_style.font_size * 0.6
                        + tabs.padding.horizontal()
                })
                .sum::<f32>()
                .max(tabs.min_size.width);
            UiSize::new(
                width,
                (line_height + tabs.padding.vertical()).max(tabs.min_size.height),
            )
        }
        UiNodeKind::Select(select) => {
            let line_height = select
                .text_style
                .line_height_or_default(select.text_style.font_size * 1.2);
            let display_text = select
                .selected_index
                .and_then(|index| select.options.get(index))
                .unwrap_or(&select.placeholder);
            let text_width =
                (display_text.chars().count() as f32 * select.text_style.font_size * 0.6).max(0.0);
            UiSize::new(
                (text_width + select.padding.horizontal() + line_height).max(select.min_size.width),
                (line_height + select.padding.vertical()).max(select.min_size.height),
            )
        }
        UiNodeKind::Table(table) => table_size(table),
        UiNodeKind::Tree(tree) => tree_size(tree),
        UiNodeKind::Spacer(spacer) => spacer.min_size,
        UiNodeKind::Divider(divider) => divider_intrinsic_size(divider),
        UiNodeKind::Image(image) => image.min_size,
        UiNodeKind::ProductSurface(surface) => surface.min_size,
        UiNodeKind::GraphCanvas(graph_canvas) => graph_canvas.min_size,
        UiNodeKind::ViewportSurfaceEmbed(embed) => embed.min_size,
        UiNodeKind::Scroll(_) => node
            .children
            .first()
            .map(measure_node)
            .unwrap_or(UiSize::ZERO),
        UiNodeKind::Stack(stack) => {
            let mut items = Vec::with_capacity(node.children.len());
            for (index, child) in node
                .children
                .iter()
                .filter(|child| !is_popup_node(child))
                .enumerate()
            {
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
        UiNodeKind::Split(split) => measure_split(node, split),
    }
}

fn measure_popup_content(node: &UiNode, popup: &PopupNode) -> UiSize {
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;
    let mut item_count = 0_usize;
    for child in node.children.iter().filter(|child| !is_popup_node(child)) {
        let measured = measure_node(child);
        width = width.max(measured.width);
        height += measured.height;
        item_count += 1;
    }
    if item_count > 1 {
        height += popup.gap * (item_count - 1) as f32;
    }
    UiSize::new(width, height)
}

fn measure_overlay_adornment_content(node: &UiNode) -> UiSize {
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;
    for child in node.children.iter().filter(|child| !is_popup_node(child)) {
        let measured = measure_node(child);
        width = width.max(measured.width);
        height += measured.height;
    }
    UiSize::new(width, height)
}

fn is_popup_node(node: &UiNode) -> bool {
    matches!(
        node.kind,
        UiNodeKind::Popup(_) | UiNodeKind::RadialMenu(_) | UiNodeKind::OverlayAdornment(_)
    )
}

fn table_size(table: &TableNode) -> UiSize {
    let width = table
        .columns
        .iter()
        .map(|column| column.min_width)
        .sum::<f32>()
        .max(table.min_size.width);
    let height = (table.rows.len() as f32 + 1.0) * table.row_height;
    UiSize::new(width, height.max(table.min_size.height))
}

fn tree_size(tree: &TreeNode) -> UiSize {
    let height = tree.rows.len().max(1) as f32 * tree.row_height;
    UiSize::new(tree.min_size.width, height.max(tree.min_size.height))
}

fn divider_size_for_bounds(divider: &DividerNode, bounds: UiRect) -> UiSize {
    let length = match divider.length_policy {
        SizePolicy::Auto | SizePolicy::Flex(_) => match divider.axis {
            Axis::Horizontal => bounds.width.max(0.0),
            Axis::Vertical => bounds.height.max(0.0),
        },
        SizePolicy::Fixed(value) => value.max(0.0),
    };
    let thickness = divider.thickness.max(0.0);

    match divider.axis {
        Axis::Horizontal => UiSize::new(length, thickness),
        Axis::Vertical => UiSize::new(thickness, length),
    }
}

fn divider_intrinsic_size(divider: &DividerNode) -> UiSize {
    let length = match divider.length_policy {
        SizePolicy::Fixed(value) => value.max(0.0),
        SizePolicy::Auto | SizePolicy::Flex(_) => 0.0,
    };
    let thickness = divider.thickness.max(0.0);

    match divider.axis {
        Axis::Horizontal => UiSize::new(length, thickness),
        Axis::Vertical => UiSize::new(thickness, length),
    }
}

fn measure_split(node: &UiNode, split: &SplitNode) -> UiSize {
    match node.children.as_slice() {
        [left, right] => {
            let left_size = measure_node(left);
            let right_size = measure_node(right);
            match split.axis {
                Axis::Horizontal => UiSize::new(
                    left_size.width + split.gap.max(0.0) + right_size.width,
                    left_size.height.max(right_size.height),
                ),
                Axis::Vertical => UiSize::new(
                    left_size.width.max(right_size.width),
                    left_size.height + split.gap.max(0.0) + right_size.height,
                ),
            }
        }
        [left, right, _] => {
            let left_size = measure_node(left);
            let right_size = measure_node(right);
            match split.axis {
                Axis::Horizontal => UiSize::new(
                    left_size.width + split.gap.max(0.0) + right_size.width,
                    left_size.height.max(right_size.height),
                ),
                Axis::Vertical => UiSize::new(
                    left_size.width.max(right_size.width),
                    left_size.height + split.gap.max(0.0) + right_size.height,
                ),
            }
        }
        [only] => measure_node(only),
        _ => UiSize::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiNode, UiNodeKind, WidgetId};
    use ui_render_data::{UiDrawKey, UiPaint};
    use ui_theme::UiColor;

    #[test]
    fn spacer_layout_preserves_min_size_and_stack_position() {
        let spacer_id = WidgetId(2);
        let button_id = WidgetId(3);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Stack(StackNode::vertical(4.0)),
            vec![
                UiNode::new(
                    spacer_id,
                    UiNodeKind::Spacer(SpacerNode::new(UiSize::new(20.0, 12.0))),
                ),
                UiNode::new(
                    button_id,
                    UiNodeKind::Button(ButtonNode::new(
                        "Apply",
                        ui_text::TextStyle::default(),
                        ui_theme::ThemeTokens::default(),
                    )),
                ),
            ],
        ));

        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(0.0, 0.0, 160.0, 80.0),
            &UiRuntimeState::default(),
        );

        let spacer = layouts.get(&spacer_id).expect("spacer layout should exist");
        let button = layouts.get(&button_id).expect("button layout should exist");

        assert_eq!(spacer.measured_size, UiSize::new(160.0, 12.0));
        assert_eq!(spacer.bounds.height, 12.0);
        assert!((button.bounds.y - 16.0).abs() < 0.001);
    }

    #[test]
    fn popup_layout_anchors_without_consuming_stack_space() {
        let anchor_id = WidgetId(2);
        let content_id = WidgetId(3);
        let popup_id = WidgetId(4);
        let popup_item_id = WidgetId(5);
        let theme = ui_theme::ThemeTokens::default();
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                WidgetId(10),
                UiNodeKind::Stack(StackNode::vertical(4.0)),
                vec![
                    UiNode::new(
                        anchor_id,
                        UiNodeKind::Button(ButtonNode::new(
                            "File",
                            ui_text::TextStyle::default(),
                            theme.clone(),
                        )),
                    ),
                    UiNode::new(
                        content_id,
                        UiNodeKind::Spacer(SpacerNode::new(UiSize::new(40.0, 30.0))),
                    ),
                    UiNode::with_children(
                        popup_id,
                        UiNodeKind::Popup(PopupNode::anchored_bottom_start(anchor_id, theme)),
                        vec![UiNode::new(
                            popup_item_id,
                            UiNodeKind::Button(ButtonNode::new(
                                "Save",
                                ui_text::TextStyle::default(),
                                ui_theme::ThemeTokens::default(),
                            )),
                        )],
                    ),
                ],
            )],
        ));

        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(0.0, 0.0, 220.0, 160.0),
            &UiRuntimeState::default(),
        );

        let anchor = layouts.get(&anchor_id).expect("anchor layout should exist");
        let content = layouts
            .get(&content_id)
            .expect("content layout should exist");
        let popup = layouts.get(&popup_id).expect("popup layout should exist");

        assert!((content.bounds.y - (anchor.bounds.y + anchor.bounds.height + 4.0)).abs() < 0.001);
        assert!(popup.bounds.y >= anchor.bounds.y + anchor.bounds.height);
        assert!(popup.bounds.y < content.bounds.y + content.bounds.height);
    }

    #[test]
    fn outside_popup_flips_above_when_bottom_space_is_insufficient() {
        let anchor_id = WidgetId(2);
        let popup_id = WidgetId(3);
        let theme = ui_theme::ThemeTokens::default();
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                WidgetId(10),
                UiNodeKind::Stack(StackNode::vertical(4.0)),
                vec![
                    UiNode::new(
                        WidgetId(11),
                        UiNodeKind::Spacer(SpacerNode::new(UiSize::new(20.0, 112.0))),
                    ),
                    UiNode::new(
                        anchor_id,
                        UiNodeKind::Button(ButtonNode::new(
                            "Menu",
                            ui_text::TextStyle::default(),
                            theme.clone(),
                        )),
                    ),
                    UiNode::with_children(
                        popup_id,
                        UiNodeKind::Popup(PopupNode::anchored_outside(
                            anchor_id,
                            PopupSide::Bottom,
                            PopupAlign::Start,
                            PopupFlipPolicy::FlipToFit,
                            theme.clone(),
                        )),
                        vec![UiNode::new(
                            WidgetId(12),
                            UiNodeKind::Button(ButtonNode::new(
                                "Item",
                                ui_text::TextStyle::default(),
                                theme,
                            )),
                        )],
                    ),
                ],
            )],
        ));

        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(0.0, 0.0, 220.0, 160.0),
            &UiRuntimeState::default(),
        );
        let anchor = layouts.get(&anchor_id).expect("anchor layout should exist");
        let popup = layouts.get(&popup_id).expect("popup layout should exist");

        assert!(
            popup.bounds.y + popup.bounds.height <= anchor.bounds.y,
            "bottom-preferred menu should flip above instead of covering its anchor"
        );
    }

    #[test]
    fn outside_popup_stays_below_when_bottom_space_fits() {
        let anchor_id = WidgetId(2);
        let popup_id = WidgetId(3);
        let theme = ui_theme::ThemeTokens::default();
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![
                UiNode::new(
                    anchor_id,
                    UiNodeKind::Button(ButtonNode::new(
                        "Menu",
                        ui_text::TextStyle::default(),
                        theme.clone(),
                    )),
                ),
                UiNode::with_children(
                    popup_id,
                    UiNodeKind::Popup(PopupNode::anchored_outside(
                        anchor_id,
                        PopupSide::Bottom,
                        PopupAlign::Start,
                        PopupFlipPolicy::FlipToFit,
                        theme.clone(),
                    )),
                    vec![UiNode::new(
                        WidgetId(12),
                        UiNodeKind::Button(ButtonNode::new(
                            "Item",
                            ui_text::TextStyle::default(),
                            theme,
                        )),
                    )],
                ),
            ],
        ));

        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(0.0, 0.0, 220.0, 160.0),
            &UiRuntimeState::default(),
        );
        let anchor = layouts.get(&anchor_id).expect("anchor layout should exist");
        let popup = layouts.get(&popup_id).expect("popup layout should exist");

        assert!(popup.bounds.y >= anchor.bounds.y + anchor.bounds.height);
    }

    #[test]
    fn outside_popup_caps_oversized_menu_to_larger_available_side() {
        let anchor_id = WidgetId(2);
        let popup_id = WidgetId(3);
        let theme = ui_theme::ThemeTokens::default();
        let popup_items = (0..12)
            .map(|index| {
                UiNode::new(
                    WidgetId(20 + index),
                    UiNodeKind::Button(ButtonNode::new(
                        format!("Item {index}"),
                        ui_text::TextStyle::default(),
                        theme.clone(),
                    )),
                )
            })
            .collect::<Vec<_>>();
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                WidgetId(10),
                UiNodeKind::Stack(StackNode::vertical(4.0)),
                vec![
                    UiNode::new(
                        WidgetId(11),
                        UiNodeKind::Spacer(SpacerNode::new(UiSize::new(20.0, 96.0))),
                    ),
                    UiNode::new(
                        anchor_id,
                        UiNodeKind::Button(ButtonNode::new(
                            "Menu",
                            ui_text::TextStyle::default(),
                            theme.clone(),
                        )),
                    ),
                    UiNode::with_children(
                        popup_id,
                        UiNodeKind::Popup(PopupNode::anchored_outside(
                            anchor_id,
                            PopupSide::Bottom,
                            PopupAlign::Start,
                            PopupFlipPolicy::FlipToFit,
                            theme,
                        )),
                        popup_items,
                    ),
                ],
            )],
        ));

        let bounds = UiRect::new(0.0, 0.0, 220.0, 180.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());
        let anchor = layouts.get(&anchor_id).expect("anchor layout should exist");
        let popup = layouts.get(&popup_id).expect("popup layout should exist");

        assert!(popup.bounds.y >= bounds.y);
        assert!(popup.bounds.y + popup.bounds.height <= anchor.bounds.y);
        assert!(popup.bounds.height < popup.measured_size.height);
    }

    #[test]
    fn popup_menu_stretches_scroll_list_items_after_clamp() {
        let anchor_id = WidgetId(2);
        let popup_id = WidgetId(3);
        let scroll_id = WidgetId(4);
        let list_id = WidgetId(5);
        let theme = ui_theme::ThemeTokens::default();
        let text_style = ui_text::TextStyle::default();
        let mut menu_items = Vec::new();
        for index in 0..10 {
            let mut button = ButtonNode::new(
                format!("Long menu item label {index}"),
                text_style.clone(),
                theme.clone(),
            );
            button.fill_width = true;
            menu_items.push(UiNode::new(
                WidgetId(20 + index),
                UiNodeKind::Button(button),
            ));
        }

        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![
                UiNode::new(
                    anchor_id,
                    UiNodeKind::Button(ButtonNode::new("Menu", text_style, theme.clone())),
                ),
                UiNode::with_children(
                    popup_id,
                    UiNodeKind::Popup(PopupNode::anchored_outside(
                        anchor_id,
                        PopupSide::Bottom,
                        PopupAlign::Start,
                        PopupFlipPolicy::FlipToFit,
                        theme.clone(),
                    )),
                    vec![UiNode::with_children(
                        scroll_id,
                        UiNodeKind::Scroll(ScrollNode::vertical(theme.clone())),
                        vec![UiNode::with_children(
                            list_id,
                            UiNodeKind::Stack(StackNode::vertical(theme.spacing.xs)),
                            menu_items,
                        )],
                    )],
                ),
            ],
        ));

        let bounds = UiRect::new(0.0, 0.0, 180.0, 128.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());
        let popup = layouts.get(&popup_id).expect("popup should lay out");
        let scroll = layouts
            .get(&scroll_id)
            .expect("scroll fallback should lay out");
        let list = layouts.get(&list_id).expect("menu list should lay out");
        let first_item = layouts
            .get(&WidgetId(20))
            .expect("first menu item should lay out");

        assert!(
            popup.bounds.width <= bounds.width,
            "popup width should clamp to available frame width"
        );
        assert!(
            popup.bounds.height < popup.measured_size.height,
            "popup should retain measured height while clamping visible height"
        );
        assert_eq!(
            scroll.bounds, popup.content_bounds,
            "single scroll child should stretch to popup content bounds after clamp"
        );
        assert_eq!(
            list.bounds.width, scroll.content_bounds.width,
            "menu list should fill the clamped scroll viewport width"
        );
        assert_eq!(
            first_item.bounds.width, list.content_bounds.width,
            "fill-width menu items should stretch to measured menu width"
        );
        assert!(
            list.bounds.height > scroll.content_bounds.height,
            "menu list should preserve overflow for scroll fallback"
        );
    }

    #[test]
    fn divider_layout_respects_axis_thickness_and_fixed_length() {
        let horizontal_id = WidgetId(10);
        let vertical_id = WidgetId(11);
        let color = UiColor::new(0.2, 0.3, 0.4, 1.0);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Stack(StackNode::vertical(2.0)),
            vec![
                UiNode::new(
                    horizontal_id,
                    UiNodeKind::Divider(DividerNode::new(
                        Axis::Horizontal,
                        2.0,
                        SizePolicy::Fixed(32.0),
                        color,
                    )),
                ),
                UiNode::new(
                    vertical_id,
                    UiNodeKind::Divider(DividerNode::new(
                        Axis::Vertical,
                        3.0,
                        SizePolicy::Fixed(24.0),
                        color,
                    )),
                ),
            ],
        ));

        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(0.0, 0.0, 120.0, 80.0),
            &UiRuntimeState::default(),
        );

        let horizontal = layouts
            .get(&horizontal_id)
            .expect("horizontal divider layout should exist");
        let vertical = layouts
            .get(&vertical_id)
            .expect("vertical divider layout should exist");

        assert_eq!(horizontal.measured_size, UiSize::new(32.0, 2.0));
        assert_eq!(horizontal.bounds.height, 2.0);
        assert_eq!(vertical.measured_size, UiSize::new(3.0, 24.0));
        assert_eq!(vertical.bounds.width, 3.0);
    }

    #[test]
    fn image_layout_preserves_min_size() {
        let image_id = WidgetId(5);
        let tree = UiTree::new(UiNode::new(
            image_id,
            UiNodeKind::Image(ImageNode::new(
                UiDrawKey::new(7, Some(8)),
                UiRect::new(0.0, 0.0, 1.0, 1.0),
                UiPaint::WHITE,
                UiSize::new(48.0, 32.0),
            )),
        ));

        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(4.0, 6.0, 24.0, 16.0),
            &UiRuntimeState::default(),
        );

        let image = layouts.get(&image_id).expect("image layout should exist");
        assert_eq!(image.bounds, UiRect::new(4.0, 6.0, 24.0, 16.0));
        assert_eq!(image.measured_size, UiSize::new(48.0, 32.0));
    }

    #[test]
    fn scroll_layout_preserves_nonzero_content_viewport_in_tight_bounds() {
        let scroll_id = WidgetId(21);
        let tree = UiTree::new(UiNode::with_children(
            scroll_id,
            UiNodeKind::Scroll(ScrollNode::vertical(ui_theme::ThemeTokens::default())),
            vec![UiNode::with_children(
                WidgetId(22),
                UiNodeKind::Stack(StackNode::vertical(2.0)),
                vec![
                    UiNode::new(
                        WidgetId(23),
                        UiNodeKind::Button(ButtonNode::new(
                            "one",
                            ui_text::TextStyle::default(),
                            ui_theme::ThemeTokens::default(),
                        )),
                    ),
                    UiNode::new(
                        WidgetId(24),
                        UiNodeKind::Button(ButtonNode::new(
                            "two",
                            ui_text::TextStyle::default(),
                            ui_theme::ThemeTokens::default(),
                        )),
                    ),
                ],
            )],
        ));

        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(0.0, 0.0, 80.0, 6.0),
            &UiRuntimeState::default(),
        );
        let scroll = layouts.get(&scroll_id).expect("scroll layout should exist");

        assert!(
            scroll.content_bounds.height > 0.0,
            "tight overflow layouts should keep a non-zero content viewport for clipping",
        );
    }
}
