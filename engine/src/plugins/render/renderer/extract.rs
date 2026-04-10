use super::*;
use crate::plugins::render::features::UiFontAtlasResource;
use ui_math::UiRect;
use ui_render_data::{
    BorderPrimitive, ClipPrimitive, GlyphRunPrimitive, ImagePrimitive, RectPrimitive, UiFrame,
    UiPrimitive, UiSortKey,
};

impl Renderer {
    pub(super) fn extract_rect_instances(frame: &UiFrame) -> Vec<FlattenedUiRectInstance> {
        let mut instances = Vec::new();
        for surface in &frame.surfaces {
            for layer in &surface.layers {
                let mut ordered = layer.primitives.iter().enumerate().collect::<Vec<_>>();
                ordered.sort_by_key(|(index, primitive)| (primitive_sort_key(primitive), *index));

                let mut clip_stack: Vec<Option<UiRect>> = Vec::new();
                for (_, primitive) in ordered {
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
                let mut ordered = layer.primitives.iter().enumerate().collect::<Vec<_>>();
                ordered.sort_by_key(|(index, primitive)| (primitive_sort_key(primitive), *index));

                let mut clip_stack: Vec<Option<UiRect>> = Vec::new();
                for (_, primitive) in ordered {
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

        if let Some(clip) = effective_clip(stack_clip, local_clip, glyph_rect) {
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

fn primitive_sort_key(primitive: &UiPrimitive) -> UiSortKey {
    match primitive {
        UiPrimitive::Rect(value) => value.sort_key,
        UiPrimitive::Border(value) => value.sort_key,
        UiPrimitive::GlyphRun(value) => value.sort_key,
        UiPrimitive::Image(value) => value.sort_key,
        UiPrimitive::Clip(ClipPrimitive::Push { sort_key, .. }) => *sort_key,
        UiPrimitive::Clip(ClipPrimitive::Pop { sort_key }) => *sort_key,
    }
}
