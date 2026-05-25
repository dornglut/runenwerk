//! Runtime-neutral UI definition persistence, migration dry-run, diff, and activation contracts.

use crate::{
    AuthoredUiDefinitionCategory, AuthoredUiNodePath, CURRENT_UI_DEFINITION_SCHEMA_VERSION,
    UiDefinitionDiagnosticSeverity, UiSourceLocation, identity::AuthoredId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub type UiPersistenceDocumentId = AuthoredId;
pub type UiPersistenceMigrationReportId = AuthoredId;
pub type UiPersistenceDiffId = AuthoredId;
pub type UiPersistenceActivationRequestId = AuthoredId;
pub type UiPersistenceTargetProfileId = AuthoredId;
pub type UiPersistenceSourcePackageId = AuthoredId;
pub type UiPersistenceDiagnosticRef = AuthoredId;
pub type UiPersistenceFieldPath = AuthoredUiNodePath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiPersistenceValidationMode {
    Inspect,
    DryRun,
    Activate,
    RollbackPreflight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiUnknownFieldPolicy {
    PreserveCompatible,
    DropCompatible,
    BlockUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiPersistenceActivationImpact {
    None,
    PreviewOnly,
    BlocksActivation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiPersistenceDiagnosticDomain {
    UiDefinition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiPersistenceDiffChangeKind {
    Insert,
    Remove,
    Move,
    Update,
    PreserveUnknown,
    DropUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPersistenceDiffChange {
    pub kind: UiPersistenceDiffChangeKind,
    pub path: UiPersistenceFieldPath,
    #[serde(default)]
    pub before: Option<String>,
    #[serde(default)]
    pub after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPersistenceDocumentDescriptor {
    pub id: UiPersistenceDocumentId,
    pub schema_version: u32,
    pub category: AuthoredUiDefinitionCategory,
    #[serde(default)]
    pub target_profiles: BTreeSet<UiPersistenceTargetProfileId>,
    #[serde(default)]
    pub compatible_unknown_fields: BTreeSet<UiPersistenceFieldPath>,
    #[serde(default)]
    pub required_unknown_fields: BTreeSet<UiPersistenceFieldPath>,
    #[serde(default)]
    pub unpreservable_unknown_fields: BTreeSet<UiPersistenceFieldPath>,
    pub source_package: UiPersistenceSourcePackageId,
    #[serde(default)]
    pub source_location: Option<UiSourceLocation>,
    pub preview_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiMigrationReportDescriptor {
    pub id: UiPersistenceMigrationReportId,
    pub document: UiPersistenceDocumentId,
    pub source_schema_version: u32,
    pub target_schema_version: u32,
    #[serde(default)]
    pub changed_paths: BTreeSet<UiPersistenceFieldPath>,
    #[serde(default)]
    pub incompatible_paths: BTreeSet<UiPersistenceFieldPath>,
    #[serde(default)]
    pub preserved_unknown_fields: BTreeSet<UiPersistenceFieldPath>,
    #[serde(default)]
    pub dropped_unknown_fields: BTreeSet<UiPersistenceFieldPath>,
    #[serde(default)]
    pub deterministic_preview: Option<String>,
    pub source_package: UiPersistenceSourcePackageId,
    #[serde(default)]
    pub source_location: Option<UiSourceLocation>,
    pub preview_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPersistenceDiffDescriptor {
    pub id: UiPersistenceDiffId,
    pub document: UiPersistenceDocumentId,
    pub before_schema_version: u32,
    pub after_schema_version: u32,
    #[serde(default)]
    pub target_profiles: BTreeSet<UiPersistenceTargetProfileId>,
    #[serde(default)]
    pub changes: Vec<UiPersistenceDiffChange>,
    #[serde(default)]
    pub deterministic_text: Option<String>,
    pub source_package: UiPersistenceSourcePackageId,
    #[serde(default)]
    pub source_location: Option<UiSourceLocation>,
    pub preview_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiActivationRequestDescriptor {
    pub id: UiPersistenceActivationRequestId,
    pub document: UiPersistenceDocumentId,
    #[serde(default)]
    pub target_profiles: BTreeSet<UiPersistenceTargetProfileId>,
    #[serde(default)]
    pub migration_report: Option<UiPersistenceMigrationReportId>,
    #[serde(default)]
    pub diff: Option<UiPersistenceDiffId>,
    #[serde(default)]
    pub expected_diagnostics: BTreeSet<UiPersistenceDiagnosticRef>,
    pub source_package: UiPersistenceSourcePackageId,
    #[serde(default)]
    pub source_location: Option<UiSourceLocation>,
    pub preview_only: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPersistenceActivationLibrary {
    #[serde(default)]
    pub documents: Vec<UiPersistenceDocumentDescriptor>,
    #[serde(default)]
    pub migration_reports: Vec<UiMigrationReportDescriptor>,
    #[serde(default)]
    pub diffs: Vec<UiPersistenceDiffDescriptor>,
    #[serde(default)]
    pub activation_requests: Vec<UiActivationRequestDescriptor>,
    #[serde(default)]
    pub known_target_profiles: BTreeSet<UiPersistenceTargetProfileId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPersistenceActivationValidationRequest {
    pub target_profile: UiPersistenceTargetProfileId,
    pub mode: UiPersistenceValidationMode,
    pub unknown_field_policy: UiUnknownFieldPolicy,
    #[serde(default)]
    pub actual_diagnostics: BTreeSet<UiPersistenceDiagnosticRef>,
}

impl UiPersistenceActivationValidationRequest {
    pub fn activate(target_profile: impl Into<UiPersistenceTargetProfileId>) -> Self {
        Self {
            target_profile: target_profile.into(),
            mode: UiPersistenceValidationMode::Activate,
            unknown_field_policy: UiUnknownFieldPolicy::PreserveCompatible,
            actual_diagnostics: BTreeSet::new(),
        }
    }

    pub fn dry_run(target_profile: impl Into<UiPersistenceTargetProfileId>) -> Self {
        Self {
            target_profile: target_profile.into(),
            mode: UiPersistenceValidationMode::DryRun,
            unknown_field_policy: UiUnknownFieldPolicy::PreserveCompatible,
            actual_diagnostics: BTreeSet::new(),
        }
    }

    pub fn with_unknown_field_policy(mut self, policy: UiUnknownFieldPolicy) -> Self {
        self.unknown_field_policy = policy;
        self
    }

    pub fn with_actual_diagnostic(
        mut self,
        diagnostic: impl Into<UiPersistenceDiagnosticRef>,
    ) -> Self {
        self.actual_diagnostics.insert(diagnostic.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPersistenceDiagnostic {
    pub severity: UiDefinitionDiagnosticSeverity,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub document: Option<UiPersistenceDocumentId>,
    #[serde(default)]
    pub migration_report: Option<UiPersistenceMigrationReportId>,
    #[serde(default)]
    pub diff: Option<UiPersistenceDiffId>,
    #[serde(default)]
    pub activation_request: Option<UiPersistenceActivationRequestId>,
    #[serde(default)]
    pub path: Option<UiPersistenceFieldPath>,
    #[serde(default)]
    pub source_location: Option<UiSourceLocation>,
    #[serde(default)]
    pub target_profile: Option<UiPersistenceTargetProfileId>,
    #[serde(default)]
    pub source_schema_version: Option<u32>,
    #[serde(default)]
    pub target_schema_version: Option<u32>,
    pub owning_domain: UiPersistenceDiagnosticDomain,
    #[serde(default)]
    pub source_package: Option<UiPersistenceSourcePackageId>,
    #[serde(default)]
    pub expected_diagnostics: Vec<UiPersistenceDiagnosticRef>,
    #[serde(default)]
    pub actual_diagnostics: Vec<UiPersistenceDiagnosticRef>,
    pub activation_impact: UiPersistenceActivationImpact,
    pub suggested_fix: String,
}

impl UiPersistenceDiagnostic {
    fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        suggested_fix: impl Into<String>,
    ) -> Self {
        Self {
            severity: UiDefinitionDiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
            document: None,
            migration_report: None,
            diff: None,
            activation_request: None,
            path: None,
            source_location: None,
            target_profile: None,
            source_schema_version: None,
            target_schema_version: None,
            owning_domain: UiPersistenceDiagnosticDomain::UiDefinition,
            source_package: None,
            expected_diagnostics: Vec::new(),
            actual_diagnostics: Vec::new(),
            activation_impact: UiPersistenceActivationImpact::BlocksActivation,
            suggested_fix: suggested_fix.into(),
        }
    }

    fn for_document(mut self, document: &UiPersistenceDocumentDescriptor) -> Self {
        self.document = Some(document.id.clone());
        self.source_location = document.source_location.clone();
        self.source_schema_version = Some(document.schema_version);
        self.source_package = Some(document.source_package.clone());
        self
    }

    fn for_migration_report(mut self, report: &UiMigrationReportDescriptor) -> Self {
        self.migration_report = Some(report.id.clone());
        self.document = Some(report.document.clone());
        self.source_location = report.source_location.clone();
        self.source_schema_version = Some(report.source_schema_version);
        self.target_schema_version = Some(report.target_schema_version);
        self.source_package = Some(report.source_package.clone());
        self
    }

    fn for_diff(mut self, diff: &UiPersistenceDiffDescriptor) -> Self {
        self.diff = Some(diff.id.clone());
        self.document = Some(diff.document.clone());
        self.source_location = diff.source_location.clone();
        self.source_schema_version = Some(diff.before_schema_version);
        self.target_schema_version = Some(diff.after_schema_version);
        self.source_package = Some(diff.source_package.clone());
        self
    }

    fn for_activation_request(mut self, activation: &UiActivationRequestDescriptor) -> Self {
        self.activation_request = Some(activation.id.clone());
        self.document = Some(activation.document.clone());
        self.source_location = activation.source_location.clone();
        self.source_package = Some(activation.source_package.clone());
        self.expected_diagnostics = activation.expected_diagnostics.iter().cloned().collect();
        self
    }

    fn with_target_profile(mut self, target_profile: UiPersistenceTargetProfileId) -> Self {
        self.target_profile = Some(target_profile);
        self
    }

    fn with_path(mut self, path: UiPersistenceFieldPath) -> Self {
        self.path = Some(path);
        self
    }

    fn with_diagnostic_mismatch(
        mut self,
        expected: Vec<UiPersistenceDiagnosticRef>,
        actual: Vec<UiPersistenceDiagnosticRef>,
    ) -> Self {
        self.expected_diagnostics = expected;
        self.actual_diagnostics = actual;
        self
    }

    fn preview_only(mut self) -> Self {
        self.activation_impact = UiPersistenceActivationImpact::PreviewOnly;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPersistenceActivationValidationReport {
    #[serde(default)]
    pub diagnostics: Vec<UiPersistenceDiagnostic>,
}

impl UiPersistenceActivationValidationReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == UiDefinitionDiagnosticSeverity::Error)
    }
}

pub fn validate_persistence_activation(
    library: &UiPersistenceActivationLibrary,
    request: &UiPersistenceActivationValidationRequest,
) -> UiPersistenceActivationValidationReport {
    let mut diagnostics = Vec::new();
    let documents = index_documents(library, request, &mut diagnostics);
    let migration_reports = index_migration_reports(library, request, &mut diagnostics);
    let diffs = index_diffs(library, request, &mut diagnostics);
    let activation_requests = index_activation_requests(library, request, &mut diagnostics);

    let mut validator = PersistenceActivationValidator {
        request,
        diagnostics,
    };

    validator.validate_documents(&documents);
    validator.validate_migration_reports(&documents, &migration_reports);
    validator.validate_diffs(&documents, &diffs);
    validator.validate_activation_requests(
        &documents,
        &migration_reports,
        &diffs,
        &activation_requests,
    );

    UiPersistenceActivationValidationReport {
        diagnostics: validator.diagnostics,
    }
}

fn index_documents<'a>(
    library: &'a UiPersistenceActivationLibrary,
    request: &UiPersistenceActivationValidationRequest,
    diagnostics: &mut Vec<UiPersistenceDiagnostic>,
) -> BTreeMap<UiPersistenceDocumentId, &'a UiPersistenceDocumentDescriptor> {
    let mut documents = BTreeMap::new();
    for document in &library.documents {
        if documents.insert(document.id.clone(), document).is_some() {
            diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.document.duplicate_id",
                    format!(
                        "Persistence document '{}' is declared more than once.",
                        document.id
                    ),
                    "Keep one persistence document descriptor for each stable document id.",
                )
                .for_document(document)
                .with_target_profile(request.target_profile.clone()),
            );
        }
    }
    documents
}

fn index_migration_reports<'a>(
    library: &'a UiPersistenceActivationLibrary,
    request: &UiPersistenceActivationValidationRequest,
    diagnostics: &mut Vec<UiPersistenceDiagnostic>,
) -> BTreeMap<UiPersistenceMigrationReportId, &'a UiMigrationReportDescriptor> {
    let mut reports = BTreeMap::new();
    for report in &library.migration_reports {
        if reports.insert(report.id.clone(), report).is_some() {
            diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.migration_report.duplicate_id",
                    format!(
                        "Migration report '{}' is declared more than once.",
                        report.id
                    ),
                    "Keep one migration report descriptor for each stable report id.",
                )
                .for_migration_report(report)
                .with_target_profile(request.target_profile.clone()),
            );
        }
    }
    reports
}

fn index_diffs<'a>(
    library: &'a UiPersistenceActivationLibrary,
    request: &UiPersistenceActivationValidationRequest,
    diagnostics: &mut Vec<UiPersistenceDiagnostic>,
) -> BTreeMap<UiPersistenceDiffId, &'a UiPersistenceDiffDescriptor> {
    let mut diffs = BTreeMap::new();
    for diff in &library.diffs {
        if diffs.insert(diff.id.clone(), diff).is_some() {
            diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.diff.duplicate_id",
                    format!("Persistence diff '{}' is declared more than once.", diff.id),
                    "Keep one diff descriptor for each stable diff id.",
                )
                .for_diff(diff)
                .with_target_profile(request.target_profile.clone()),
            );
        }
    }
    diffs
}

fn index_activation_requests<'a>(
    library: &'a UiPersistenceActivationLibrary,
    request: &UiPersistenceActivationValidationRequest,
    diagnostics: &mut Vec<UiPersistenceDiagnostic>,
) -> BTreeMap<UiPersistenceActivationRequestId, &'a UiActivationRequestDescriptor> {
    let mut activations = BTreeMap::new();
    for activation in &library.activation_requests {
        if activations
            .insert(activation.id.clone(), activation)
            .is_some()
        {
            diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.duplicate_id",
                    format!(
                        "Activation request '{}' is declared more than once.",
                        activation.id
                    ),
                    "Keep one activation request descriptor for each stable activation request id.",
                )
                .for_activation_request(activation)
                .with_target_profile(request.target_profile.clone()),
            );
        }
    }
    activations
}

struct PersistenceActivationValidator<'a> {
    request: &'a UiPersistenceActivationValidationRequest,
    diagnostics: Vec<UiPersistenceDiagnostic>,
}

impl PersistenceActivationValidator<'_> {
    fn validate_documents(
        &mut self,
        documents: &BTreeMap<UiPersistenceDocumentId, &UiPersistenceDocumentDescriptor>,
    ) {
        for document in documents.values() {
            self.validate_document_target_profile(document);
            self.validate_document_schema_version(document);
            self.validate_unknown_required_fields(document);
            self.validate_unpreservable_unknown_fields(document);
            self.validate_unknown_field_policy(document);
            self.validate_preview_only_document(document);
        }
    }

    fn validate_document_target_profile(&mut self, document: &UiPersistenceDocumentDescriptor) {
        if !document.target_profiles.is_empty()
            && !document
                .target_profiles
                .contains(&self.request.target_profile)
        {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.document.target_profile_unsupported",
                    format!(
                        "Persistence document '{}' does not support target profile '{}'.",
                        document.id, self.request.target_profile
                    ),
                    "Add the target profile to the document descriptor or validate with a compatible target profile.",
                )
                .for_document(document)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_document_schema_version(&mut self, document: &UiPersistenceDocumentDescriptor) {
        if document.schema_version > CURRENT_UI_DEFINITION_SCHEMA_VERSION {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.schema.unsupported_version",
                    format!(
                        "Persistence document '{}' uses unsupported schema version '{}'.",
                        document.id, document.schema_version
                    ),
                    "Run a supported migration path before activation.",
                )
                .for_document(document)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_unknown_required_fields(&mut self, document: &UiPersistenceDocumentDescriptor) {
        for path in &document.required_unknown_fields {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.unknown_required_field",
                    format!(
                        "Persistence document '{}' contains an unknown required field.",
                        document.id
                    ),
                    "Add a schema migration that understands the required field before activation.",
                )
                .for_document(document)
                .with_path(path.clone())
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_unpreservable_unknown_fields(
        &mut self,
        document: &UiPersistenceDocumentDescriptor,
    ) {
        for path in &document.unpreservable_unknown_fields {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.unknown_field.unpreservable",
                    format!(
                        "Persistence document '{}' contains an unknown field that cannot be preserved.",
                        document.id
                    ),
                    "Reject activation or add an owning migration for the unknown field.",
                )
                .for_document(document)
                .with_path(path.clone())
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_unknown_field_policy(&mut self, document: &UiPersistenceDocumentDescriptor) {
        if self.request.unknown_field_policy == UiUnknownFieldPolicy::BlockUnknown
            && !document.compatible_unknown_fields.is_empty()
        {
            for path in &document.compatible_unknown_fields {
                self.diagnostics.push(
                    UiPersistenceDiagnostic::error(
                        "ui.persistence.unknown_field.blocked_by_policy",
                        format!(
                            "Persistence document '{}' contains an unknown field while unknown fields are blocked.",
                            document.id
                        ),
                        "Use preserve-compatible policy or remove the unknown field before activation.",
                    )
                    .for_document(document)
                    .with_path(path.clone())
                    .with_target_profile(self.request.target_profile.clone()),
                );
            }
        }
    }

    fn validate_preview_only_document(&mut self, document: &UiPersistenceDocumentDescriptor) {
        if document.preview_only && self.request.mode == UiPersistenceValidationMode::Activate {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.document.preview_only_activation",
                    format!(
                        "Persistence document '{}' is preview-only and cannot activate.",
                        document.id
                    ),
                    "Use dry-run validation or remove the preview-only flag before activation.",
                )
                .for_document(document)
                .with_target_profile(self.request.target_profile.clone())
                .preview_only(),
            );
        }
    }

    fn validate_migration_reports(
        &mut self,
        documents: &BTreeMap<UiPersistenceDocumentId, &UiPersistenceDocumentDescriptor>,
        reports: &BTreeMap<UiPersistenceMigrationReportId, &UiMigrationReportDescriptor>,
    ) {
        for report in reports.values() {
            let document = match documents.get(&report.document) {
                Some(document) => *document,
                None => {
                    self.diagnostics.push(
                        UiPersistenceDiagnostic::error(
                            "ui.persistence.migration_report.document_unknown",
                            format!(
                                "Migration report '{}' references unknown document '{}'.",
                                report.id, report.document
                            ),
                            "Create the referenced persistence document descriptor or update the migration report reference.",
                        )
                        .for_migration_report(report)
                        .with_target_profile(self.request.target_profile.clone()),
                    );
                    continue;
                }
            };

            self.validate_migration_schema_match(report, document);
            self.validate_migration_target_schema(report);
            self.validate_incompatible_migration_paths(report);
            self.validate_migration_unknown_field_policy(report, document);
            self.validate_migration_preview_only(report);
        }
    }

    fn validate_migration_schema_match(
        &mut self,
        report: &UiMigrationReportDescriptor,
        document: &UiPersistenceDocumentDescriptor,
    ) {
        if report.source_schema_version != document.schema_version {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.migration_report.schema_mismatch",
                    format!(
                        "Migration report '{}' source schema version does not match document '{}'.",
                        report.id, document.id
                    ),
                    "Regenerate the migration report from the current persisted document.",
                )
                .for_migration_report(report)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_migration_target_schema(&mut self, report: &UiMigrationReportDescriptor) {
        if report.target_schema_version != CURRENT_UI_DEFINITION_SCHEMA_VERSION {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.migration_report.target_version_unsupported",
                    format!(
                        "Migration report '{}' targets unsupported schema version '{}'.",
                        report.id, report.target_schema_version
                    ),
                    "Target the current UI definition schema version before activation.",
                )
                .for_migration_report(report)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_incompatible_migration_paths(&mut self, report: &UiMigrationReportDescriptor) {
        for path in &report.incompatible_paths {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.migration.incompatible_path",
                    format!(
                        "Migration report '{}' contains an incompatible field path.",
                        report.id
                    ),
                    "Resolve the incompatible migration before activation.",
                )
                .for_migration_report(report)
                .with_path(path.clone())
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_migration_unknown_field_policy(
        &mut self,
        report: &UiMigrationReportDescriptor,
        document: &UiPersistenceDocumentDescriptor,
    ) {
        if self.request.unknown_field_policy == UiUnknownFieldPolicy::PreserveCompatible {
            for path in document
                .compatible_unknown_fields
                .difference(&report.preserved_unknown_fields)
            {
                self.diagnostics.push(
                    UiPersistenceDiagnostic::error(
                        "ui.persistence.unknown_field.not_preserved",
                        format!(
                            "Migration report '{}' does not preserve a compatible unknown field.",
                            report.id
                        ),
                        "Preserve compatible unknown fields or change policy before activation.",
                    )
                    .for_migration_report(report)
                    .with_path(path.clone())
                    .with_target_profile(self.request.target_profile.clone()),
                );
            }
        }

        if self.request.unknown_field_policy == UiUnknownFieldPolicy::PreserveCompatible {
            for path in &report.dropped_unknown_fields {
                self.diagnostics.push(
                    UiPersistenceDiagnostic::error(
                        "ui.persistence.unknown_field.dropped",
                        format!(
                            "Migration report '{}' drops an unknown field while preserve-compatible policy is active.",
                            report.id
                        ),
                        "Preserve the field or mark it unpreservable with a blocking diagnostic.",
                    )
                    .for_migration_report(report)
                    .with_path(path.clone())
                    .with_target_profile(self.request.target_profile.clone()),
                );
            }
        }
    }

    fn validate_migration_preview_only(&mut self, report: &UiMigrationReportDescriptor) {
        if report.preview_only && self.request.mode == UiPersistenceValidationMode::Activate {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.migration_report.preview_only_activation",
                    format!(
                        "Migration report '{}' is preview-only and cannot activate.",
                        report.id
                    ),
                    "Use dry-run validation or regenerate an activation-capable migration report.",
                )
                .for_migration_report(report)
                .with_target_profile(self.request.target_profile.clone())
                .preview_only(),
            );
        }
    }

    fn validate_diffs(
        &mut self,
        documents: &BTreeMap<UiPersistenceDocumentId, &UiPersistenceDocumentDescriptor>,
        diffs: &BTreeMap<UiPersistenceDiffId, &UiPersistenceDiffDescriptor>,
    ) {
        for diff in diffs.values() {
            if !documents.contains_key(&diff.document) {
                self.diagnostics.push(
                    UiPersistenceDiagnostic::error(
                        "ui.persistence.diff.document_unknown",
                        format!(
                            "Persistence diff '{}' references unknown document '{}'.",
                            diff.id, diff.document
                        ),
                        "Create the referenced persistence document descriptor or update the diff reference.",
                    )
                    .for_diff(diff)
                    .with_target_profile(self.request.target_profile.clone()),
                );
                continue;
            }

            self.validate_diff_target_profile(diff);
            self.validate_diff_target_schema(diff);
            self.validate_deterministic_diff_text(diff);
            self.validate_diff_preview_only(diff);
        }
    }

    fn validate_diff_target_profile(&mut self, diff: &UiPersistenceDiffDescriptor) {
        if !diff.target_profiles.is_empty()
            && !diff.target_profiles.contains(&self.request.target_profile)
        {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.diff.target_profile_unsupported",
                    format!(
                        "Persistence diff '{}' does not support target profile '{}'.",
                        diff.id, self.request.target_profile
                    ),
                    "Add the target profile to the diff descriptor or validate with a compatible target profile.",
                )
                .for_diff(diff)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_diff_target_schema(&mut self, diff: &UiPersistenceDiffDescriptor) {
        if diff.after_schema_version != CURRENT_UI_DEFINITION_SCHEMA_VERSION {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.diff.target_version_unsupported",
                    format!(
                        "Persistence diff '{}' targets unsupported schema version '{}'.",
                        diff.id, diff.after_schema_version
                    ),
                    "Regenerate the diff against the current UI definition schema version.",
                )
                .for_diff(diff)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_deterministic_diff_text(&mut self, diff: &UiPersistenceDiffDescriptor) {
        if diff
            .deterministic_text
            .as_ref()
            .is_none_or(|text| text.is_empty())
        {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.diff.non_deterministic",
                    format!("Persistence diff '{}' does not provide deterministic diff text.", diff.id),
                    "Serialize the migration or edit into deterministic textual diff output before activation.",
                )
                .for_diff(diff)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_diff_preview_only(&mut self, diff: &UiPersistenceDiffDescriptor) {
        if diff.preview_only && self.request.mode == UiPersistenceValidationMode::Activate {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.diff.preview_only_activation",
                    format!(
                        "Persistence diff '{}' is preview-only and cannot activate.",
                        diff.id
                    ),
                    "Use dry-run validation or produce an activation-capable deterministic diff.",
                )
                .for_diff(diff)
                .with_target_profile(self.request.target_profile.clone())
                .preview_only(),
            );
        }
    }

    fn validate_activation_requests(
        &mut self,
        documents: &BTreeMap<UiPersistenceDocumentId, &UiPersistenceDocumentDescriptor>,
        reports: &BTreeMap<UiPersistenceMigrationReportId, &UiMigrationReportDescriptor>,
        diffs: &BTreeMap<UiPersistenceDiffId, &UiPersistenceDiffDescriptor>,
        activations: &BTreeMap<UiPersistenceActivationRequestId, &UiActivationRequestDescriptor>,
    ) {
        for activation in activations.values() {
            let document = match documents.get(&activation.document) {
                Some(document) => *document,
                None => {
                    self.diagnostics.push(
                        UiPersistenceDiagnostic::error(
                            "ui.persistence.activation.document_unknown",
                            format!(
                                "Activation request '{}' references unknown document '{}'.",
                                activation.id, activation.document
                            ),
                            "Create the referenced persistence document descriptor or update the activation request.",
                        )
                        .for_activation_request(activation)
                        .with_target_profile(self.request.target_profile.clone()),
                    );
                    continue;
                }
            };

            self.validate_activation_target_profile(activation);
            self.validate_activation_preview_only(activation);
            self.validate_activation_migration_report(activation, document, reports);
            self.validate_activation_diff(activation, document, diffs);
            self.validate_activation_diagnostic_expectations(activation);
        }
    }

    fn validate_activation_target_profile(&mut self, activation: &UiActivationRequestDescriptor) {
        if !activation.target_profiles.is_empty()
            && !activation
                .target_profiles
                .contains(&self.request.target_profile)
        {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.target_profile_unsupported",
                    format!(
                        "Activation request '{}' does not support target profile '{}'.",
                        activation.id, self.request.target_profile
                    ),
                    "Add the target profile to the activation request or validate with a compatible target profile.",
                )
                .for_activation_request(activation)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_activation_preview_only(&mut self, activation: &UiActivationRequestDescriptor) {
        if activation.preview_only && self.request.mode == UiPersistenceValidationMode::Activate {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.preview_only_activation",
                    format!(
                        "Activation request '{}' is preview-only and cannot activate.",
                        activation.id
                    ),
                    "Use dry-run validation or remove the preview-only flag before activation.",
                )
                .for_activation_request(activation)
                .with_target_profile(self.request.target_profile.clone())
                .preview_only(),
            );
        }
    }

    fn validate_activation_migration_report(
        &mut self,
        activation: &UiActivationRequestDescriptor,
        document: &UiPersistenceDocumentDescriptor,
        reports: &BTreeMap<UiPersistenceMigrationReportId, &UiMigrationReportDescriptor>,
    ) {
        let Some(report_id) = &activation.migration_report else {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.migration_report_missing",
                    format!(
                        "Activation request '{}' does not reference a migration report.",
                        activation.id
                    ),
                    "Run migration dry-run and attach the report before activation.",
                )
                .for_activation_request(activation)
                .with_target_profile(self.request.target_profile.clone()),
            );
            return;
        };

        let Some(report) = reports.get(report_id) else {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.migration_report_unknown",
                    format!(
                        "Activation request '{}' references unknown migration report '{}'.",
                        activation.id, report_id
                    ),
                    "Create the referenced migration report or update the activation request.",
                )
                .for_activation_request(activation)
                .with_target_profile(self.request.target_profile.clone()),
            );
            return;
        };

        if report.document != document.id {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.migration_report_document_mismatch",
                    format!(
                        "Activation request '{}' references a migration report for a different document.",
                        activation.id
                    ),
                    "Attach a migration report generated for the activation document.",
                )
                .for_activation_request(activation)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_activation_diff(
        &mut self,
        activation: &UiActivationRequestDescriptor,
        document: &UiPersistenceDocumentDescriptor,
        diffs: &BTreeMap<UiPersistenceDiffId, &UiPersistenceDiffDescriptor>,
    ) {
        let Some(diff_id) = &activation.diff else {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.diff_missing",
                    format!(
                        "Activation request '{}' does not reference a deterministic diff.",
                        activation.id
                    ),
                    "Attach a deterministic diff descriptor before activation.",
                )
                .for_activation_request(activation)
                .with_target_profile(self.request.target_profile.clone()),
            );
            return;
        };

        let Some(diff) = diffs.get(diff_id) else {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.diff_unknown",
                    format!(
                        "Activation request '{}' references unknown diff '{}'.",
                        activation.id, diff_id
                    ),
                    "Create the referenced deterministic diff or update the activation request.",
                )
                .for_activation_request(activation)
                .with_target_profile(self.request.target_profile.clone()),
            );
            return;
        };

        if diff.document != document.id {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.diff_document_mismatch",
                    format!(
                        "Activation request '{}' references a diff for a different document.",
                        activation.id
                    ),
                    "Attach a deterministic diff generated for the activation document.",
                )
                .for_activation_request(activation)
                .with_target_profile(self.request.target_profile.clone()),
            );
        }
    }

    fn validate_activation_diagnostic_expectations(
        &mut self,
        activation: &UiActivationRequestDescriptor,
    ) {
        if activation.expected_diagnostics != self.request.actual_diagnostics {
            self.diagnostics.push(
                UiPersistenceDiagnostic::error(
                    "ui.persistence.activation.expected_diagnostics_mismatch",
                    format!(
                        "Activation request '{}' expected diagnostics do not match actual diagnostics.",
                        activation.id
                    ),
                    "Update expected diagnostics or fix the persistence input that produced different diagnostics.",
                )
                .for_activation_request(activation)
                .with_target_profile(self.request.target_profile.clone())
                .with_diagnostic_mismatch(
                    activation.expected_diagnostics.iter().cloned().collect(),
                    self.request.actual_diagnostics.iter().cloned().collect(),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> AuthoredId {
        AuthoredId::from(value)
    }

    fn path(value: &str) -> AuthoredUiNodePath {
        AuthoredUiNodePath(value.to_string())
    }

    fn target_profiles(values: &[&str]) -> BTreeSet<UiPersistenceTargetProfileId> {
        values.iter().map(|value| id(value)).collect()
    }

    fn source_package() -> UiPersistenceSourcePackageId {
        id("ui.package.core")
    }

    fn document(
        id_value: &str,
        category: AuthoredUiDefinitionCategory,
    ) -> UiPersistenceDocumentDescriptor {
        UiPersistenceDocumentDescriptor {
            id: id(id_value),
            schema_version: CURRENT_UI_DEFINITION_SCHEMA_VERSION,
            category,
            target_profiles: target_profiles(&["editor.workbench", "game.runtime"]),
            compatible_unknown_fields: BTreeSet::new(),
            required_unknown_fields: BTreeSet::new(),
            unpreservable_unknown_fields: BTreeSet::new(),
            source_package: source_package(),
            source_location: None,
            preview_only: false,
        }
    }

    fn migration_report(document_id: &str) -> UiMigrationReportDescriptor {
        UiMigrationReportDescriptor {
            id: id(&format!("{document_id}.migration")),
            document: id(document_id),
            source_schema_version: CURRENT_UI_DEFINITION_SCHEMA_VERSION,
            target_schema_version: CURRENT_UI_DEFINITION_SCHEMA_VERSION,
            changed_paths: BTreeSet::from([path("root/title")]),
            incompatible_paths: BTreeSet::new(),
            preserved_unknown_fields: BTreeSet::new(),
            dropped_unknown_fields: BTreeSet::new(),
            deterministic_preview: Some("schema_version = 1".to_string()),
            source_package: source_package(),
            source_location: None,
            preview_only: false,
        }
    }

    fn diff(document_id: &str) -> UiPersistenceDiffDescriptor {
        UiPersistenceDiffDescriptor {
            id: id(&format!("{document_id}.diff")),
            document: id(document_id),
            before_schema_version: CURRENT_UI_DEFINITION_SCHEMA_VERSION,
            after_schema_version: CURRENT_UI_DEFINITION_SCHEMA_VERSION,
            target_profiles: target_profiles(&["editor.workbench", "game.runtime"]),
            changes: vec![UiPersistenceDiffChange {
                kind: UiPersistenceDiffChangeKind::Update,
                path: path("root/title"),
                before: Some("Old".to_string()),
                after: Some("New".to_string()),
            }],
            deterministic_text: Some("- title: Old\n+ title: New\n".to_string()),
            source_package: source_package(),
            source_location: None,
            preview_only: false,
        }
    }

    fn activation(document_id: &str) -> UiActivationRequestDescriptor {
        UiActivationRequestDescriptor {
            id: id(&format!("{document_id}.activate")),
            document: id(document_id),
            target_profiles: target_profiles(&["editor.workbench", "game.runtime"]),
            migration_report: Some(id(&format!("{document_id}.migration"))),
            diff: Some(id(&format!("{document_id}.diff"))),
            expected_diagnostics: BTreeSet::new(),
            source_package: source_package(),
            source_location: None,
            preview_only: false,
        }
    }

    fn library() -> UiPersistenceActivationLibrary {
        UiPersistenceActivationLibrary {
            documents: vec![
                document("editor.inspector", AuthoredUiDefinitionCategory::Editor),
                document("runtime.hud", AuthoredUiDefinitionCategory::GameUi),
            ],
            migration_reports: vec![
                migration_report("editor.inspector"),
                migration_report("runtime.hud"),
            ],
            diffs: vec![diff("editor.inspector"), diff("runtime.hud")],
            activation_requests: vec![activation("editor.inspector"), activation("runtime.hud")],
            known_target_profiles: target_profiles(&["editor.workbench", "game.runtime"]),
        }
    }

    fn request(target_profile: &str) -> UiPersistenceActivationValidationRequest {
        UiPersistenceActivationValidationRequest::activate(target_profile)
    }

    fn codes(report: &UiPersistenceActivationValidationReport) -> BTreeSet<&str> {
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect()
    }

    #[test]
    fn persistence_activation_validates_editor_and_runtime_examples_without_shared_ownership() {
        let editor = validate_persistence_activation(&library(), &request("editor.workbench"));
        let runtime = validate_persistence_activation(&library(), &request("game.runtime"));

        assert!(
            !editor.has_errors(),
            "editor diagnostics: {:?}",
            editor.diagnostics
        );
        assert!(
            !runtime.has_errors(),
            "runtime diagnostics: {:?}",
            runtime.diagnostics
        );
    }

    #[test]
    fn persistence_activation_rejects_unsupported_schema_version() {
        let mut library = library();
        library.documents[0].schema_version = CURRENT_UI_DEFINITION_SCHEMA_VERSION + 1;

        let report = validate_persistence_activation(&library, &request("editor.workbench"));

        assert!(codes(&report).contains("ui.persistence.schema.unsupported_version"));
    }

    #[test]
    fn persistence_activation_rejects_incompatible_migration() {
        let mut library = library();
        library.migration_reports[0]
            .incompatible_paths
            .insert(path("root/legacy"));

        let report = validate_persistence_activation(&library, &request("editor.workbench"));

        assert!(codes(&report).contains("ui.persistence.migration.incompatible_path"));
    }

    #[test]
    fn persistence_activation_preserves_compatible_unknown_fields() {
        let mut library = library();
        library.documents[0]
            .compatible_unknown_fields
            .insert(path("root/extension"));
        library.migration_reports[0]
            .preserved_unknown_fields
            .insert(path("root/extension"));

        let report = validate_persistence_activation(&library, &request("editor.workbench"));

        assert!(
            !report.has_errors(),
            "diagnostics: {:?}",
            report.diagnostics
        );
    }

    #[test]
    fn persistence_activation_rejects_unpreservable_unknown_fields() {
        let mut library = library();
        library.documents[0]
            .unpreservable_unknown_fields
            .insert(path("root/plugin_state"));

        let report = validate_persistence_activation(&library, &request("editor.workbench"));

        assert!(codes(&report).contains("ui.persistence.unknown_field.unpreservable"));
    }

    #[test]
    fn persistence_activation_rejects_non_deterministic_diff() {
        let mut library = library();
        library.diffs[0].deterministic_text = None;

        let report = validate_persistence_activation(&library, &request("editor.workbench"));

        assert!(codes(&report).contains("ui.persistence.diff.non_deterministic"));
    }

    #[test]
    fn persistence_activation_requires_migration_report_and_diff() {
        let mut library = library();
        library.activation_requests[0].migration_report = None;
        library.activation_requests[0].diff = None;

        let report = validate_persistence_activation(&library, &request("editor.workbench"));
        let codes = codes(&report);

        assert!(codes.contains("ui.persistence.activation.migration_report_missing"));
        assert!(codes.contains("ui.persistence.activation.diff_missing"));
    }

    #[test]
    fn game_runtime_persistence_activation_requires_migration_diff_and_determinism() {
        let mut library = library();
        library.activation_requests[0].migration_report = None;
        library.activation_requests[0].diff = None;
        library.diffs[0].deterministic_text = None;

        let report = validate_persistence_activation(&library, &request("game.runtime"));
        let codes = codes(&report);

        assert!(codes.contains("ui.persistence.activation.migration_report_missing"));
        assert!(codes.contains("ui.persistence.activation.diff_missing"));
        assert!(codes.contains("ui.persistence.diff.non_deterministic"));
    }

    #[test]
    fn persistence_activation_rejects_unsupported_target_profile() {
        let report = validate_persistence_activation(&library(), &request("console.runtime"));
        let codes = codes(&report);

        assert!(codes.contains("ui.persistence.document.target_profile_unsupported"));
        assert!(codes.contains("ui.persistence.diff.target_profile_unsupported"));
        assert!(codes.contains("ui.persistence.activation.target_profile_unsupported"));
    }

    #[test]
    fn persistence_activation_rejects_expected_diagnostic_mismatches() {
        let mut library = library();
        library.activation_requests[0]
            .expected_diagnostics
            .insert(id("ui.persistence.expected"));

        let report = validate_persistence_activation(
            &library,
            &UiPersistenceActivationValidationRequest::activate("editor.workbench")
                .with_actual_diagnostic("ui.persistence.actual"),
        );

        assert!(codes(&report).contains("ui.persistence.activation.expected_diagnostics_mismatch"));
    }

    #[test]
    fn persistence_activation_rejects_preview_only_activation() {
        let mut library = library();
        library.documents[0].preview_only = true;
        library.migration_reports[0].preview_only = true;
        library.diffs[0].preview_only = true;
        library.activation_requests[0].preview_only = true;

        let report = validate_persistence_activation(&library, &request("editor.workbench"));
        let codes = codes(&report);

        assert!(codes.contains("ui.persistence.document.preview_only_activation"));
        assert!(codes.contains("ui.persistence.migration_report.preview_only_activation"));
        assert!(codes.contains("ui.persistence.diff.preview_only_activation"));
        assert!(codes.contains("ui.persistence.activation.preview_only_activation"));
    }
}
