use super::model::{DiagnosticsAttachment, DiagnosticsEntry, DiagnosticsFrameReport};
use anyhow::anyhow;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsValidationIssue {
    pub code: &'static str,
    pub message: String,
}

pub fn validate_entry_contract(
    entry: &DiagnosticsEntry,
) -> Result<(), Vec<DiagnosticsValidationIssue>> {
    let mut issues = Vec::<DiagnosticsValidationIssue>::new();

    require_non_empty(&mut issues, "entry.entry_id", entry.entry_id.as_str());
    require_non_empty(&mut issues, "entry.producer_id", entry.producer_id.as_str());
    require_non_empty(&mut issues, "entry.domain_id", entry.domain_id.as_str());
    require_non_empty(&mut issues, "entry.schema_id", entry.schema_id.as_str());

    if entry.schema_version == 0 {
        issues.push(DiagnosticsValidationIssue {
            code: "entry.schema_version.zero",
            message: "schema_version must be > 0".to_string(),
        });
    }

    if !entry.payload_json.is_object() {
        issues.push(DiagnosticsValidationIssue {
            code: "entry.payload_json.not_object",
            message: "payload_json must be an object serialized from a typed DTO".to_string(),
        });
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

pub fn validate_attachment_contract(
    attachment: &DiagnosticsAttachment,
) -> Result<(), Vec<DiagnosticsValidationIssue>> {
    let mut issues = Vec::<DiagnosticsValidationIssue>::new();

    require_non_empty(
        &mut issues,
        "attachment.attachment_id",
        attachment.attachment_id.as_str(),
    );
    require_non_empty(&mut issues, "attachment.kind", attachment.kind.as_str());
    require_non_empty(&mut issues, "attachment.label", attachment.label.as_str());
    require_non_empty(
        &mut issues,
        "attachment.producer_id",
        attachment.producer_id.as_str(),
    );
    require_non_empty(
        &mut issues,
        "attachment.domain_id",
        attachment.domain_id.as_str(),
    );
    require_non_empty(
        &mut issues,
        "attachment.path_or_handle",
        attachment.path_or_handle.as_str(),
    );

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

pub fn validate_submission_contract(
    entry: &DiagnosticsEntry,
    attachments: &[DiagnosticsAttachment],
) -> anyhow::Result<()> {
    let mut issues = Vec::<DiagnosticsValidationIssue>::new();

    if let Err(entry_issues) = validate_entry_contract(entry) {
        issues.extend(entry_issues);
    }

    let mut seen_attachment_ids = BTreeSet::<String>::new();
    for attachment in attachments {
        if let Err(attachment_issues) = validate_attachment_contract(attachment) {
            issues.extend(attachment_issues);
        }
        if !seen_attachment_ids.insert(attachment.attachment_id.clone()) {
            issues.push(DiagnosticsValidationIssue {
                code: "attachment.attachment_id.duplicate",
                message: format!(
                    "duplicate attachment_id '{}' inside single submission",
                    attachment.attachment_id
                ),
            });
        }
    }

    for attachment_id in &entry.attachment_ids {
        if !seen_attachment_ids.contains(attachment_id) {
            issues.push(DiagnosticsValidationIssue {
                code: "entry.attachment_id.missing",
                message: format!(
                    "entry attachment_id '{}' is not present in submission attachments",
                    attachment_id
                ),
            });
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "diagnostics submission contract validation failed: {}",
            format_issues(&issues)
        ))
    }
}

pub fn validate_report_contract(
    report: &DiagnosticsFrameReport,
) -> Result<(), Vec<DiagnosticsValidationIssue>> {
    let mut issues = Vec::<DiagnosticsValidationIssue>::new();

    require_non_empty(&mut issues, "report.report_id", report.report_id.as_str());
    if report.report_sequence == 0 {
        issues.push(DiagnosticsValidationIssue {
            code: "report.report_sequence.zero",
            message: "report_sequence must be assigned before persistence".to_string(),
        });
    }

    let mut attachment_ids = BTreeSet::<String>::new();
    for attachment in &report.attachments {
        if let Err(attachment_issues) = validate_attachment_contract(attachment) {
            issues.extend(attachment_issues);
        }
        if !attachment_ids.insert(attachment.attachment_id.clone()) {
            issues.push(DiagnosticsValidationIssue {
                code: "report.attachment_id.duplicate",
                message: format!(
                    "duplicate attachment_id '{}' in report",
                    attachment.attachment_id
                ),
            });
        }
    }

    for entry in &report.entries {
        if let Err(entry_issues) = validate_entry_contract(entry) {
            issues.extend(entry_issues);
        }
        for attachment_id in &entry.attachment_ids {
            if !attachment_ids.contains(attachment_id) {
                issues.push(DiagnosticsValidationIssue {
                    code: "report.entry.attachment_id.missing",
                    message: format!(
                        "entry '{}' references unknown attachment_id '{}'",
                        entry.entry_id, attachment_id
                    ),
                });
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

pub fn validate_before_persist(report: &DiagnosticsFrameReport) -> anyhow::Result<()> {
    match validate_report_contract(report) {
        Ok(()) => Ok(()),
        Err(issues) => Err(anyhow!(
            "diagnostics persistence validation failed for report '{}': {}",
            report.report_id,
            format_issues(&issues)
        )),
    }
}

fn require_non_empty(
    issues: &mut Vec<DiagnosticsValidationIssue>,
    code_prefix: &'static str,
    value: &str,
) {
    if value.trim().is_empty() {
        issues.push(DiagnosticsValidationIssue {
            code: "field.empty",
            message: format!("{} must not be empty", code_prefix),
        });
    }
}

fn format_issues(issues: &[DiagnosticsValidationIssue]) -> String {
    issues
        .iter()
        .map(|issue| format!("{}: {}", issue.code, issue.message))
        .collect::<Vec<_>>()
        .join(" | ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::diagnostics::core::model::{
        DiagnosticsSeverity, DiagnosticsStatus, DiagnosticsSummary,
    };

    #[test]
    fn entry_requires_schema_and_object_payload() {
        let invalid_entry = DiagnosticsEntry {
            entry_id: "entry".to_string(),
            producer_id: "render.inspect".to_string(),
            domain_id: "render".to_string(),
            schema_id: "schema".to_string(),
            schema_version: 0,
            payload_json: serde_json::json!("not-an-object"),
            severity: DiagnosticsSeverity::Info,
            status: DiagnosticsStatus::Ok,
            attachment_ids: Vec::new(),
        };

        let issues = validate_entry_contract(&invalid_entry)
            .expect_err("invalid entry should fail contract validation");
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == "entry.schema_version.zero")
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == "entry.payload_json.not_object")
        );
    }

    #[test]
    fn report_requires_all_attachment_fields() {
        let mut report = DiagnosticsFrameReport {
            report_id: "r".to_string(),
            report_sequence: 1,
            frame_index: 1,
            simulation_tick: 1,
            collected_at_unix_ms: 0,
            entries: vec![DiagnosticsEntry {
                entry_id: "entry.1".to_string(),
                producer_id: "render.inspect".to_string(),
                domain_id: "render".to_string(),
                schema_id: "schema".to_string(),
                schema_version: 1,
                payload_json: serde_json::json!({"a": 1}),
                severity: DiagnosticsSeverity::Info,
                status: DiagnosticsStatus::Ok,
                attachment_ids: vec!["att.1".to_string()],
            }],
            summary: DiagnosticsSummary::default(),
            attachments: vec![DiagnosticsAttachment {
                attachment_id: "att.1".to_string(),
                kind: String::new(),
                label: "label".to_string(),
                producer_id: "render.inspect".to_string(),
                domain_id: "render".to_string(),
                path_or_handle: "path".to_string(),
                metadata: None,
            }],
            warning_count: 0,
            error_count: 0,
        };
        report.recompute_summary();

        let issues = validate_report_contract(&report)
            .expect_err("report with invalid attachment should fail");
        assert!(issues.iter().any(|issue| issue.code == "field.empty"));
    }
}
