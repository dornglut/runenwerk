use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::InputState;
use crate::plugins::time::domain::Time;
use crate::runtime::{RenderPrepare, Res, ResMut, Startup};
use crate::state::{
    DebugMetricsState, OverlayDrawCmd, SceneRuntimeState, StartupPhase, StartupState,
    UiOverlayState,
};
use winit::keyboard::KeyCode;

pub struct DebugMetricsPlugin;

const ACTION_TOGGLE_METRICS: &str = "debug.metrics.toggle";

impl Plugin for DebugMetricsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugMetricsState>();
        app.init_resource::<StartupState>();
        app.init_resource::<SceneRuntimeState>();
        app.init_resource::<UiOverlayState>();
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
    mut debug_metrics: ResMut<DebugMetricsState>,
    mut ui: ResMut<UiOverlayState>,
) {
    if input.action_pressed(ACTION_TOGGLE_METRICS) {
        debug_metrics.visible = !debug_metrics.visible;
    }

    debug_metrics.observe_frame_delta(time.delta_seconds);
    ui.debug_draw_list.commands.clear();

    if !debug_metrics.visible {
        return;
    }

    let (screen_w, _screen_h) = ui.screen_size;
    let scale = ui.scale.max(0.5);
    let x = 12.0 * scale;
    let y = 12.0 * scale;
    let w = (380.0 * scale).min((screen_w - x * 2.0).max(120.0));
    let h = 170.0 * scale;
    let clip = Some([x, y, w, h]);

    ui.debug_draw_list.commands.push(OverlayDrawCmd::Rect {
        x,
        y,
        w,
        h,
        color: [0.03, 0.05, 0.08, 0.9],
        radius: 8.0 * scale,
    });

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

    let line_h = 16.0 * scale;
    for (idx, line) in lines.into_iter().enumerate() {
        ui.debug_draw_list.commands.push(OverlayDrawCmd::Text {
            x: x + 10.0 * scale,
            y: y + 10.0 * scale + line_h * idx as f32,
            content: line,
            color: [0.84, 0.92, 0.98, 1.0],
            size: 12.0 * scale,
            clip,
        });
    }
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
        assert!(!overlay.debug_draw_list.commands.is_empty());
    }
}
