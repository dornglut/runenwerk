//! File: domain/editor/editor_persistence/src/change_log.rs
//! Purpose: Durable retained-ratification change log contracts.

use serde::{Deserialize, Serialize};

pub const RETAINED_CHANGE_LOG_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedRatifiedChangeRecordV1 {
    pub ratification_id: u64,
    pub transaction_id: u64,
    pub transaction_label: String,
    pub causality_id: u64,
    pub origin: String,
    pub authority_scope: String,
    pub affected_domains: Vec<String>,
    pub affected_scopes: Vec<String>,
    pub base_version: u64,
    pub result_version: u64,
    pub semantic_operations: Vec<String>,
    pub ratification_class: String,
    pub reversibility_class: String,
    pub retention_hint: String,
    pub stability_class: String,
    pub reconciliation_policy: String,
    pub propagation_structure: String,
    pub timestamp_unix_millis: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedRatifiedChangeLogV1 {
    pub version: u32,
    pub entries: Vec<RetainedRatifiedChangeRecordV1>,
}
