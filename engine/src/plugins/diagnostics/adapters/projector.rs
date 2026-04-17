use crate::plugins::diagnostics::core::model::{DiagnosticsFrameReport, DiagnosticsStatus};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct DiagnosticsProjectedSummary {
    pub report_id: String,
    pub frame_index: u64,
    pub simulation_tick: u64,
    pub warning_count: usize,
    pub error_count: usize,
    pub total_entries: usize,
    pub primary_line: String,
    pub detail_lines: Vec<String>,
    pub fingerprint: u64,
}

pub fn project_report_summary(report: &DiagnosticsFrameReport) -> DiagnosticsProjectedSummary {
    let status_label = status_label(report.summary.status);
    let primary_line = format!(
        "[diagnostics] frame={} tick={} entries={} warnings={} errors={} status={}",
        report.frame_index,
        report.simulation_tick,
        report.summary.total_entries,
        report.warning_count,
        report.error_count,
        status_label,
    );

    let mut detail_lines = Vec::<String>::with_capacity(report.entries.len());
    for entry in &report.entries {
        detail_lines.push(format!(
            "[diagnostics.entry] producer={} domain={} schema={}@{} severity={:?} status={:?} attachments={}",
            entry.producer_id,
            entry.domain_id,
            entry.schema_id,
            entry.schema_version,
            entry.severity,
            entry.status,
            entry.attachment_ids.len()
        ));
    }

    let fingerprint = fingerprint_summary(primary_line.as_str(), &detail_lines);

    DiagnosticsProjectedSummary {
        report_id: report.report_id.clone(),
        frame_index: report.frame_index,
        simulation_tick: report.simulation_tick,
        warning_count: report.warning_count,
        error_count: report.error_count,
        total_entries: report.summary.total_entries,
        primary_line,
        detail_lines,
        fingerprint,
    }
}

pub fn should_emit_projected_summary(
    projected: &DiagnosticsProjectedSummary,
    last_info_frame_index: Option<u64>,
    last_fingerprint: Option<u64>,
    info_interval_frames: u64,
    dedupe_enabled: bool,
) -> bool {
    if projected.error_count > 0 || projected.warning_count > 0 {
        return true;
    }

    let fingerprint_changed = match last_fingerprint {
        Some(previous) => previous != projected.fingerprint,
        None => true,
    };

    if !dedupe_enabled || fingerprint_changed {
        return true;
    }

    match last_info_frame_index {
        Some(previous_frame) => {
            projected.frame_index.saturating_sub(previous_frame) >= info_interval_frames.max(1)
        }
        None => true,
    }
}

fn status_label(status: DiagnosticsStatus) -> &'static str {
    match status {
        DiagnosticsStatus::Ok => "ok",
        DiagnosticsStatus::Degraded => "degraded",
        DiagnosticsStatus::Failed => "failed",
        DiagnosticsStatus::Skipped => "skipped",
    }
}

fn fingerprint_summary(primary_line: &str, detail_lines: &[String]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    primary_line.hash(&mut hasher);
    for detail in detail_lines {
        detail.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::diagnostics::core::model::{
        DiagnosticsAttachment, DiagnosticsEntry, DiagnosticsFrameReport, DiagnosticsSeverity,
        DiagnosticsStatus,
    };

    #[test]
    fn projector_emits_when_warning_or_error_exists() {
        let mut report = DiagnosticsFrameReport::new_pending(10, 20);
        report.append_entry(
            DiagnosticsEntry {
                entry_id: "entry.1".to_string(),
                producer_id: "render.inspect".to_string(),
                domain_id: "render".to_string(),
                schema_id: "schema".to_string(),
                schema_version: 1,
                payload_json: serde_json::json!({"ok": true}),
                severity: DiagnosticsSeverity::Warning,
                status: DiagnosticsStatus::Degraded,
                attachment_ids: vec!["att.1".to_string()],
            },
            vec![DiagnosticsAttachment {
                attachment_id: "att.1".to_string(),
                kind: "file".to_string(),
                label: "manifest".to_string(),
                producer_id: "render.inspect".to_string(),
                domain_id: "render".to_string(),
                path_or_handle: "target/file.json".to_string(),
                metadata: None,
            }],
        );
        report.assign_identity(1);

        let projected = project_report_summary(&report);
        assert!(should_emit_projected_summary(
            &projected,
            Some(projected.frame_index),
            Some(projected.fingerprint),
            120,
            true,
        ));
    }
}
