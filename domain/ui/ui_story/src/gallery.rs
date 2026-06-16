//! Checked-in UI gallery story catalog.

use crate::{
    manifest::{
        UiStoryCategory, UiStoryExpectedOutcome, UiStoryHostProfile, UiStoryManifest,
        UiStoryMountPolicy, UiStorySource, UiStoryThemeProfile, UiStoryViewportProfile,
    },
    registry::UiStoryRegistry,
    report::UiStoryStageKind,
};

pub const RUNENWERK_CONTROL_PACKAGE_ID: &str = "runenwerk.ui.controls@1";
pub const EDITOR_DARK_THEME_ID: &str = "editor.dark";

const BUTTON_REQUIRED_STAGES: &[UiStoryStageKind] = &[
    UiStoryStageKind::Manifest,
    UiStoryStageKind::SourceLoad,
    UiStoryStageKind::SourceParse,
    UiStoryStageKind::ProgramFormation,
    UiStoryStageKind::Compiler,
    UiStoryStageKind::RuntimeView,
    UiStoryStageKind::RenderPrimitives,
    UiStoryStageKind::RenderData,
    UiStoryStageKind::StaticMount,
    UiStoryStageKind::PreviewFrame,
    UiStoryStageKind::MountEligibility,
];

const MISSING_SOURCE_REQUIRED_STAGES: &[UiStoryStageKind] = &[
    UiStoryStageKind::Manifest,
    UiStoryStageKind::SourceLoad,
    UiStoryStageKind::MountEligibility,
];

pub const CHECKED_IN_GALLERY_STORIES: &[UiGalleryStorySpec] = &[
    UiGalleryStorySpec {
        story_id: "ui.gallery.button.basic",
        category: "controls/button",
        title: "Button / Basic",
        source_path: "assets/ui_gallery/button/basic.ron",
        source_id: "assets.ui_gallery.button.basic",
        program_id: "ui.gallery.button.basic",
        required_stages: BUTTON_REQUIRED_STAGES,
        expected_failure: false,
        mount_policy: UiStoryMountPolicy::EligibleWhenPassed,
        host_bools: &[],
    },
    UiGalleryStorySpec {
        story_id: "ui.gallery.button.selected",
        category: "controls/button",
        title: "Button / Selected",
        source_path: "assets/ui_gallery/button/selected.ron",
        source_id: "assets.ui_gallery.button.selected",
        program_id: "ui.gallery.button.selected",
        required_stages: BUTTON_REQUIRED_STAGES,
        expected_failure: false,
        mount_policy: UiStoryMountPolicy::EligibleWhenPassed,
        host_bools: &[UiGalleryHostBool {
            endpoint: "ui_gallery.button.selected.active",
            value: true,
        }],
    },
    UiGalleryStorySpec {
        story_id: "ui.gallery.button.missing_source",
        category: "controls/button/failure",
        title: "Button / Missing Source",
        source_path: "assets/ui_gallery/button/missing.ron",
        source_id: "assets.ui_gallery.button.missing",
        program_id: "ui.gallery.button.missing_source",
        required_stages: MISSING_SOURCE_REQUIRED_STAGES,
        expected_failure: true,
        mount_policy: UiStoryMountPolicy::Never,
        host_bools: &[],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiGalleryHostBool {
    pub endpoint: &'static str,
    pub value: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiGalleryStorySpec {
    pub story_id: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub source_path: &'static str,
    pub source_id: &'static str,
    pub program_id: &'static str,
    pub required_stages: &'static [UiStoryStageKind],
    pub expected_failure: bool,
    pub mount_policy: UiStoryMountPolicy,
    pub host_bools: &'static [UiGalleryHostBool],
}

impl UiGalleryStorySpec {
    pub fn manifest(self) -> UiStoryManifest {
        UiStoryManifest {
            story_id: crate::manifest::UiStoryId::new(self.story_id),
            category: UiStoryCategory::new(self.category),
            title: self.title.to_owned(),
            source: UiStorySource::node(self.source_path, self.source_id),
            program_id: self.program_id.to_owned(),
            control_package: RUNENWERK_CONTROL_PACKAGE_ID.to_owned(),
            host_profile: UiStoryHostProfile::headless(),
            viewport_matrix: vec![UiStoryViewportProfile::new(
                "gallery-default",
                320,
                128,
                1.0,
            )],
            theme_profile: UiStoryThemeProfile::new(EDITOR_DARK_THEME_ID),
            expected: if self.expected_failure {
                UiStoryExpectedOutcome::fail(self.required_stages.to_vec())
            } else {
                UiStoryExpectedOutcome::pass(self.required_stages.to_vec())
            },
            mount_policy: self.mount_policy,
            diagnostic_expectations: Vec::new(),
        }
    }
}

pub fn checked_in_gallery_story_specs() -> &'static [UiGalleryStorySpec] {
    CHECKED_IN_GALLERY_STORIES
}

pub fn checked_in_gallery_manifests() -> Vec<UiStoryManifest> {
    CHECKED_IN_GALLERY_STORIES
        .iter()
        .map(|story| story.manifest())
        .collect()
}

pub fn checked_in_gallery_registry() -> UiStoryRegistry {
    UiStoryRegistry::from_manifests(checked_in_gallery_manifests())
}

pub fn checked_in_gallery_story_spec(story_id: &str) -> Option<&'static UiGalleryStorySpec> {
    CHECKED_IN_GALLERY_STORIES
        .iter()
        .find(|story| story.story_id == story_id)
}

#[cfg(test)]
mod tests {
    use crate::{
        mount::{UiStoryMountEligibility, UiStoryMountEligibilityReason},
        report::{UiStoryStageKind, UiStoryVerdictStatus},
        runner::UiStoryRunner,
    };

    use super::*;

    #[test]
    fn checked_in_gallery_registry_is_deterministic_and_valid() {
        let registry = checked_in_gallery_registry();

        assert!(registry.is_valid());
        assert_eq!(registry.stories().count(), CHECKED_IN_GALLERY_STORIES.len());
        assert!(registry.contains(&crate::manifest::UiStoryId::new("ui.gallery.button.basic")));
    }

    #[test]
    fn checked_in_pass_stories_require_rendering_proof_stages() {
        let basic = checked_in_gallery_story_spec("ui.gallery.button.basic")
            .expect("basic story should exist")
            .manifest();

        assert!(
            basic
                .expected
                .required_stages
                .contains(&UiStoryStageKind::RenderPrimitives)
        );
        assert!(
            basic
                .expected
                .required_stages
                .contains(&UiStoryStageKind::RenderData)
        );
        assert!(
            basic
                .expected
                .required_stages
                .contains(&UiStoryStageKind::StaticMount)
        );
        assert!(
            basic
                .expected
                .required_stages
                .contains(&UiStoryStageKind::PreviewFrame)
        );
        assert_eq!(basic.mount_policy, UiStoryMountPolicy::EligibleWhenPassed);
    }

    #[test]
    fn missing_source_story_is_first_class_expected_failure() {
        let registry = checked_in_gallery_registry();
        let runner = UiStoryRunner::new(&registry);

        let report = runner.run_story_with_stage_reports(
            &registry.run_request("ui.gallery.button.missing_source"),
            [crate::report::UiStoryStageReport::failed(
                UiStoryStageKind::SourceLoad,
                vec![crate::report::UiStoryDiagnostic::error(
                    "ui_gallery.story.source.read_failed",
                    "source fixture is intentionally absent",
                    UiStoryStageKind::SourceLoad,
                )],
            )],
        );
        let eligibility = UiStoryMountEligibility::from_report(&report);

        assert_eq!(report.verdict.status, UiStoryVerdictStatus::Passed);
        assert_eq!(
            eligibility.reason,
            UiStoryMountEligibilityReason::ExpectedFailureStory
        );
    }
}
