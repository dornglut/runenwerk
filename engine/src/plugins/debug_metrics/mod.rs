use std::sync::OnceLock;

use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::InputState;
use crate::plugins::render::features::{DEFAULT_EDITOR_FONT_ID, UiFontAtlasResource};
use crate::plugins::render::inspect::{RenderDebugTimingsState, WorldRuntimeInspectorSnapshot};
use crate::plugins::time::domain::Time;
use crate::runtime::{RenderPrepare, Res, ResMut, Startup};
use crate::state::{
    DebugMetricsState, SceneRuntimeState, StartupPhase, StartupState, UiOverlayState,
};
use ui_math::{UiInsets, UiRect, UiSize};
use ui_runtime::{
    ComputedLayout, ComputedLayoutMap, InteractionVisualState, LabelNode, PanelNode, UiNode,
    UiNodeKind, UiTree, WidgetId, build_ui_frame,
};
use ui_text::{FontId, TextAlign, TextOverflow, TextStyle, TextWrap};
use ui_theme::{ThemeTokens, UiColor};
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
        app.init_resource::<RenderDebugTimingsState>();
        app.add_systems(Startup, setup_debug_metrics_input_binding);
        app.add_systems(RenderPrepare, debug_metrics_overlay_system);
    }
}

fn setup_debug_metrics_input_binding(mut input: ResMut<InputState>) {
    input.map_key(ACTION_TOGGLE_METRICS, KeyCode::F10);
}

#[allow(clippy::too_many_arguments)]
fn debug_metrics_overlay_system(
    input: Res<InputState>,
    time: Res<Time>,
    startup: Res<StartupState>,
    scene: Res<SceneRuntimeState>,
    world_runtime: Res<WorldRuntimeInspectorSnapshot>,
    render_debug_timings: Res<RenderDebugTimingsState>,
    mut debug_metrics: ResMut<DebugMetricsState>,
    mut ui: ResMut<UiOverlayState>,
) {
    if input.action_pressed(ACTION_TOGGLE_METRICS) {
        debug_metrics.visible = !debug_metrics.visible;
    }

    debug_metrics.observe_frame_delta(time.delta_seconds);
    ui.debug_frame = ui_render_data::UiFrame::default();

    if !debug_metrics.visible {
        return;
    }

    let (screen_w, screen_h) = ui.screen_size;
    let scale = ui.scale.max(0.5);
    let x = 12.0 * scale;
    let y = 12.0 * scale;
    let w = (380.0 * scale).min((screen_w - x * 2.0).max(120.0));
    let h = 350.0 * scale;

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
            + t.renderer.preflight_ms
            + t.renderer.flow_encode_ms
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
        lines.push(format!(
            "pre={:.2} flow={:.2} diag={:.2}",
            t.renderer.preflight_ms,
            t.renderer.flow_encode_ms,
            render_debug_timings.diagnostics_report_ms
        ));
        lines.push(format!(
            "shader={} {:.2}ms",
            render_debug_timings
                .shader_reload_poll_status
                .as_deref()
                .unwrap_or("--"),
            render_debug_timings.shader_reload_poll_ms
        ));
    } else {
        lines.push("acq=-- ui=-- mesh=--".to_string());
        lines.push("world=-- enc=-- pres=--".to_string());
        lines.push("pre=-- flow=-- diag=--".to_string());
        lines.push("shader=-- --ms".to_string());
    }
    if let Some(workload) = workload_ms {
        lines.push(format!("workload={workload:.2}ms"));
    }
    lines.push(format!(
        "pace={} cap={} next={}",
        render_debug_timings
            .frame_pacing_mode
            .as_deref()
            .unwrap_or("--"),
        render_debug_timings
            .frame_pacing_target_fps
            .map(|value| value.to_string())
            .unwrap_or_else(|| "--".to_string()),
        render_debug_timings
            .frame_pacing_next_delay_ms
            .map(|value| format!("{value:.1}ms"))
            .unwrap_or_else(|| "--".to_string())
    ));
    lines.push(format!(
        "preflight={} source={}",
        render_debug_timings
            .preflight_cache_status
            .as_deref()
            .unwrap_or("--"),
        render_debug_timings
            .preflight_report_source
            .as_deref()
            .unwrap_or("--")
    ));
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
) -> ui_render_data::UiFrame {
    let panel_rect = UiRect::new(panel_x, panel_y, panel_w.max(0.0), panel_h.max(0.0));
    let text_padding = UiInsets::all(10.0 * scale);
    let content_bounds = panel_rect.inset(text_padding);

    let mut panel_theme = ThemeTokens::default().scaled_by(scale);
    panel_theme.background_panel = UiColor::new(0.03, 0.05, 0.08, 0.90);
    panel_theme.border = UiColor::new(0.24, 0.30, 0.40, 0.90);
    panel_theme.border_width = scale.max(0.5);
    panel_theme.radius.sm = 8.0 * scale;
    panel_theme.radius.md = 8.0 * scale;
    panel_theme.radius.lg = 8.0 * scale;

    let line_height = 16.0 * scale;
    let text_style = TextStyle {
        font_id: FontId(DEFAULT_EDITOR_FONT_ID.0),
        font_size: (12.0 * scale).max(1.0),
        color: [0.84, 0.92, 0.98, 1.0],
        line_height: Some(line_height.max(1.0)),
        align: TextAlign::Start,
        vertical_align: ui_text::TextVerticalAlign::LineBoxCenter,
        wrap: TextWrap::NoWrap,
        overflow: TextOverflow::Clip,
    };

    let mut children = Vec::with_capacity(lines.len());
    let mut layouts = ComputedLayoutMap::new();
    let mut next_id = 2_u64;

    for (index, line) in lines.iter().enumerate() {
        let id = WidgetId(next_id);
        next_id = next_id.saturating_add(1);
        children.push(UiNode::new(
            id,
            UiNodeKind::Label(LabelNode::new(line, text_style.clone())),
        ));

        let y = content_bounds.y + line_height * index as f32;
        let bounds = UiRect::new(
            content_bounds.x,
            y,
            content_bounds.width.max(0.0),
            line_height,
        );
        layouts.insert(id, ComputedLayout::new(bounds, bounds, bounds.size()));
    }

    let root_id = WidgetId(1);
    let panel_node = UiNode::with_children(
        root_id,
        UiNodeKind::Panel(PanelNode::new(panel_theme)),
        children,
    );
    layouts.insert(
        root_id,
        ComputedLayout::new(panel_rect, content_bounds, panel_rect.size()),
    );

    let tree = UiTree::new(panel_node);
    let surface_size = UiSize::new(screen_size.0.max(1.0), screen_size.1.max(1.0));
    build_ui_frame(
        &tree,
        &layouts,
        surface_size,
        InteractionVisualState::default(),
        debug_overlay_font_atlas(),
    )
}

fn debug_overlay_font_atlas() -> &'static UiFontAtlasResource {
    static ATLAS: OnceLock<UiFontAtlasResource> = OnceLock::new();
    ATLAS.get_or_init(UiFontAtlasResource::default)
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
