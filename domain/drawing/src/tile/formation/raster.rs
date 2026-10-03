//! Private stroke reconstruction, dab generation, rasterization, and blending helpers.

use crate::{
    BrushDab, BrushDabStream, BrushDescriptor, CanvasCoordinate, CanvasRect, DynamicsCurve,
    StrokeReconstructedPath, StrokeReconstructionPolicy, StrokeRecord, StrokeSample,
    StrokeSampleTimeline,
};

use super::{DrawingInkTilePayload, DrawingTileFormationPolicy};
use super::spatial::rects_intersect;

pub(super) fn rasterize_stroke(
    payload: &mut DrawingInkTilePayload,
    tile_bounds: CanvasRect,
    stroke: &StrokeRecord,
    brush: &BrushDescriptor,
    policy: DrawingTileFormationPolicy,
) {
    let context = StrokeRasterContext {
        tile_bounds,
        stroke,
        policy,
    };
    let timeline = StrokeSampleTimeline::from_samples(stroke.samples.iter().copied());
    let reconstructed_path =
        StrokeReconstructedPath::from_timeline(&timeline, StrokeReconstructionPolicy::identity());
    let dab_stream = brush_dab_stream_for_path(&reconstructed_path, brush);
    for dab in dab_stream.dabs() {
        rasterize_dab(payload, context, *dab);
    }
}

#[derive(Clone, Copy)]
struct StrokeRasterContext<'a> {
    tile_bounds: CanvasRect,
    stroke: &'a StrokeRecord,
    policy: DrawingTileFormationPolicy,
}

fn brush_dab_stream_for_path(
    path: &StrokeReconstructedPath,
    brush: &BrushDescriptor,
) -> BrushDabStream {
    let mut dabs = Vec::new();
    match path.samples() {
        [] => {}
        [sample] => dabs.push(brush_dab_for_sample(brush, sample)),
        samples => {
            for (index, pair) in samples.windows(2).enumerate() {
                append_segment_dabs(&mut dabs, brush, &pair[0], &pair[1], index == 0);
            }
        }
    }
    BrushDabStream::new(dabs)
}

fn brush_dab_for_sample(brush: &BrushDescriptor, sample: &StrokeSample) -> BrushDab {
    BrushDab {
        center: sample.position,
        radius: f64::from(sample_size(brush, sample) * 0.5).max(0.5),
        opacity: sample_opacity(brush, sample),
        flow: sample_flow(brush, sample),
        edge_softness: brush.ink.edge_softness,
    }
}

fn append_segment_dabs(
    dabs: &mut Vec<BrushDab>,
    brush: &BrushDescriptor,
    start: &StrokeSample,
    end: &StrokeSample,
    include_start: bool,
) {
    let dx = end.position.x - start.position.x;
    let dy = end.position.y - start.position.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f64::EPSILON {
        if include_start {
            dabs.push(brush_dab_for_sample(brush, start));
        }
        return;
    }

    let spacing = dab_spacing(brush, start, end);
    let min_index = if include_start { 0 } else { 1 };
    let first_index = min_index;
    let last_index = (length / spacing).floor() as u32;
    let mut drew_end = false;
    for index in first_index..=last_index {
        let distance = (f64::from(index) * spacing).min(length);
        let t = (distance / length).clamp(0.0, 1.0);
        if (1.0 - t).abs() <= 0.000_001 {
            drew_end = true;
        }
        dabs.push(brush_dab_at_t(brush, start, end, t));
    }
    if !drew_end {
        dabs.push(brush_dab_at_t(brush, start, end, 1.0));
    }
}

fn brush_dab_at_t(
    brush: &BrushDescriptor,
    start: &StrokeSample,
    end: &StrokeSample,
    t: f64,
) -> BrushDab {
    let center = CanvasCoordinate::new(
        lerp_f64(start.position.x, end.position.x, t),
        lerp_f64(start.position.y, end.position.y, t),
    );
    let radius = lerp_f64(
        f64::from(sample_size(brush, start) * 0.5).max(0.5),
        f64::from(sample_size(brush, end) * 0.5).max(0.5),
        t,
    );
    let opacity = lerp_f32(sample_opacity(brush, start), sample_opacity(brush, end), t);
    let flow = lerp_f32(sample_flow(brush, start), sample_flow(brush, end), t);
    BrushDab {
        center,
        radius,
        opacity,
        flow,
        edge_softness: brush.ink.edge_softness,
    }
}

fn rasterize_dab(
    payload: &mut DrawingInkTilePayload,
    context: StrokeRasterContext<'_>,
    dab: BrushDab,
) {
    let radius = dab.radius.max(0.5);
    let min_x = dab.center.x - radius;
    let max_x = dab.center.x + radius;
    let min_y = dab.center.y - radius;
    let max_y = dab.center.y + radius;
    let dab_bounds = CanvasRect::new(
        CanvasCoordinate::new(min_x, min_y),
        CanvasCoordinate::new(max_x, max_y),
    );
    if !rects_intersect(dab_bounds, context.tile_bounds) {
        return;
    }

    let pixel_w = context.policy.tile_size_canvas_units / f64::from(payload.width);
    let pixel_h = context.policy.tile_size_canvas_units / f64::from(payload.height);
    let min_px = (((min_x - context.tile_bounds.min.x) / pixel_w).floor() as i64)
        .clamp(0, i64::from(payload.width.saturating_sub(1))) as u32;
    let max_px = (((max_x - context.tile_bounds.min.x) / pixel_w).floor() as i64)
        .clamp(0, i64::from(payload.width.saturating_sub(1))) as u32;
    let min_py = (((min_y - context.tile_bounds.min.y) / pixel_h).floor() as i64)
        .clamp(0, i64::from(payload.height.saturating_sub(1))) as u32;
    let max_py = (((max_y - context.tile_bounds.min.y) / pixel_h).floor() as i64)
        .clamp(0, i64::from(payload.height.saturating_sub(1))) as u32;

    let opacity = dab.opacity.clamp(0.0, 1.0)
        * dab.flow.clamp(0.0, 1.0)
        * context.stroke.color.a.clamp(0.0, 1.0);
    if opacity <= 0.0 {
        return;
    }
    let edge_softness = dab.edge_softness.clamp(0.0, 1.0);
    let fade_width = (radius * f64::from(edge_softness))
        .max(pixel_w.max(pixel_h) * 0.5)
        .min(radius);
    let hard_radius = (radius - fade_width).max(0.0);

    for py in min_py..=max_py {
        for px in min_px..=max_px {
            let pixel_center = CanvasCoordinate::new(
                context.tile_bounds.min.x + (f64::from(px) + 0.5) * pixel_w,
                context.tile_bounds.min.y + (f64::from(py) + 0.5) * pixel_h,
            );
            let distance = distance_between(pixel_center, dab.center);
            if distance > radius {
                continue;
            }
            let coverage = if distance <= hard_radius {
                1.0
            } else if fade_width <= f64::EPSILON {
                0.0
            } else {
                1.0 - ((distance - hard_radius) / fade_width).clamp(0.0, 1.0)
            } as f32;
            let alpha = (opacity * coverage).clamp(0.0, 1.0);
            blend_pixel(
                payload,
                px,
                py,
                context.stroke.color.r,
                context.stroke.color.g,
                context.stroke.color.b,
                alpha,
            );
        }
    }
}

fn sample_size(brush: &BrushDescriptor, sample: &StrokeSample) -> f32 {
    range_value(
        brush.ink.size.min,
        brush.ink.size.max,
        pressure_value(sample, brush.ink.dynamics.pressure_to_size),
    )
}

fn sample_opacity(brush: &BrushDescriptor, sample: &StrokeSample) -> f32 {
    range_value(
        brush.ink.opacity.min,
        brush.ink.opacity.max,
        pressure_value(sample, brush.ink.dynamics.pressure_to_opacity),
    )
}

fn sample_flow(brush: &BrushDescriptor, sample: &StrokeSample) -> f32 {
    range_value(
        brush.ink.flow.min,
        brush.ink.flow.max,
        pressure_value(sample, brush.ink.dynamics.pressure_to_opacity),
    )
}

fn pressure_value(sample: &StrokeSample, curve: DynamicsCurve) -> f32 {
    let pressure = sample.pressure.unwrap_or(1.0).clamp(0.0, 1.0);
    if !curve.enabled {
        return pressure;
    }
    curve.minimum_scale + (1.0 - curve.minimum_scale) * pressure.powf(curve.gamma)
}

fn range_value(min: f32, max: f32, t: f32) -> f32 {
    min + (max - min) * t.clamp(0.0, 1.0)
}

fn dab_spacing(brush: &BrushDescriptor, start: &StrokeSample, end: &StrokeSample) -> f64 {
    let start_size = f64::from(sample_size(brush, start)).max(1.0);
    let end_size = f64::from(sample_size(brush, end)).max(1.0);
    (start_size.min(end_size) * 0.25).clamp(0.75, 8.0)
}

fn lerp_f64(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t.clamp(0.0, 1.0)
}

fn lerp_f32(start: f32, end: f32, t: f64) -> f32 {
    start + (end - start) * t.clamp(0.0, 1.0) as f32
}

fn distance_between(a: CanvasCoordinate, b: CanvasCoordinate) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

fn blend_pixel(
    payload: &mut DrawingInkTilePayload,
    px: u32,
    py: u32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    let index = ((py as usize * payload.width as usize + px as usize) * 4)
        .min(payload.rgba8_premultiplied.len().saturating_sub(4));
    let src_a = a.clamp(0.0, 1.0);
    let src_r = r.clamp(0.0, 1.0) * src_a;
    let src_g = g.clamp(0.0, 1.0) * src_a;
    let src_b = b.clamp(0.0, 1.0) * src_a;

    let dst_r = f32::from(payload.rgba8_premultiplied[index]) / 255.0;
    let dst_g = f32::from(payload.rgba8_premultiplied[index + 1]) / 255.0;
    let dst_b = f32::from(payload.rgba8_premultiplied[index + 2]) / 255.0;
    let dst_a = f32::from(payload.rgba8_premultiplied[index + 3]) / 255.0;
    let inv = 1.0 - src_a;

    payload.rgba8_premultiplied[index] = to_u8(src_r + dst_r * inv);
    payload.rgba8_premultiplied[index + 1] = to_u8(src_g + dst_g * inv);
    payload.rgba8_premultiplied[index + 2] = to_u8(src_b + dst_b * inv);
    payload.rgba8_premultiplied[index + 3] = to_u8(src_a + dst_a * inv);
}

fn to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

