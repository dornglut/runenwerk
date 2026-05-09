use std::collections::BTreeSet;

use foundation_ratification::{RatificationIssue, RatificationReport, Ratifier};
use serde::{Deserialize, Serialize};

use crate::{ArtifactValidity, AssetArtifactDescriptor, AssetCatalog, AssetSourceDescriptor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetRatificationIssueCode {
    EmptySourcePath,
    AbsoluteSourcePath,
    EmptyStableName,
    MissingPrimarySource,
    MissingArtifactPath,
    FailedArtifactWithoutDiagnostic,
    DuplicateStableName,
    ArtifactAssetMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetRatificationSubject {
    Source(u64),
    Artifact(u64),
    Asset(u64),
    Catalog,
}

pub struct AssetSourceRatifier;
pub struct AssetArtifactRatifier;
pub struct AssetCatalogRatifier;

impl Ratifier<AssetSourceDescriptor> for AssetSourceRatifier {
    type Code = AssetRatificationIssueCode;
    type Subject = AssetRatificationSubject;

    fn ratify(
        &self,
        candidate: &AssetSourceDescriptor,
    ) -> RatificationReport<Self::Code, Self::Subject> {
        let mut report = RatificationReport::accepted();
        let trimmed = candidate.relative_path.trim();
        if trimmed.is_empty() {
            report.push(RatificationIssue::error(
                AssetRatificationIssueCode::EmptySourcePath,
                AssetRatificationSubject::Source(candidate.source_id.raw()),
                "asset source path must not be empty",
            ));
        }
        if trimmed.starts_with('/') || trimmed.contains(':') {
            report.push(RatificationIssue::error(
                AssetRatificationIssueCode::AbsoluteSourcePath,
                AssetRatificationSubject::Source(candidate.source_id.raw()),
                "asset source path must be project-relative",
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
        if candidate
            .artifact_path
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            report.push(RatificationIssue::error(
                AssetRatificationIssueCode::MissingArtifactPath,
                AssetRatificationSubject::Artifact(candidate.artifact_id.raw()),
                "asset artifact path must be present before catalog publication",
            ));
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
        }
        for artifact in candidate.artifacts.values() {
            if !candidate.records.contains_key(&artifact.asset_id) {
                report.push(RatificationIssue::error(
                    AssetRatificationIssueCode::ArtifactAssetMissing,
                    AssetRatificationSubject::Artifact(artifact.artifact_id.raw()),
                    "asset artifact must point at an asset record in the same catalog",
                ));
            }
        }
        report
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetKind, AssetRecord, AssetSourceDescriptor, asset_id, asset_source_id};

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
}
