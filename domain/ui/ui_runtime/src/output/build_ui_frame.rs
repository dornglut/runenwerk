//! File: domain/ui/ui_runtime/src/output/build_ui_frame.rs
//! Purpose: Convert retained tree + computed layout into UiFrame.

use crate::{
    ButtonNode, ComputedLayoutMap, LabelNode, NumericInputNode, PanelNode, ScrollNode, TabsNode,
    TextInputNode, ToggleNode, UiNode, UiNodeKind, UiTree, ViewportSurfaceEmbedNode,
};
use ui_math::{UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, ClipPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer,
    UiLayerId, UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
    ViewportSurfaceEmbedPrimitive,
};
use ui_text::{AtlasTextLayouter, FontAtlasSource, TextLayoutRequest, TextLayouter};

pub fn build_ui_frame(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    surface_size: UiSize,
    atlas_source: &dyn FontAtlasSource,
) -> UiFrame {
    let mut layer = UiLayer::new(UiLayerId(0));
    let mut primitive_order = 0u32;
    let layouter = AtlasTextLayouter;

    emit_node(
        &tree.root,
        layouts,
        &mut layer,
        surface_size,
        atlas_source,
        &layouter,
        0,
        &mut primitive_order,
    );

    UiFrame::with_surfaces(vec![UiSurface::with_layers(
        UiSurfaceId(0),
        surface_size,
        vec![layer],
    )])
}

fn emit_node(
    node: &UiNode,
    layouts: &ComputedLayoutMap,
    layer: &mut UiLayer,
    surface_size: UiSize,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
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
            button,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            depth,
            primitive_order,
        ),
        UiNodeKind::TextInput(text_input) => emit_text_input(
            text_input,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            depth,
            primitive_order,
        ),
        UiNodeKind::Toggle(toggle) => emit_toggle(
            toggle,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            depth,
            primitive_order,
        ),
        UiNodeKind::NumericInput(numeric) => emit_numeric_input(
            numeric,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            depth,
            primitive_order,
        ),
        UiNodeKind::Tabs(tabs) => emit_tabs(
            tabs,
            layout.bounds,
            layout.content_bounds,
            layer,
            atlas_source,
            layouter,
            depth,
            primitive_order,
        ),
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
            child,
            layouts,
            layer,
            surface_size,
            atlas_source,
            layouter,
            depth + 1,
            primitive_order,
        );
    }

    match &node.kind {
        UiNodeKind::Panel(_)
        | UiNodeKind::Button(_)
        | UiNodeKind::TextInput(_)
        | UiNodeKind::NumericInput(_)
        | UiNodeKind::Tabs(_) => {
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

fn emit_scrollbar(
    node: &UiNode,
    scroll: &ScrollNode,
    layouts: &ComputedLayoutMap,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    let track_width = (bounds.width - content_bounds.width).max(0.0);
    if track_width <= f32::EPSILON || content_bounds.height <= f32::EPSILON {
        return;
    }

    let track_rect = UiRect::new(
        content_bounds.x + content_bounds.width,
        content_bounds.y,
        track_width,
        content_bounds.height,
    );

    let Some(child) = node.children.first() else {
        return;
    };
    let Some(child_layout) = layouts.get(&child.id) else {
        return;
    };

    let viewport_height = content_bounds.height.max(0.0);
    let content_height = child_layout.measured_size.height.max(viewport_height);
    let max_offset = (content_height - viewport_height).max(0.0);
    let scroll_offset = (content_bounds.y - child_layout.bounds.y).clamp(0.0, max_offset);

    let thumb_height = if max_offset <= f32::EPSILON {
        track_rect.height
    } else {
        let min_thumb_height = scroll.min_thumb_height.min(track_rect.height).max(0.0);
        let natural_thumb_height = (viewport_height / content_height) * track_rect.height;
        natural_thumb_height.clamp(min_thumb_height, track_rect.height)
    };
    let thumb_range = (track_rect.height - thumb_height).max(0.0);
    let thumb_y = if max_offset <= f32::EPSILON {
        track_rect.y
    } else {
        track_rect.y + thumb_range * (scroll_offset / max_offset)
    };
    let thumb_rect = UiRect::new(track_rect.x, thumb_y, track_rect.width, thumb_height);
    let radius = scroll.theme.radius.sm.min(track_rect.width * 0.5);

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

fn normalized_uv_rect(bounds: UiRect, surface_size: UiSize) -> UiRect {
    let width = surface_size.width.max(1.0);
    let height = surface_size.height.max(1.0);

    let u0 = (bounds.x / width).clamp(0.0, 1.0);
    let v0 = (bounds.y / height).clamp(0.0, 1.0);
    let u1 = ((bounds.x + bounds.width) / width).clamp(0.0, 1.0);
    let v1 = ((bounds.y + bounds.height) / height).clamp(0.0, 1.0);

    UiRect::new(u0, v0, (u1 - u0).max(0.0), (v1 - v0).max(0.0))
}

fn emit_button(
    button: &ButtonNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    depth: u32,
    primitive_order: &mut u32,
) {
    let background = if button.enabled {
        button.theme.accent
    } else {
        button.theme.border
    };

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
        paint_from_color(button.theme.border),
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
        text_style: button.text_style.clone(),
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

fn emit_text_input(
    text_input: &TextInputNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        text_input.theme.radius.sm,
        paint_from_color(text_input.theme.background_panel),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        text_input.theme.radius.sm,
        text_input.theme.border_width,
        paint_from_color(text_input.theme.border),
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

fn emit_toggle(
    toggle: &ToggleNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        toggle.theme.radius.sm,
        paint_from_color(toggle.theme.background_panel),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        toggle.theme.radius.sm,
        toggle.theme.border_width,
        paint_from_color(toggle.theme.border),
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

fn emit_numeric_input(
    numeric: &NumericInputNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        numeric.theme.radius.sm,
        paint_from_color(numeric.theme.background_panel),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        numeric.theme.radius.sm,
        numeric.theme.border_width,
        paint_from_color(numeric.theme.border),
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

fn emit_tabs(
    tabs: &TabsNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    depth: u32,
    primitive_order: &mut u32,
) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        tabs.theme.radius.sm,
        paint_from_color(tabs.theme.background_panel),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        tabs.theme.radius.sm,
        tabs.theme.border_width,
        paint_from_color(tabs.theme.border),
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

fn paint_from_color(color: ui_theme::UiColor) -> UiPaint {
    UiPaint::rgba(color.r, color.g, color.b, color.a)
}

fn default_draw_key() -> UiDrawKey {
    UiDrawKey::new(0, None)
}

fn sort_key(depth: u32, primitive_order: u32) -> UiSortKey {
    UiSortKey::new(0, depth, primitive_order)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{UiRuntimeState, WidgetId, compute_tree_layout};
    use ui_render_data::ViewportSurfaceSlot;
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

        let frame = build_ui_frame(&tree, &layouts, UiSize::new(320.0, 180.0), &atlas_source);
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
                ViewportSurfaceSlot::Primary,
            )),
        ));
        let layouts = compute_tree_layout(
            &tree,
            UiRect::new(10.0, 20.0, 100.0, 50.0),
            &UiRuntimeState::default(),
        );

        let frame = build_ui_frame(&tree, &layouts, UiSize::new(200.0, 100.0), &atlas_source);
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
