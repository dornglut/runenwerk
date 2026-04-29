//! File: domain/ui/ui_runtime/src/output/build_ui_frame.rs
//! Purpose: Convert retained tree + computed layout into UiFrame.

use crate::{
    ButtonNode, ComputedLayoutMap, DividerNode, ImageNode, LabelNode, NumericInputNode, PanelNode,
    ScrollNode, SelectNode, TableNode, TabsNode, TextInputNode, ToggleNode, TreeNode, UiNode,
    UiNodeKind, UiTree, ViewportSurfaceEmbedNode, WidgetId,
};
use ui_math::{UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, ClipPrimitive, GlyphRunPrimitive, ImagePrimitive, RectPrimitive, UiDrawKey,
    UiFrame, UiLayer, UiLayerId, UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
    ViewportSurfaceEmbedPrimitive,
};
use ui_text::{AtlasTextLayouter, FontAtlasSource, TextLayoutRequest, TextLayouter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InteractionVisualState {
    pub hovered_widget: Option<WidgetId>,
    pub pressed_widget: Option<WidgetId>,
    pub focused_widget: Option<WidgetId>,
}

pub fn build_ui_frame(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    surface_size: UiSize,
    interaction_state: InteractionVisualState,
    atlas_source: &dyn FontAtlasSource,
) -> UiFrame {
    let mut layer = UiLayer::new(UiLayerId(0));
    let mut primitive_order = 0u32;
    let layouter = AtlasTextLayouter;

    emit_node(
        tree,
        &tree.root,
        layouts,
        &mut layer,
        surface_size,
        atlas_source,
        &layouter,
        interaction_state,
        0,
        &mut primitive_order,
    );

    UiFrame::with_surfaces(vec![UiSurface::with_layers(
        UiSurfaceId(0),
        surface_size,
        vec![layer],
    )])
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_node(
    tree: &UiTree,
    node: &UiNode,
    layouts: &ComputedLayoutMap,
    layer: &mut UiLayer,
    surface_size: UiSize,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let Some(layout) = layouts.get(&node.id) else {
        return;
    };

    match &node.kind {
        UiNodeKind::Panel(panel) => emit_panel(
            panel,
            layout.bounds,
            layout.content_bounds,
            layer,
            depth,
            primitive_order,
        ),
        UiNodeKind::Label(label) => emit_label(
            label,
            layout.bounds,
            layer,
            atlas_source,
            layouter,
            depth,
            primitive_order,
        ),
        UiNodeKind::Button(button) => emit_button(
            node.id,
            button,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state,
            depth,
            primitive_order,
        ),
        UiNodeKind::TextInput(text_input) => emit_text_input(
            node.id,
            text_input,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state,
            depth,
            primitive_order,
        ),
        UiNodeKind::Toggle(toggle) => emit_toggle(
            node.id,
            toggle,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state,
            depth,
            primitive_order,
        ),
        UiNodeKind::NumericInput(numeric) => emit_numeric_input(
            node.id,
            numeric,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state,
            depth,
            primitive_order,
        ),
        UiNodeKind::Tabs(tabs) => emit_tabs(
            node.id,
            tabs,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state,
            depth,
            primitive_order,
        ),
        UiNodeKind::Select(select) => emit_select(
            node.id,
            select,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state,
            depth,
            primitive_order,
        ),
        UiNodeKind::Table(table) => emit_table(
            node.id,
            table,
            layout.bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state,
            depth,
            primitive_order,
        ),
        UiNodeKind::Tree(tree) => emit_tree(
            node.id,
            tree,
            layout.bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state,
            depth,
            primitive_order,
        ),
        UiNodeKind::Spacer(_) => {}
        UiNodeKind::Divider(divider) => {
            emit_divider(divider, layout.bounds, layer, depth, primitive_order)
        }
        UiNodeKind::Image(image) => emit_image(image, layout.bounds, layer, depth, primitive_order),
        UiNodeKind::ViewportSurfaceEmbed(embed) => emit_viewport_surface_embed(
            embed,
            layout.bounds,
            surface_size,
            layer,
            depth,
            primitive_order,
        ),
        UiNodeKind::Scroll(scroll) => emit_scroll_begin(
            scroll,
            layout.bounds,
            layout.content_bounds,
            layer,
            depth,
            primitive_order,
        ),
        UiNodeKind::Stack(_) | UiNodeKind::Split(_) => {}
    }

    for child in &node.children {
        emit_node(
            tree,
            child,
            layouts,
            layer,
            surface_size,
            atlas_source,
            layouter,
            interaction_state,
            depth + 1,
            primitive_order,
        );
    }

    match &node.kind {
        UiNodeKind::Panel(_)
        | UiNodeKind::Button(_)
        | UiNodeKind::TextInput(_)
        | UiNodeKind::NumericInput(_)
        | UiNodeKind::Tabs(_)
        | UiNodeKind::Select(_)
        | UiNodeKind::Table(_)
        | UiNodeKind::Tree(_) => {
            layer.push(UiPrimitive::Clip(ClipPrimitive::Pop {
                sort_key: sort_key(depth, *primitive_order),
            }));
            *primitive_order += 1;
        }
        UiNodeKind::Scroll(scroll) => {
            layer.push(UiPrimitive::Clip(ClipPrimitive::Pop {
                sort_key: sort_key(depth, *primitive_order),
            }));
            *primitive_order += 1;

            emit_scrollbar(
                tree,
                node,
                scroll,
                layouts,
                layout.bounds,
                layout.content_bounds,
                layer,
                depth,
                primitive_order,
            );
        }
        UiNodeKind::Label(_)
        | UiNodeKind::Toggle(_)
        | UiNodeKind::Spacer(_)
        | UiNodeKind::Divider(_)
        | UiNodeKind::Image(_)
        | UiNodeKind::ViewportSurfaceEmbed(_)
        | UiNodeKind::Stack(_)
        | UiNodeKind::Split(_) => {}
    }
}

fn emit_panel(
    panel: &PanelNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        panel.theme.radius.md,
        paint_from_color(panel.theme.background_panel),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        panel.theme.radius.md,
        panel.theme.border_width,
        paint_from_color(panel.theme.border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: content_bounds,
        sort_key: sort_key(depth, *primitive_order),
    }));
    *primitive_order += 1;
}

fn emit_scroll_begin(
    _scroll: &ScrollNode,
    _bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: content_bounds,
        sort_key: sort_key(depth, *primitive_order),
    }));
    *primitive_order += 1;
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_scrollbar(
    tree: &UiTree,
    node: &UiNode,
    scroll: &ScrollNode,
    layouts: &ComputedLayoutMap,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    let Some(child) = node.children.first() else {
        return;
    };
    let Some(child_layout) = layouts.get(&child.id) else {
        return;
    };
    let (track_rect, thumb_rect, radius) = match scroll.axis {
        ui_math::Axis::Vertical => {
            let track_width = (bounds.width - content_bounds.width).max(0.0);
            if track_width <= f32::EPSILON || content_bounds.height <= f32::EPSILON {
                return;
            }
            let track_x = if let Some(parent) = find_parent_node(tree, node.id) {
                if let UiNodeKind::Scroll(parent_scroll) = &parent.kind {
                    if parent_scroll.axis == ui_math::Axis::Horizontal {
                        layouts
                            .get(&parent.id)
                            .map(|layout| {
                                layout.content_bounds.x + layout.content_bounds.width - track_width
                            })
                            .unwrap_or(content_bounds.x + content_bounds.width)
                    } else {
                        content_bounds.x + content_bounds.width
                    }
                } else {
                    content_bounds.x + content_bounds.width
                }
            } else {
                content_bounds.x + content_bounds.width
            };
            let track_rect = UiRect::new(
                track_x,
                content_bounds.y,
                track_width,
                content_bounds.height,
            );
            let viewport_extent = content_bounds.height.max(0.0);
            let content_extent = child_layout.bounds.height.max(viewport_extent);
            let max_offset = (content_extent - viewport_extent).max(0.0);
            if max_offset <= f32::EPSILON {
                return;
            }
            let scroll_offset = (content_bounds.y - child_layout.bounds.y).clamp(0.0, max_offset);
            let min_thumb = scroll.min_thumb_main_size.min(track_rect.height).max(0.0);
            let natural = (viewport_extent / content_extent) * track_rect.height;
            let thumb_extent = natural.clamp(min_thumb, track_rect.height);
            let thumb_range = (track_rect.height - thumb_extent).max(0.0);
            let thumb_y = track_rect.y + thumb_range * (scroll_offset / max_offset);
            let thumb_rect = UiRect::new(track_rect.x, thumb_y, track_rect.width, thumb_extent);
            let radius = scroll.theme.radius.sm.min(track_rect.width * 0.5);
            (track_rect, thumb_rect, radius)
        }
        ui_math::Axis::Horizontal => {
            let track_height = (bounds.height - content_bounds.height).max(0.0);
            if track_height <= f32::EPSILON || content_bounds.width <= f32::EPSILON {
                return;
            }
            let track_rect = UiRect::new(
                content_bounds.x,
                content_bounds.y + content_bounds.height,
                content_bounds.width,
                track_height,
            );
            let viewport_extent = content_bounds.width.max(0.0);
            let content_extent = child_layout.bounds.width.max(viewport_extent);
            let max_offset = (content_extent - viewport_extent).max(0.0);
            if max_offset <= f32::EPSILON {
                return;
            }
            let scroll_offset = (content_bounds.x - child_layout.bounds.x).clamp(0.0, max_offset);
            let min_thumb = scroll.min_thumb_main_size.min(track_rect.width).max(0.0);
            let natural = (viewport_extent / content_extent) * track_rect.width;
            let thumb_extent = natural.clamp(min_thumb, track_rect.width);
            let thumb_range = (track_rect.width - thumb_extent).max(0.0);
            let thumb_x = track_rect.x + thumb_range * (scroll_offset / max_offset);
            let thumb_rect = UiRect::new(thumb_x, track_rect.y, thumb_extent, track_rect.height);
            let radius = scroll.theme.radius.sm.min(track_rect.height * 0.5);
            (track_rect, thumb_rect, radius)
        }
    };

    let mut track_color = scroll.theme.border;
    track_color.a = (track_color.a * 0.35).clamp(0.0, 1.0);
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        track_rect,
        radius,
        paint_from_color(track_color),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    let mut thumb_color = scroll.theme.accent;
    thumb_color.a = (thumb_color.a * 0.80).clamp(0.0, 1.0);
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        thumb_rect,
        radius,
        paint_from_color(thumb_color),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
}

fn find_parent_node(tree: &UiTree, target: WidgetId) -> Option<&UiNode> {
    find_parent_node_inner(&tree.root, target)
}

fn find_parent_node_inner(node: &UiNode, target: WidgetId) -> Option<&UiNode> {
    for child in &node.children {
        if child.id == target {
            return Some(node);
        }
        if let Some(found) = find_parent_node_inner(child, target) {
            return Some(found);
        }
    }
    None
}

fn emit_viewport_surface_embed(
    embed: &ViewportSurfaceEmbedNode,
    bounds: UiRect,
    surface_size: UiSize,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    let uv_rect = normalized_uv_rect(bounds, surface_size);
    layer.push(UiPrimitive::ViewportSurfaceEmbed(
        ViewportSurfaceEmbedPrimitive::new(
            embed.viewport_id,
            embed.slot,
            bounds,
            uv_rect,
            UiPaint::rgba(1.0, 1.0, 1.0, 1.0),
            sort_key(depth, *primitive_order),
        ),
    ));
    *primitive_order += 1;
}

fn emit_divider(
    divider: &DividerNode,
    bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        0.0,
        paint_from_color(divider.color),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
}

fn emit_image(
    image: &ImageNode,
    bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Image(ImagePrimitive::new(
        bounds,
        image.uv_rect,
        image.tint,
        image.draw_key,
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
}

fn normalized_uv_rect(bounds: UiRect, surface_size: UiSize) -> UiRect {
    let width = surface_size.width.max(1.0);
    let height = surface_size.height.max(1.0);

    let u0 = (bounds.x / width).clamp(0.0, 1.0);
    let v0 = (bounds.y / height).clamp(0.0, 1.0);
    let u1 = ((bounds.x + bounds.width) / width).clamp(0.0, 1.0);
    let v1 = ((bounds.y + bounds.height) / height).clamp(0.0, 1.0);

    UiRect::new(u0, v0, (u1 - u0).max(0.0), (v1 - v0).max(0.0))
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_button(
    widget_id: WidgetId,
    button: &ButtonNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let interaction = interaction_state.for_widget(widget_id);
    let mut background = if button.enabled {
        if button.selected {
            button.selected_fill.unwrap_or(button.theme.accent)
        } else {
            button.theme.background_panel
        }
    } else {
        with_alpha(button.theme.border, 0.35)
    };
    if interaction.hovered {
        background = brighten(background, 1.08);
    }
    if interaction.pressed {
        background = darken(background, 0.88);
    }

    let mut border = if button.selected {
        button.selected_border.unwrap_or(button.theme.accent)
    } else {
        button.theme.border
    };
    if interaction.focused {
        border = brighten(button.theme.accent, 1.04);
    } else if interaction.hovered {
        border = brighten(border, 1.08);
    }

    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        button.theme.radius.sm,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        button.theme.radius.sm,
        button.theme.border_width,
        paint_from_color(border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: content_bounds,
        sort_key: sort_key(depth, *primitive_order),
    }));
    *primitive_order += 1;

    let text_rect = UiRect::new(
        content_bounds.x,
        content_bounds.y,
        content_bounds.width,
        content_bounds.height,
    );

    let label_node = LabelNode {
        text: button.label.clone(),
        text_style: button_text_style(button, interaction),
        constraints: ui_layout::LayoutConstraints::tight(text_rect.size()),
    };

    emit_label(
        &label_node,
        text_rect,
        layer,
        atlas_source,
        layouter,
        depth + 1,
        primitive_order,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_text_input(
    widget_id: WidgetId,
    text_input: &TextInputNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let interaction = interaction_state.for_widget(widget_id);
    let mut background = if text_input.editable {
        text_input.theme.background_panel
    } else {
        with_alpha(text_input.theme.border, 0.30)
    };
    if interaction.hovered {
        background = brighten(background, 1.04);
    }
    if interaction.focused {
        background = brighten(background, 1.06);
    }

    let mut border = text_input.theme.border;
    if interaction.focused {
        border = brighten(text_input.theme.accent, 1.03);
    } else if interaction.hovered {
        border = brighten(border, 1.08);
    }

    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        text_input.theme.radius.sm,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        text_input.theme.radius.sm,
        text_input.theme.border_width,
        paint_from_color(border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: content_bounds,
        sort_key: sort_key(depth, *primitive_order),
    }));
    *primitive_order += 1;

    let mut text_style = text_input.text_style.clone();
    let text = if text_input.value.is_empty() {
        text_style.color[3] = (text_style.color[3] * 0.6).clamp(0.0, 1.0);
        text_input.placeholder.clone()
    } else {
        text_input.value.clone()
    };
    let label = LabelNode {
        text,
        text_style,
        constraints: ui_layout::LayoutConstraints::tight(content_bounds.size()),
    };
    emit_label(
        &label,
        content_bounds,
        layer,
        atlas_source,
        layouter,
        depth + 1,
        primitive_order,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_toggle(
    widget_id: WidgetId,
    toggle: &ToggleNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let interaction = interaction_state.for_widget(widget_id);
    let mut background = if toggle.enabled {
        toggle.theme.background_panel
    } else {
        with_alpha(toggle.theme.border, 0.30)
    };
    if interaction.hovered {
        background = brighten(background, 1.04);
    }
    let mut border = toggle.theme.border;
    if interaction.focused {
        border = brighten(toggle.theme.accent, 1.03);
    } else if interaction.hovered {
        border = brighten(border, 1.08);
    }

    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        toggle.theme.radius.sm,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        toggle.theme.radius.sm,
        toggle.theme.border_width,
        paint_from_color(border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    let indicator_size = content_bounds.height.min(content_bounds.width).max(0.0);
    let indicator_rect = UiRect::new(
        content_bounds.x,
        content_bounds.y,
        indicator_size,
        indicator_size,
    );
    let mut indicator_color = if toggle.checked {
        toggle.theme.accent
    } else {
        toggle.theme.border
    };
    if !toggle.enabled {
        indicator_color.a = (indicator_color.a * 0.5).clamp(0.0, 1.0);
    }
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        indicator_rect,
        toggle.theme.radius.sm.min(indicator_size * 0.4),
        paint_from_color(indicator_color),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    let text_bounds = UiRect::new(
        indicator_rect.x + indicator_rect.width + toggle.theme.spacing.sm,
        content_bounds.y,
        (content_bounds.width - indicator_rect.width - toggle.theme.spacing.sm).max(0.0),
        content_bounds.height,
    );
    let label = LabelNode {
        text: toggle.label.clone(),
        text_style: toggle.text_style.clone(),
        constraints: ui_layout::LayoutConstraints::tight(text_bounds.size()),
    };
    emit_label(
        &label,
        text_bounds,
        layer,
        atlas_source,
        layouter,
        depth + 1,
        primitive_order,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_numeric_input(
    widget_id: WidgetId,
    numeric: &NumericInputNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let interaction = interaction_state.for_widget(widget_id);
    let mut background = if numeric.enabled {
        numeric.theme.background_panel
    } else {
        with_alpha(numeric.theme.border, 0.30)
    };
    if interaction.hovered {
        background = brighten(background, 1.04);
    }
    let mut border = numeric.theme.border;
    if interaction.focused {
        border = brighten(numeric.theme.accent, 1.03);
    } else if interaction.hovered {
        border = brighten(border, 1.08);
    }

    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        numeric.theme.radius.sm,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        numeric.theme.radius.sm,
        numeric.theme.border_width,
        paint_from_color(border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: content_bounds,
        sort_key: sort_key(depth, *primitive_order),
    }));
    *primitive_order += 1;

    let value_text = format!("{:.*}", usize::from(numeric.precision), numeric.value);
    let label = LabelNode {
        text: value_text,
        text_style: numeric.text_style.clone(),
        constraints: ui_layout::LayoutConstraints::tight(content_bounds.size()),
    };
    emit_label(
        &label,
        content_bounds,
        layer,
        atlas_source,
        layouter,
        depth + 1,
        primitive_order,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_tabs(
    widget_id: WidgetId,
    tabs: &TabsNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let interaction = interaction_state.for_widget(widget_id);
    let mut background = tabs.theme.background_panel;
    if interaction.hovered {
        background = brighten(background, 1.04);
    }
    let mut border = tabs.theme.border;
    if interaction.focused {
        border = brighten(tabs.theme.accent, 1.03);
    } else if interaction.hovered {
        border = brighten(border, 1.08);
    }

    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        tabs.theme.radius.sm,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        tabs.theme.radius.sm,
        tabs.theme.border_width,
        paint_from_color(border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: content_bounds,
        sort_key: sort_key(depth, *primitive_order),
    }));
    *primitive_order += 1;

    if tabs.labels.is_empty() {
        return;
    }

    let segment_width = content_bounds.width / tabs.labels.len() as f32;
    let selected = tabs.selected_index.min(tabs.labels.len() - 1);
    let selected_rect = UiRect::new(
        content_bounds.x + segment_width * selected as f32,
        content_bounds.y,
        segment_width.max(0.0),
        content_bounds.height.max(0.0),
    );
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        selected_rect,
        tabs.theme.radius.sm.min(selected_rect.height * 0.5),
        paint_from_color(tabs.theme.accent),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    for (index, label_text) in tabs.labels.iter().enumerate() {
        let tab_bounds = UiRect::new(
            content_bounds.x + segment_width * index as f32,
            content_bounds.y,
            segment_width.max(0.0),
            content_bounds.height.max(0.0),
        );
        let mut style = tabs.text_style.clone();
        if index != selected {
            style.color[3] = (style.color[3] * 0.7).clamp(0.0, 1.0);
        }
        let label = LabelNode {
            text: label_text.clone(),
            text_style: style,
            constraints: ui_layout::LayoutConstraints::tight(tab_bounds.size()),
        };
        emit_label(
            &label,
            tab_bounds,
            layer,
            atlas_source,
            layouter,
            depth + 1,
            primitive_order,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_select(
    widget_id: WidgetId,
    select: &SelectNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let interaction = interaction_state.for_widget(widget_id);
    let mut background = if select.enabled {
        select.theme.background_panel
    } else {
        with_alpha(select.theme.border, 0.30)
    };
    if interaction.hovered {
        background = brighten(background, 1.04);
    }
    let mut border = select.theme.border;
    if interaction.focused {
        border = brighten(select.theme.accent, 1.03);
    } else if interaction.hovered {
        border = brighten(border, 1.08);
    }
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        select.theme.radius.sm,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        select.theme.radius.sm,
        select.theme.border_width,
        paint_from_color(border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: content_bounds,
        sort_key: sort_key(depth, *primitive_order),
    }));
    *primitive_order += 1;

    let text = select
        .selected_index
        .and_then(|index| select.options.get(index))
        .cloned()
        .unwrap_or_else(|| select.placeholder.clone());
    let label = LabelNode {
        text: format!("{text} v"),
        text_style: select.text_style.clone(),
        constraints: ui_layout::LayoutConstraints::tight(content_bounds.size()),
    };
    emit_label(
        &label,
        content_bounds,
        layer,
        atlas_source,
        layouter,
        depth + 1,
        primitive_order,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_table(
    widget_id: WidgetId,
    table: &TableNode,
    bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let interaction = interaction_state.for_widget(widget_id);
    let mut background = table.theme.background_panel;
    if interaction.hovered {
        background = brighten(background, 1.03);
    }
    let mut border = table.theme.border;
    if interaction.focused {
        border = brighten(table.theme.accent, 1.03);
    } else if interaction.hovered {
        border = brighten(border, 1.08);
    }
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        table.theme.radius.sm,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        table.theme.radius.sm,
        table.theme.border_width,
        paint_from_color(border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: bounds,
        sort_key: sort_key(depth, *primitive_order),
    }));
    *primitive_order += 1;

    let column_widths = table_column_widths(table, bounds.width);
    let header_rect = UiRect::new(bounds.x, bounds.y, bounds.width, table.row_height);
    let mut header_color = table.theme.border;
    header_color.a = (header_color.a * 0.45).clamp(0.0, 1.0);
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        header_rect,
        0.0,
        paint_from_color(header_color),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    emit_table_cells(
        table
            .columns
            .iter()
            .map(|column| column.label.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
        &column_widths,
        header_rect,
        &table.header_text_style,
        layer,
        atlas_source,
        layouter,
        depth + 1,
        primitive_order,
    );

    for (row_index, row) in table.rows.iter().enumerate() {
        let row_rect = UiRect::new(
            bounds.x,
            bounds.y + table.row_height * (row_index as f32 + 1.0),
            bounds.width,
            table.row_height,
        );
        if row.selected {
            let mut selected = table.theme.accent;
            selected.a = (selected.a * 0.55).clamp(0.0, 1.0);
            layer.push(UiPrimitive::Rect(RectPrimitive::new(
                row_rect,
                0.0,
                paint_from_color(selected),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            )));
            *primitive_order += 1;
        }
        let cells = row.cells.iter().map(String::as_str).collect::<Vec<_>>();
        emit_table_cells(
            &cells,
            &column_widths,
            row_rect,
            &table.text_style,
            layer,
            atlas_source,
            layouter,
            depth + 1,
            primitive_order,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_tree(
    widget_id: WidgetId,
    tree: &TreeNode,
    bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let interaction = interaction_state.for_widget(widget_id);
    let mut background = tree.theme.background_panel;
    if interaction.hovered {
        background = brighten(background, 1.03);
    }
    let mut border = tree.theme.border;
    if interaction.focused {
        border = brighten(tree.theme.accent, 1.03);
    } else if interaction.hovered {
        border = brighten(border, 1.08);
    }
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        tree.theme.radius.sm,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        tree.theme.radius.sm,
        tree.theme.border_width,
        paint_from_color(border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: bounds,
        sort_key: sort_key(depth, *primitive_order),
    }));
    *primitive_order += 1;

    for (index, row) in tree.rows.iter().enumerate() {
        let row_rect = UiRect::new(
            bounds.x,
            bounds.y + tree.row_height * index as f32,
            bounds.width,
            tree.row_height,
        );
        if row.selected {
            let mut selected = tree.theme.accent;
            selected.a = (selected.a * 0.55).clamp(0.0, 1.0);
            layer.push(UiPrimitive::Rect(RectPrimitive::new(
                row_rect,
                0.0,
                paint_from_color(selected),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            )));
            *primitive_order += 1;
        }
        let marker = if row.has_children {
            if row.expanded { "v" } else { ">" }
        } else {
            " "
        };
        let text = format!(
            "{}{marker} {}",
            " ".repeat(row.depth.saturating_mul(2)),
            row.label
        );
        let label_bounds = UiRect::new(
            row_rect.x + tree.theme.spacing.xs,
            row_rect.y,
            (row_rect.width - tree.theme.spacing.xs * 2.0).max(0.0),
            row_rect.height,
        );
        let label = LabelNode {
            text,
            text_style: tree.text_style.clone(),
            constraints: ui_layout::LayoutConstraints::tight(label_bounds.size()),
        };
        emit_label(
            &label,
            label_bounds,
            layer,
            atlas_source,
            layouter,
            depth + 1,
            primitive_order,
        );
    }
}

fn table_column_widths(table: &TableNode, available_width: f32) -> Vec<f32> {
    if table.columns.is_empty() {
        return Vec::new();
    }
    let minimum = table
        .columns
        .iter()
        .map(|column| column.min_width)
        .collect::<Vec<_>>();
    let minimum_sum = minimum.iter().sum::<f32>().max(1.0);
    let scale = (available_width.max(minimum_sum)) / minimum_sum;
    minimum.into_iter().map(|width| width * scale).collect()
}

#[expect(
    clippy::too_many_arguments,
    reason = "private render-emission helpers keep explicit render state at each emission boundary"
)]
fn emit_table_cells(
    cells: &[&str],
    column_widths: &[f32],
    row_rect: UiRect,
    text_style: &ui_text::TextStyle,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    depth: u32,
    primitive_order: &mut u32,
) {
    let mut x = row_rect.x;
    for (column_index, width) in column_widths.iter().copied().enumerate() {
        let cell_text = cells.get(column_index).copied().unwrap_or("");
        let cell_bounds = UiRect::new(x + 4.0, row_rect.y, (width - 8.0).max(0.0), row_rect.height);
        let label = LabelNode {
            text: cell_text.to_string(),
            text_style: text_style.clone(),
            constraints: ui_layout::LayoutConstraints::tight(cell_bounds.size()),
        };
        emit_label(
            &label,
            cell_bounds,
            layer,
            atlas_source,
            layouter,
            depth,
            primitive_order,
        );
        x += width;
    }
}

fn emit_label(
    label: &LabelNode,
    bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    depth: u32,
    primitive_order: &mut u32,
) {
    let Some(mut glyph_run) = layouter.layout(
        atlas_source,
        TextLayoutRequest {
            text: &label.text,
            style: &label.text_style,
            max_width: Some(bounds.width.max(0.0)),
        },
    ) else {
        return;
    };

    for glyph in &mut glyph_run.glyphs {
        glyph.origin.x += bounds.x;
        glyph.origin.y += bounds.y;
    }

    layer.push(UiPrimitive::GlyphRun(GlyphRunPrimitive::new(
        glyph_run,
        Some(bounds),
        UiPaint::rgba(
            label.text_style.color[0],
            label.text_style.color[1],
            label.text_style.color[2],
            label.text_style.color[3],
        ),
        UiDrawKey::new(0, Some(label.text_style.font_id.0)),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct WidgetInteraction {
    hovered: bool,
    pressed: bool,
    focused: bool,
}

impl InteractionVisualState {
    fn for_widget(&self, widget_id: WidgetId) -> WidgetInteraction {
        WidgetInteraction {
            hovered: self.hovered_widget == Some(widget_id),
            pressed: self.pressed_widget == Some(widget_id),
            focused: self.focused_widget == Some(widget_id),
        }
    }
}

fn button_text_style(button: &ButtonNode, interaction: WidgetInteraction) -> ui_text::TextStyle {
    let mut text_style = button.text_style.clone();
    if !button.enabled {
        text_style.color[3] = (text_style.color[3] * 0.55).clamp(0.0, 1.0);
        return text_style;
    }
    if button.selected || interaction.pressed {
        text_style.color[0] = (text_style.color[0] + 0.08).clamp(0.0, 1.0);
        text_style.color[1] = (text_style.color[1] + 0.08).clamp(0.0, 1.0);
        text_style.color[2] = (text_style.color[2] + 0.08).clamp(0.0, 1.0);
    } else if interaction.hovered {
        text_style.color[3] = (text_style.color[3] * 0.95).clamp(0.0, 1.0);
    }
    text_style
}

fn brighten(color: ui_theme::UiColor, factor: f32) -> ui_theme::UiColor {
    ui_theme::UiColor::new(
        (color.r * factor).clamp(0.0, 1.0),
        (color.g * factor).clamp(0.0, 1.0),
        (color.b * factor).clamp(0.0, 1.0),
        color.a,
    )
}

fn darken(color: ui_theme::UiColor, factor: f32) -> ui_theme::UiColor {
    ui_theme::UiColor::new(
        (color.r * factor).clamp(0.0, 1.0),
        (color.g * factor).clamp(0.0, 1.0),
        (color.b * factor).clamp(0.0, 1.0),
        color.a,
    )
}

fn with_alpha(color: ui_theme::UiColor, alpha_mul: f32) -> ui_theme::UiColor {
    ui_theme::UiColor::new(
        color.r,
        color.g,
        color.b,
        (color.a * alpha_mul).clamp(0.0, 1.0),
    )
}

fn paint_from_color(color: ui_theme::UiColor) -> UiPaint {
    UiPaint::rgba(color.r, color.g, color.b, color.a)
}

fn default_draw_key() -> UiDrawKey {
    UiDrawKey::new(0, None)
}

fn sort_key(_depth: u32, primitive_order: u32) -> UiSortKey {
    UiSortKey::new(0, 0, primitive_order)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{UiRuntimeState, WidgetId, compute_tree_layout};
    use ui_render_data::ViewportSurfaceEmbedSlotId;
    use ui_text::{
        FontFaceMetrics, FontId, GlyphMetrics, MsdfFontAtlas, TextAlign, TextOverflow, TextStyle,
        TextWrap,
    };
    use ui_theme::ThemeTokens;

    #[derive(Debug, Clone)]
    struct TestAtlasSource {
        atlas: MsdfFontAtlas,
    }

    impl FontAtlasSource for TestAtlasSource {
        fn atlas(&self, font_id: FontId) -> Option<&MsdfFontAtlas> {
            (self.atlas.font_id == font_id).then_some(&self.atlas)
        }
    }

    #[test]
    fn build_ui_frame_panel_label_snapshot_signature() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let theme = ThemeTokens::default();
        let text_style = TextStyle {
            font_id: FontId(1),
            font_size: 14.0,
            color: [0.9, 0.95, 1.0, 1.0],
            line_height: Some(18.0),
            align: TextAlign::Start,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        };
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme)),
            vec![UiNode::new(
                WidgetId(2),
                UiNodeKind::Label(LabelNode::new("Overlay", text_style)),
            )],
        ));
        let bounds = UiRect::new(12.0, 16.0, 240.0, 96.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());

        let frame = build_ui_frame(
            &tree,
            &layouts,
            UiSize::new(320.0, 180.0),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let signature = frame_signature(&frame);
        let expected = [
            "Rect(x=12.0 y=16.0 w=240.0 h=96.0)",
            "Border(x=12.0 y=16.0 w=240.0 h=96.0)",
            "ClipPush(x=20.0 y=24.0 w=224.0 h=80.0)",
            "GlyphRun(text=\"Overl\" clip=true)",
            "ClipPop",
        ]
        .join("\n");
        assert_eq!(signature, expected);
    }

    #[test]
    fn build_ui_frame_emits_viewport_embed_with_normalized_uv() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let tree = UiTree::new(UiNode::new(
            WidgetId(7),
            UiNodeKind::ViewportSurfaceEmbed(ViewportSurfaceEmbedNode::new(
                9,
                ViewportSurfaceEmbedSlotId::new(1),
            )),
        ));
        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(10.0, 20.0, 100.0, 50.0),
            &UiRuntimeState::default(),
        );

        let frame = build_ui_frame(
            &tree,
            &layouts,
            UiSize::new(200.0, 100.0),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let layer = &frame.surfaces[0].layers[0];
        let embed = layer
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                UiPrimitive::ViewportSurfaceEmbed(value) => Some(value),
                _ => None,
            })
            .expect("viewport embed primitive should exist");

        assert!((embed.uv_rect.x - 0.05).abs() < 0.000_1);
        assert!((embed.uv_rect.y - 0.20).abs() < 0.000_1);
        assert!((embed.uv_rect.width - 0.50).abs() < 0.000_1);
        assert!((embed.uv_rect.height - 0.50).abs() < 0.000_1);
    }

    #[test]
    fn build_ui_frame_emits_divider_as_rect_and_spacer_as_no_primitive() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Stack(crate::StackNode::vertical(0.0)),
            vec![
                UiNode::new(
                    WidgetId(2),
                    UiNodeKind::Spacer(crate::SpacerNode::new(UiSize::new(8.0, 4.0))),
                ),
                UiNode::new(
                    WidgetId(3),
                    UiNodeKind::Divider(crate::DividerNode::new(
                        ui_math::Axis::Horizontal,
                        2.0,
                        ui_layout::SizePolicy::Fixed(40.0),
                        ui_theme::UiColor::new(0.3, 0.4, 0.5, 1.0),
                    )),
                ),
            ],
        ));
        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(0.0, 0.0, 100.0, 40.0),
            &UiRuntimeState::default(),
        );

        let frame = build_ui_frame(
            &tree,
            &layouts,
            UiSize::new(100.0, 40.0),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let layer = &frame.surfaces[0].layers[0];
        let rects = layer
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, UiPrimitive::Rect(_)))
            .count();

        assert_eq!(rects, 1);
        assert_eq!(layer.primitives.len(), 1);
    }

    #[test]
    fn build_ui_frame_emits_image_primitive() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let draw_key = UiDrawKey::new(5, Some(12));
        let uv_rect = UiRect::new(0.25, 0.25, 0.5, 0.5);
        let tint = UiPaint::rgba(0.8, 0.9, 1.0, 0.75);
        let tree = UiTree::new(UiNode::new(
            WidgetId(4),
            UiNodeKind::Image(crate::ImageNode::new(
                draw_key,
                uv_rect,
                tint,
                UiSize::new(32.0, 24.0),
            )),
        ));
        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(10.0, 20.0, 64.0, 48.0),
            &UiRuntimeState::default(),
        );

        let frame = build_ui_frame(
            &tree,
            &layouts,
            UiSize::new(100.0, 80.0),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let image = frame.surfaces[0].layers[0]
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                UiPrimitive::Image(value) => Some(value),
                _ => None,
            })
            .expect("image primitive should exist");

        assert_eq!(image.rect, UiRect::new(10.0, 20.0, 64.0, 48.0));
        assert_eq!(image.uv_rect, uv_rect);
        assert_eq!(image.tint, tint);
        assert_eq!(image.draw_key, draw_key);
    }

    #[test]
    fn build_ui_frame_emits_scrollbar_only_when_content_overflows() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let scroll_id = WidgetId(1);

        let overflow_tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Scroll(crate::ScrollNode::vertical(theme.clone())),
            vec![UiNode::with_children(
                WidgetId(2),
                UiNodeKind::Stack(crate::StackNode::vertical(theme.spacing.xs)),
                vec![
                    UiNode::new(
                        WidgetId(3),
                        UiNodeKind::Button(crate::ButtonNode::new(
                            "First",
                            text_style.clone(),
                            theme.clone(),
                        )),
                    ),
                    UiNode::new(
                        WidgetId(4),
                        UiNodeKind::Button(crate::ButtonNode::new(
                            "Second",
                            text_style.clone(),
                            theme.clone(),
                        )),
                    ),
                    UiNode::new(
                        WidgetId(5),
                        UiNodeKind::Button(crate::ButtonNode::new(
                            "Third",
                            text_style.clone(),
                            theme.clone(),
                        )),
                    ),
                ],
            )],
        ));
        let overflow_bounds = UiRect::new(0.0, 0.0, 120.0, 64.0);
        let overflow_layouts =
            compute_tree_layout(&overflow_tree, overflow_bounds, &UiRuntimeState::default());
        let overflow_frame = build_ui_frame(
            &overflow_tree,
            &overflow_layouts,
            overflow_bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );

        let scroll_layout = overflow_layouts
            .get(&scroll_id)
            .expect("scroll layout should exist");
        let track_rect = UiRect::new(
            scroll_layout.content_bounds.x + scroll_layout.content_bounds.width,
            scroll_layout.content_bounds.y,
            (scroll_layout.bounds.width - scroll_layout.content_bounds.width).max(0.0),
            scroll_layout.content_bounds.height,
        );
        assert!(
            has_rect_primitive(&overflow_frame, track_rect),
            "overflowing scroll should emit a scrollbar track primitive",
        );

        let no_overflow_tree = UiTree::new(UiNode::with_children(
            WidgetId(11),
            UiNodeKind::Scroll(crate::ScrollNode::vertical(theme.clone())),
            vec![UiNode::new(
                WidgetId(12),
                UiNodeKind::Button(crate::ButtonNode::new("One", text_style, theme)),
            )],
        ));
        let no_overflow_bounds = UiRect::new(0.0, 0.0, 240.0, 128.0);
        let no_overflow_layouts = compute_tree_layout(
            &no_overflow_tree,
            no_overflow_bounds,
            &UiRuntimeState::default(),
        );
        let no_overflow_frame = build_ui_frame(
            &no_overflow_tree,
            &no_overflow_layouts,
            no_overflow_bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let no_overflow_scroll = no_overflow_layouts
            .get(&WidgetId(11))
            .expect("scroll layout should exist");
        let no_overflow_track = UiRect::new(
            no_overflow_scroll.content_bounds.x + no_overflow_scroll.content_bounds.width,
            no_overflow_scroll.content_bounds.y,
            (no_overflow_scroll.bounds.width - no_overflow_scroll.content_bounds.width).max(0.0),
            no_overflow_scroll.content_bounds.height,
        );
        assert!(
            !has_rect_primitive(&no_overflow_frame, no_overflow_track),
            "non-overflowing scroll should not emit a scrollbar track primitive",
        );
    }

    #[test]
    fn build_ui_frame_applies_hover_and_focus_visual_states_to_button() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let theme = ThemeTokens::default();
        let button_id = WidgetId(21);
        let tree = UiTree::new(UiNode::new(
            button_id,
            UiNodeKind::Button(crate::ButtonNode::new(
                "Apply",
                TextStyle::default(),
                theme.clone(),
            )),
        ));
        let bounds = UiRect::new(0.0, 0.0, 140.0, 36.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());

        let base_frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let hover_frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState {
                hovered_widget: Some(button_id),
                pressed_widget: None,
                focused_widget: None,
            },
            &atlas_source,
        );
        let focus_frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState {
                hovered_widget: None,
                pressed_widget: None,
                focused_widget: Some(button_id),
            },
            &atlas_source,
        );

        let base_background = first_rect_paint(&base_frame).expect("base button rect should exist");
        let hover_background =
            first_rect_paint(&hover_frame).expect("hover button rect should exist");
        assert_ne!(
            base_background, hover_background,
            "hovered button should render a different background paint"
        );

        let base_border = first_border_paint(&base_frame).expect("base button border should exist");
        let focus_border =
            first_border_paint(&focus_frame).expect("focused button border should exist");
        assert_ne!(
            base_border, focus_border,
            "focused button should render a different border paint"
        );
    }

    fn frame_signature(frame: &UiFrame) -> String {
        let layer = &frame.surfaces[0].layers[0];
        layer
            .primitives
            .iter()
            .map(|primitive| match primitive {
                UiPrimitive::Rect(value) => format!(
                    "Rect(x={:.1} y={:.1} w={:.1} h={:.1})",
                    value.rect.x, value.rect.y, value.rect.width, value.rect.height
                ),
                UiPrimitive::Border(value) => format!(
                    "Border(x={:.1} y={:.1} w={:.1} h={:.1})",
                    value.rect.x, value.rect.y, value.rect.width, value.rect.height
                ),
                UiPrimitive::Clip(ClipPrimitive::Push { rect, .. }) => format!(
                    "ClipPush(x={:.1} y={:.1} w={:.1} h={:.1})",
                    rect.x, rect.y, rect.width, rect.height
                ),
                UiPrimitive::Clip(ClipPrimitive::Pop { .. }) => "ClipPop".to_string(),
                UiPrimitive::GlyphRun(value) => {
                    let text = value
                        .glyph_run
                        .glyphs
                        .iter()
                        .map(|glyph| glyph.ch)
                        .collect::<String>();
                    format!(
                        "GlyphRun(text=\"{}\" clip={})",
                        text,
                        value.baseline_origin_clip.is_some()
                    )
                }
                UiPrimitive::ViewportSurfaceEmbed(value) => format!(
                    "ViewportSurfaceEmbed(viewport={} slot={:?})",
                    value.viewport_id, value.slot
                ),
                UiPrimitive::Image(value) => format!(
                    "Image(x={:.1} y={:.1} w={:.1} h={:.1})",
                    value.rect.x, value.rect.y, value.rect.width, value.rect.height
                ),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn has_rect_primitive(frame: &UiFrame, expected_rect: UiRect) -> bool {
        frame.surfaces.iter().any(|surface| {
            surface.layers.iter().any(|layer| {
                layer.primitives.iter().any(|primitive| match primitive {
                    UiPrimitive::Rect(value) => rect_approx_eq(value.rect, expected_rect),
                    _ => false,
                })
            })
        })
    }

    fn first_rect_paint(frame: &UiFrame) -> Option<UiPaint> {
        frame
            .surfaces
            .iter()
            .flat_map(|surface| surface.layers.iter())
            .flat_map(|layer| layer.primitives.iter())
            .find_map(|primitive| match primitive {
                UiPrimitive::Rect(value) => Some(value.paint),
                _ => None,
            })
    }

    fn first_border_paint(frame: &UiFrame) -> Option<UiPaint> {
        frame
            .surfaces
            .iter()
            .flat_map(|surface| surface.layers.iter())
            .flat_map(|layer| layer.primitives.iter())
            .find_map(|primitive| match primitive {
                UiPrimitive::Border(value) => Some(value.paint),
                _ => None,
            })
    }

    fn rect_approx_eq(left: UiRect, right: UiRect) -> bool {
        (left.x - right.x).abs() <= 0.001
            && (left.y - right.y).abs() <= 0.001
            && (left.width - right.width).abs() <= 0.001
            && (left.height - right.height).abs() <= 0.001
    }

    fn atlas_with_ascii(font_id: FontId) -> MsdfFontAtlas {
        let mut glyphs = HashMap::new();
        for ch in 32_u8..=126_u8 {
            glyphs.insert(
                char::from(ch),
                GlyphMetrics {
                    advance: 10.0,
                    plane_left: 0.0,
                    plane_top: 8.0,
                    plane_right: 8.0,
                    plane_bottom: -2.0,
                    atlas_left: 0.0,
                    atlas_top: 0.0,
                    atlas_right: 0.1,
                    atlas_bottom: 0.1,
                },
            );
        }
        MsdfFontAtlas {
            font_id,
            texture_width: 256,
            texture_height: 256,
            metrics: FontFaceMetrics {
                ascender: 9.0,
                descender: -3.0,
                line_height: 12.0,
                base_size: 12.0,
            },
            glyphs,
        }
    }
}
