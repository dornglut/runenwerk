//! File: domain/ui/ui_runtime/src/output/build_ui_frame.rs
//! Purpose: Convert retained tree + computed layout into UiFrame.

use crate::{ButtonNode, ComputedLayoutMap, LabelNode, PanelNode, UiNode, UiNodeKind, UiTree};
use ui_math::{UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, ClipPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer,
    UiLayerId, UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};
use ui_text::{GlyphRun, PositionedGlyph};

pub fn build_ui_frame(tree: &UiTree, layouts: &ComputedLayoutMap, surface_size: UiSize) -> UiFrame {
    let mut layer = UiLayer::new(UiLayerId(0));
    let mut primitive_order = 0u32;

    emit_node(&tree.root, layouts, &mut layer, 0, &mut primitive_order);

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
        UiNodeKind::Label(label) => emit_label(label, layout.bounds, layer, depth, primitive_order),
        UiNodeKind::Button(button) => emit_button(
            button,
            layout.bounds,
            layout.content_bounds,
            layer,
            depth,
            primitive_order,
        ),
        UiNodeKind::Stack(_) | UiNodeKind::Split(_) => {}
    }

    for child in &node.children {
        emit_node(child, layouts, layer, depth + 1, primitive_order);
    }

    if matches!(node.kind, UiNodeKind::Panel(_) | UiNodeKind::Button(_)) {
        layer.push(UiPrimitive::Clip(ClipPrimitive::Pop {
            sort_key: sort_key(depth, *primitive_order),
        }));
        *primitive_order += 1;
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
        1.0,
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

fn emit_button(
    button: &ButtonNode,
    bounds: UiRect,
    content_bounds: UiRect,
    layer: &mut UiLayer,
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
        1.0,
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

    emit_label(&label_node, text_rect, layer, depth + 1, primitive_order);
}

fn emit_label(
    label: &LabelNode,
    bounds: UiRect,
    layer: &mut UiLayer,
    depth: u32,
    primitive_order: &mut u32,
) {
    let glyph_run = estimate_glyph_run(label, bounds);

    layer.push(UiPrimitive::GlyphRun(GlyphRunPrimitive::new(
        glyph_run,
        Some(bounds),
        UiPaint::rgba(
            label.text_style.color[0],
            label.text_style.color[1],
            label.text_style.color[2],
            label.text_style.color[3],
        ),
        default_draw_key(),
        sort_key(depth, *primitive_order),
    )));
    *primitive_order += 1;
}

fn estimate_glyph_run(label: &LabelNode, bounds: UiRect) -> GlyphRun {
    let advance = label.text_style.font_size * 0.6;
    let baseline_y = bounds.y + label.text_style.font_size;

    let glyphs = label
        .text
        .chars()
        .enumerate()
        .map(|(index, ch)| PositionedGlyph {
            ch,
            origin: ui_math::UiPoint::new(bounds.x + advance * index as f32, baseline_y),
            advance,
        })
        .collect();

    GlyphRun {
        font_id: label.text_style.font_id,
        font_size: label.text_style.font_size,
        glyphs,
        size: bounds.size(),
    }
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
