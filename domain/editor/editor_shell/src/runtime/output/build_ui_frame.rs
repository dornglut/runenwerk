//! File: domain/ui/ui_runtime/src/output/build_ui_frame.rs
//! Purpose: Convert retained tree + computed layout into UiFrame.

use crate::{ButtonNode, ComputedLayoutMap, LabelNode, PanelNode, UiNode, UiNodeKind, UiTree};
use ui_math::{UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, ClipPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer,
    UiLayerId, UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
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
        UiNodeKind::Stack(_) | UiNodeKind::Split(_) => {}
    }

    for child in &node.children {
        emit_node(
            child,
            layouts,
            layer,
            atlas_source,
            layouter,
            depth + 1,
            primitive_order,
        );
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
