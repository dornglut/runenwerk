//! UI-independent app view projection contracts.

use std::collections::BTreeMap;

use crate::ids::{AppModelId, AppModelRevision, AppModelVersion, AppProgramId, AppProgramVersion};
use crate::model::AppModelValue;
use crate::report::AppDiagnostic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppViewProjection {
    pub program_id: AppProgramId,
    pub program_version: AppProgramVersion,
    pub model_id: AppModelId,
    pub model_version: AppModelVersion,
    pub model_revision: AppModelRevision,
    pub screen_id: String,
    pub route_ids: Vec<String>,
    pub values: BTreeMap<String, AppModelValue>,
}

impl AppViewProjection {
    pub fn new(
        program_id: AppProgramId,
        program_version: AppProgramVersion,
        model_id: AppModelId,
        model_version: AppModelVersion,
        model_revision: AppModelRevision,
        screen_id: impl Into<String>,
    ) -> Self {
        Self {
            program_id,
            program_version,
            model_id,
            model_version,
            model_revision,
            screen_id: screen_id.into(),
            route_ids: Vec::new(),
            values: BTreeMap::new(),
        }
    }

    pub fn with_route(mut self, route_id: impl Into<String>) -> Self {
        self.route_ids.push(route_id.into());
        self.route_ids.sort();
        self.route_ids.dedup();
        self
    }

    pub fn with_value(mut self, key: impl Into<String>, value: AppModelValue) -> Self {
        self.values.insert(key.into(), value);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppViewProjectionReport {
    pub source_model_revision: AppModelRevision,
    pub projection: Option<AppViewProjection>,
    pub diagnostics: Vec<AppDiagnostic>,
}

impl AppViewProjectionReport {
    pub fn accepted(projection: AppViewProjection) -> Self {
        Self {
            source_model_revision: projection.model_revision,
            projection: Some(projection),
            diagnostics: Vec::new(),
        }
    }

    pub fn rejected(
        source_model_revision: AppModelRevision,
        diagnostics: Vec<AppDiagnostic>,
    ) -> Self {
        Self {
            source_model_revision,
            projection: None,
            diagnostics,
        }
    }

    pub fn passed(&self) -> bool {
        self.projection.is_some() && !self.diagnostics.iter().any(AppDiagnostic::is_error)
    }
}
