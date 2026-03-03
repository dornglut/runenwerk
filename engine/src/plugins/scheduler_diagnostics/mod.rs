use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::time::domain::Time;
use crate::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
use crate::runtime_v2::{RenderSubmit, Res, WindowState};
use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SchedulerDiagnosticsPlugin;

impl Plugin for SchedulerDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(RenderSubmit, typed_scheduler_diagnostics_system);
    }
}

impl EnginePlugin for SchedulerDiagnosticsPlugin {
    fn name(&self) -> &'static str {
        "scheduler_diagnostics"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "scheduler_diagnostics",
            scheduler_diagnostics_system,
            &["frame_render_submit"],
        );
        Ok(())
    }
}

static DIAGNOSTIC_FRAME_COUNTER: AtomicU64 = AtomicU64::new(0);
const LOG_INTERVAL_FRAMES: u64 = 120;

pub fn scheduler_diagnostics_system(data: &mut EngineData) -> anyhow::Result<()> {
    let frame = DIAGNOSTIC_FRAME_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    if frame % LOG_INTERVAL_FRAMES != 0 {
        return Ok(());
    }

    tracing::info!(
        frame,
        dt = data.time.delta_seconds,
        world_scene = data.scene.world.active.label(),
        overlay_scene = data.scene.active_overlay().label(),
        overlay_visible = data.scene.overlay_visible(),
        world_paused = data.scene.world.paused,
        "scheduler diagnostics"
    );
    Ok(())
}

fn typed_scheduler_diagnostics_system(time: Res<Time>, window: Res<WindowState>) {
    let frame = DIAGNOSTIC_FRAME_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    if frame % LOG_INTERVAL_FRAMES != 0 {
        return;
    }

    tracing::info!(
        frame,
        dt = time.delta_seconds,
        window_title = %window.title,
        window_size = ?window.size_px,
        headless = window.is_headless(),
        "typed scheduler diagnostics"
    );
}

#[cfg(test)]
mod tests {
    use super::SchedulerDiagnosticsPlugin;
    use crate::runtime::{EnginePlugin, EngineScheduleBuilder};

    #[test]
    fn scheduler_diagnostics_plugin_requires_render_submit_node() {
        let mut builder = EngineScheduleBuilder::new();
        SchedulerDiagnosticsPlugin
            .configure(&mut builder)
            .expect("scheduler diagnostics plugin should configure");
        assert!(builder.build_scheduler().is_err());
    }
}
