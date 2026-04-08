use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::InputState;
use crate::plugins::render::inspect::WorldRuntimeInspectorSnapshot;
use crate::plugins::time::domain::Time;
use crate::runtime::{RenderPrepare, Res, ResMut, Startup};
use crate::state::{
    DebugMetricsState, SceneRuntimeState, StartupPhase, StartupState, UiOverlayState,
};
use ui_math::{UiPoint, UiRect, UiSize};
use ui_render_data::{
    ClipPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId,
    UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};
use ui_text::{FontId, GlyphRun, PositionedGlyph};
use winit::keyboard::KeyCode;

pub struct DebugMetricsPlugin;

const ACTION_TOGGLE_METRICS: &str = "debug.metrics.toggle";

impl Plugin for DebugMetricsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugMetricsState>();
        app.init_resource::<StartupState>();
        app.init_resource::<SceneRuntimeState>();
        app.init_resource::<UiOverlayState>();
        app.init_resource::<WorldRuntimeInspectorSnapshot>();
        app.add_systems(Startup, setup_debug_metrics_input_binding);
        app.add_systems(RenderPrepare, debug_metrics_overlay_system);
    }
}

fn setup_debug_metrics_input_binding(mut input: ResMut<InputState>) {
    input.map_key(ACTION_TOGGLE_METRICS, KeyCode::F10);
}

fn debug_metrics_overlay_system(
    input: Res<InputState>,
    time: Res<Time>,
    startup: Res<StartupState>,
    scene: Res<SceneRuntimeState>,
    world_runtime: Res<WorldRuntimeInspectorSnapshot>,
    mut debug_metrics: ResMut<DebugMetricsState>,
    mut ui: ResMut<UiOverlayState>,
) {
    if input.action_pressed(ACTION_TOGGLE_METRICS) {
        debug_metrics.visible = !debug_metrics.visible;
    }

    debug_metrics.observe_frame_delta(time.delta_seconds);
    ui.debug_frame = UiFrame::default();

    if !debug_metrics.visible {
        return;
    }

    let (screen_w, screen_h) = ui.screen_size;
    let scale = ui.scale.max(0.5);
    let x = 12.0 * scale;
    let y = 12.0 * scale;
    let w = (380.0 * scale).min((screen_w - x * 2.0).max(120.0));
    let h = 282.0 * scale;

    let phase = match startup.phase {
        StartupPhase::Loading => "loading",
        StartupPhase::Ready => "ready",
    };
    let fps = debug_metrics.fps_ema;
    let frame_ms = debug_metrics.frame_ms_ema;
    let timings = debug_metrics.last_timings;
    let workload_ms = timings.map(|t| {
        t.renderer.prepare_ui_ms
            + t.renderer.prepare_mesh_ms
            + t.renderer.world_prepare_ms
            + t.renderer.encode_submit_ms
    });

    let mut lines = Vec::new();
    lines.push("Diagnostics (F10)".to_string());
    lines.push(format!("fps={fps:>6.1} frame={frame_ms:>6.2}ms"));
    lines.push(format!(
        "startup={} stable={}/{}",
        phase, startup.stable_frames, startup.required_stable_frames
    ));
    lines.push(format!(
        "scene={} overlay={}",
        scene.world_scene_label, scene.overlay_scene_label
    ));
    lines.push(format!(
        "overlay_visible={} world_paused={}",
        scene.overlay_visible, scene.world_paused
    ));

    if let Some(t) = timings {
        lines.push(format!(
            "acq={:.2} ui={:.2} mesh={:.2}",
            t.acquire_ms, t.renderer.prepare_ui_ms, t.renderer.prepare_mesh_ms
        ));
        lines.push(format!(
            "world={:.2} enc={:.2} pres={:.2}",
            t.renderer.world_prepare_ms, t.renderer.encode_submit_ms, t.present_ms
        ));
    } else {
        lines.push("acq=-- ui=-- mesh=--".to_string());
        lines.push("world=-- enc=-- pres=--".to_string());
    }
    if let Some(workload) = workload_ms {
        lines.push(format!("workload={workload:.2}ms"));
    }
    lines.push(format!(
        "world rev={} dirty={} q={}/{}",
        world_runtime.world_revision,
        world_runtime.chunk_dirty_count,
        world_runtime.queued_interactive,
        world_runtime.queued_background
    ));
    lines.push(format!(
        "build int={} drop={} oplog={}",
        world_runtime.integrated_build_outputs,
        world_runtime.dropped_stale_outputs,
        world_runtime.op_log_count
    ));
    lines.push(format!(
        "ingress={} invalid={}",
        world_runtime.ingress_operations, world_runtime.invalidated_chunks
    ));
    lines.push(format!(
        "collision q={} miss={}",
        world_runtime.collision_queries, world_runtime.collision_authority_misses
    ));
    lines.push(format!(
        "stream conn={} resync={} lag={} rlag={}",
        world_runtime.streaming_connection_count,
        world_runtime.streaming_needs_resync_count,
        world_runtime.streaming_max_cursor_lag,
        world_runtime.streaming_max_region_sequence_lag
    ));
    lines.push(format!(
        "region seq={} records={}",
        world_runtime.region_journal_latest_sequence, world_runtime.region_journal_record_count
    ));

    ui.debug_frame = build_debug_metrics_frame((screen_w, screen_h), x, y, w, h, scale, &lines);
}

fn build_debug_metrics_frame(
    screen_size: (f32, f32),
    panel_x: f32,
    panel_y: f32,
    panel_w: f32,
    panel_h: f32,
    scale: f32,
    lines: &[String],
) -> UiFrame {
    let mut layer = UiLayer::new(UiLayerId(0));
    let mut primitive_order = 0u32;
    let panel_rect = UiRect::new(panel_x, panel_y, panel_w, panel_h);

    push_rect(
        &mut layer,
        &mut primitive_order,
        panel_rect,
        [0.03, 0.05, 0.08, 0.9],
        8.0 * scale,
    );
    layer.push(UiPrimitive::Clip(ClipPrimitive::Push {
        rect: panel_rect,
        sort_key: next_sort_key(&mut primitive_order),
    }));

    let line_h = 16.0 * scale;
    for (idx, line) in lines.iter().enumerate() {
        let text_x = panel_x + 10.0 * scale;
        let text_y = panel_y + 10.0 * scale + line_h * idx as f32;
        let glyph_run = estimate_glyph_run(line, text_x, text_y, 12.0 * scale);
        layer.push(UiPrimitive::GlyphRun(GlyphRunPrimitive::new(
            glyph_run,
            Some(panel_rect),
            UiPaint::rgba(0.84, 0.92, 0.98, 1.0),
            default_draw_key(),
            next_sort_key(&mut primitive_order),
        )));
    }

    layer.push(UiPrimitive::Clip(ClipPrimitive::Pop {
        sort_key: next_sort_key(&mut primitive_order),
    }));

    let surface_size = UiSize::new(screen_size.0.max(1.0), screen_size.1.max(1.0));
    UiFrame::with_surfaces(vec![UiSurface::with_layers(
        UiSurfaceId(0),
        surface_size,
        vec![layer],
    )])
}

fn push_rect(
    layer: &mut UiLayer,
    primitive_order: &mut u32,
    rect: UiRect,
    color: [f32; 4],
    radius: f32,
) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        rect,
        radius.max(0.0),
        UiPaint::rgba(color[0], color[1], color[2], color[3]),
        default_draw_key(),
        next_sort_key(primitive_order),
    )));
}

fn estimate_glyph_run(text: &str, x: f32, y: f32, font_size: f32) -> GlyphRun {
    let advance = font_size.max(1.0) * 0.55;
    let baseline_y = y + font_size.max(1.0);
    let glyphs = text
        .chars()
        .enumerate()
        .map(|(index, ch)| PositionedGlyph {
            ch,
            origin: UiPoint::new(x + advance * index as f32, baseline_y),
            advance,
        })
        .collect::<Vec<_>>();

    GlyphRun {
        font_id: FontId(0),
        font_size,
        size: UiSize::new(advance * text.chars().count() as f32, font_size * 1.25),
        glyphs,
    }
}

fn default_draw_key() -> UiDrawKey {
    UiDrawKey::new(0, None)
}

fn next_sort_key(primitive_order: &mut u32) -> UiSortKey {
    let key = UiSortKey::new(0, 0, *primitive_order);
    *primitive_order = primitive_order.saturating_add(1);
    key
}

#[cfg(test)]
mod tests {
    use super::DebugMetricsPlugin;
    use crate::plugins::InputState;
    use crate::prelude::*;
    use winit::event::ElementState;
    use winit::keyboard::KeyCode;

    fn inject_f10(mut input: ResMut<InputState>) {
        input.handle_keyboard_input(KeyCode::F10, ElementState::Pressed, None);
    }

    #[test]
    fn debug_metrics_plugin_populates_overlay_draw_state() {
        let mut app = App::headless();
        app.add_plugin(DebugMetricsPlugin);
        app.add_systems(Update, inject_f10);
        let app = app.run_for_frames(1).expect("debug metrics should run");

        let metrics = app.world().resource::<DebugMetricsState>().unwrap();
        assert!(metrics.visible);
        let overlay = app.world().resource::<UiOverlayState>().unwrap();
        assert!(!overlay.debug_frame.is_empty());
    }
}
