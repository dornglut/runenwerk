//! Checked-in UI gallery story catalog.

use crate::{
    manifest::{UiStoryId, UiStoryManifest, UiStoryManifestParseError},
    registry::{UiStoryRegistry, UiStoryRegistryDiagnostic},
};

pub const RUNENWERK_CONTROL_PACKAGE_ID: &str = "runenwerk.ui.controls@1";
pub const EDITOR_DARK_THEME_ID: &str = "editor.dark";

pub const CHECKED_IN_GALLERY_STORY_MANIFESTS: &[UiCheckedInGalleryStoryManifest] = &[
    UiCheckedInGalleryStoryManifest {
        path: "assets/ui_gallery/stories/controls/button/basic.story.ron",
        source: include_str!(
            "../../../../assets/ui_gallery/stories/controls/button/basic.story.ron"
        ),
    },
    UiCheckedInGalleryStoryManifest {
        path: "assets/ui_gallery/stories/controls/button/selected.story.ron",
        source: include_str!(
            "../../../../assets/ui_gallery/stories/controls/button/selected.story.ron"
        ),
    },
    UiCheckedInGalleryStoryManifest {
        path: "assets/ui_gallery/stories/controls/button/missing_source.failure.story.ron",
        source: include_str!(
            "../../../../assets/ui_gallery/stories/controls/button/missing_source.failure.story.ron"
        ),
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiCheckedInGalleryStoryManifest {
    pub path: &'static str,
    pub source: &'static str,
}

impl UiCheckedInGalleryStoryManifest {
    pub fn parse(self) -> Result<UiStoryManifest, UiStoryManifestParseError> {
        UiStoryManifest::from_ron_str(self.source)
    }
}

pub fn checked_in_gallery_story_manifest_sources() -> &'static [UiCheckedInGalleryStoryManifest] {
    CHECKED_IN_GALLERY_STORY_MANIFESTS
}

pub fn checked_in_gallery_manifests() -> Vec<UiStoryManifest> {
    CHECKED_IN_GALLERY_STORY_MANIFESTS
        .iter()
        .map(|manifest| {
            manifest.parse().unwrap_or_else(|error| {
                panic!("failed to parse {}: {}", manifest.path, error.message)
            })
        })
        .collect()
}

pub fn checked_in_gallery_registry() -> UiStoryRegistry {
    let mut registry = UiStoryRegistry::new();
    for manifest_source in CHECKED_IN_GALLERY_STORY_MANIFESTS {
        match manifest_source.parse() {
            Ok(manifest) => registry.insert(manifest),
            Err(error) => registry.push_diagnostic(UiStoryRegistryDiagnostic::new(
                error.code,
                format!(
                    "failed to parse {}: {}",
                    manifest_source.path, error.message
                ),
            )),
        }
    }
    registry
}

pub fn checked_in_gallery_manifest(story_id: &str) -> Option<UiStoryManifest> {
    checked_in_gallery_manifests()
        .into_iter()
        .find(|story| story.story_id == UiStoryId::new(story_id))
}

#[cfg(test)]
mod tests {
    use crate::{
        manifest::UiStoryMountPolicy,
        mount::{UiStoryMountEligibility, UiStoryMountEligibilityReason},
        report::{UiStoryStageKind, UiStoryVerdictStatus},
        runner::UiStoryRunner,
    };

    use super::*;

    #[test]
    fn checked_in_gallery_registry_is_deterministic_and_valid() {
        let registry = checked_in_gallery_registry();

        assert!(registry.is_valid(), "{:?}", registry.diagnostics());
        assert_eq!(
            registry.stories().count(),
            CHECKED_IN_GALLERY_STORY_MANIFESTS.len()
        );
        assert!(registry.contains(&crate::manifest::UiStoryId::new("ui.gallery.button.basic")));
    }

    #[test]
    fn checked_in_pass_stories_require_rendering_proof_stages() {
        let basic = checked_in_gallery_manifest("ui.gallery.button.basic")
            .expect("basic story should exist");

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

    #[test]
    fn checked_in_story_manifests_round_trip_and_reject_future_schema() {
        for manifest_source in checked_in_gallery_story_manifest_sources() {
            let manifest = manifest_source.parse().expect("manifest should parse");
            let serialized = manifest
                .to_ron_string_pretty()
                .expect("manifest should serialize");
            let reparsed =
                UiStoryManifest::from_ron_str(&serialized).expect("manifest should reparse");

            assert_eq!(manifest, reparsed, "{}", manifest_source.path);
        }

        let future_schema = checked_in_gallery_story_manifest_sources()[0]
            .source
            .replacen("schema_version: 1", "schema_version: 999", 1);
        let error = UiStoryManifest::from_ron_str(&future_schema)
            .expect_err("future schema should be rejected");

        assert_eq!(
            error.code,
            crate::manifest::DIAGNOSTIC_MANIFEST_SCHEMA_UNSUPPORTED
        );
    }
}
