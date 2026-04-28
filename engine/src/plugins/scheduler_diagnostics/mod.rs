use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::diagnostics::core::ingest::{
    DiagnosticsEntrySubmission, submit_diagnostics_entry,
};
use crate::plugins::diagnostics::core::model::{
    DiagnosticsEntry, DiagnosticsSeverity, DiagnosticsStatus,
};
use crate::plugins::time::domain::Time;
use crate::runtime::{RenderSubmit, SimulationTick, WindowState, WorldMut};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SchedulerDiagnosticsPlugin;

impl Plugin for SchedulerDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(RenderSubmit, scheduler_diagnostics_system);
    }
}

static DIAGNOSTIC_FRAME_COUNTER: AtomicU64 = AtomicU64::new(0);
const LOG_INTERVAL_FRAMES: u64 = 120;
const SCHEDULER_PRODUCER_ID: &str = "scheduler.runtime";
const SCHEDULER_DOMAIN_ID: &str = "scheduler";
const SCHEDULER_SCHEMA_ID: &str = "runenwerk.scheduler.frame_snapshot";
const SCHEDULER_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone)]
struct SchedulerDiagnosticsSnapshot {
    simulation_tick: u64,
    dt_seconds: f32,
    window_title: String,
    window_size_px: (u32, u32),
    headless: bool,
}

#[derive(Debug, Serialize)]
struct SchedulerDiagnosticsEntryDto {
    frame: u64,
    simulation_tick: u64,
    dt_seconds: f32,
    window_title: String,
    window_size_px: [u32; 2],
    headless: bool,
}

fn scheduler_diagnostics_system(mut world: WorldMut) {
    let frame = DIAGNOSTIC_FRAME_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    if !frame.is_multiple_of(LOG_INTERVAL_FRAMES) {
        return;
    }

    let snapshot = {
        let Ok(time) = world.resource::<Time>() else {
            return;
        };
        let Ok(window) = world.resource::<WindowState>() else {
            return;
        };
        let simulation_tick = world
            .resource::<SimulationTick>()
            .map(|value| value.0)
            .unwrap_or_default();

        SchedulerDiagnosticsSnapshot {
            simulation_tick,
            dt_seconds: time.delta_seconds,
            window_title: window.title.clone(),
            window_size_px: window.size_px,
            headless: window.is_headless(),
        }
    };

    let submission = match map_scheduler_snapshot_to_entry(frame, &snapshot) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(
                frame,
                error = %error,
                "failed to map scheduler diagnostics snapshot into canonical diagnostics entry"
            );
            return;
        }
    };

    if let Err(error) = submit_diagnostics_entry(&mut world, submission) {
        tracing::warn!(
            frame,
            error = %error,
            "failed submitting scheduler diagnostics entry to canonical diagnostics core"
        );
    }
}

fn map_scheduler_snapshot_to_entry(
    frame: u64,
    snapshot: &SchedulerDiagnosticsSnapshot,
) -> anyhow::Result<DiagnosticsEntrySubmission> {
    let payload_json = serde_json::to_value(SchedulerDiagnosticsEntryDto {
        frame,
        simulation_tick: snapshot.simulation_tick,
        dt_seconds: snapshot.dt_seconds,
        window_title: snapshot.window_title.clone(),
        window_size_px: [snapshot.window_size_px.0, snapshot.window_size_px.1],
        headless: snapshot.headless,
    })?;

    Ok(DiagnosticsEntrySubmission {
        frame_index: frame,
        simulation_tick: snapshot.simulation_tick,
        entry: DiagnosticsEntry {
            entry_id: format!("scheduler.runtime.frame.{}", frame),
            producer_id: SCHEDULER_PRODUCER_ID.to_string(),
            domain_id: SCHEDULER_DOMAIN_ID.to_string(),
            schema_id: SCHEDULER_SCHEMA_ID.to_string(),
            schema_version: SCHEDULER_SCHEMA_VERSION,
            payload_json,
            severity: DiagnosticsSeverity::Info,
            status: DiagnosticsStatus::Ok,
            attachment_ids: Vec::new(),
        },
        attachments: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_mapping_uses_locked_schema_contract() {
        let snapshot = SchedulerDiagnosticsSnapshot {
            simulation_tick: 42,
            dt_seconds: 1.0 / 60.0,
            window_title: "Runenwerk".to_string(),
            window_size_px: (1280, 720),
            headless: false,
        };

        let submission = map_scheduler_snapshot_to_entry(120, &snapshot)
            .expect("scheduler mapping should succeed");

        assert_eq!(submission.entry.producer_id, "scheduler.runtime");
        assert_eq!(submission.entry.domain_id, "scheduler");
        assert_eq!(
            submission.entry.schema_id,
            "runenwerk.scheduler.frame_snapshot"
        );
        assert_eq!(submission.entry.schema_version, 1);
    }
}
