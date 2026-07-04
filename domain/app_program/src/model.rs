//! App model snapshot contracts.

use std::collections::BTreeMap;

use crate::ids::{AppModelId, AppModelRevision, AppModelVersion};
use crate::report::AppDiagnostic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppModelSnapshot {
    pub model_id: AppModelId,
    pub model_version: AppModelVersion,
    pub revision: AppModelRevision,
    pub values: BTreeMap<String, AppModelValue>,
    pub diagnostics: Vec<AppDiagnostic>,
}

impl AppModelSnapshot {
    pub fn new(
        model_id: AppModelId,
        model_version: AppModelVersion,
        revision: AppModelRevision,
    ) -> Self {
        Self {
            model_id,
            model_version,
            revision,
            values: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn with_value(mut self, key: impl Into<String>, value: AppModelValue) -> Self {
        self.values.insert(key.into(), value);
        self
    }

    pub fn with_revision(mut self, revision: AppModelRevision) -> Self {
        self.revision = revision;
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: AppDiagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    pub fn has_error_diagnostics(&self) -> bool {
        self.diagnostics.iter().any(AppDiagnostic::is_error)
    }

    pub fn value(&self, key: &str) -> Option<&AppModelValue> {
        self.values.get(key)
    }

    pub fn integer(&self, key: &str) -> Option<i64> {
        self.value(key).and_then(AppModelValue::as_integer)
    }

    pub fn string(&self, key: &str) -> Option<&str> {
        self.value(key).and_then(AppModelValue::as_str)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppModelValue {
    Integer(i64),
    Bool(bool),
    String(String),
}

impl AppModelValue {
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }
}
