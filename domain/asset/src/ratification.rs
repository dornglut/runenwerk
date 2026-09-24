use std::collections::BTreeSet;

use foundation_ratification::{RatificationIssue, RatificationReport, Ratifier};
use serde::{Deserialize, Serialize};

use crate::{
    ASSET_CATALOG_VERSION_V1, ASSET_PROJECT_CATALOG_DESCRIPTOR_VERSION_V1, ArtifactValidity,
    AssetArtifactDescriptor, AssetCatalog, AssetProjectCatalogDescriptor, AssetSourceDescriptor,
    ImportPlan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetRatificationIssueCode {
    EmptySourcePath,
    AbsoluteSourcePath,
    ParentTraversalSourcePath,
    SourceAssetMissing,
    SourceRootMissing,
    EmptyStableName,
    MissingPrimarySource,
    PrimarySourceAssetMismatch,
    MissingArtifactPath,
    AbsoluteArtifactPath,
    ParentTraversalArtifactPath,
    ArtifactSourceMissing,
    ArtifactSourceAssetMismatch,
    FailedArtifactWithoutDiagnostic,
    DuplicateStableName,
    ArtifactAssetMissing,
    DependencyAssetMissing,
    SelfDependency,
    UnsupportedCatalogVersion,
    ImportPlanWithoutExpectedArtifact,
    ImportPlanWithoutRequiredArtifact,
    ImportPlanSourceMismatch,
    ImportPlanHashMismatch,
    UnsupportedImportSettingsForSource,
    UnsupportedArtifactKindForImportSettings,
    ImportPlanCacheKeyMismatch,
    DuplicateImportPlanDependency,
    ImportPlanSelfDependency,
    UnsupportedProjectCatalogVersion,
    EmptyProjectCatalogPath,
    AbsoluteProjectCatalogPath,
    ParentTraversalProjectCatalogPath,
    EmptySourceRootName,
    DuplicateSourceRootId,
    DuplicateSourceRootPath,
    EmptyImportProfileName,
    ImportProfileSettingsMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetRatificationSubject {
    Source(u64),
    Artifact(u64),
    Asset(u64),
    SourceRoot(u64),
    ImportPlan(u64),
    Dependency { asset_id: u64, depends_on: u64 },
    ProjectCatalog,
    Catalog,
}

pub struct AssetSourceRatifier;
pub struct AssetArtifactRatifier;
pub struct AssetCatalogRatifier;
pub struct AssetImportPlanRatifier;
pub struct AssetProjectCatalogDescriptorRatifier;

impl Ratifier<AssetSourceDescriptor> for AssetSourceRatifier {
    type Code = AssetRatificationIssueCode;
    type Subject = AssetRatificationSubject;

    fn ratify(
        &self,
        candidate: &AssetSourceDescriptor,
    ) -> RatificationReport<Self::Code, Self::Subject> {
        let mut report = RatificationReport::accepted();
        if let Some(violation) = project_relative_path_violation(&candidate.relative_path) {
            let code = match violation {
                ProjectRelativePathViolation::Empty => AssetRatificationIssueCode::EmptySourcePath,
                ProjectRelativePathViolation::Absolute => {
                    AssetRatificationIssueCode::AbsoluteSourcePath
                }
                ProjectRelativePathViolation::ParentTraversal => {
                    AssetRatificationIssueCode::ParentTraversalSourcePath
                }
            };
            let message = match violation {
                ProjectRelativePathViolation::Empty => "asset source path must not be empty",
                ProjectRelativePathViolation::Absolute => {
                    "asset source path must be project-relative"
                }
                ProjectRelativePathViolation::ParentTraversal => {
                    "asset source path must not traverse outside the project"
                }
            };
            report.push(RatificationIssue::error(
                code,
                AssetRatificationSubject::Source(candidate.source_id.raw()),
                message,
            ));
        }
        report
    }
}

impl Ratifier<AssetArtifactDescriptor> for AssetArtifactRatifier {
    type Code = AssetRatificationIssueCode;
    type Subject = AssetRatificationSubject;

    fn ratify(
        &self,
        candidate: &AssetArtifactDescriptor,
    ) -> RatificationReport<Self::Code, Self::Subject> {
        let mut report = RatificationReport::accepted();
        match candidate.artifact_path.as_deref() {
            Some(path) => {
                if let Some(violation) = project_relative_path_violation(path) {
                    let code = match violation {
                        ProjectRelativePathViolation::Empty => {
                            AssetRatificationIssueCode::MissingArtifactPath
                        }
                        ProjectRelativePathViolation::Absolute => {
                            AssetRatificationIssueCode::AbsoluteArtifactPath
                        }
                        ProjectRelativePathViolation::ParentTraversal => {
                            AssetRatificationIssueCode::ParentTraversalArtifactPath
                        }
                    };
                    let message = match violation {
                        ProjectRelativePathViolation::Empty => {
                            "asset artifact path must be present before catalog publication"
                        }
                        ProjectRelativePathViolation::Absolute => {
                            "asset artifact path must be project-relative"
                        }
                        ProjectRelativePathViolation::ParentTraversal => {
                            "asset artifact path must not traverse outside the project"
                        }
                    };
                    report.push(RatificationIssue::error(
                        code,
                        AssetRatificationSubject::Artifact(candidate.artifact_id.raw()),
                        message,
                    ));
                }
            }
            None => {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::MissingArtifactPath,
                    AssetRatificationSubject::Artifact(candidate.artifact_id.raw()),
                    "asset artifact path must be present before catalog publication",
                ));
            }
        }
        if candidate.validity == ArtifactValidity::FailedPreserved
            && candidate.diagnostics.is_empty()
        {
            report.push(RatificationIssue::error(
                AssetRatificationIssueCode::FailedArtifactWithoutDiagnostic,
                AssetRatificationSubject::Artifact(candidate.artifact_id.raw()),
                "failed-preserved artifacts must include diagnostics explaining the failure",
            ));
        }
        report
    }
}

impl Ratifier<AssetCatalog> for AssetCatalogRatifier {
    type Code = AssetRatificationIssueCode;
    type Subject = AssetRatificationSubject;

    fn ratify(&self, candidate: &AssetCatalog) -> RatificationReport<Self::Code, Self::Subject> {
        let mut report = RatificationReport::accepted();
        if candidate.version != ASSET_CATALOG_VERSION_V1 {
            report.push(RatificationIssue::error(
                AssetRatificationIssueCode::UnsupportedCatalogVersion,
                AssetRatificationSubject::Catalog,
                "asset catalog version is not supported by the V1 domain contract",
            ));
        }
        let mut stable_names = BTreeSet::new();
        for record in candidate.records.values() {
            if record.stable_name.trim().is_empty() {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::EmptyStableName,
                    AssetRatificationSubject::Asset(record.asset_id.raw()),
                    "asset stable name must not be empty",
                ));
            }
            if !stable_names.insert(record.stable_name.as_str()) {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::DuplicateStableName,
                    AssetRatificationSubject::Asset(record.asset_id.raw()),
                    "asset stable names must be unique within one catalog",
                ));
            }
            if let Some(source_id) = record.primary_source_id
                && !candidate.sources.contains_key(&source_id)
            {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::MissingPrimarySource,
                    AssetRatificationSubject::Asset(record.asset_id.raw()),
                    "asset primary source must exist in the same catalog",
                ));
            }
            if let Some(source_id) = record.primary_source_id
                && let Some(source) = candidate.sources.get(&source_id)
                && source.asset_id != record.asset_id
            {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::PrimarySourceAssetMismatch,
                    AssetRatificationSubject::Asset(record.asset_id.raw()),
                    "asset primary source must belong to the same asset record",
                ));
            }
        }
        for source in candidate.sources.values() {
            report.merge(ratify_asset_source(source));
            if !candidate.records.contains_key(&source.asset_id) {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::SourceAssetMissing,
                    AssetRatificationSubject::Source(source.source_id.raw()),
                    "asset source must point at an asset record in the same catalog",
                ));
            }
            if let Some(root_id) = source.source_root_id
                && !candidate.source_roots.contains_key(&root_id)
            {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::SourceRootMissing,
                    AssetRatificationSubject::Source(source.source_id.raw()),
                    "asset source root must exist in the same catalog",
                ));
            }
        }
        for artifact in candidate.artifacts.values() {
            report.merge(ratify_asset_artifact(artifact));
            if !candidate.records.contains_key(&artifact.asset_id) {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::ArtifactAssetMissing,
                    AssetRatificationSubject::Artifact(artifact.artifact_id.raw()),
                    "asset artifact must point at an asset record in the same catalog",
                ));
            }
            if let Some(source_id) = artifact.source_id {
                let Some(source) = candidate.sources.get(&source_id) else {
                    report.push(RatificationIssue::error(
                        AssetRatificationIssueCode::ArtifactSourceMissing,
                        AssetRatificationSubject::Artifact(artifact.artifact_id.raw()),
                        "asset artifact source must exist in the same catalog",
                    ));
                    continue;
                };
                if source.asset_id != artifact.asset_id {
                    report.push(RatificationIssue::error(
                        AssetRatificationIssueCode::ArtifactSourceAssetMismatch,
                        AssetRatificationSubject::Artifact(artifact.artifact_id.raw()),
                        "asset artifact source must belong to the artifact asset",
                    ));
                }
            }
        }
        for (asset_id, depends_on) in candidate.dependency_graph.dependency_edges() {
            if asset_id == depends_on {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::SelfDependency,
                    AssetRatificationSubject::Dependency {
                        asset_id: asset_id.raw(),
                        depends_on: depends_on.raw(),
                    },
                    "asset dependency graph must not contain self-dependencies",
                ));
            }
            if !candidate.records.contains_key(&asset_id)
                || !candidate.records.contains_key(&depends_on)
            {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::DependencyAssetMissing,
                    AssetRatificationSubject::Dependency {
                        asset_id: asset_id.raw(),
                        depends_on: depends_on.raw(),
                    },
                    "asset dependency graph edges must reference catalog asset records",
                ));
            }
        }
        report
    }
}

impl Ratifier<ImportPlan> for AssetImportPlanRatifier {
    type Code = AssetRatificationIssueCode;
    type Subject = AssetRatificationSubject;

    fn ratify(&self, candidate: &ImportPlan) -> RatificationReport<Self::Code, Self::Subject> {
        let mut report = RatificationReport::accepted();
        if candidate.expected_artifacts.is_empty() {
            report.push(RatificationIssue::error(
                AssetRatificationIssueCode::ImportPlanWithoutExpectedArtifact,
                AssetRatificationSubject::ImportPlan(candidate.job_id.raw()),
                "deterministic import plans must declare at least one expected artifact",
            ));
        }
        if !candidate
            .expected_artifacts
            .iter()
            .any(|artifact| artifact.required)
        {
            report.push(RatificationIssue::error(
                AssetRatificationIssueCode::ImportPlanWithoutRequiredArtifact,
                AssetRatificationSubject::ImportPlan(candidate.job_id.raw()),
                "deterministic import plans must declare at least one required artifact",
            ));
        }
        for artifact in &candidate.expected_artifacts {
            if artifact.cache_key != candidate.cache_key {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::ImportPlanCacheKeyMismatch,
                    AssetRatificationSubject::ImportPlan(candidate.job_id.raw()),
                    "expected artifact cache key must match the deterministic import plan cache key",
                ));
            }
            if !candidate.settings.supports_artifact_kind(artifact.kind) {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::UnsupportedArtifactKindForImportSettings,
                    AssetRatificationSubject::ImportPlan(candidate.job_id.raw()),
                    "import settings must be compatible with every expected artifact kind",
                ));
            }
        }
        let mut dependencies = BTreeSet::new();
        for dependency in &candidate.dependencies {
            if *dependency == candidate.asset_id {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::ImportPlanSelfDependency,
                    AssetRatificationSubject::ImportPlan(candidate.job_id.raw()),
                    "import plans must not depend on their own asset",
                ));
            }
            if !dependencies.insert(*dependency) {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::DuplicateImportPlanDependency,
                    AssetRatificationSubject::ImportPlan(candidate.job_id.raw()),
                    "import plan dependencies must be unique",
                ));
            }
        }
        report
    }
}

impl Ratifier<AssetProjectCatalogDescriptor> for AssetProjectCatalogDescriptorRatifier {
    type Code = AssetRatificationIssueCode;
    type Subject = AssetRatificationSubject;

    fn ratify(
        &self,
        candidate: &AssetProjectCatalogDescriptor,
    ) -> RatificationReport<Self::Code, Self::Subject> {
        let mut report = RatificationReport::accepted();
        if candidate.version != ASSET_PROJECT_CATALOG_DESCRIPTOR_VERSION_V1 {
            report.push(RatificationIssue::error(
                AssetRatificationIssueCode::UnsupportedProjectCatalogVersion,
                AssetRatificationSubject::ProjectCatalog,
                "asset project catalog descriptor version is not supported",
            ));
        }
        for root in &candidate.source_roots {
            if root.display_name.trim().is_empty() {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::EmptySourceRootName,
                    AssetRatificationSubject::SourceRoot(root.root_id.raw()),
                    "asset source root display name must not be empty",
                ));
            }
        }
        let mut source_root_ids = BTreeSet::new();
        let mut source_root_paths = BTreeSet::new();
        for root in &candidate.source_roots {
            if !source_root_ids.insert(root.root_id) {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::DuplicateSourceRootId,
                    AssetRatificationSubject::SourceRoot(root.root_id.raw()),
                    "asset project catalog source root ids must be unique",
                ));
            }
            let normalized_path = normalize_project_relative_path_for_identity(&root.relative_path);
            if !normalized_path.is_empty() && !source_root_paths.insert(normalized_path) {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::DuplicateSourceRootPath,
                    AssetRatificationSubject::SourceRoot(root.root_id.raw()),
                    "asset project catalog source root paths must be unique",
                ));
            }
            ratify_project_relative_path(
                &mut report,
                AssetRatificationSubject::SourceRoot(root.root_id.raw()),
                &root.relative_path,
                "asset source root path must be project-relative",
            );
        }
        for default in &candidate.import_profile_defaults {
            if default.profile_name.trim().is_empty() {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::EmptyImportProfileName,
                    AssetRatificationSubject::ProjectCatalog,
                    "asset import profile defaults must have a stable profile name",
                ));
            }
            if !default.settings.supports_source_kind(default.asset_kind) {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::ImportProfileSettingsMismatch,
                    AssetRatificationSubject::ProjectCatalog,
                    "asset import profile defaults must use settings compatible with the declared source kind",
                ));
            }
        }
        for path in [
            candidate.artifact_cache_root.as_str(),
            candidate.field_product_cache_root.as_str(),
            candidate.catalog_file_path.as_str(),
        ] {
            ratify_project_relative_path(
                &mut report,
                AssetRatificationSubject::ProjectCatalog,
                path,
                "asset project catalog paths must be project-relative",
            );
        }
        report
    }
}

fn ratify_project_relative_path(
    report: &mut RatificationReport<AssetRatificationIssueCode, AssetRatificationSubject>,
    subject: AssetRatificationSubject,
    path: &str,
    absolute_message: &'static str,
) {
    if let Some(violation) = project_relative_path_violation(path) {
        let code = match violation {
            ProjectRelativePathViolation::Empty => {
                AssetRatificationIssueCode::EmptyProjectCatalogPath
            }
            ProjectRelativePathViolation::Absolute => {
                AssetRatificationIssueCode::AbsoluteProjectCatalogPath
            }
            ProjectRelativePathViolation::ParentTraversal => {
                AssetRatificationIssueCode::ParentTraversalProjectCatalogPath
            }
        };
        let message = match violation {
            ProjectRelativePathViolation::Empty => "asset project catalog paths must not be empty",
            ProjectRelativePathViolation::Absolute => absolute_message,
            ProjectRelativePathViolation::ParentTraversal => {
                "asset project catalog paths must not traverse outside the project"
            }
        };
        report.push(RatificationIssue::error(code, subject, message));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectRelativePathViolation {
    Empty,
    Absolute,
    ParentTraversal,
}

fn project_relative_path_violation(path: &str) -> Option<ProjectRelativePathViolation> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Some(ProjectRelativePathViolation::Empty);
    }
    if trimmed.starts_with('/') || trimmed.starts_with('\\') || trimmed.contains(':') {
        return Some(ProjectRelativePathViolation::Absolute);
    }
    if trimmed
        .split(['/', '\\'])
        .any(|component| component == "..")
    {
        return Some(ProjectRelativePathViolation::ParentTraversal);
    }
    None
}

fn normalize_project_relative_path_for_identity(path: &str) -> String {
    path.trim().replace('\\', "/")
}

pub fn ratify_asset_source(
    candidate: &AssetSourceDescriptor,
) -> RatificationReport<AssetRatificationIssueCode, AssetRatificationSubject> {
    AssetSourceRatifier.ratify(candidate)
}

pub fn ratify_asset_artifact(
    candidate: &AssetArtifactDescriptor,
) -> RatificationReport<AssetRatificationIssueCode, AssetRatificationSubject> {
    AssetArtifactRatifier.ratify(candidate)
}

pub fn ratify_asset_catalog(
    candidate: &AssetCatalog,
) -> RatificationReport<AssetRatificationIssueCode, AssetRatificationSubject> {
    AssetCatalogRatifier.ratify(candidate)
}

pub fn ratify_asset_import_plan(
    candidate: &ImportPlan,
) -> RatificationReport<AssetRatificationIssueCode, AssetRatificationSubject> {
    AssetImportPlanRatifier.ratify(candidate)
}

pub fn ratify_asset_import_plan_against_source(
    plan: &ImportPlan,
    source: &AssetSourceDescriptor,
) -> RatificationReport<AssetRatificationIssueCode, AssetRatificationSubject> {
    let mut report = ratify_asset_import_plan(plan);
    if plan.asset_id != source.asset_id || plan.source_id != source.source_id {
        report.push(RatificationIssue::error(
            AssetRatificationIssueCode::ImportPlanSourceMismatch,
            AssetRatificationSubject::ImportPlan(plan.job_id.raw()),
            "import plan must target the provided source descriptor",
        ));
    }
    if plan.source_hash != source.source_hash {
        report.push(RatificationIssue::error(
            AssetRatificationIssueCode::ImportPlanHashMismatch,
            AssetRatificationSubject::ImportPlan(plan.job_id.raw()),
            "import plan source hash must match the source descriptor revision hash",
        ));
    }
    if !plan.settings.supports_source_kind(source.kind) {
        report.push(RatificationIssue::error(
            AssetRatificationIssueCode::UnsupportedImportSettingsForSource,
            AssetRatificationSubject::ImportPlan(plan.job_id.raw()),
            "import settings must be compatible with the source asset kind",
        ));
    }
    report
}

pub fn ratify_asset_project_catalog_descriptor(
    candidate: &AssetProjectCatalogDescriptor,
) -> RatificationReport<AssetRatificationIssueCode, AssetRatificationSubject> {
    AssetProjectCatalogDescriptorRatifier.ratify(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ArtifactCacheKey, AssetArtifactDescriptor, AssetImportProfileDefault, AssetKind,
        AssetProjectCatalogDescriptor, AssetRecord, AssetSourceDescriptor, AssetSourceRoot,
        AssetSourceRootKind, ExpectedArtifact, FieldProductResolution, ImportSettings, SourceHash,
        asset_artifact_id, asset_id, asset_source_id, asset_source_root_id, import_job_id,
    };

    #[test]
    fn source_ratifier_rejects_absolute_paths() {
        let source = AssetSourceDescriptor::new(
            asset_source_id(1),
            asset_id(1),
            AssetKind::SdfGraph,
            "/tmp/source.ron",
        );

        assert!(ratify_asset_source(&source).has_blocking_issues());
    }

    #[test]
    fn source_ratifier_rejects_windows_rooted_and_parent_traversal_paths() {
        let rooted = AssetSourceDescriptor::new(
            asset_source_id(1),
            asset_id(1),
            AssetKind::SdfGraph,
            "\\assets\\source.ron",
        );
        let traversal = AssetSourceDescriptor::new(
            asset_source_id(2),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/../outside.ron",
        );

        assert!(
            ratify_asset_source(&rooted)
                .issues()
                .iter()
                .any(|issue| issue.code() == &AssetRatificationIssueCode::AbsoluteSourcePath)
        );
        assert!(
            ratify_asset_source(&traversal)
                .issues()
                .iter()
                .any(|issue| issue.code()
                    == &AssetRatificationIssueCode::ParentTraversalSourcePath)
        );
    }

    #[test]
    fn catalog_ratifier_rejects_duplicate_stable_names() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(1),
            "field",
            "Field A",
            AssetKind::SdfGraph,
        ));
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(2),
            "field",
            "Field B",
            AssetKind::SdfGraph,
        ));

        assert!(ratify_asset_catalog(&catalog).has_blocking_issues());
    }

    #[test]
    fn catalog_ratifier_rejects_missing_source_roots_and_dependency_assets() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(1),
            "field",
            "Field",
            AssetKind::SdfGraph,
        ));
        catalog.insert_source(
            AssetSourceDescriptor::new(
                asset_source_id(1),
                asset_id(1),
                AssetKind::SdfGraph,
                "assets/fields/field.ron",
            )
            .with_source_root(asset_source_root_id(9)),
        );
        catalog
            .dependency_graph
            .add_dependency(asset_id(1), asset_id(99));

        let report = ratify_asset_catalog(&catalog);

        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.code() == &AssetRatificationIssueCode::SourceRootMissing)
        );
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.code() == &AssetRatificationIssueCode::DependencyAssetMissing)
        );
    }

    #[test]
    fn catalog_ratifier_composes_source_and_artifact_ratifiers() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(1),
            "field",
            "Field",
            AssetKind::SdfGraph,
        ));
        catalog.insert_source(AssetSourceDescriptor::new(
            asset_source_id(1),
            asset_id(1),
            AssetKind::SdfGraph,
            "../field.ron",
        ));
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(3),
                asset_id(1),
                AssetKind::FormedFieldProduct,
                crate::ArtifactPayloadKind::FormedFieldProduct {
                    product_id: "field".to_string(),
                },
                crate::ArtifactCacheKey::new("field"),
            )
            .with_artifact_path("../artifacts/field.ron"),
        );

        let report = ratify_asset_catalog(&catalog);

        assert!(
            report.issues().iter().any(|issue| issue.code()
                == &AssetRatificationIssueCode::ParentTraversalSourcePath)
        );
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.code()
                    == &AssetRatificationIssueCode::ParentTraversalArtifactPath)
        );
    }

    #[test]
    fn catalog_ratifier_accepts_source_artifact_and_dependency_contracts() {
        let mut catalog = AssetCatalog::new();
        catalog.insert_source_root(AssetSourceRoot::new(
            asset_source_root_id(1),
            AssetSourceRootKind::ProjectAssets,
            "Project assets",
            "assets",
        ));
        catalog.insert_asset_record(
            AssetRecord::new(asset_id(1), "field", "Field", AssetKind::SdfGraph)
                .with_primary_source(asset_source_id(1)),
        );
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(2),
            "material",
            "Material",
            AssetKind::MaterialGraph,
        ));
        catalog.insert_source(
            AssetSourceDescriptor::new(
                asset_source_id(1),
                asset_id(1),
                AssetKind::SdfGraph,
                "assets/fields/field.ron",
            )
            .with_source_root(asset_source_root_id(1)),
        );
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(3),
                asset_id(1),
                AssetKind::FormedFieldProduct,
                crate::ArtifactPayloadKind::FormedFieldProduct {
                    product_id: "field".to_string(),
                },
                crate::ArtifactCacheKey::new("field"),
            )
            .with_source(asset_source_id(1), crate::asset_source_revision_id(1))
            .with_artifact_path(".runenwerk/artifacts/field.ron"),
        );
        catalog
            .dependency_graph
            .add_dependency(asset_id(2), asset_id(1));

        assert!(ratify_asset_catalog(&catalog).is_accepted());
    }

    #[test]
    fn import_plan_ratifier_checks_source_revision_and_settings() {
        let source = AssetSourceDescriptor::new(
            asset_source_id(1),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/fields/field.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let plan = ImportPlan::deterministic(
            import_job_id(4),
            &source,
            ImportSettings::MaterialGraph {
                lowering_target: "preview".to_string(),
            },
            AssetKind::Material,
        );

        let report = ratify_asset_import_plan_against_source(&plan, &source);

        assert!(
            report.issues().iter().any(|issue| issue.code()
                == &AssetRatificationIssueCode::UnsupportedImportSettingsForSource)
        );
    }

    #[test]
    fn import_plan_ratifier_checks_required_artifact_cache_key_dependencies_and_output_kind() {
        let source = AssetSourceDescriptor::new(
            asset_source_id(1),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/fields/field.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let mut plan = ImportPlan::deterministic(
            import_job_id(4),
            &source,
            ImportSettings::SdfGraph {
                resolution: FieldProductResolution::new(32, 32, 1),
            },
            AssetKind::FormedFieldProduct,
        );
        plan.expected_artifacts[0].required = false;
        plan.expected_artifacts.push(ExpectedArtifact {
            kind: AssetKind::Material,
            cache_key: ArtifactCacheKey::new("wrong-cache-key"),
            required: false,
        });
        plan.dependencies = vec![asset_id(1), asset_id(2), asset_id(2)];

        let report = ratify_asset_import_plan(&plan);

        for code in [
            AssetRatificationIssueCode::ImportPlanWithoutRequiredArtifact,
            AssetRatificationIssueCode::ImportPlanCacheKeyMismatch,
            AssetRatificationIssueCode::UnsupportedArtifactKindForImportSettings,
            AssetRatificationIssueCode::ImportPlanSelfDependency,
            AssetRatificationIssueCode::DuplicateImportPlanDependency,
        ] {
            assert!(
                report.issues().iter().any(|issue| issue.code() == &code),
                "missing expected issue {code:?}"
            );
        }
    }

    #[test]
    fn project_catalog_descriptor_ratifier_rejects_absolute_paths() {
        let descriptor = AssetProjectCatalogDescriptor::new(
            [AssetSourceRoot::new(
                asset_source_root_id(1),
                AssetSourceRootKind::ProjectAssets,
                "Project assets",
                "C:/absolute/assets",
            )],
            "/tmp/artifacts",
            ".runenwerk/field-products",
            "assets/catalog.ron",
        )
        .with_import_profile_default(AssetImportProfileDefault::new(
            AssetKind::SdfGraph,
            "default-sdf",
            ImportSettings::SdfGraph {
                resolution: FieldProductResolution::new(32, 32, 1),
            },
        ));

        assert!(ratify_asset_project_catalog_descriptor(&descriptor).has_blocking_issues());
    }

    #[test]
    fn project_catalog_descriptor_ratifier_rejects_duplicate_roots_and_bad_defaults() {
        let descriptor = AssetProjectCatalogDescriptor::new(
            [
                AssetSourceRoot::new(
                    asset_source_root_id(1),
                    AssetSourceRootKind::ProjectAssets,
                    "Project assets",
                    "assets",
                ),
                AssetSourceRoot::new(
                    asset_source_root_id(1),
                    AssetSourceRootKind::GameAssets,
                    "Game assets",
                    "assets",
                ),
            ],
            ".runenwerk/artifacts",
            ".runenwerk/field-products",
            "assets/catalog.ron",
        )
        .with_import_profile_default(AssetImportProfileDefault::new(
            AssetKind::SdfGraph,
            "",
            ImportSettings::MaterialGraph {
                lowering_target: "preview".to_string(),
            },
        ));

        let report = ratify_asset_project_catalog_descriptor(&descriptor);

        for code in [
            AssetRatificationIssueCode::DuplicateSourceRootId,
            AssetRatificationIssueCode::DuplicateSourceRootPath,
            AssetRatificationIssueCode::EmptyImportProfileName,
            AssetRatificationIssueCode::ImportProfileSettingsMismatch,
        ] {
            assert!(
                report.issues().iter().any(|issue| issue.code() == &code),
                "missing expected issue {code:?}"
            );
        }
    }
}
