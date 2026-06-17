//! Story runner orchestration over registered manifests.

use serde::{Deserialize, Serialize};

use std::collections::BTreeSet;

use crate::{
    manifest::{UiStoryId, UiStoryManifestDiagnostic},
    mount::UiStoryMountEligibility,
    registry::UiStoryRegistry,
    report::{
        UiStoryDiagnostic, UiStoryDiagnosticSeverity, UiStoryRunReport, UiStoryStageKind,
        UiStoryStageReport,
    },
};

pub const DIAGNOSTIC_RUNNER_UNKNOWN_STORY: &str = "ui.story.runner.unknown_story";
pub const DIAGNOSTIC_RUNNER_MISSING_STAGE_PROOF: &str = "ui.story.runner.missing_stage_proof";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryRunRequest {
    pub story_id: UiStoryId,
}

impl UiStoryRunRequest {
    pub fn new(story_id: UiStoryId) -> Self {
        Self { story_id }
    }
}

#[derive(Clone, Debug)]
pub struct UiStoryRunner<'a> {
    registry: &'a UiStoryRegistry,
}

impl<'a> UiStoryRunner<'a> {
    pub fn new(registry: &'a UiStoryRegistry) -> Self {
        Self { registry }
    }

    pub fn run_story(&self, request: &UiStoryRunRequest) -> UiStoryRunReport {
        self.run_story_with_stage_reports(request, [])
    }

    pub fn run_story_with_stage_reports(
        &self,
        request: &UiStoryRunRequest,
        stage_reports: impl IntoIterator<Item = UiStoryStageReport>,
    ) -> UiStoryRunReport {
        let Some(manifest) = self.registry.get(&request.story_id) else {
            return UiStoryRunReport::unknown_story(
                request.story_id.clone(),
                UiStoryDiagnostic::error(
                    DIAGNOSTIC_RUNNER_UNKNOWN_STORY,
                    format!("story {} is not registered", request.story_id.as_str()),
                    UiStoryStageKind::Manifest,
                ),
            );
        };

        let manifest_diagnostics = manifest.validate();
        let mut stages = if manifest_diagnostics.is_empty() {
            vec![UiStoryStageReport::passed(UiStoryStageKind::Manifest)]
        } else {
            vec![UiStoryStageReport::failed(
                UiStoryStageKind::Manifest,
                manifest_diagnostics
                    .into_iter()
                    .map(manifest_diagnostic_to_story_diagnostic)
                    .collect(),
            )]
        };
        let mut proven_stages = stages
            .iter()
            .map(|stage| stage.stage)
            .collect::<BTreeSet<_>>();

        for stage_report in stage_reports {
            proven_stages.insert(stage_report.stage);
            stages.push(stage_report);
        }

        for required_stage in manifest.required_stage_kinds() {
            if required_stage == UiStoryStageKind::Manifest
                || required_stage == UiStoryStageKind::MountEligibility
            {
                continue;
            }
            if proven_stages.contains(&required_stage) {
                continue;
            }
            stages.push(UiStoryStageReport::missing_proof(
                required_stage,
                UiStoryDiagnostic::new(
                    DIAGNOSTIC_RUNNER_MISSING_STAGE_PROOF,
                    format!(
                        "stage {:?} has no proof producer in this runner slice",
                        required_stage
                    ),
                    required_stage,
                    UiStoryDiagnosticSeverity::Error,
                ),
            ));
        }

        let base_report = UiStoryRunReport::from_manifest_and_stages(manifest.clone(), stages);
        let mount_eligibility = UiStoryMountEligibility::from_report(&base_report);
        let mut stages_with_mount = base_report.stages;
        stages_with_mount.push(mount_eligibility.stage_report());
        UiStoryRunReport::from_manifest_and_stages(manifest.clone(), stages_with_mount)
    }
}

fn manifest_diagnostic_to_story_diagnostic(
    diagnostic: UiStoryManifestDiagnostic,
) -> UiStoryDiagnostic {
    UiStoryDiagnostic::error(
        diagnostic.code,
        diagnostic.message,
        UiStoryStageKind::Manifest,
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        manifest::{
            UI_STORY_MANIFEST_SCHEMA_VERSION, UiStoryCategory, UiStoryCompatibilityPolicy,
            UiStoryDiagnosticExpectation, UiStoryExpectedOutcome, UiStoryHostProfile,
            UiStoryManifest, UiStoryMigrationPolicy, UiStoryMountPolicy, UiStorySource,
            UiStoryThemeProfile, UiStoryViewportProfile,
        },
        mount::UiStoryMountEligibilityReason,
        proof::{
            UI_STORY_PROOF_CONTRACT_VERSION, UiStoryProofContract, UiStoryProofRequirement,
            UiStoryProofSubject,
        },
        report::{
            DIAGNOSTIC_EXPECTED_FAILURE_DIAGNOSTIC_MISSING,
            DIAGNOSTIC_EXPECTED_FAILURE_UNEXPECTED_ERROR, UiStoryDiagnostic,
            UiStoryDiagnosticSeverity, UiStoryStageKind, UiStoryStageReport, UiStoryVerdictStatus,
        },
    };

    use super::*;

    #[test]
    fn valid_manifest_reports_mount_eligible_when_required_stages_pass() {
        let manifest = basic_manifest(UiStoryMountPolicy::EligibleWhenPassed);
        let registry = UiStoryRegistry::from_manifests([manifest]);
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story(&registry.run_request("ui.controls.button.basic"));
        let eligibility = UiStoryMountEligibility::from_report(&report);

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Passed);
        assert_eq!(
            report.stage(UiStoryStageKind::Manifest).unwrap().status,
            crate::report::UiStoryStageStatus::Passed
        );
        assert!(eligibility.eligible);
        assert_eq!(eligibility.reason, UiStoryMountEligibilityReason::Eligible);
    }

    #[test]
    fn invalid_manifest_fails_closed() {
        let mut manifest = basic_manifest(UiStoryMountPolicy::EligibleWhenPassed);
        manifest.source.path.clear();
        let registry = UiStoryRegistry::from_manifests([manifest]);
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story(&registry.run_request("ui.controls.button.basic"));
        let eligibility = UiStoryMountEligibility::from_report(&report);

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Failed);
        assert_eq!(
            report.verdict.first_failing_stage,
            Some(UiStoryStageKind::Manifest)
        );
        assert!(!eligibility.eligible);
        assert_eq!(
            eligibility.reason,
            UiStoryMountEligibilityReason::ReportFailed
        );
    }

    #[test]
    fn missing_required_stage_is_not_success_shaped() {
        let mut manifest = basic_manifest(UiStoryMountPolicy::EligibleWhenPassed);
        manifest
            .expected
            .required_stages
            .push(UiStoryStageKind::StaticMount);
        let registry = UiStoryRegistry::from_manifests([manifest]);
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story(&registry.run_request("ui.controls.button.basic"));

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Failed);
        assert_eq!(
            report.verdict.first_failing_stage,
            Some(UiStoryStageKind::StaticMount)
        );
    }

    #[test]
    fn duplicate_story_ids_record_registry_diagnostic() {
        let manifest = basic_manifest(UiStoryMountPolicy::GalleryOnly);

        let registry = UiStoryRegistry::from_manifests([manifest.clone(), manifest]);

        assert!(!registry.is_valid());
        assert_eq!(
            registry.diagnostics()[0].code,
            crate::registry::DIAGNOSTIC_REGISTRY_DUPLICATE_ID
        );
    }

    #[test]
    fn expected_failure_requires_declared_diagnostic() {
        let mut manifest = expected_failure_manifest();
        manifest.diagnostic_expectations.clear();
        let registry = UiStoryRegistry::from_manifests([manifest]);
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story_with_stage_reports(
            &registry.run_request("ui.controls.button.missing_source"),
            [source_load_failure(
                "ui_gallery.story.source.read_failed",
                UiStoryDiagnosticSeverity::Error,
            )],
        );

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Failed);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code
                == crate::manifest::DIAGNOSTIC_MANIFEST_EXPECTED_FAILURE_EXPECTATION_MISSING
        }));
    }

    #[test]
    fn expected_failure_fails_on_wrong_code() {
        let registry = UiStoryRegistry::from_manifests([expected_failure_manifest()]);
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story_with_stage_reports(
            &registry.run_request("ui.controls.button.missing_source"),
            [source_load_failure(
                "ui_gallery.story.source.other",
                UiStoryDiagnosticSeverity::Error,
            )],
        );

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Failed);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DIAGNOSTIC_EXPECTED_FAILURE_DIAGNOSTIC_MISSING
        }));
    }

    #[test]
    fn expected_failure_fails_on_wrong_stage_or_proof_key() {
        let mut manifest = expected_failure_manifest();
        manifest.diagnostic_expectations = vec![UiStoryDiagnosticExpectation::for_stage_error(
            UiStoryStageKind::SourceParse,
            "ui_gallery.story.source.read_failed",
        )];
        let registry = UiStoryRegistry::from_manifests([manifest]);
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story_with_stage_reports(
            &registry.run_request("ui.controls.button.missing_source"),
            [source_load_failure(
                "ui_gallery.story.source.read_failed",
                UiStoryDiagnosticSeverity::Error,
            )],
        );

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Failed);
        assert!(
            report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == DIAGNOSTIC_EXPECTED_FAILURE_UNEXPECTED_ERROR
            })
        );
    }

    #[test]
    fn expected_failure_fails_on_wrong_severity() {
        let registry = UiStoryRegistry::from_manifests([expected_failure_manifest()]);
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story_with_stage_reports(
            &registry.run_request("ui.controls.button.missing_source"),
            [source_load_failure(
                "ui_gallery.story.source.read_failed",
                UiStoryDiagnosticSeverity::Warning,
            )],
        );

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Failed);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DIAGNOSTIC_EXPECTED_FAILURE_DIAGNOSTIC_MISSING
        }));
    }

    #[test]
    fn expected_failure_fails_on_extra_unexpected_error() {
        let registry = UiStoryRegistry::from_manifests([expected_failure_manifest()]);
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story_with_stage_reports(
            &registry.run_request("ui.controls.button.missing_source"),
            [
                source_load_failure(
                    "ui_gallery.story.source.read_failed",
                    UiStoryDiagnosticSeverity::Error,
                ),
                UiStoryStageReport::failed(
                    UiStoryStageKind::SourceParse,
                    vec![UiStoryDiagnostic::error(
                        "ui_gallery.story.source.parse_failed",
                        "unexpected parse error",
                        UiStoryStageKind::SourceParse,
                    )],
                ),
            ],
        );

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Failed);
        assert!(
            report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == DIAGNOSTIC_EXPECTED_FAILURE_UNEXPECTED_ERROR
            })
        );
    }

    #[test]
    fn expected_failure_passes_only_declared_diagnostic_and_never_mounts() {
        let registry = UiStoryRegistry::from_manifests([expected_failure_manifest()]);
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story_with_stage_reports(
            &registry.run_request("ui.controls.button.missing_source"),
            [source_load_failure(
                "ui_gallery.story.source.read_failed",
                UiStoryDiagnosticSeverity::Error,
            )],
        );
        let eligibility = crate::mount::UiStoryMountEligibility::from_report(&report);

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Passed);
        assert_eq!(
            eligibility.reason,
            UiStoryMountEligibilityReason::ExpectedFailureStory
        );
    }

    fn basic_manifest(mount_policy: UiStoryMountPolicy) -> UiStoryManifest {
        UiStoryManifest {
            schema_version: UI_STORY_MANIFEST_SCHEMA_VERSION,
            story_revision: 1,
            proof_contract_version: UI_STORY_PROOF_CONTRACT_VERSION,
            compatibility_policy: UiStoryCompatibilityPolicy::ExactVersion,
            migration_policy: UiStoryMigrationPolicy::RejectUnsupported,
            story_id: UiStoryId::new("ui.controls.button.basic"),
            category: UiStoryCategory::new("controls/button"),
            title: "Button / Basic".to_owned(),
            source: UiStorySource::node(
                "assets/ui_gallery/button/basic.ron",
                "assets.ui_gallery.button.basic",
            ),
            program_id: "ui.gallery.button.basic".to_owned(),
            control_package: "runenwerk.ui.controls@1".to_owned(),
            host_profile: UiStoryHostProfile::headless(),
            viewport_matrix: vec![UiStoryViewportProfile::new("default", 240, 96, 1.0)],
            theme_profile: UiStoryThemeProfile::new("editor.dark"),
            expected: UiStoryExpectedOutcome::pass([
                UiStoryStageKind::Manifest,
                UiStoryStageKind::MountEligibility,
            ]),
            mount_policy,
            diagnostic_expectations: Vec::new(),
            host_inputs: Vec::new(),
            proof_contract: UiStoryProofContract::new([
                UiStoryProofRequirement::required_stage(
                    UiStoryStageKind::Manifest,
                    UiStoryProofSubject::Story,
                ),
                UiStoryProofRequirement::required_stage(
                    UiStoryStageKind::MountEligibility,
                    UiStoryProofSubject::MountEligibility,
                ),
            ]),
        }
    }

    fn expected_failure_manifest() -> UiStoryManifest {
        UiStoryManifest {
            schema_version: UI_STORY_MANIFEST_SCHEMA_VERSION,
            story_revision: 1,
            proof_contract_version: UI_STORY_PROOF_CONTRACT_VERSION,
            compatibility_policy: UiStoryCompatibilityPolicy::ExactVersion,
            migration_policy: UiStoryMigrationPolicy::RejectUnsupported,
            story_id: UiStoryId::new("ui.controls.button.missing_source"),
            category: UiStoryCategory::new("controls/button/failure"),
            title: "Button / Missing Source".to_owned(),
            source: UiStorySource::node(
                "assets/ui_gallery/button/missing.ron",
                "assets.ui_gallery.button.missing",
            ),
            program_id: "ui.gallery.button.missing_source".to_owned(),
            control_package: "runenwerk.ui.controls@1".to_owned(),
            host_profile: UiStoryHostProfile::headless(),
            viewport_matrix: vec![UiStoryViewportProfile::new("default", 240, 96, 1.0)],
            theme_profile: UiStoryThemeProfile::new("editor.dark"),
            expected: UiStoryExpectedOutcome::fail([
                UiStoryStageKind::Manifest,
                UiStoryStageKind::SourceLoad,
                UiStoryStageKind::MountEligibility,
            ]),
            mount_policy: UiStoryMountPolicy::Never,
            diagnostic_expectations: vec![UiStoryDiagnosticExpectation::for_stage_error(
                UiStoryStageKind::SourceLoad,
                "ui_gallery.story.source.read_failed",
            )],
            host_inputs: Vec::new(),
            proof_contract: UiStoryProofContract::new([
                UiStoryProofRequirement::required_stage(
                    UiStoryStageKind::Manifest,
                    UiStoryProofSubject::Story,
                ),
                UiStoryProofRequirement::required_stage(
                    UiStoryStageKind::SourceLoad,
                    UiStoryProofSubject::Source,
                ),
                UiStoryProofRequirement::required_stage(
                    UiStoryStageKind::MountEligibility,
                    UiStoryProofSubject::MountEligibility,
                ),
            ]),
        }
    }

    fn source_load_failure(
        code: &'static str,
        severity: UiStoryDiagnosticSeverity,
    ) -> UiStoryStageReport {
        UiStoryStageReport::failed(
            UiStoryStageKind::SourceLoad,
            vec![UiStoryDiagnostic::new(
                code,
                "source fixture is intentionally absent",
                UiStoryStageKind::SourceLoad,
                severity,
            )],
        )
    }
}
