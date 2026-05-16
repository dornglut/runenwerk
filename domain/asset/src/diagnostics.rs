use diagnostics::{DiagnosticCode, DiagnosticDomain};
use serde::{Deserialize, Serialize};

pub const ASSET_DIAGNOSTIC_DOMAIN: DiagnosticDomain =
    DiagnosticDomain::from_static_unchecked("asset");

pub const ASSET_SOURCE_MISSING_CODE: DiagnosticCode =
    DiagnosticCode::from_static_unchecked("asset.source.missing");
pub const ASSET_SOURCE_HASH_MISMATCH_CODE: DiagnosticCode =
    DiagnosticCode::from_static_unchecked("asset.source.hash_mismatch");
pub const ASSET_ARTIFACT_OUTSIDE_CACHE_CODE: DiagnosticCode =
    DiagnosticCode::from_static_unchecked("asset.artifact.outside_cache");
pub const ASSET_ARTIFACT_KIND_MISMATCH_CODE: DiagnosticCode =
    DiagnosticCode::from_static_unchecked("asset.artifact.kind_mismatch");
pub const ASSET_CATALOG_DUPLICATE_ID_CODE: DiagnosticCode =
    DiagnosticCode::from_static_unchecked("asset.catalog.duplicate_id");
pub const ASSET_IMPORT_TOOL_MISSING_CODE: DiagnosticCode =
    DiagnosticCode::from_static_unchecked("asset.import.tool_missing");
pub const ASSET_IMPORT_PROFILE_REJECTED_CODE: DiagnosticCode =
    DiagnosticCode::from_static_unchecked("asset.import.profile_rejected");
pub const ASSET_RATIFICATION_REJECTED_CODE: DiagnosticCode =
    DiagnosticCode::from_static_unchecked("asset.ratification.rejected");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetDiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetDiagnosticCode {
    SourceMissing,
    SourceHashMismatch,
    ArtifactOutsideCache,
    ArtifactKindMismatch,
    CatalogDuplicateId,
    ImportToolMissing,
    ImportProfileRejected,
    RatificationRejected,
}

impl AssetDiagnosticCode {
    pub const fn diagnostic_code(self) -> DiagnosticCode {
        match self {
            Self::SourceMissing => ASSET_SOURCE_MISSING_CODE,
            Self::SourceHashMismatch => ASSET_SOURCE_HASH_MISMATCH_CODE,
            Self::ArtifactOutsideCache => ASSET_ARTIFACT_OUTSIDE_CACHE_CODE,
            Self::ArtifactKindMismatch => ASSET_ARTIFACT_KIND_MISMATCH_CODE,
            Self::CatalogDuplicateId => ASSET_CATALOG_DUPLICATE_ID_CODE,
            Self::ImportToolMissing => ASSET_IMPORT_TOOL_MISSING_CODE,
            Self::ImportProfileRejected => ASSET_IMPORT_PROFILE_REJECTED_CODE,
            Self::RatificationRejected => ASSET_RATIFICATION_REJECTED_CODE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetDiagnosticRecord {
    pub code: AssetDiagnosticCode,
    pub severity: AssetDiagnosticSeverity,
    pub message: String,
    pub subject: Option<String>,
}

impl AssetDiagnosticRecord {
    pub fn new(
        code: AssetDiagnosticCode,
        severity: AssetDiagnosticSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            subject: None,
        }
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn warning(code: AssetDiagnosticCode, message: impl Into<String>) -> Self {
        Self::new(code, AssetDiagnosticSeverity::Warning, message)
    }

    pub fn error(code: AssetDiagnosticCode, message: impl Into<String>) -> Self {
        Self::new(code, AssetDiagnosticSeverity::Error, message)
    }
}
