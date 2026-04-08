use super::*;
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
                            if let Some(entry) = flattened_rect(rect, current_clip(&clip_stack), None)
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
                            if let Some(entry) = flattened_image(image, current_clip(&clip_stack), None)
                            {
                                instances.push(entry);
                            }
                        }
                        UiPrimitive::GlyphRun(run) => {
                            flatten_glyph_run(run, current_clip(&clip_stack), &mut instances);
                        }
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
    instances: &mut Vec<FlattenedUiRectInstance>,
) {
    let local_clip = run.baseline_origin_clip;
    for glyph in &run.glyph_run.glyphs {
        let glyph_rect = UiRect::new(
            glyph.origin.x,
            glyph.origin.y - run.glyph_run.font_size,
            glyph.advance.max(1.0),
            run.glyph_run.font_size.max(1.0),
        );
        if let Some(clip) = effective_clip(stack_clip, local_clip, glyph_rect)
            && let Some(flattened) = flattened_rect_raw(
                glyph_rect,
                [run.tint.r, run.tint.g, run.tint.b, run.tint.a],
                0.0,
                clip,
            )
        {
            instances.push(flattened);
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
        [border.paint.r, border.paint.g, border.paint.b, border.paint.a],
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
