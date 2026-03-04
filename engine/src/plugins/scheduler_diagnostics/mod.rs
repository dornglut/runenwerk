use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::time::domain::Time;
use crate::runtime::{RenderSubmit, Res, WindowState};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SchedulerDiagnosticsPlugin;

impl Plugin for SchedulerDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(RenderSubmit, scheduler_diagnostics_system);
    }
}

static DIAGNOSTIC_FRAME_COUNTER: AtomicU64 = AtomicU64::new(0);
const LOG_INTERVAL_FRAMES: u64 = 120;

fn scheduler_diagnostics_system(time: Res<Time>, window: Res<WindowState>) {
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
        "scheduler diagnostics"
    );
}
