use super::*;
use crate::plugins::render::features::UiFontAtlasResource;
use ui_math::UiRect;
use ui_render_data::{
    BorderPrimitive, ClipPrimitive, GlyphRunPrimitive, ImagePrimitive, ProductSurfacePrimitive,
    RectPrimitive, StrokePrimitive, UiFrame, UiPrimitive, ViewportSurfaceEmbedPrimitive,
};

impl Renderer {
    pub(super) fn extract_rect_instances(frame: &UiFrame) -> Vec<FlattenedUiRectInstance> {
        let mut instances = Vec::new();
        for surface in &frame.surfaces {
            for layer in &surface.layers {
                let mut clip_stack: Vec<ClipRegion> = Vec::new();
                for primitive in &layer.primitives {
                    match primitive {
                        UiPrimitive::Clip(ClipPrimitive::Push { rect, .. }) => {
                            let next = clip_stack
                                .last()
                                .copied()
                                .unwrap_or(ClipRegion::Unbounded)
                                .intersect(*rect);
                            clip_stack.push(next);
                        }
                        UiPrimitive::Clip(ClipPrimitive::Pop { .. }) => {
                            let _ = clip_stack.pop();
                        }
                        UiPrimitive::Rect(rect) => {
                            if let Some(entry) =
                                flattened_rect(rect, current_clip_region(&clip_stack), None)
                            {
                                instances.push(entry);
                            }
                        }
                        UiPrimitive::Border(border) => {
                            if let Some(entry) =
                                flattened_border(border, current_clip_region(&clip_stack), None)
                            {
                                instances.push(entry);
                            }
                        }
                        UiPrimitive::Image(image) => {
                            if let Some(entry) =
                                flattened_image(image, current_clip_region(&clip_stack), None)
                            {
                                instances.push(entry);
                            }
                        }
                        UiPrimitive::GlyphRun(_)
                        | UiPrimitive::Stroke(_)
                        | UiPrimitive::ProductSurface(_) => {}
                        UiPrimitive::ViewportSurfaceEmbed(_) => {}
                    }
                }
            }
        }
        instances
    }

    pub(super) fn extract_stroke_instances(
        frame: &UiFrame,
    ) -> Vec<FlattenedUiStrokeSegmentInstance> {
        let mut instances = Vec::new();
        for surface in &frame.surfaces {
            for layer in &surface.layers {
                let mut clip_stack: Vec<ClipRegion> = Vec::new();
                for primitive in &layer.primitives {
                    match primitive {
                        UiPrimitive::Clip(ClipPrimitive::Push { rect, .. }) => {
                            let next = clip_stack
                                .last()
                                .copied()
                                .unwrap_or(ClipRegion::Unbounded)
                                .intersect(*rect);
                            clip_stack.push(next);
                        }
                        UiPrimitive::Clip(ClipPrimitive::Pop { .. }) => {
                            let _ = clip_stack.pop();
                        }
                        UiPrimitive::Stroke(stroke) => {
                            flatten_stroke(
                                stroke,
                                current_clip_region(&clip_stack),
                                &mut instances,
                            );
                        }
                        UiPrimitive::Rect(_)
                        | UiPrimitive::Border(_)
                        | UiPrimitive::Image(_)
                        | UiPrimitive::GlyphRun(_)
                        | UiPrimitive::ProductSurface(_)
                        | UiPrimitive::ViewportSurfaceEmbed(_) => {}
                    }
                }
            }
        }
        instances
    }

    pub(super) fn extract_glyph_instances(
        frame: &UiFrame,
        atlas_resource: &UiFontAtlasResource,
    ) -> Vec<FlattenedUiGlyphInstance> {
        let mut instances = Vec::new();
        for surface in &frame.surfaces {
            for layer in &surface.layers {
                let mut clip_stack: Vec<ClipRegion> = Vec::new();
                for primitive in &layer.primitives {
                    match primitive {
                        UiPrimitive::Clip(ClipPrimitive::Push { rect, .. }) => {
                            let next = clip_stack
                                .last()
                                .copied()
                                .unwrap_or(ClipRegion::Unbounded)
                                .intersect(*rect);
                            clip_stack.push(next);
                        }
                        UiPrimitive::Clip(ClipPrimitive::Pop { .. }) => {
                            let _ = clip_stack.pop();
                        }
                        UiPrimitive::GlyphRun(run) => {
                            flatten_glyph_run(
                                run,
                                current_clip_region(&clip_stack),
                                atlas_resource,
                                &mut instances,
                            );
                        }
                        UiPrimitive::Rect(_)
                        | UiPrimitive::Border(_)
                        | UiPrimitive::Image(_)
                        | UiPrimitive::Stroke(_)
                        | UiPrimitive::ProductSurface(_)
                        | UiPrimitive::ViewportSurfaceEmbed(_) => {}
                    }
                }
            }
        }
        instances
    }

    pub(super) fn extract_viewport_embed_instances(
        frame: &UiFrame,
    ) -> Vec<FlattenedUiViewportEmbedInstance> {
        let mut instances = Vec::new();
        for surface in &frame.surfaces {
            for layer in &surface.layers {
                let mut clip_stack: Vec<ClipRegion> = Vec::new();
                for primitive in &layer.primitives {
                    match primitive {
                        UiPrimitive::Clip(ClipPrimitive::Push { rect, .. }) => {
                            let next = clip_stack
                                .last()
                                .copied()
                                .unwrap_or(ClipRegion::Unbounded)
                                .intersect(*rect);
                            clip_stack.push(next);
                        }
                        UiPrimitive::Clip(ClipPrimitive::Pop { .. }) => {
                            let _ = clip_stack.pop();
                        }
                        UiPrimitive::ViewportSurfaceEmbed(embed) => {
                            if let Some(entry) = flattened_viewport_embed(
                                embed,
                                current_clip_region(&clip_stack),
                                None,
                            ) {
                                instances.push(entry);
                            }
                        }
                        UiPrimitive::Rect(_)
                        | UiPrimitive::Border(_)
                        | UiPrimitive::Image(_)
                        | UiPrimitive::Stroke(_)
                        | UiPrimitive::ProductSurface(_)
                        | UiPrimitive::GlyphRun(_) => {}
                    }
                }
            }
        }
        instances
    }

    pub(super) fn extract_product_surface_instances(
        frame: &UiFrame,
    ) -> Vec<FlattenedUiProductSurfaceInstance> {
        let mut instances = Vec::new();
        for surface in &frame.surfaces {
            for layer in &surface.layers {
                let mut clip_stack: Vec<ClipRegion> = Vec::new();
                for primitive in &layer.primitives {
                    match primitive {
                        UiPrimitive::Clip(ClipPrimitive::Push { rect, .. }) => {
                            let next = clip_stack
                                .last()
                                .copied()
                                .unwrap_or(ClipRegion::Unbounded)
                                .intersect(*rect);
                            clip_stack.push(next);
                        }
                        UiPrimitive::Clip(ClipPrimitive::Pop { .. }) => {
                            let _ = clip_stack.pop();
                        }
                        UiPrimitive::ProductSurface(surface) => {
                            if let Some(entry) = flattened_product_surface(
                                surface,
                                current_clip_region(&clip_stack),
                                None,
                            ) {
                                instances.push(entry);
                            }
                        }
                        UiPrimitive::Rect(_)
                        | UiPrimitive::Border(_)
                        | UiPrimitive::Image(_)
                        | UiPrimitive::Stroke(_)
                        | UiPrimitive::ViewportSurfaceEmbed(_)
                        | UiPrimitive::GlyphRun(_) => {}
                    }
                }
            }
        }
        instances
    }
}

fn flatten_stroke(
    stroke: &StrokePrimitive,
    stack_clip: ClipRegion,
    instances: &mut Vec<FlattenedUiStrokeSegmentInstance>,
) {
    if stroke.width <= f32::EPSILON || stroke.paint.a <= f32::EPSILON || stroke.points.is_empty() {
        return;
    }

    if stroke.points.len() == 1 {
        push_stroke_segment(
            stroke,
            stack_clip,
            stroke.points[0],
            stroke.points[0],
            instances,
        );
        return;
    }

    for pair in stroke.points.windows(2) {
        push_stroke_segment(stroke, stack_clip, pair[0], pair[1], instances);
    }
}

fn push_stroke_segment(
    stroke: &StrokePrimitive,
    stack_clip: ClipRegion,
    start: ui_math::UiPoint,
    end: ui_math::UiPoint,
    instances: &mut Vec<FlattenedUiStrokeSegmentInstance>,
) {
    let bounds = stroke_segment_bounds(start, end, stroke.width);
    let Some(clip) = effective_clip(stack_clip, stroke.clip, bounds) else {
        return;
    };
    instances.push(FlattenedUiStrokeSegmentInstance {
        raw: StrokeSegmentInstanceRaw {
            start: [start.x, start.y],
            end: [end.x, end.y],
            color: [
                stroke.paint.r,
                stroke.paint.g,
                stroke.paint.b,
                stroke.paint.a,
            ],
            width: stroke.width,
            _pad: [0.0; 3],
        },
        clip: clip.map(|value| [value.x, value.y, value.width, value.height]),
        layer_order: stroke.sort_key.layer_order,
        primitive_order: stroke.sort_key.primitive_order,
    });
}

fn stroke_segment_bounds(start: ui_math::UiPoint, end: ui_math::UiPoint, width: f32) -> UiRect {
    let half = (width * 0.5).max(0.5);
    UiRect::new(
        start.x.min(end.x) - half,
        start.y.min(end.y) - half,
        (start.x - end.x).abs() + width.max(1.0),
        (start.y - end.y).abs() + width.max(1.0),
    )
}

fn flatten_glyph_run(
    run: &GlyphRunPrimitive,
    stack_clip: ClipRegion,
    atlas_resource: &UiFontAtlasResource,
    instances: &mut Vec<FlattenedUiGlyphInstance>,
) {
    let texture_id = run.draw_key.texture_id.unwrap_or(run.glyph_run.font_id.0);
    let Some((atlas, _atlas_image)) = atlas_resource.atlas_for_texture_id(texture_id) else {
        return;
    };
    let base_size = atlas.metrics.base_size.max(f32::EPSILON);
    let scale = run.glyph_run.font_size / base_size;
    let local_clip = run.baseline_origin_clip;

    for glyph in &run.glyph_run.glyphs {
        let Some(metrics) = atlas
            .glyphs
            .get(&glyph.ch)
            .or_else(|| atlas.glyphs.get(&'?'))
        else {
            continue;
        };

        let width = (metrics.plane_right - metrics.plane_left).abs().max(1.0) * scale;
        let height = (metrics.plane_top - metrics.plane_bottom).abs().max(1.0) * scale;
        let glyph_rect = UiRect::new(
            glyph.origin.x + metrics.plane_left * scale,
            glyph.origin.y - metrics.plane_top * scale,
            width,
            height,
        );
        let clip = effective_clip(stack_clip, local_clip, glyph_rect);
        if glyph_diagnostics_enabled() && glyph_is_diagnostic_target(glyph.ch) {
            eprintln!(
                "[glyph] ch='{}' origin=({:.3},{:.3}) plane=({:.3},{:.3},{:.3},{:.3}) rect=({:.3},{:.3},{:.3},{:.3}) clip={:?}",
                glyph.ch,
                glyph.origin.x,
                glyph.origin.y,
                metrics.plane_left,
                metrics.plane_top,
                metrics.plane_right,
                metrics.plane_bottom,
                glyph_rect.x,
                glyph_rect.y,
                glyph_rect.width,
                glyph_rect.height,
                clip,
            );
        }

        if let Some(clip) = clip {
            instances.push(FlattenedUiGlyphInstance {
                raw: GlyphInstanceRaw {
                    rect: [
                        glyph_rect.x,
                        glyph_rect.y,
                        glyph_rect.width,
                        glyph_rect.height,
                    ],
                    uv_rect: [
                        metrics.atlas_left,
                        metrics.atlas_top,
                        (metrics.atlas_right - metrics.atlas_left).max(0.0),
                        (metrics.atlas_bottom - metrics.atlas_top).max(0.0),
                    ],
                    color: [run.tint.r, run.tint.g, run.tint.b, run.tint.a],
                },
                clip: clip
                    .map(|clip_rect| [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height]),
                texture_id,
                layer_order: run.sort_key.layer_order,
                primitive_order: run.sort_key.primitive_order,
            });
        }
    }
}

fn flattened_rect(
    rect: &RectPrimitive,
    stack_clip: ClipRegion,
    local_clip: Option<UiRect>,
) -> Option<FlattenedUiRectInstance> {
    let clip = effective_clip(stack_clip, local_clip, rect.rect)?;
    flattened_rect_raw(
        rect.rect,
        [rect.paint.r, rect.paint.g, rect.paint.b, rect.paint.a],
        rect.radius,
        clip,
        rect.sort_key.layer_order,
        rect.sort_key.primitive_order,
    )
}

fn flattened_border(
    border: &BorderPrimitive,
    stack_clip: ClipRegion,
    local_clip: Option<UiRect>,
) -> Option<FlattenedUiRectInstance> {
    let clip = effective_clip(stack_clip, local_clip, border.rect)?;
    flattened_rect_raw(
        border.rect,
        [
            border.paint.r,
            border.paint.g,
            border.paint.b,
            border.paint.a,
        ],
        border.radius,
        clip,
        border.sort_key.layer_order,
        border.sort_key.primitive_order,
    )
}

fn flattened_image(
    image: &ImagePrimitive,
    stack_clip: ClipRegion,
    local_clip: Option<UiRect>,
) -> Option<FlattenedUiRectInstance> {
    let clip = effective_clip(stack_clip, local_clip, image.rect)?;
    flattened_rect_raw(
        image.rect,
        [image.tint.r, image.tint.g, image.tint.b, image.tint.a],
        0.0,
        clip,
        image.sort_key.layer_order,
        image.sort_key.primitive_order,
    )
}

fn flattened_viewport_embed(
    embed: &ViewportSurfaceEmbedPrimitive,
    stack_clip: ClipRegion,
    local_clip: Option<UiRect>,
) -> Option<FlattenedUiViewportEmbedInstance> {
    let clip = effective_clip(stack_clip, local_clip, embed.rect)?;
    if embed.rect.width <= f32::EPSILON
        || embed.rect.height <= f32::EPSILON
        || embed.tint.a <= f32::EPSILON
    {
        return None;
    }

    Some(FlattenedUiViewportEmbedInstance {
        raw: ViewportEmbedInstanceRaw {
            rect: [
                embed.rect.x,
                embed.rect.y,
                embed.rect.width,
                embed.rect.height,
            ],
            uv_rect: [
                embed.uv_rect.x,
                embed.uv_rect.y,
                embed.uv_rect.width,
                embed.uv_rect.height,
            ],
            tint: [embed.tint.r, embed.tint.g, embed.tint.b, embed.tint.a],
        },
        clip: clip.map(|value| [value.x, value.y, value.width, value.height]),
        viewport_id: embed.viewport_id,
        slot: embed.slot,
        layer_order: embed.sort_key.layer_order,
        primitive_order: embed.sort_key.primitive_order,
    })
}

fn flattened_product_surface(
    surface: &ProductSurfacePrimitive,
    stack_clip: ClipRegion,
    local_clip: Option<UiRect>,
) -> Option<FlattenedUiProductSurfaceInstance> {
    let clip = effective_clip(stack_clip, local_clip, surface.rect)?;
    if surface.rect.width <= f32::EPSILON
        || surface.rect.height <= f32::EPSILON
        || surface.tint.a <= f32::EPSILON
    {
        return None;
    }

    Some(FlattenedUiProductSurfaceInstance {
        raw: ViewportEmbedInstanceRaw {
            rect: [
                surface.rect.x,
                surface.rect.y,
                surface.rect.width,
                surface.rect.height,
            ],
            uv_rect: [
                surface.uv_rect.x,
                surface.uv_rect.y,
                surface.uv_rect.width,
                surface.uv_rect.height,
            ],
            tint: [
                surface.tint.r,
                surface.tint.g,
                surface.tint.b,
                surface.tint.a,
            ],
        },
        clip: clip.map(|value| [value.x, value.y, value.width, value.height]),
        source: surface.source.clone(),
        layer_order: surface.sort_key.layer_order,
        primitive_order: surface.sort_key.primitive_order,
    })
}

fn flattened_rect_raw(
    rect: UiRect,
    color: [f32; 4],
    radius: f32,
    clip: Option<UiRect>,
    layer_order: u32,
    primitive_order: u32,
) -> Option<FlattenedUiRectInstance> {
    if rect.width <= f32::EPSILON || rect.height <= f32::EPSILON || color[3] <= f32::EPSILON {
        return None;
    }

    Some(FlattenedUiRectInstance {
        raw: RectInstanceRaw {
            rect: [rect.x, rect.y, rect.width, rect.height],
            color,
            radius,
            _pad: [0.0; 3],
        },
        clip: clip.map(|clip| [clip.x, clip.y, clip.width, clip.height]),
        layer_order,
        primitive_order,
    })
}

fn current_clip_region(clip_stack: &[ClipRegion]) -> ClipRegion {
    clip_stack.last().copied().unwrap_or(ClipRegion::Unbounded)
}

fn effective_clip(
    stack_clip: ClipRegion,
    local_clip: Option<UiRect>,
    primitive_rect: UiRect,
) -> Option<Option<UiRect>> {
    let mut clip = stack_clip;

    clip = match (clip, local_clip) {
        (ClipRegion::Empty, _) => ClipRegion::Empty,
        (ClipRegion::Unbounded, Some(local)) => ClipRegion::Rect(local),
        (ClipRegion::Unbounded, None) => ClipRegion::Unbounded,
        (ClipRegion::Rect(existing), Some(local)) => match existing.intersect(local) {
            Some(intersection) => ClipRegion::Rect(intersection),
            None => ClipRegion::Empty,
        },
        (ClipRegion::Rect(existing), None) => ClipRegion::Rect(existing),
    };

    match clip {
        ClipRegion::Empty => None,
        ClipRegion::Unbounded => Some(None),
        ClipRegion::Rect(active_clip) => {
            if active_clip.intersect(primitive_rect).is_none() {
                None
            } else {
                Some(Some(active_clip))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClipRegion {
    Unbounded,
    Rect(UiRect),
    Empty,
}

impl ClipRegion {
    fn intersect(self, rect: UiRect) -> Self {
        match self {
            Self::Unbounded => Self::Rect(rect),
            Self::Rect(existing) => match existing.intersect(rect) {
                Some(intersection) => Self::Rect(intersection),
                None => Self::Empty,
            },
            Self::Empty => Self::Empty,
        }
    }
}

fn glyph_diagnostics_enabled() -> bool {
    std::env::var("RUNENWERK_EDITOR_DEBUG_GLYPH_TRACE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn glyph_is_diagnostic_target(ch: char) -> bool {
    matches!(ch, 'N' | 'o' | 'n' | 'g' | 'e' | 'p' | 'y')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::features::{DEFAULT_EDITOR_FONT_ID, UiFontAtlasResource};
    use ui_math::UiSize;
    use ui_render_data::{
        ClipPrimitive, GlyphRunPrimitive, RectPrimitive, StrokePrimitive, UiDrawKey, UiLayer,
        UiLayerId, UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
    };
    use ui_text::{
        AtlasTextLayouter, TextAlign, TextLayoutRequest, TextLayouter, TextOverflow, TextStyle,
        TextWrap,
    };

    #[test]
    fn extract_rect_instances_preserves_clip_stack_emission_order() {
        let parent_clip = UiRect::new(0.0, 0.0, 100.0, 100.0);
        let child_clip = UiRect::new(10.0, 10.0, 20.0, 20.0);
        let rect = UiRect::new(10.0, 10.0, 20.0, 20.0);
        let layer = UiLayer::with_primitives(
            UiLayerId(0),
            vec![
                UiPrimitive::Clip(ClipPrimitive::Push {
                    rect: parent_clip,
                    sort_key: UiSortKey::new(0, 0, 0),
                }),
                UiPrimitive::Clip(ClipPrimitive::Push {
                    rect: child_clip,
                    sort_key: UiSortKey::new(0, 1, 1),
                }),
                UiPrimitive::Rect(RectPrimitive::new(
                    rect,
                    0.0,
                    UiPaint::rgba(1.0, 1.0, 1.0, 1.0),
                    UiDrawKey::new(0, None),
                    UiSortKey::new(0, 2, 2),
                )),
                UiPrimitive::Clip(ClipPrimitive::Pop {
                    sort_key: UiSortKey::new(0, 1, 3),
                }),
                UiPrimitive::Clip(ClipPrimitive::Pop {
                    sort_key: UiSortKey::new(0, 0, 4),
                }),
            ],
        );
        let frame = UiFrame::with_surfaces(vec![UiSurface::with_layers(
            UiSurfaceId(0),
            UiSize::new(200.0, 200.0),
            vec![layer],
        )]);

        let instances = Renderer::extract_rect_instances(&frame);
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].clip, Some([10.0, 10.0, 20.0, 20.0]));
    }

    #[test]
    fn extract_rect_instances_culls_primitives_when_nested_clip_intersection_is_empty() {
        let layer = UiLayer::with_primitives(
            UiLayerId(0),
            vec![
                UiPrimitive::Clip(ClipPrimitive::Push {
                    rect: UiRect::new(0.0, 0.0, 20.0, 20.0),
                    sort_key: UiSortKey::new(0, 0, 0),
                }),
                UiPrimitive::Clip(ClipPrimitive::Push {
                    rect: UiRect::new(100.0, 100.0, 20.0, 20.0),
                    sort_key: UiSortKey::new(0, 1, 1),
                }),
                UiPrimitive::Rect(RectPrimitive::new(
                    UiRect::new(0.0, 0.0, 200.0, 200.0),
                    0.0,
                    UiPaint::rgba(1.0, 0.2, 0.2, 1.0),
                    UiDrawKey::new(0, None),
                    UiSortKey::new(0, 2, 2),
                )),
                UiPrimitive::Clip(ClipPrimitive::Pop {
                    sort_key: UiSortKey::new(0, 1, 3),
                }),
                UiPrimitive::Clip(ClipPrimitive::Pop {
                    sort_key: UiSortKey::new(0, 0, 4),
                }),
            ],
        );
        let frame = UiFrame::with_surfaces(vec![UiSurface::with_layers(
            UiSurfaceId(0),
            UiSize::new(256.0, 256.0),
            vec![layer],
        )]);

        let instances = Renderer::extract_rect_instances(&frame);
        assert!(
            instances.is_empty(),
            "rect should be culled when nested clip intersection is empty",
        );
    }

    #[test]
    fn extract_stroke_instances_preserves_order_clip_and_width() {
        let clip = UiRect::new(0.0, 0.0, 80.0, 80.0);
        let stroke = StrokePrimitive::new(
            [
                ui_math::UiPoint::new(10.0, 10.0),
                ui_math::UiPoint::new(20.0, 12.0),
                ui_math::UiPoint::new(28.0, 20.0),
            ],
            6.0,
            UiPaint::rgba(0.1, 0.2, 0.3, 0.8),
            UiDrawKey::new(1, None),
            UiSortKey::new(0, 7, 13),
        )
        .with_clip(clip);
        let layer = UiLayer::with_primitives(UiLayerId(0), vec![UiPrimitive::Stroke(stroke)]);
        let frame = UiFrame::with_surfaces(vec![UiSurface::with_layers(
            UiSurfaceId(0),
            UiSize::new(200.0, 200.0),
            vec![layer],
        )]);

        let instances = Renderer::extract_stroke_instances(&frame);
        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0].raw.width, 6.0);
        assert_eq!(instances[0].raw.start, [10.0, 10.0]);
        assert_eq!(instances[0].raw.end, [20.0, 12.0]);
        assert_eq!(instances[1].raw.start, [20.0, 12.0]);
        assert_eq!(instances[1].raw.end, [28.0, 20.0]);
        assert_eq!(instances[0].clip, Some([0.0, 0.0, 80.0, 80.0]));
        assert_eq!(instances[0].layer_order, 7);
        assert_eq!(instances[0].primitive_order, 13);
    }

    #[test]
    fn flatten_glyph_run_preserves_baseline_origin_and_quad_formula() {
        let atlas = UiFontAtlasResource::default();
        let style = TextStyle {
            font_id: DEFAULT_EDITOR_FONT_ID,
            font_size: 14.0,
            color: [1.0, 1.0, 1.0, 1.0],
            line_height: None,
            align: TextAlign::Start,
            vertical_align: ui_text::TextVerticalAlign::LineBoxCenter,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        };
        let glyph_run = AtlasTextLayouter
            .layout(
                &atlas,
                TextLayoutRequest {
                    text: "Nonge",
                    style: &style,
                    max_width: None,
                },
            )
            .expect("expected atlas text layout");
        let baseline = glyph_run.glyphs[0].origin.y;
        assert!(
            glyph_run
                .glyphs
                .iter()
                .all(|glyph| (glyph.origin.y - baseline).abs() <= f32::EPSILON),
            "all glyph origins in run should share one baseline",
        );

        let run = GlyphRunPrimitive::new(
            glyph_run.clone(),
            Some(UiRect::new(0.0, 0.0, 400.0, 64.0)),
            UiPaint::rgba(1.0, 1.0, 1.0, 1.0),
            UiDrawKey::new(0, Some(DEFAULT_EDITOR_FONT_ID.0)),
            UiSortKey::new(0, 0, 0),
        );
        let mut instances = Vec::new();
        flatten_glyph_run(&run, ClipRegion::Unbounded, &atlas, &mut instances);
        assert_eq!(instances.len(), glyph_run.glyphs.len());

        let (atlas_metrics, _) = atlas
            .atlas_for_texture_id(DEFAULT_EDITOR_FONT_ID.0)
            .expect("default atlas should be available");
        let scale = glyph_run.font_size / atlas_metrics.metrics.base_size.max(f32::EPSILON);
        for (glyph, instance) in glyph_run.glyphs.iter().zip(instances.iter()) {
            let metrics = atlas_metrics
                .glyphs
                .get(&glyph.ch)
                .or_else(|| atlas_metrics.glyphs.get(&'?'))
                .expect("metrics should exist for glyph");
            let expected_top = glyph.origin.y - metrics.plane_top * scale;
            assert!(
                (instance.raw.rect[1] - expected_top).abs() <= 0.001,
                "glyph '{}' quad top must match baseline conversion",
                glyph.ch,
            );
            assert!(
                instance.clip.is_some(),
                "glyph '{}' should keep final clip in flattened output",
                glyph.ch,
            );
        }
    }

    #[test]
    fn flatten_glyph_run_places_descenders_below_baseline_for_representative_word() {
        let atlas = UiFontAtlasResource::default();
        let style = TextStyle {
            font_id: DEFAULT_EDITOR_FONT_ID,
            font_size: 14.0,
            color: [1.0, 1.0, 1.0, 1.0],
            line_height: None,
            align: TextAlign::Start,
            vertical_align: ui_text::TextVerticalAlign::LineBoxCenter,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        };
        let glyph_run = AtlasTextLayouter
            .layout(
                &atlas,
                TextLayoutRequest {
                    text: "Nongepy",
                    style: &style,
                    max_width: None,
                },
            )
            .expect("expected atlas text layout");
        let baseline = glyph_run
            .glyphs
            .first()
            .map(|glyph| glyph.origin.y)
            .expect("representative run should contain glyphs");

        let run = GlyphRunPrimitive::new(
            glyph_run.clone(),
            Some(UiRect::new(0.0, 0.0, 480.0, 96.0)),
            UiPaint::rgba(1.0, 1.0, 1.0, 1.0),
            UiDrawKey::new(0, Some(DEFAULT_EDITOR_FONT_ID.0)),
            UiSortKey::new(0, 0, 0),
        );
        let mut instances = Vec::new();
        flatten_glyph_run(&run, ClipRegion::Unbounded, &atlas, &mut instances);
        assert_eq!(instances.len(), glyph_run.glyphs.len());

        let mut by_char = std::collections::HashMap::new();
        for (glyph, instance) in glyph_run.glyphs.iter().zip(instances.iter()) {
            by_char.entry(glyph.ch).or_insert(instance);
        }

        for ch in ['g', 'p', 'y'] {
            let instance = by_char
                .get(&ch)
                .unwrap_or_else(|| panic!("flattened run should contain '{ch}'"));
            let bottom = instance.raw.rect[1] + instance.raw.rect[3];
            assert!(
                bottom > baseline + 0.5,
                "descender glyph '{ch}' should render below baseline; bottom={bottom} baseline={baseline}",
            );
        }

        for ch in ['N', 'o', 'n', 'e'] {
            let instance = by_char
                .get(&ch)
                .unwrap_or_else(|| panic!("flattened run should contain '{ch}'"));
            let bottom = instance.raw.rect[1] + instance.raw.rect[3];
            assert!(
                bottom < baseline + 0.5,
                "non-descender glyph '{ch}' should stay near baseline; bottom={bottom} baseline={baseline}",
            );
        }

        let top_n = by_char
            .get(&'N')
            .expect("flattened run should contain 'N'")
            .raw
            .rect[1];
        let top_g = by_char
            .get(&'g')
            .expect("flattened run should contain 'g'")
            .raw
            .rect[1];
        assert!(
            top_n + 0.5 < top_g,
            "cap-height glyph 'N' should render above descender glyph 'g'; top_n={top_n} top_g={top_g}",
        );
    }
}
