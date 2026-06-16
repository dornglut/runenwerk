//! Mount eligibility derived from story run reports.

use serde::{Deserialize, Serialize};

use crate::{
    manifest::{UiStoryExpectedVerdict, UiStoryId, UiStoryMountPolicy},
    report::{
        UiStoryDiagnostic, UiStoryRunReport, UiStoryStageKind, UiStoryStageReport,
        UiStoryStageStatus, UiStoryVerdictStatus,
    },
};

pub const DIAGNOSTIC_MOUNT_MANIFEST_MISSING: &str = "ui.story.mount.manifest_missing";
pub const DIAGNOSTIC_MOUNT_REPORT_FAILED: &str = "ui.story.mount.report_failed";
pub const DIAGNOSTIC_MOUNT_EXPECTED_FAILURE: &str = "ui.story.mount.expected_failure";
pub const DIAGNOSTIC_MOUNT_GALLERY_ONLY: &str = "ui.story.mount.gallery_only";
pub const DIAGNOSTIC_MOUNT_NEVER: &str = "ui.story.mount.never";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryMountEligibilityReason {
    Eligible,
    ManifestMissing,
    ReportFailed,
    ExpectedFailureStory,
    GalleryOnly,
    Never,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryMountEligibility {
    #[serde(default)]
    pub story_id: Option<UiStoryId>,
    pub eligible: bool,
    pub reason: UiStoryMountEligibilityReason,
    #[serde(default)]
    pub diagnostics: Vec<UiStoryDiagnostic>,
}

impl UiStoryMountEligibility {
    pub fn from_report(report: &UiStoryRunReport) -> Self {
        let Some(manifest) = report.manifest.as_ref() else {
            return Self::denied(
                None,
                UiStoryMountEligibilityReason::ManifestMissing,
                UiStoryDiagnostic::error(
                    DIAGNOSTIC_MOUNT_MANIFEST_MISSING,
                    "mount eligibility requires a parsed story manifest",
                    UiStoryStageKind::MountEligibility,
                ),
            );
        };

        if report.verdict.status != UiStoryVerdictStatus::Passed {
            return Self::denied(
                Some(manifest.story_id.clone()),
                UiStoryMountEligibilityReason::ReportFailed,
                UiStoryDiagnostic::error(
                    DIAGNOSTIC_MOUNT_REPORT_FAILED,
                    "mount eligibility requires a passing story run report",
                    UiStoryStageKind::MountEligibility,
                ),
            );
        }

        if manifest.expected.verdict == UiStoryExpectedVerdict::Fail {
            return Self::denied(
                Some(manifest.story_id.clone()),
                UiStoryMountEligibilityReason::ExpectedFailureStory,
                UiStoryDiagnostic::info(
                    DIAGNOSTIC_MOUNT_EXPECTED_FAILURE,
                    "expected-failure stories are never mount eligible",
                    UiStoryStageKind::MountEligibility,
                ),
            );
        }

        match manifest.mount_policy {
            UiStoryMountPolicy::EligibleWhenPassed => Self {
                story_id: Some(manifest.story_id.clone()),
                eligible: true,
                reason: UiStoryMountEligibilityReason::Eligible,
                diagnostics: Vec::new(),
            },
            UiStoryMountPolicy::GalleryOnly => Self::denied(
                Some(manifest.story_id.clone()),
                UiStoryMountEligibilityReason::GalleryOnly,
                UiStoryDiagnostic::info(
                    DIAGNOSTIC_MOUNT_GALLERY_ONLY,
                    "story is valid for gallery proof only",
                    UiStoryStageKind::MountEligibility,
                ),
            ),
            UiStoryMountPolicy::Never => Self::denied(
                Some(manifest.story_id.clone()),
                UiStoryMountEligibilityReason::Never,
                UiStoryDiagnostic::info(
                    DIAGNOSTIC_MOUNT_NEVER,
                    "story manifest declares mount_policy never",
                    UiStoryStageKind::MountEligibility,
                ),
            ),
        }
    }

    pub fn stage_report(&self) -> UiStoryStageReport {
        let has_error = self.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == crate::report::UiStoryDiagnosticSeverity::Error
        });
        UiStoryStageReport {
            stage: UiStoryStageKind::MountEligibility,
            status: if has_error {
                UiStoryStageStatus::Failed
            } else {
                UiStoryStageStatus::Passed
            },
            diagnostics: self.diagnostics.clone(),
            elapsed_micros: None,
        }
    }

    fn denied(
        story_id: Option<UiStoryId>,
        reason: UiStoryMountEligibilityReason,
        diagnostic: UiStoryDiagnostic,
    ) -> Self {
        Self {
            story_id,
            eligible: false,
            reason,
            diagnostics: vec![diagnostic],
        }
    }
}
