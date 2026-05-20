//! File: domain/ui/ui_runtime/src/output/build_ui_frame.rs
//! Purpose: Convert retained tree + computed layout into UiFrame.

use crate::{
    ButtonNode, ComputedLayoutMap, DividerNode, GraphCanvasNode, ImageNode, LabelNode,
    NumericInputNode, PanelNode, PopupNode, ProductSurfaceNode, RadialMenuNode, ScrollNode,
    ScrollbarAxisOpacities, ScrollbarAxisTarget, SelectNode, TableNode, TabsNode, TextInputNode,
    ToggleNode, TreeNode, UiNode, UiNodeKind, UiTree, ViewportSurfaceEmbedNode, WidgetId,
};
use std::collections::BTreeMap;
use ui_math::{Axis, UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, ClipPrimitive, GlyphRunPrimitive, GraphCanvasPrimitiveBatch,
    GraphCanvasPrimitiveRole, ImagePrimitive, ProductSurfacePrimitive, RectPrimitive,
    StrokePrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId, UiPaint, UiPrimitive, UiSortKey,
    UiSurface, UiSurfaceId, ViewportSurfaceEmbedPrimitive,
};
use ui_text::{
    AtlasTextLayouter, FontAtlasSource, TextAlign, TextLayoutRequest, TextLayouter,
    TextVerticalAlign,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InteractionVisualState {
    pub hovered_widget: Option<WidgetId>,
    pub pressed_widget: Option<WidgetId>,
    pub focused_widget: Option<WidgetId>,
    pub hovered_scrollbar: Option<ScrollbarAxisTarget>,
    pub active_scrollbar: Option<ScrollbarAxisTarget>,
    pub scrollbar_opacity_by_widget_id: BTreeMap<WidgetId, ScrollbarAxisOpacities>,
    pub graph_canvas_gestures: BTreeMap<WidgetId, ui_graph_editor::GraphCanvasGestureState>,
    pub graph_canvas_viewports: BTreeMap<WidgetId, ui_graph_editor::GraphViewport>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScrollbarGeometry {
    pub scroll_widget_id: WidgetId,
    pub axis: ui_math::Axis,
    pub track_rect: UiRect,
    pub thumb_rect: UiRect,
    pub max_offset: f32,
}

const BASE_LAYER_ORDER: u32 = 0;
#[cfg(test)]
const POPUP_LAYER_ORDER: u32 = 1;

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
        BASE_LAYER_ORDER,
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
    render_layer_order: u32,
    primitive_order: &mut u32,
) {
    let Some(layout) = layouts.get(&node.id) else {
        return;
    };
    let node_layer_order = match &node.kind {
        UiNodeKind::Popup(popup) => popup.layer_order,
        UiNodeKind::RadialMenu(radial) => radial.layer_order,
        UiNodeKind::OverlayAdornment(_) => render_layer_order,
        _ => render_layer_order,
    };

    match &node.kind {
        UiNodeKind::Panel(panel) => emit_panel(
            panel,
            layout.bounds,
            layout.content_bounds,
            layer,
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::Popup(popup) => emit_popup(
            popup,
            layout.bounds,
            layout.content_bounds,
            layer,
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::RadialMenu(radial) => emit_radial_menu(
            radial,
            layout.bounds,
            layout.content_bounds,
            layer,
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::OverlayAdornment(_) => {}
        UiNodeKind::Label(label) => emit_label(
            label,
            layout.bounds,
            layer,
            atlas_source,
            layouter,
            node_layer_order,
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
            interaction_state.clone(),
            node_layer_order,
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
            interaction_state.clone(),
            node_layer_order,
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
            interaction_state.clone(),
            node_layer_order,
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
            interaction_state.clone(),
            node_layer_order,
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
            interaction_state.clone(),
            node_layer_order,
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
            interaction_state.clone(),
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::Table(table) => emit_table(
            node.id,
            table,
            layout.bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state.clone(),
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::Tree(tree) => emit_tree(
            node.id,
            tree,
            layout.bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state.clone(),
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::Spacer(_) => {}
        UiNodeKind::Divider(divider) => emit_divider(
            divider,
            layout.bounds,
            layer,
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::Image(image) => emit_image(
            image,
            layout.bounds,
            layer,
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::ProductSurface(surface) => emit_product_surface(
            surface,
            layout.bounds,
            layer,
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::GraphCanvas(graph_canvas) => emit_graph_canvas(
            node.id,
            graph_canvas,
            layout.bounds,
            layer,
            atlas_source,
            layouter,
            interaction_state.clone(),
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::ViewportSurfaceEmbed(embed) => emit_viewport_surface_embed(
            embed,
            layout.bounds,
            surface_size,
            layer,
            node_layer_order,
            primitive_order,
        ),
        UiNodeKind::Scroll(scroll) => emit_scroll_begin(
            scroll,
            layout.bounds,
            layout.content_bounds,
            layer,
            node_layer_order,
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
            interaction_state.clone(),
            node_layer_order,
            primitive_order,
        );
    }

    match &node.kind {
        UiNodeKind::Panel(_)
        | UiNodeKind::Popup(_)
        | UiNodeKind::RadialMenu(_)
        | UiNodeKind::Button(_)
        | UiNodeKind::TextInput(_)
        | UiNodeKind::NumericInput(_)
        | UiNodeKind::Tabs(_)
        | UiNodeKind::Select(_)
        | UiNodeKind::Table(_)
        | UiNodeKind::Tree(_) => {
            layer.push(UiPrimitive::Clip(ClipPrimitive::Pop {
                sort_key: sort_key(node_layer_order, *primitive_order),
            }));
            *primitive_order += 1;
        }
        UiNodeKind::Scroll(scroll) => {
            layer.push(UiPrimitive::Clip(ClipPrimitive::Pop {
                sort_key: sort_key(node_layer_order, *primitive_order),
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
                interaction_state.clone(),
                node_layer_order,
                primitive_order,
            );
        }
        UiNodeKind::Label(_)
        | UiNodeKind::OverlayAdornment(_)
        | UiNodeKind::Toggle(_)
        | UiNodeKind::Spacer(_)
        | UiNodeKind::Divider(_)
        | UiNodeKind::Image(_)
        | UiNodeKind::ProductSurface(_)
        | UiNodeKind::GraphCanvas(_)
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

fn emit_popup(
    popup: &PopupNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        popup.theme.radius.md,
        paint_from_color(popup.theme.background_panel),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        popup.theme.radius.md,
        popup.theme.border_width,
        paint_from_color(popup.theme.border),
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

fn emit_radial_menu(
    radial: &RadialMenuNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        radial.outer_radius,
        paint_from_color(radial.theme.background_panel),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        radial.outer_radius,
        radial.theme.border_width,
        paint_from_color(radial.theme.border),
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
    _scroll: &ScrollNode,
    layouts: &ComputedLayoutMap,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let geometries = scrollbar_geometries(tree, node.id, layouts, bounds, content_bounds);
    if geometries.is_empty() {
        return;
    }
    let UiNodeKind::Scroll(scroll) = &node.kind else {
        return;
    };

    for geometry in geometries {
        let target = ScrollbarAxisTarget::new(node.id, geometry.axis);
        let scrollbar_opacity = if interaction_state.active_scrollbar == Some(target)
            || interaction_state.hovered_scrollbar == Some(target)
        {
            1.0
        } else {
            interaction_state
                .scrollbar_opacity_by_widget_id
                .get(&node.id)
                .map(|opacities| opacities.for_axis(geometry.axis))
                .unwrap_or(0.0)
                .clamp(0.0, 1.0)
        };
        if scrollbar_opacity <= 0.0 {
            continue;
        }
        let radius = match geometry.axis {
            Axis::Vertical => scroll.theme.radius.sm.min(geometry.track_rect.width * 0.5),
            Axis::Horizontal => scroll.theme.radius.sm.min(geometry.track_rect.height * 0.5),
        };

        let mut track_color = scroll.theme.border;
        track_color.a = (track_color.a * 0.35 * scrollbar_opacity).clamp(0.0, 1.0);
        layer.push(UiPrimitive::Rect(RectPrimitive::new(
            geometry.track_rect,
            radius,
            paint_from_color(track_color),
            default_draw_key(),
            sort_key(depth, *primitive_order),
        )));
        *primitive_order += 1;

        let mut thumb_color = scroll.theme.accent;
        thumb_color.a = (thumb_color.a * 0.80 * scrollbar_opacity).clamp(0.0, 1.0);
        layer.push(UiPrimitive::Rect(RectPrimitive::new(
            geometry.thumb_rect,
            radius,
            paint_from_color(thumb_color),
            default_draw_key(),
            sort_key(depth, *primitive_order),
        )));
        *primitive_order += 1;
    }
}

#[cfg(test)]
pub(crate) fn scrollbar_geometry(
    tree: &UiTree,
    scroll_widget_id: WidgetId,
    layouts: &ComputedLayoutMap,
    bounds: UiRect,
    content_bounds: UiRect,
) -> Option<ScrollbarGeometry> {
    scrollbar_geometry_for_axis(
        tree,
        scroll_widget_id,
        layouts,
        bounds,
        content_bounds,
        Axis::Vertical,
    )
    .or_else(|| {
        scrollbar_geometry_for_axis(
            tree,
            scroll_widget_id,
            layouts,
            bounds,
            content_bounds,
            Axis::Horizontal,
        )
    })
}

pub(crate) fn scrollbar_geometries(
    tree: &UiTree,
    scroll_widget_id: WidgetId,
    layouts: &ComputedLayoutMap,
    bounds: UiRect,
    content_bounds: UiRect,
) -> Vec<ScrollbarGeometry> {
    [Axis::Vertical, Axis::Horizontal]
        .into_iter()
        .filter_map(|axis| {
            scrollbar_geometry_for_axis(
                tree,
                scroll_widget_id,
                layouts,
                bounds,
                content_bounds,
                axis,
            )
        })
        .collect()
}

pub(crate) fn scrollbar_geometry_for_axis(
    tree: &UiTree,
    scroll_widget_id: WidgetId,
    layouts: &ComputedLayoutMap,
    bounds: UiRect,
    content_bounds: UiRect,
    axis: Axis,
) -> Option<ScrollbarGeometry> {
    let node = tree.walk().find(|node| node.id == scroll_widget_id)?;
    let UiNodeKind::Scroll(scroll) = &node.kind else {
        return None;
    };
    if !scroll.axes.contains(axis) {
        return None;
    }
    let child = node.children.first()?;
    let child_layout = layouts.get(&child.id)?;
    let vertical_max_offset =
        scroll_max_offset_for_axis(scroll, child_layout.bounds, content_bounds, Axis::Vertical);
    let horizontal_max_offset = scroll_max_offset_for_axis(
        scroll,
        child_layout.bounds,
        content_bounds,
        Axis::Horizontal,
    );
    match axis {
        Axis::Vertical => {
            let track_width = scroll.bar_thickness.min(bounds.width.max(0.0));
            if track_width <= f32::EPSILON || content_bounds.height <= f32::EPSILON {
                return None;
            }
            let horizontal_track_height = if horizontal_max_offset > f32::EPSILON {
                scroll.bar_thickness.min(bounds.height.max(0.0))
            } else {
                0.0
            };
            let track_x = bounds.x + bounds.width - track_width;
            let track_height = (content_bounds.height - horizontal_track_height).max(0.0);
            if track_height <= f32::EPSILON {
                return None;
            }
            let track_rect = UiRect::new(track_x, content_bounds.y, track_width, track_height);
            let viewport_extent = content_bounds.height.max(0.0);
            let content_extent = child_layout.bounds.height.max(viewport_extent);
            let max_offset = vertical_max_offset;
            if max_offset <= f32::EPSILON {
                return None;
            }
            let scroll_offset = (content_bounds.y - child_layout.bounds.y).clamp(0.0, max_offset);
            let min_thumb = scroll.min_thumb_main_size.min(track_rect.height).max(0.0);
            let natural = (viewport_extent / content_extent) * track_rect.height;
            let thumb_extent = natural.clamp(min_thumb, track_rect.height);
            let thumb_range = (track_rect.height - thumb_extent).max(0.0);
            let thumb_y = track_rect.y + thumb_range * (scroll_offset / max_offset);
            Some(ScrollbarGeometry {
                scroll_widget_id,
                axis,
                track_rect,
                thumb_rect: UiRect::new(track_rect.x, thumb_y, track_rect.width, thumb_extent),
                max_offset,
            })
        }
        Axis::Horizontal => {
            let track_height = scroll.bar_thickness.min(bounds.height.max(0.0));
            if track_height <= f32::EPSILON || content_bounds.width <= f32::EPSILON {
                return None;
            }
            let vertical_track_width = if vertical_max_offset > f32::EPSILON {
                scroll.bar_thickness.min(bounds.width.max(0.0))
            } else {
                0.0
            };
            let track_width = (content_bounds.width - vertical_track_width).max(0.0);
            if track_width <= f32::EPSILON {
                return None;
            }
            let track_rect = UiRect::new(
                content_bounds.x,
                bounds.y + bounds.height - track_height,
                track_width,
                track_height,
            );
            let viewport_extent = content_bounds.width.max(0.0);
            let content_extent = child_layout.bounds.width.max(viewport_extent);
            let max_offset = horizontal_max_offset;
            if max_offset <= f32::EPSILON {
                return None;
            }
            let scroll_offset = (content_bounds.x - child_layout.bounds.x).clamp(0.0, max_offset);
            let min_thumb = scroll.min_thumb_main_size.min(track_rect.width).max(0.0);
            let natural = (viewport_extent / content_extent) * track_rect.width;
            let thumb_extent = natural.clamp(min_thumb, track_rect.width);
            let thumb_range = (track_rect.width - thumb_extent).max(0.0);
            let thumb_x = track_rect.x + thumb_range * (scroll_offset / max_offset);
            Some(ScrollbarGeometry {
                scroll_widget_id,
                axis,
                track_rect,
                thumb_rect: UiRect::new(thumb_x, track_rect.y, thumb_extent, track_rect.height),
                max_offset,
            })
        }
    }
}

fn scroll_max_offset_for_axis(
    scroll: &ScrollNode,
    child_bounds: UiRect,
    content_bounds: UiRect,
    axis: Axis,
) -> f32 {
    if !scroll.axes.contains(axis) {
        return 0.0;
    }
    match axis {
        Axis::Vertical => {
            let viewport_height = content_bounds.height.max(0.0);
            let content_height = child_bounds.height.max(viewport_height);
            (content_height - viewport_height).max(0.0)
        }
        Axis::Horizontal => {
            let viewport_width = content_bounds.width.max(0.0);
            let content_width = child_bounds.width.max(viewport_width);
            (content_width - viewport_width).max(0.0)
        }
    }
}

fn emit_viewport_surface_embed(
    embed: &ViewportSurfaceEmbedNode,
    bounds: UiRect,
    _surface_size: UiSize,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::ViewportSurfaceEmbed(
        ViewportSurfaceEmbedPrimitive::new(
            embed.viewport_id,
            embed.slot,
            bounds,
            UiRect::new(0.0, 0.0, 1.0, 1.0),
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

fn emit_product_surface(
    surface: &ProductSurfaceNode,
    bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::ProductSurface(ProductSurfacePrimitive::new(
        surface.source.clone(),
        bounds,
        surface.uv_rect,
        surface.tint,
        surface.alpha_mode,
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
}

#[expect(
    clippy::too_many_arguments,
    reason = "graph canvas emission maps a retained graph view model into concrete render primitives"
)]
fn emit_graph_canvas(
    widget_id: WidgetId,
    graph_canvas: &GraphCanvasNode,
    bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    interaction_state: InteractionVisualState,
    depth: u32,
    primitive_order: &mut u32,
) {
    let interaction = interaction_state.for_widget(widget_id);
    let viewport = interaction_state
        .graph_canvas_viewports
        .get(&widget_id)
        .copied()
        .unwrap_or(graph_canvas.canvas.viewport);
    let mut background = graph_canvas.theme.background_panel;
    if interaction.hovered {
        background = brighten(background, 1.03);
    }
    let mut border = graph_canvas.theme.border;
    if interaction.focused {
        border = brighten(graph_canvas.theme.accent, 1.04);
    }

    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        graph_canvas.theme.radius.sm,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        graph_canvas.theme.radius.sm,
        graph_canvas.theme.border_width,
        paint_from_color(border),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    if graph_canvas.clip {
        layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
            rect: bounds,
            sort_key: sort_key(depth, *primitive_order),
        }));
        *primitive_order += 1;
    }

    let mut batch = GraphCanvasPrimitiveBatch::new();
    emit_graph_grid(
        &mut batch,
        bounds,
        viewport,
        with_alpha(graph_canvas.theme.border, 0.28),
        with_alpha(graph_canvas.theme.accent, 0.18),
        depth,
        primitive_order,
    );
    for edge in &graph_canvas.canvas.edges {
        batch.push(
            GraphCanvasPrimitiveRole::Edge,
            StrokePrimitive::new(
                [
                    graph_point_to_ui(bounds, viewport, edge.from),
                    graph_point_to_ui(bounds, viewport, edge.to),
                ],
                if edge.selected { 3.0 } else { 2.0 },
                paint_from_color(if edge.selected {
                    graph_canvas.theme.accent
                } else {
                    graph_canvas.theme.border
                }),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            )
            .with_clip(bounds),
        );
        *primitive_order += 1;
    }

    if let Some(gesture) = interaction_state.graph_canvas_gestures.get(&widget_id)
        && let Some(ui_graph_editor::GraphActiveGesture::ConnectionPreview(connection)) =
            gesture.active
    {
        batch.push(
            GraphCanvasPrimitiveRole::ConnectionPreview,
            StrokePrimitive::new(
                [
                    graph_point_to_ui(bounds, viewport, connection.start),
                    graph_point_to_ui(bounds, viewport, connection.current),
                ],
                2.0,
                paint_from_color(graph_canvas.theme.accent),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            )
            .with_clip(bounds),
        );
        *primitive_order += 1;
    }

    for node in &graph_canvas.canvas.nodes {
        let rect = graph_rect_to_ui(bounds, viewport, node.rect);
        let header_rect = graph_rect_to_ui(
            bounds,
            viewport,
            ui_graph_editor::GraphRect::new(node.rect.x, node.rect.y, node.rect.width, 30),
        );
        batch.push(
            GraphCanvasPrimitiveRole::NodeBox,
            RectPrimitive::new(
                rect,
                graph_canvas.theme.radius.sm,
                paint_from_color(darken(graph_canvas.theme.background_panel, 0.92)),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            ),
        );
        *primitive_order += 1;
        batch.push(
            GraphCanvasPrimitiveRole::NodeBox,
            RectPrimitive::new(
                header_rect,
                graph_canvas.theme.radius.sm,
                paint_from_color(with_alpha(graph_canvas.theme.accent, 0.28)),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            ),
        );
        *primitive_order += 1;
        batch.push(
            GraphCanvasPrimitiveRole::NodeBox,
            BorderPrimitive::new(
                rect,
                graph_canvas.theme.radius.sm,
                graph_canvas.theme.border_width.max(1.0),
                paint_from_color(brighten(graph_canvas.theme.border, 1.12)),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            ),
        );
        *primitive_order += 1;
        if node.selected || graph_canvas.canvas.selection.nodes.contains(&node.node) {
            batch.push(
                GraphCanvasPrimitiveRole::SelectionOutline,
                BorderPrimitive::new(
                    rect,
                    graph_canvas.theme.radius.sm,
                    graph_canvas.theme.border_width.max(1.0),
                    paint_from_color(graph_canvas.theme.accent),
                    default_draw_key(),
                    sort_key(depth, *primitive_order),
                ),
            );
            *primitive_order += 1;
        }
        let label_rect = UiRect::new(
            rect.x + graph_canvas.theme.spacing.xs,
            rect.y + graph_canvas.theme.spacing.xs,
            (rect.width - graph_canvas.theme.spacing.xs * 2.0).max(0.0),
            graph_canvas.text_style.font_size * 1.4,
        );
        let label = LabelNode {
            text: node.title.clone(),
            text_style: graph_canvas.text_style.clone(),
            constraints: ui_layout::LayoutConstraints::tight(label_rect.size()),
        };
        emit_label(
            &label,
            label_rect,
            layer,
            atlas_source,
            layouter,
            depth,
            primitive_order,
        );
    }

    for port in &graph_canvas.canvas.ports {
        let port_rect = graph_rect_to_ui(bounds, viewport, port.rect);
        batch.push(
            GraphCanvasPrimitiveRole::Port,
            RectPrimitive::new(
                port_rect,
                4.0,
                paint_from_color(match port.direction {
                    ui_graph_editor::GraphPortDirection::Input => {
                        brighten(graph_canvas.theme.border, 1.45)
                    }
                    ui_graph_editor::GraphPortDirection::Output => graph_canvas.theme.accent,
                }),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            ),
        );
        *primitive_order += 1;
        let mut port_text_style = graph_canvas.text_style.clone();
        port_text_style.color[3] = (port_text_style.color[3] * 0.82).clamp(0.0, 1.0);
        port_text_style.align = match port.direction {
            ui_graph_editor::GraphPortDirection::Input => TextAlign::Start,
            ui_graph_editor::GraphPortDirection::Output => TextAlign::End,
        };
        let label_width = 150.0;
        let label_rect = match port.direction {
            ui_graph_editor::GraphPortDirection::Input => UiRect::new(
                port_rect.x + port_rect.width + graph_canvas.theme.spacing.xs,
                port_rect.y - 3.0,
                label_width,
                graph_canvas.text_style.font_size * 1.4,
            ),
            ui_graph_editor::GraphPortDirection::Output => UiRect::new(
                port_rect.x - label_width - graph_canvas.theme.spacing.xs,
                port_rect.y - 3.0,
                label_width,
                graph_canvas.text_style.font_size * 1.4,
            ),
        };
        let label = LabelNode {
            text: port.label.clone(),
            text_style: port_text_style,
            constraints: ui_layout::LayoutConstraints::tight(label_rect.size()),
        };
        emit_label(
            &label,
            label_rect,
            layer,
            atlas_source,
            layouter,
            depth,
            primitive_order,
        );
    }

    for overlay in &graph_canvas.canvas.overlays {
        batch.push(
            GraphCanvasPrimitiveRole::Overlay,
            BorderPrimitive::new(
                graph_rect_to_ui(bounds, viewport, overlay.rect),
                graph_canvas.theme.radius.sm,
                if overlay.active {
                    graph_canvas.theme.border_width.max(2.0)
                } else {
                    graph_canvas.theme.border_width.max(1.0)
                },
                paint_from_color(match overlay.severity {
                    ui_graph_editor::GraphOverlaySeverity::Info => graph_canvas.theme.accent,
                    ui_graph_editor::GraphOverlaySeverity::Warning => {
                        brighten(graph_canvas.theme.accent, 1.20)
                    }
                    ui_graph_editor::GraphOverlaySeverity::Error => {
                        ui_theme::UiColor::new(1.0, 0.12, 0.18, 1.0)
                    }
                }),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            ),
        );
        *primitive_order += 1;
    }

    for primitive in batch.into_ui_primitives() {
        layer.push(primitive);
    }

    if graph_canvas.clip {
        layer.push(UiPrimitive::Clip(ClipPrimitive::Pop {
            sort_key: sort_key(depth, *primitive_order),
        }));
        *primitive_order += 1;
    }
}

fn emit_graph_grid(
    batch: &mut GraphCanvasPrimitiveBatch,
    bounds: UiRect,
    viewport: ui_graph_editor::GraphViewport,
    minor_color: ui_theme::UiColor,
    major_color: ui_theme::UiColor,
    depth: u32,
    primitive_order: &mut u32,
) {
    const GRID_STEP: i32 = 64;
    const MAJOR_EVERY: i32 = 4;
    const MAX_LINES_PER_AXIS: i32 = 96;

    let zoom = viewport.zoom_milli.max(1) as f32 / 1000.0;
    let min_x = ((-viewport.pan_x as f32) / zoom).floor() as i32;
    let max_x = (((bounds.width - viewport.pan_x as f32) / zoom).ceil() as i32).max(min_x);
    let min_y = ((-viewport.pan_y as f32) / zoom).floor() as i32;
    let max_y = (((bounds.height - viewport.pan_y as f32) / zoom).ceil() as i32).max(min_y);
    let first_x = floor_to_grid(min_x, GRID_STEP);
    let first_y = floor_to_grid(min_y, GRID_STEP);

    let mut x = first_x;
    let mut x_count = 0;
    while x <= max_x && x_count < MAX_LINES_PER_AXIS {
        let major = (x / GRID_STEP).rem_euclid(MAJOR_EVERY) == 0;
        batch.push(
            GraphCanvasPrimitiveRole::Edge,
            StrokePrimitive::new(
                [
                    graph_point_to_ui(bounds, viewport, ui_graph_editor::GraphPoint::new(x, min_y)),
                    graph_point_to_ui(bounds, viewport, ui_graph_editor::GraphPoint::new(x, max_y)),
                ],
                if major { 1.1 } else { 0.7 },
                paint_from_color(if major { major_color } else { minor_color }),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            )
            .with_clip(bounds),
        );
        *primitive_order += 1;
        x += GRID_STEP;
        x_count += 1;
    }

    let mut y = first_y;
    let mut y_count = 0;
    while y <= max_y && y_count < MAX_LINES_PER_AXIS {
        let major = (y / GRID_STEP).rem_euclid(MAJOR_EVERY) == 0;
        batch.push(
            GraphCanvasPrimitiveRole::Edge,
            StrokePrimitive::new(
                [
                    graph_point_to_ui(bounds, viewport, ui_graph_editor::GraphPoint::new(min_x, y)),
                    graph_point_to_ui(bounds, viewport, ui_graph_editor::GraphPoint::new(max_x, y)),
                ],
                if major { 1.1 } else { 0.7 },
                paint_from_color(if major { major_color } else { minor_color }),
                default_draw_key(),
                sort_key(depth, *primitive_order),
            )
            .with_clip(bounds),
        );
        *primitive_order += 1;
        y += GRID_STEP;
        y_count += 1;
    }
}

fn floor_to_grid(value: i32, step: i32) -> i32 {
    value.div_euclid(step) * step
}

fn graph_rect_to_ui(
    canvas_bounds: UiRect,
    viewport: ui_graph_editor::GraphViewport,
    rect: ui_graph_editor::GraphRect,
) -> UiRect {
    let origin = graph_point_to_ui(
        canvas_bounds,
        viewport,
        ui_graph_editor::GraphPoint::new(rect.x, rect.y),
    );
    let zoom = viewport.zoom_milli.max(1) as f32 / 1000.0;
    UiRect::new(
        origin.x,
        origin.y,
        rect.width.max(0) as f32 * zoom,
        rect.height.max(0) as f32 * zoom,
    )
}

fn graph_point_to_ui(
    canvas_bounds: UiRect,
    viewport: ui_graph_editor::GraphViewport,
    point: ui_graph_editor::GraphPoint,
) -> ui_math::UiPoint {
    let zoom = viewport.zoom_milli.max(1) as f32 / 1000.0;
    ui_math::UiPoint::new(
        canvas_bounds.x + viewport.pan_x as f32 + point.x as f32 * zoom,
        canvas_bounds.y + viewport.pan_y as f32 + point.y as f32 * zoom,
    )
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
    if let Some(anchor) = button.reveal_on_hover_anchor {
        let anchor_interaction = interaction_state.for_widget(anchor);
        if !(interaction.hovered
            || interaction.pressed
            || anchor_interaction.hovered
            || anchor_interaction.pressed)
        {
            return;
        }
    }
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

    let radius = button_radius(button, bounds);
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        radius,
        paint_from_color(background),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;

    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        radius,
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

    emit_button_label(
        &button.label,
        &button_text_style(button, interaction),
        text_rect,
        layer,
        atlas_source,
        layouter,
        depth,
        primitive_order,
    );
}

fn button_radius(button: &ButtonNode, bounds: UiRect) -> f32 {
    button
        .corner_radius
        .unwrap_or(button.theme.radius.sm)
        .min(bounds.width.max(0.0) * 0.5)
        .min(bounds.height.max(0.0) * 0.5)
        .max(0.0)
}

#[expect(
    clippy::too_many_arguments,
    reason = "button label emission mirrors the surrounding primitive emission boundary"
)]
fn emit_button_label(
    text: &str,
    text_style: &ui_text::TextStyle,
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
            text,
            style: text_style,
            max_width: Some(bounds.width.max(0.0)),
        },
    ) else {
        return;
    };

    let align_offset = match text_style.align {
        TextAlign::Start => 0.0,
        TextAlign::Center => ((bounds.width - glyph_run.size.width) * 0.5).max(0.0),
        TextAlign::End => (bounds.width - glyph_run.size.width).max(0.0),
    };
    let vertical_offset =
        vertical_alignment_offset(&glyph_run, text_style, bounds.height, atlas_source);

    for glyph in &mut glyph_run.glyphs {
        glyph.origin.x += bounds.x + align_offset;
        glyph.origin.y += bounds.y + vertical_offset;
    }

    layer.push(UiPrimitive::GlyphRun(GlyphRunPrimitive::new(
        glyph_run,
        Some(bounds),
        UiPaint::rgba(
            text_style.color[0],
            text_style.color[1],
            text_style.color[2],
            text_style.color[3],
        ),
        UiDrawKey::new(0, Some(text_style.font_id.0)),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
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
        depth,
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
        depth,
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
        depth,
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
            depth,
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
        depth,
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
        depth,
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
            depth,
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
            depth,
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

    let align_offset = match label.text_style.align {
        TextAlign::Start => 0.0,
        TextAlign::Center => ((bounds.width - glyph_run.size.width) * 0.5).max(0.0),
        TextAlign::End => (bounds.width - glyph_run.size.width).max(0.0),
    };

    let vertical_offset =
        vertical_alignment_offset(&glyph_run, &label.text_style, bounds.height, atlas_source);
    for glyph in &mut glyph_run.glyphs {
        glyph.origin.x += bounds.x + align_offset;
        glyph.origin.y += bounds.y + vertical_offset;
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

fn vertical_alignment_offset(
    glyph_run: &ui_text::GlyphRun,
    text_style: &ui_text::TextStyle,
    bounds_height: f32,
    atlas_source: &dyn FontAtlasSource,
) -> f32 {
    match text_style.vertical_align {
        TextVerticalAlign::LineBoxCenter => {
            ((bounds_height - glyph_run.size.height) * 0.5).max(0.0)
        }
        TextVerticalAlign::InkBoundsCenter | TextVerticalAlign::CapHeightCenter => {
            ink_bounds_vertical_offset(glyph_run, text_style, bounds_height, atlas_source)
                .unwrap_or_else(|| ((bounds_height - glyph_run.size.height) * 0.5).max(0.0))
        }
    }
}

fn ink_bounds_vertical_offset(
    glyph_run: &ui_text::GlyphRun,
    text_style: &ui_text::TextStyle,
    bounds_height: f32,
    atlas_source: &dyn FontAtlasSource,
) -> Option<f32> {
    let atlas = atlas_source.atlas(text_style.font_id)?;
    let scale = text_style.font_size / atlas.metrics.base_size.max(f32::EPSILON);
    let mut top = f32::INFINITY;
    let mut bottom = f32::NEG_INFINITY;
    for glyph in &glyph_run.glyphs {
        let metrics = atlas
            .glyphs
            .get(&glyph.ch)
            .or_else(|| atlas.glyphs.get(&'?'))?;
        top = top.min(glyph.origin.y - metrics.plane_top * scale);
        bottom = bottom.max(glyph.origin.y - metrics.plane_bottom * scale);
    }
    if !top.is_finite() || !bottom.is_finite() {
        return None;
    }

    Some(bounds_height * 0.5 - (top + bottom) * 0.5)
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
    text_style.align = TextAlign::Center;
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

fn sort_key(layer_order: u32, primitive_order: u32) -> UiSortKey {
    UiSortKey::new(0, layer_order, primitive_order)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{UiRuntimeState, WidgetId, compute_tree_layout};
    use ui_render_data::ViewportSurfaceEmbedSlotId;
    use ui_text::{
        FontFaceMetrics, FontId, GlyphMetrics, MsdfFontAtlas, TextAlign, TextOverflow, TextStyle,
        TextVerticalAlign, TextWrap,
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
            vertical_align: TextVerticalAlign::LineBoxCenter,
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
            "ClipPush(x=14.0 y=18.0 w=236.0 h=92.0)",
            "GlyphRun(text=\"Overl\" clip=true)",
            "ClipPop",
        ]
        .join("\n");
        assert_eq!(signature, expected);
    }

    #[test]
    fn build_ui_frame_emits_viewport_embed_with_full_product_uv() {
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

        assert_eq!(embed.uv_rect, UiRect::new(0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn popup_button_emits_text_on_popup_layer() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let theme = ThemeTokens::default();
        let anchor_id = WidgetId(2);
        let popup_id = WidgetId(3);
        let item_id = WidgetId(4);
        let text_style = theme.body_small_text_style(FontId(1));
        let mut popup_button = ButtonNode::new("Save", text_style.clone(), theme.clone());
        popup_button.fill_width = true;
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![
                UiNode::new(
                    anchor_id,
                    UiNodeKind::Button(ButtonNode::new("File", text_style.clone(), theme.clone())),
                ),
                UiNode::with_children(
                    popup_id,
                    UiNodeKind::Popup(PopupNode::anchored_bottom_start(anchor_id, theme.clone())),
                    vec![UiNode::new(item_id, UiNodeKind::Button(popup_button))],
                ),
            ],
        ));
        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(0.0, 0.0, 240.0, 160.0),
            &UiRuntimeState::default(),
        );

        let frame = build_ui_frame(
            &tree,
            &layouts,
            UiSize::new(240.0, 160.0),
            InteractionVisualState::default(),
            &atlas_source,
        );

        let popup_text = frame.surfaces[0].layers[0]
            .primitives
            .iter()
            .find_map(|primitive| {
                let UiPrimitive::GlyphRun(run) = primitive else {
                    return None;
                };
                let text = run
                    .glyph_run
                    .glyphs
                    .iter()
                    .map(|glyph| glyph.ch)
                    .collect::<String>();
                (text == "Save").then_some(run)
            })
            .unwrap_or_else(|| {
                panic!(
                    "popup button text should emit a glyph run; frame:\n{}",
                    frame_signature(&frame)
                )
            });

        assert_eq!(popup_text.sort_key.layer_order, POPUP_LAYER_ORDER + 1);

        let popup_background = frame.surfaces[0].layers[0]
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                UiPrimitive::Rect(rect)
                    if rect.sort_key.layer_order == POPUP_LAYER_ORDER + 1
                        && rect.sort_key.primitive_order < popup_text.sort_key.primitive_order =>
                {
                    Some(rect)
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "popup background should render on the popup layer before text; frame:\n{}",
                    frame_signature(&frame)
                )
            });

        assert_eq!(popup_background.sort_key.layer_order, POPUP_LAYER_ORDER + 1);
        assert!(
            popup_background.sort_key.primitive_order < popup_text.sort_key.primitive_order,
            "popup background must not render after popup text; frame:\n{}",
            frame_signature(&frame)
        );
        assert!(
            frame.surfaces[0].layers[0]
                .primitives
                .iter()
                .all(|primitive| primitive_sort_key(primitive).layer_order <= POPUP_LAYER_ORDER + 1),
            "test popup frame should not emit a higher overlay layer than the popup text; frame:\n{}",
            frame_signature(&frame)
        );
    }

    #[test]
    fn inside_top_end_adornment_stays_in_scroll_layer_under_scrollbar() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let theme = ThemeTokens::default();
        let scroll_id = WidgetId(10);
        let row_id = WidgetId(11);
        let anchor_id = WidgetId(12);
        let filler_id = WidgetId(13);
        let popup_id = WidgetId(14);
        let close_id = WidgetId(15);
        let text_style = theme.body_small_text_style(FontId(1));
        let mut filler = ButtonNode::new("Very wide tab title", text_style.clone(), theme.clone());
        filler.min_size.width = 180.0;
        let mut close = ButtonNode::new("x", text_style.clone(), theme.clone());
        close.reveal_on_hover_anchor = Some(anchor_id);
        close.min_size = UiSize::new(18.0, 18.0);
        close.padding = ui_math::UiInsets::ZERO;
        let mut popup = PopupNode::anchored_inside_top_end(anchor_id, theme.clone());
        popup.offset = theme.spacing.xs;

        let tree = UiTree::new(UiNode::with_children(
            scroll_id,
            UiNodeKind::Scroll(crate::ScrollNode::horizontal(theme.clone())),
            vec![UiNode::with_children(
                row_id,
                UiNodeKind::Stack(crate::StackNode::horizontal(theme.spacing.xs)),
                vec![
                    UiNode::new(
                        anchor_id,
                        UiNodeKind::Button(ButtonNode::new("Tab", text_style, theme.clone())),
                    ),
                    UiNode::new(filler_id, UiNodeKind::Button(filler)),
                    UiNode::with_children(
                        popup_id,
                        UiNodeKind::Popup(popup),
                        vec![UiNode::new(close_id, UiNodeKind::Button(close))],
                    ),
                ],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 96.0, 36.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());
        let frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState {
                hovered_widget: Some(anchor_id),
                active_scrollbar: Some(ScrollbarAxisTarget::new(scroll_id, Axis::Horizontal)),
                ..Default::default()
            },
            &atlas_source,
        );
        let close_text = frame.surfaces[0].layers[0]
            .primitives
            .iter()
            .find_map(|primitive| {
                let UiPrimitive::GlyphRun(run) = primitive else {
                    return None;
                };
                let text = run
                    .glyph_run
                    .glyphs
                    .iter()
                    .map(|glyph| glyph.ch)
                    .collect::<String>();
                (text == "x").then_some(run)
            })
            .unwrap_or_else(|| {
                panic!(
                    "inside adornment close text should emit; frame:\n{}",
                    frame_signature(&frame)
                )
            });
        let scroll_layout = layouts.get(&scroll_id).expect("scroll layout should exist");
        let scrollbar_track = scrollbar_geometry(
            &tree,
            scroll_id,
            &layouts,
            scroll_layout.bounds,
            scroll_layout.content_bounds,
        )
        .expect("overflowing horizontal scroll should have a scrollbar")
        .track_rect;
        let track_rect = frame.surfaces[0].layers[0]
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                UiPrimitive::Rect(rect) if rect_approx_eq(rect.rect, scrollbar_track) => Some(rect),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "active scrollbar track should emit; frame:\n{}",
                    frame_signature(&frame)
                )
            });

        assert_eq!(close_text.sort_key.layer_order, BASE_LAYER_ORDER);
        assert_eq!(track_rect.sort_key.layer_order, BASE_LAYER_ORDER);
        assert!(
            track_rect.sort_key.primitive_order > close_text.sort_key.primitive_order,
            "scrollbar must paint over in-scroll adornments; frame:\n{}",
            frame_signature(&frame)
        );
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
        let inactive_overflow_frame = build_ui_frame(
            &overflow_tree,
            &overflow_layouts,
            overflow_bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let active_overflow_frame = build_ui_frame(
            &overflow_tree,
            &overflow_layouts,
            overflow_bounds.size(),
            InteractionVisualState {
                active_scrollbar: Some(ScrollbarAxisTarget::new(scroll_id, Axis::Vertical)),
                ..Default::default()
            },
            &atlas_source,
        );

        let scroll_layout = overflow_layouts
            .get(&scroll_id)
            .expect("scroll layout should exist");
        let track_rect = scrollbar_geometry(
            &overflow_tree,
            scroll_id,
            &overflow_layouts,
            scroll_layout.bounds,
            scroll_layout.content_bounds,
        )
        .expect("overflowing scroll should have scrollbar geometry")
        .track_rect;
        assert!(
            !has_rect_primitive(&inactive_overflow_frame, track_rect),
            "overflowing scrollbars should stay hidden until active",
        );
        assert!(
            has_rect_primitive(&active_overflow_frame, track_rect),
            "active overflowing scroll should emit an overlay scrollbar track primitive",
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
        let _no_overflow_frame = build_ui_frame(
            &no_overflow_tree,
            &no_overflow_layouts,
            no_overflow_bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let no_overflow_scroll = no_overflow_layouts
            .get(&WidgetId(11))
            .expect("scroll layout should exist");
        assert!(
            scrollbar_geometry(
                &no_overflow_tree,
                WidgetId(11),
                &no_overflow_layouts,
                no_overflow_scroll.bounds,
                no_overflow_scroll.content_bounds,
            )
            .is_none(),
            "non-overflowing scroll should not emit a scrollbar track primitive",
        );
    }

    #[test]
    fn build_ui_frame_reveals_two_axis_scrollbars_per_axis() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let scroll_id = WidgetId(31);
        let child_id = WidgetId(32);
        let rows = (0..8)
            .map(|row| {
                let columns = (0..5)
                    .map(|column| {
                        let mut button = ButtonNode::new(
                            format!("Cell {row}-{column}"),
                            text_style.clone(),
                            theme.clone(),
                        );
                        button.min_size = UiSize::new(96.0, 28.0);
                        UiNode::new(
                            WidgetId(1_000 + row * 10 + column),
                            UiNodeKind::Button(button),
                        )
                    })
                    .collect::<Vec<_>>();
                UiNode::with_children(
                    WidgetId(2_000 + row),
                    UiNodeKind::Stack(crate::StackNode::horizontal(theme.spacing.xs)),
                    columns,
                )
            })
            .collect::<Vec<_>>();
        let tree = UiTree::new(UiNode::with_children(
            scroll_id,
            UiNodeKind::Scroll(crate::ScrollNode::both(theme.clone())),
            vec![UiNode::with_children(
                child_id,
                UiNodeKind::Stack(crate::StackNode::vertical(theme.spacing.xs)),
                rows,
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 180.0, 96.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());
        let scroll_layout = layouts.get(&scroll_id).expect("scroll layout should exist");
        let vertical_track = scrollbar_geometry_for_axis(
            &tree,
            scroll_id,
            &layouts,
            scroll_layout.bounds,
            scroll_layout.content_bounds,
            Axis::Vertical,
        )
        .expect("vertical scrollbar should exist")
        .track_rect;
        let horizontal_track = scrollbar_geometry_for_axis(
            &tree,
            scroll_id,
            &layouts,
            scroll_layout.bounds,
            scroll_layout.content_bounds,
            Axis::Horizontal,
        )
        .expect("horizontal scrollbar should exist")
        .track_rect;

        let inactive_frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );
        assert!(!has_rect_primitive(&inactive_frame, vertical_track));
        assert!(!has_rect_primitive(&inactive_frame, horizontal_track));

        let vertical_frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState {
                active_scrollbar: Some(ScrollbarAxisTarget::new(scroll_id, Axis::Vertical)),
                ..Default::default()
            },
            &atlas_source,
        );
        assert!(has_rect_primitive(&vertical_frame, vertical_track));
        assert!(!has_rect_primitive(&vertical_frame, horizontal_track));

        let horizontal_frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState {
                active_scrollbar: Some(ScrollbarAxisTarget::new(scroll_id, Axis::Horizontal)),
                ..Default::default()
            },
            &atlas_source,
        );
        assert!(!has_rect_primitive(&horizontal_frame, vertical_track));
        assert!(has_rect_primitive(&horizontal_frame, horizontal_track));

        let hovered_horizontal_frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState {
                hovered_scrollbar: Some(ScrollbarAxisTarget::new(scroll_id, Axis::Horizontal)),
                ..Default::default()
            },
            &atlas_source,
        );
        assert!(!has_rect_primitive(
            &hovered_horizontal_frame,
            vertical_track
        ));
        assert!(has_rect_primitive(
            &hovered_horizontal_frame,
            horizontal_track
        ));
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
                ..Default::default()
            },
            &atlas_source,
        );
        let focus_frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState {
                focused_widget: Some(button_id),
                ..Default::default()
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

    #[test]
    fn button_emission_supports_round_close_shape_and_centered_label() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let theme = ThemeTokens::default();
        let button_id = WidgetId(221);
        let style = TextStyle {
            font_id: FontId(1),
            font_size: 12.0,
            ..TextStyle::default()
        };
        let mut button = crate::ButtonNode::new("x", style, theme);
        button.padding = ui_math::UiInsets::ZERO;
        button.min_size = UiSize::new(18.0, 18.0);
        button.corner_radius = Some(f32::MAX);
        let tree = UiTree::new(UiNode::new(button_id, UiNodeKind::Button(button)));
        let bounds = UiRect::new(0.0, 0.0, 18.0, 18.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());
        let frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );

        let rect = frame
            .surfaces
            .iter()
            .flat_map(|surface| surface.layers.iter())
            .flat_map(|layer| layer.primitives.iter())
            .find_map(|primitive| match primitive {
                UiPrimitive::Rect(rect) => Some(rect),
                _ => None,
            })
            .expect("button should emit a background rect");
        assert!(
            (rect.radius - 9.0).abs() <= 0.001,
            "full corner radius should clamp to a circular 50% radius"
        );

        let glyph = frame
            .surfaces
            .iter()
            .flat_map(|surface| surface.layers.iter())
            .flat_map(|layer| layer.primitives.iter())
            .find_map(|primitive| match primitive {
                UiPrimitive::GlyphRun(run) => run.glyph_run.glyphs.first(),
                _ => None,
            })
            .expect("button label should emit a glyph");
        assert!(
            glyph.origin.x > 0.0 && glyph.origin.y > 0.0,
            "button label should be centered away from the top-left edge"
        );
    }

    #[test]
    fn icon_button_uses_ink_bounds_vertical_centering() {
        let mut atlas = atlas_with_ascii(FontId(1));
        let metrics = atlas.glyphs.get_mut(&'x').expect("x glyph should exist");
        metrics.plane_top = 3.0;
        metrics.plane_bottom = 0.0;
        let atlas_source = TestAtlasSource { atlas };
        let theme = ThemeTokens::default();
        let button_id = WidgetId(231);
        let mut style = TextStyle {
            font_id: FontId(1),
            font_size: 12.0,
            ..TextStyle::default()
        };
        style.vertical_align = TextVerticalAlign::InkBoundsCenter;
        let mut button = crate::ButtonNode::new("x", style, theme);
        button.padding = ui_math::UiInsets::ZERO;
        button.min_size = UiSize::new(18.0, 18.0);
        let tree = UiTree::new(UiNode::new(button_id, UiNodeKind::Button(button)));
        let bounds = UiRect::new(0.0, 0.0, 18.0, 18.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());
        let frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let glyph = first_glyph(&frame).expect("button label should emit a glyph");
        let rendered_top = glyph.origin.y - 3.0;
        let rendered_bottom = glyph.origin.y;
        let ink_center = (rendered_top + rendered_bottom) * 0.5;

        assert!(
            (ink_center - 9.0).abs() <= 0.001,
            "icon glyph ink bounds should be centered in the button"
        );
    }

    #[test]
    fn normal_label_keeps_line_box_vertical_centering() {
        let mut atlas = atlas_with_ascii(FontId(1));
        let metrics = atlas.glyphs.get_mut(&'x').expect("x glyph should exist");
        metrics.plane_top = 3.0;
        metrics.plane_bottom = 0.0;
        let atlas_source = TestAtlasSource { atlas };
        let style = TextStyle {
            font_id: FontId(1),
            font_size: 12.0,
            line_height: Some(12.0),
            vertical_align: TextVerticalAlign::LineBoxCenter,
            ..TextStyle::default()
        };
        let mut label = LabelNode::new("x", style);
        label.constraints = ui_layout::LayoutConstraints::tight(UiSize::new(18.0, 18.0));
        let tree = UiTree::new(UiNode::new(WidgetId(241), UiNodeKind::Label(label)));
        let bounds = UiRect::new(0.0, 0.0, 18.0, 18.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());
        let frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let glyph = first_glyph(&frame).expect("label should emit a glyph");

        assert!(
            (glyph.origin.y - 12.0).abs() <= 0.001,
            "normal labels should keep typographic line-box centering"
        );
    }

    #[test]
    fn scroll_children_and_overlays_remain_inside_scroll_clip_stack() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let theme = ThemeTokens::default();
        let scroll_id = WidgetId(301);
        let anchor_id = WidgetId(302);
        let popup_id = WidgetId(303);
        let popup_button_id = WidgetId(304);
        let mut anchor = crate::ButtonNode::new("Wide tab", TextStyle::default(), theme.clone());
        anchor.min_size = UiSize::new(200.0, 24.0);
        let mut close = crate::ButtonNode::new("x", TextStyle::default(), theme.clone());
        close.min_size = UiSize::new(18.0, 18.0);
        let mut popup = PopupNode::anchored_inside_top_end(anchor_id, theme.clone());
        popup.offset = 0.0;
        let row = UiNode::with_children(
            WidgetId(305),
            UiNodeKind::Stack(crate::StackNode::horizontal(0.0)),
            vec![
                UiNode::new(anchor_id, UiNodeKind::Button(anchor)),
                UiNode::with_children(
                    popup_id,
                    UiNodeKind::Popup(popup),
                    vec![UiNode::new(popup_button_id, UiNodeKind::Button(close))],
                ),
            ],
        );
        let tree = UiTree::new(UiNode::with_children(
            scroll_id,
            UiNodeKind::Scroll(crate::ScrollNode::horizontal(theme)),
            vec![row],
        ));
        let bounds = UiRect::new(0.0, 0.0, 80.0, 24.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());
        let frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );

        let mut clip_depth = 0_i32;
        let mut popup_rect_depth = None;
        for primitive in &frame.surfaces[0].layers[0].primitives {
            match primitive {
                UiPrimitive::Clip(ClipPrimitive::Push { .. }) => clip_depth += 1,
                UiPrimitive::Clip(ClipPrimitive::Pop { .. }) => clip_depth -= 1,
                UiPrimitive::Rect(rect) if rect.rect.x >= 80.0 => {
                    popup_rect_depth = Some(clip_depth);
                    break;
                }
                _ => {}
            }
        }

        assert!(
            popup_rect_depth.is_some_and(|depth| depth > 0),
            "overflowing overlay primitives should still be emitted under the scroll clip"
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
                UiPrimitive::ProductSurface(value) => format!(
                    "ProductSurface(x={:.1} y={:.1} w={:.1} h={:.1})",
                    value.rect.x, value.rect.y, value.rect.width, value.rect.height
                ),
                UiPrimitive::Stroke(value) => {
                    format!(
                        "Stroke(points={} width={:.1})",
                        value.points.len(),
                        value.width
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn primitive_sort_key(primitive: &UiPrimitive) -> UiSortKey {
        match primitive {
            UiPrimitive::Rect(value) => value.sort_key,
            UiPrimitive::Border(value) => value.sort_key,
            UiPrimitive::GlyphRun(value) => value.sort_key,
            UiPrimitive::Image(value) => value.sort_key,
            UiPrimitive::Stroke(value) => value.sort_key,
            UiPrimitive::ViewportSurfaceEmbed(value) => value.sort_key,
            UiPrimitive::ProductSurface(value) => value.sort_key,
            UiPrimitive::Clip(ClipPrimitive::Push { sort_key, .. }) => *sort_key,
            UiPrimitive::Clip(ClipPrimitive::Pop { sort_key }) => *sort_key,
        }
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

    fn first_glyph(frame: &UiFrame) -> Option<&ui_text::PositionedGlyph> {
        frame
            .surfaces
            .iter()
            .flat_map(|surface| surface.layers.iter())
            .flat_map(|layer| layer.primitives.iter())
            .find_map(|primitive| match primitive {
                UiPrimitive::GlyphRun(run) => run.glyph_run.glyphs.first(),
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
