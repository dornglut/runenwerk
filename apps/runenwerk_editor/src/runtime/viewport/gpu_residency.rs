//! File: apps/runenwerk_editor/src/runtime/viewport/gpu_residency.rs
//! Purpose: App-owned viewport GPU residency diagnostics bridge.

use engine::plugins::render::{RenderGpuResidencyResource, RenderGpuResidencySummary};
use engine::runtime::{Res, ResMut};

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::resources::EditorHostResource;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EditorViewportGpuResidencySummary {
    pub resident_count: usize,
    pub allocated_count: usize,
    pub preserved_count: usize,
    pub invalidated_count: usize,
    pub evicted_count: usize,
    pub rejected_count: usize,
    pub diagnostic_count: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EditorViewportGpuResidencyJournalEntry {
    pub summary: EditorViewportGpuResidencySummary,
}

pub fn summarize_viewport_gpu_residency_system(
    mut host: ResMut<EditorHostResource>,
    residency: Res<RenderGpuResidencyResource>,
) {
    record_viewport_gpu_residency_summary(&mut host.app, residency.last_summary());
}

pub fn record_viewport_gpu_residency_summary(
    app: &mut RunenwerkEditorApp,
    summary: RenderGpuResidencySummary,
) -> bool {
    let summary = EditorViewportGpuResidencySummary {
        resident_count: summary.resident_count,
        allocated_count: summary.allocated_count,
        preserved_count: summary.preserved_count,
        invalidated_count: summary.invalidated_count,
        evicted_count: summary.evicted_count,
        rejected_count: summary.rejected_count,
        diagnostic_count: summary.diagnostic_count,
    };

    if !app.update_viewport_gpu_residency_summary(summary) {
        return false;
    }

    app.record_viewport_gpu_residency(EditorViewportGpuResidencyJournalEntry { summary });
    app.append_console_line(format!(
        "[gpu_residency] resident={} allocated={} preserved={} invalidated={} evicted={} rejected={} diagnostics={}",
        summary.resident_count,
        summary.allocated_count,
        summary.preserved_count,
        summary.invalidated_count,
        summary.evicted_count,
        summary.rejected_count,
        summary.diagnostic_count
    ));
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_residency_summary_logs_only_when_changed() {
        let mut app = RunenwerkEditorApp::new();
        let summary = RenderGpuResidencySummary {
            resident_count: 1,
            allocated_count: 1,
            ..RenderGpuResidencySummary::default()
        };

        assert!(record_viewport_gpu_residency_summary(&mut app, summary));
        assert!(!record_viewport_gpu_residency_summary(&mut app, summary));

        assert_eq!(app.viewport_gpu_residency_journal().len(), 1);
        assert_eq!(
            app.console_lines()
                .iter()
                .filter(|line| line.text.contains("[gpu_residency]"))
                .count(),
            1
        );
    }
}
