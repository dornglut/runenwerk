use super::*;
use crate::plugins::render::features::UiFontAtlasResource;
use ui_math::UiRect;
use ui_render_data::{
    BorderPrimitive, ClipPrimitive, GlyphRunPrimitive, ImagePrimitive, RectPrimitive, UiFrame,
    UiPrimitive,
};

impl Renderer {
    pub(super) fn extract_rect_instances(frame: &UiFrame) -> Vec<FlattenedUiRectInstance> {
        let mut instances = Vec::new();
        for surface in &frame.surfaces {
            for layer in &surface.layers {
                let mut clip_stack: Vec<Option<UiRect>> = Vec::new();
                for primitive in &layer.primitives {
                    match primitive {
                        UiPrimitive::Clip(ClipPrimitive::Push { rect, .. }) => {
                            let next = match clip_stack.last().copied() {
                                Some(Some(parent)) => parent.intersect(*rect),
                                Some(None) => None,
                                None => Some(*rect),
                            };
                            clip_stack.push(next);
                        }
                        UiPrimitive::Clip(ClipPrimitive::Pop { .. }) => {
                            let _ = clip_stack.pop();
                        }
                        UiPrimitive::Rect(rect) => {
                            if let Some(entry) =
                                flattened_rect(rect, current_clip(&clip_stack), None)
                            {
                                instances.push(entry);
                            }
                        }
                        UiPrimitive::Border(border) => {
                            if let Some(entry) =
                                flattened_border(border, current_clip(&clip_stack), None)
                            {
                                instances.push(entry);
                            }
                        }
                        UiPrimitive::Image(image) => {
                            if let Some(entry) =
                                flattened_image(image, current_clip(&clip_stack), None)
                            {
                                instances.push(entry);
                            }
                        }
                        UiPrimitive::GlyphRun(_) => {}
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
                let mut clip_stack: Vec<Option<UiRect>> = Vec::new();
                for primitive in &layer.primitives {
                    match primitive {
                        UiPrimitive::Clip(ClipPrimitive::Push { rect, .. }) => {
                            let next = match clip_stack.last().copied() {
                                Some(Some(parent)) => parent.intersect(*rect),
                                Some(None) => None,
                                None => Some(*rect),
                            };
                            clip_stack.push(next);
                        }
                        UiPrimitive::Clip(ClipPrimitive::Pop { .. }) => {
                            let _ = clip_stack.pop();
                        }
                        UiPrimitive::GlyphRun(run) => {
                            flatten_glyph_run(
                                run,
                                current_clip(&clip_stack),
                                atlas_resource,
                                &mut instances,
                            );
                        }
                        UiPrimitive::Rect(_) | UiPrimitive::Border(_) | UiPrimitive::Image(_) => {}
                    }
                }
            }
        }
        instances
    }
}

fn flatten_glyph_run(
    run: &GlyphRunPrimitive,
    stack_clip: Option<UiRect>,
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
            });
        }
    }
}

fn flattened_rect(
    rect: &RectPrimitive,
    stack_clip: Option<UiRect>,
    local_clip: Option<UiRect>,
) -> Option<FlattenedUiRectInstance> {
    let clip = effective_clip(stack_clip, local_clip, rect.rect)?;
    flattened_rect_raw(
        rect.rect,
        [rect.paint.r, rect.paint.g, rect.paint.b, rect.paint.a],
        rect.radius,
        clip,
    )
}

fn flattened_border(
    border: &BorderPrimitive,
    stack_clip: Option<UiRect>,
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
    )
}

fn flattened_image(
    image: &ImagePrimitive,
    stack_clip: Option<UiRect>,
    local_clip: Option<UiRect>,
) -> Option<FlattenedUiRectInstance> {
    let clip = effective_clip(stack_clip, local_clip, image.rect)?;
    flattened_rect_raw(
        image.rect,
        [image.tint.r, image.tint.g, image.tint.b, image.tint.a],
        0.0,
        clip,
    )
}

fn flattened_rect_raw(
    rect: UiRect,
    color: [f32; 4],
    radius: f32,
    clip: Option<UiRect>,
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
    })
}

fn current_clip(clip_stack: &[Option<UiRect>]) -> Option<UiRect> {
    clip_stack.last().copied().flatten()
}

fn effective_clip(
    stack_clip: Option<UiRect>,
    local_clip: Option<UiRect>,
    primitive_rect: UiRect,
) -> Option<Option<UiRect>> {
    let mut clip = stack_clip;

    if let Some(local) = local_clip {
        clip = match clip {
            Some(existing) => existing.intersect(local),
            None => Some(local),
        };
        if clip.is_none() {
            return None;
        }
    }

    if let Some(active_clip) = clip
        && active_clip.intersect(primitive_rect).is_none()
    {
        return None;
    }

    Some(clip)
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
        ClipPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiLayer, UiLayerId, UiPaint,
        UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
    };
    use ui_text::{AtlasTextLayouter, TextAlign, TextLayoutRequest, TextLayouter, TextOverflow, TextStyle, TextWrap};

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
    fn flatten_glyph_run_preserves_baseline_origin_and_quad_formula() {
        let atlas = UiFontAtlasResource::default();
        let style = TextStyle {
            font_id: DEFAULT_EDITOR_FONT_ID,
            font_size: 14.0,
            color: [1.0, 1.0, 1.0, 1.0],
            line_height: None,
            align: TextAlign::Start,
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
        flatten_glyph_run(&run, None, &atlas, &mut instances);
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
}
