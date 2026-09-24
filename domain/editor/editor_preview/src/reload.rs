use asset::{AssetId, AssetKind, AssetRevisionId};
use serde::{Deserialize, Serialize};
use world_sdf::FieldProductId;

use crate::RuntimeProductRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReloadDecision {
    LiveReload,
    PreviewSessionRestartRequired,
    RuntimeProcessRestartRequired,
    Unsupported,
    FailedPreserved,
    Rejected,
}

impl ReloadDecision {
    pub const fn preserves_prior_valid_product(self) -> bool {
        matches!(self, Self::FailedPreserved)
    }

    pub const fn can_apply_live(self) -> bool {
        matches!(self, Self::LiveReload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReloadSubjectKind {
    Asset,
    FieldProduct,
    WorldSdfPayload,
    Shader,
    Scene,
    UiDefinition,
    FutureKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReloadSubject {
    pub kind: ReloadSubjectKind,
    pub asset_kind: Option<AssetKind>,
    pub asset_id: Option<AssetId>,
    pub asset_revision: Option<AssetRevisionId>,
    pub field_product_id: Option<FieldProductId>,
    pub label: String,
}

impl ReloadSubject {
    pub fn new(kind: ReloadSubjectKind, label: impl Into<String>) -> Self {
        Self {
            kind,
            asset_kind: None,
            asset_id: None,
            asset_revision: None,
            field_product_id: None,
            label: label.into(),
        }
    }

    pub fn asset(asset_id: AssetId, asset_kind: AssetKind, revision: AssetRevisionId) -> Self {
        Self {
            kind: ReloadSubjectKind::Asset,
            asset_kind: Some(asset_kind),
            asset_id: Some(asset_id),
            asset_revision: Some(revision),
            field_product_id: None,
            label: format!("{asset_kind:?}:{asset_id:?}"),
        }
    }

    pub fn field_product(product_id: FieldProductId) -> Self {
        Self {
            kind: ReloadSubjectKind::FieldProduct,
            asset_kind: None,
            asset_id: None,
            asset_revision: None,
            field_product_id: Some(product_id),
            label: format!("field_product:{}", product_id.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReloadStatus {
    pub subject: ReloadSubject,
    pub decision: ReloadDecision,
    pub message: String,
    pub runtime_product: Option<RuntimeProductRef>,
    pub prior_valid_product: Option<RuntimeProductRef>,
    pub diagnostics: Vec<String>,
}

impl ReloadStatus {
    pub fn new(
        subject: ReloadSubject,
        decision: ReloadDecision,
        message: impl Into<String>,
    ) -> Self {
        Self {
            subject,
            decision,
            message: message.into(),
            runtime_product: None,
            prior_valid_product: None,
            diagnostics: Vec::new(),
        }
    }

    pub fn with_runtime_product(mut self, product: RuntimeProductRef) -> Self {
        self.runtime_product = Some(product);
        self
    }

    pub fn with_prior_valid_product(mut self, product: RuntimeProductRef) -> Self {
        self.prior_valid_product = Some(product);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostics.push(diagnostic.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_preserved_is_the_only_preservation_decision() {
        assert!(ReloadDecision::FailedPreserved.preserves_prior_valid_product());
        assert!(!ReloadDecision::Rejected.preserves_prior_valid_product());
        assert!(ReloadDecision::LiveReload.can_apply_live());
    }
}
