use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiagnosticsSeverity {
    #[default]
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiagnosticsStatus {
    #[default]
    Ok,
    Degraded,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsAttachment {
    pub attachment_id: String,
    pub kind: String,
    pub label: String,
    pub producer_id: String,
    pub domain_id: String,
    pub path_or_handle: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsEntry {
    pub entry_id: String,
    pub producer_id: String,
    pub domain_id: String,
    pub schema_id: String,
    pub schema_version: u32,
    pub payload_json: Value,
    pub severity: DiagnosticsSeverity,
    pub status: DiagnosticsStatus,
    pub attachment_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticsSummary {
    pub total_entries: usize,
    pub info_entries: usize,
    pub warning_entries: usize,
    pub error_entries: usize,
    pub status: DiagnosticsStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsFrameReport {
    pub report_id: String,
    pub report_sequence: u64,
    pub frame_index: u64,
    pub simulation_tick: u64,
    pub collected_at_unix_ms: u128,
    pub entries: Vec<DiagnosticsEntry>,
    pub summary: DiagnosticsSummary,
    pub attachments: Vec<DiagnosticsAttachment>,
    pub warning_count: usize,
    pub error_count: usize,
}

impl DiagnosticsFrameReport {
    pub fn new_pending(frame_index: u64, simulation_tick: u64) -> Self {
        Self {
            report_id: String::new(),
            report_sequence: 0,
            frame_index,
            simulation_tick,
            collected_at_unix_ms: unix_ms_now(),
            entries: Vec::new(),
            summary: DiagnosticsSummary::default(),
            attachments: Vec::new(),
            warning_count: 0,
            error_count: 0,
        }
    }

    pub fn append_entry(
        &mut self,
        entry: DiagnosticsEntry,
        mut attachments: Vec<DiagnosticsAttachment>,
    ) {
        self.entries.push(entry);
        self.attachments.append(&mut attachments);
        self.normalize_attachment_index();
        self.recompute_summary();
    }

    pub fn assign_identity(&mut self, report_sequence: u64) {
        self.report_sequence = report_sequence;
        self.report_id = format!("diagnostics.frame.{}.{}", self.frame_index, report_sequence);
        self.collected_at_unix_ms = unix_ms_now();
    }

    pub fn recompute_summary(&mut self) {
        let mut summary = DiagnosticsSummary {
            total_entries: self.entries.len(),
            ..DiagnosticsSummary::default()
        };
        for entry in &self.entries {
            match entry.severity {
                DiagnosticsSeverity::Info => {
                    summary.info_entries = summary.info_entries.saturating_add(1)
                }
                DiagnosticsSeverity::Warning => {
                    summary.warning_entries = summary.warning_entries.saturating_add(1)
                }
                DiagnosticsSeverity::Error => {
                    summary.error_entries = summary.error_entries.saturating_add(1)
                }
            }
        }

        summary.status = if summary.error_entries > 0 {
            DiagnosticsStatus::Failed
        } else if summary.warning_entries > 0 {
            DiagnosticsStatus::Degraded
        } else {
            DiagnosticsStatus::Ok
        };

        self.warning_count = summary.warning_entries;
        self.error_count = summary.error_entries;
        self.summary = summary;
    }

    fn normalize_attachment_index(&mut self) {
        let mut seen = BTreeSet::<String>::new();
        self.attachments.retain(|attachment| {
            if seen.contains(&attachment.attachment_id) {
                false
            } else {
                seen.insert(attachment.attachment_id.clone());
                true
            }
        });
    }
}

fn unix_ms_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_summary_is_recomputed_from_entry_severity() {
        let mut report = DiagnosticsFrameReport::new_pending(12, 34);
        report.append_entry(
            DiagnosticsEntry {
                entry_id: "entry.1".to_string(),
                producer_id: "render.inspect".to_string(),
                domain_id: "render".to_string(),
                schema_id: "runenwerk.render.frame".to_string(),
                schema_version: 1,
                payload_json: serde_json::json!({"k": "v"}),
                severity: DiagnosticsSeverity::Warning,
                status: DiagnosticsStatus::Degraded,
                attachment_ids: Vec::new(),
            },
            Vec::new(),
        );

        assert_eq!(report.summary.total_entries, 1);
        assert_eq!(report.summary.warning_entries, 1);
        assert_eq!(report.warning_count, 1);
        assert_eq!(report.error_count, 0);
        assert_eq!(report.summary.status, DiagnosticsStatus::Degraded);
    }

    #[test]
    fn report_keeps_unique_attachments_by_attachment_id() {
        let mut report = DiagnosticsFrameReport::new_pending(1, 1);
        let attachment = DiagnosticsAttachment {
            attachment_id: "att.1".to_string(),
            kind: "file".to_string(),
            label: "artifact".to_string(),
            producer_id: "render.inspect".to_string(),
            domain_id: "render".to_string(),
            path_or_handle: "target/file.json".to_string(),
            metadata: None,
        };

        report.append_entry(
            DiagnosticsEntry {
                entry_id: "entry.1".to_string(),
                producer_id: "render.inspect".to_string(),
                domain_id: "render".to_string(),
                schema_id: "schema".to_string(),
                schema_version: 1,
                payload_json: serde_json::json!({"a": 1}),
                severity: DiagnosticsSeverity::Info,
                status: DiagnosticsStatus::Ok,
                attachment_ids: vec!["att.1".to_string()],
            },
            vec![attachment.clone(), attachment],
        );

        assert_eq!(report.attachments.len(), 1);
        assert_eq!(report.attachments[0].attachment_id, "att.1");
    }
}
