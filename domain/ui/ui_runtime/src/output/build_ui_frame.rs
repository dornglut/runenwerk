//! File: domain/ui/ui_runtime/src/output/build_ui_frame.rs
//! Purpose: Convert retained tree + computed layout into UiFrame.

use crate::{
    ComputedLayoutMap, ScrollbarAxisOpacities, ScrollbarAxisTarget, UiNode, UiNodeKind, UiTree,
    WidgetId,
};
use std::collections::BTreeMap;

#[cfg(test)]
pub(crate) use super::emit::containers::scrollbar_geometry;
pub(crate) use super::emit::containers::{
    ScrollbarGeometry, scrollbar_geometries, scrollbar_geometry_for_axis,
};
use super::emit::containers::{
    emit_panel, emit_popup, emit_radial_menu, emit_scroll_begin, emit_scrollbar,
};
use super::emit::controls::{
    emit_button, emit_label, emit_numeric_input, emit_select, emit_table, emit_tabs,
    emit_text_input, emit_toggle, emit_tree,
};
use super::emit::graph_canvas::emit_graph_canvas;
use super::emit::surface::{
    emit_divider, emit_image, emit_product_surface, emit_viewport_surface_embed,
};
use super::primitives::sort_key;
use ui_math::UiSize;
use ui_render_data::{
    ClipPrimitive, UiFrame, UiLayer, UiLayerId, UiPrimitive, UiSurface, UiSurfaceId,
};
use ui_text::{AtlasTextLayouter, FontAtlasSource, TextLayouter};

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

#[cfg(test)]
mod tests {
    use super::super::test_support::{
        TestAtlasSource, atlas_with_ascii, first_border_paint, first_glyph, first_rect_paint,
        first_text_glyph, frame_signature, has_rect_primitive, primitive_sort_key, rect_approx_eq,
    };
    use super::*;
    use crate::{
        ButtonNode, LabelNode, PanelNode, PopupNode, UiRuntimeState, ViewportSurfaceEmbedNode,
        WidgetId, compute_tree_layout,
    };
    use ui_math::{Axis, UiRect};
    use ui_render_data::{UiDrawKey, UiPaint, ViewportSurfaceEmbedSlotId};
    use ui_text::{
        FontId, TextEllipsisPlacement, TextLineHeightPolicy, TextOverflowPolicy, TextStyle,
    };
    use ui_theme::ThemeTokens;

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
            line_height: TextLineHeightPolicy::Absolute(18.0),
            ..TextStyle::default()
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
            "GlyphRun(text=\"Overlay\" clip=true)",
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
                let text = crate::text_emission::text_from_primitive(run);
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
                let text = crate::text_emission::text_from_primitive(run);
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
                UiPrimitive::GlyphRun(run) => first_text_glyph(run),
                _ => None,
            })
            .expect("button label should emit a glyph");
        assert!(
            glyph.origin.x > 0.0 && glyph.origin.y > 0.0,
            "button label should be centered away from the top-left edge"
        );
    }

    #[test]
    fn icon_button_uses_line_box_vertical_centering() {
        let mut atlas = atlas_with_ascii(FontId(1));
        let metrics = atlas.glyphs.get_mut(&'x').expect("x glyph should exist");
        metrics.plane_top = 3.0;
        metrics.plane_bottom = 0.0;
        let atlas_source = TestAtlasSource { atlas };
        let theme = ThemeTokens::default();
        let button_id = WidgetId(231);
        let style = TextStyle {
            font_id: FontId(1),
            font_size: 12.0,
            ..TextStyle::default()
        };
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
        assert!(
            (glyph.origin.y - 12.0).abs() <= 0.001,
            "button label line box should be centered in the button"
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
            line_height: TextLineHeightPolicy::Absolute(12.0),
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
    fn label_node_text_layout_policy_drives_runtime_overflow() {
        let atlas_source = TestAtlasSource {
            atlas: atlas_with_ascii(FontId(1)),
        };
        let style = TextStyle {
            font_id: FontId(1),
            font_size: 12.0,
            line_height: TextLineHeightPolicy::Absolute(12.0),
            ..TextStyle::default()
        };
        let mut label = LabelNode::new("overflowing label", style);
        label.constraints = ui_layout::LayoutConstraints::tight(UiSize::new(28.0, 18.0));
        label.text_layout.overflow = TextOverflowPolicy::Ellipsis(TextEllipsisPlacement::End);

        let tree = UiTree::new(UiNode::new(WidgetId(242), UiNodeKind::Label(label)));
        let bounds = UiRect::new(0.0, 0.0, 28.0, 18.0);
        let layouts = compute_tree_layout(&tree, bounds, &UiRuntimeState::default());
        let frame = build_ui_frame(
            &tree,
            &layouts,
            bounds.size(),
            InteractionVisualState::default(),
            &atlas_source,
        );
        let run = frame.surfaces[0].layers[0]
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                UiPrimitive::GlyphRun(run) => Some(run),
                _ => None,
            })
            .expect("label should emit a glyph run");

        assert!(run.overflow_evidence.ellipsized);
        assert_eq!(
            run.overflow_evidence.ellipsis_placement,
            Some(TextEllipsisPlacement::End)
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
}
