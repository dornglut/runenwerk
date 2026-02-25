use crate::plugins::ui::domain::UiDrawCmd;
use crate::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder, StartupPhase};
use anyhow::Result;
use winit::keyboard::KeyCode;

pub struct DebugMetricsPlugin;

const ACTION_TOGGLE_METRICS: &str = "debug.metrics.toggle";

impl EnginePlugin for DebugMetricsPlugin {
    fn name(&self) -> &'static str {
        "debug_metrics"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "debug_metrics_overlay",
            debug_metrics_overlay_system,
            &["overlay_ui_render_extract"],
        );
        builder.add_edge("debug_metrics_overlay", "frame_render_prepare");
        Ok(())
    }

    fn setup(&self, data: &mut EngineData) -> Result<()> {
        data.input.map_key(ACTION_TOGGLE_METRICS, KeyCode::F10);
        Ok(())
    }
}

pub fn debug_metrics_overlay_system(data: &mut EngineData) -> anyhow::Result<()> {
    if data.input.action_pressed(ACTION_TOGGLE_METRICS) {
        data.debug_metrics.visible = !data.debug_metrics.visible;
    }

    data.debug_metrics
        .observe_frame_delta(data.time.delta_seconds);

    if !data.debug_metrics.visible {
        return Ok(());
    }

    let world_scene_label = data.scene.world.active.label().to_string();
    let overlay_scene_label = data.scene.active_overlay().label().to_string();
    let ui = &mut data.scene.overlay_runtime.ui;
    let (screen_w, _screen_h) = ui.screen_size;
    let scale = ui.scale.max(0.5);
    let x = 12.0 * scale;
    let y = 12.0 * scale;
    let w = (380.0 * scale).min((screen_w - x * 2.0).max(120.0));
    let h = 170.0 * scale;
    let clip = Some([x, y, w, h]);

    ui.draw_list.commands.push(UiDrawCmd::Rect {
        x,
        y,
        w,
        h,
        color: [0.03, 0.05, 0.08, 0.9],
        radius: 8.0 * scale,
    });

    let phase = match data.startup.phase {
        StartupPhase::Loading => "loading",
        StartupPhase::Ready => "ready",
    };
    let fps = data.debug_metrics.fps_ema;
    let frame_ms = data.debug_metrics.frame_ms_ema;
    let timings = data.debug_metrics.last_timings;
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
        phase, data.startup.stable_frames, data.startup.required_stable_frames
    ));
    lines.push(format!(
        "scene={} overlay={}",
        world_scene_label, overlay_scene_label
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

    let line_h = 16.0 * scale;
    for (idx, line) in lines.into_iter().enumerate() {
        ui.draw_list.commands.push(UiDrawCmd::Text {
            x: x + 10.0 * scale,
            y: y + 10.0 * scale + line_h * idx as f32,
            content: line,
            color: [0.84, 0.92, 0.98, 1.0],
            size: 12.0 * scale,
            clip,
        });
    }

    Ok(())
}
