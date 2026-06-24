use anyhow::{Result, bail};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--inspect-stories") | Some("inspect-stories") => {
            let report = runenwerk_editor::runtime::inspect_checked_in_gallery_stories();
            println!("{}", report.render_text());
            if report.passed() {
                Ok(())
            } else {
                bail!("one or more UI gallery stories failed unexpectedly")
            }
        }
        Some("--help") | Some("-h") => {
            println!("runenwerk_ui_gallery [--inspect-stories]");
            Ok(())
        }
        Some(arg) => bail!("unknown argument {arg}; expected --inspect-stories"),
        None => runenwerk_editor::runtime::run_ui_gallery_workbench(),
    }
}

#[cfg(test)]
mod tests {
    use engine::plugins::render::UiFontAtlasResource;
    use ui_math::{UiRect, UiSize};
    use ui_render_data::{UiFrame, UiPrimitive, UiSurfaceId};
    use ui_story::{
        UiStoryDiagnostic, UiStoryDiagnosticOrigin, UiStoryDiagnosticSubject, UiStoryId,
        UiStoryMountBlockReasonV2, UiStoryMountDecisionV2, UiStoryMountPolicyV2, UiStoryOutcomeV2,
        UiStoryWorkflowReportV2, checked_in_story_registry_v2,
    };
    use ui_theme::ThemeTokens;

    #[test]
    fn editor_gallery_v2_builds_checked_in_story_registry() {
        let registry =
            checked_in_story_registry_v2().expect("checked-in V2 story registry should build");

        assert_eq!(registry.len(), 3);
        assert!(registry.contains(&UiStoryId::new("ui.gallery.button.basic")));
        assert!(registry.contains(&UiStoryId::new("ui.gallery.button.selected")));
        assert!(registry.contains(&UiStoryId::new("ui.gallery.button.missing_source")));
    }

    #[test]
    fn story_inspection_runs_checked_in_gallery_reports() {
        let report = runenwerk_editor::runtime::inspect_checked_in_gallery_stories();
        let rendered = report.render_text();

        assert!(report.passed(), "{rendered}");
        assert!(rendered.contains("ui.gallery.button.basic: outcome=Passed"));
        assert!(rendered.contains("ui.gallery.button.selected: outcome=Passed"));
        assert!(
            rendered.contains("ui.gallery.button.missing_source: outcome=ExpectedFailureMatched")
        );
        assert!(rendered.contains("mount_allowed=true"));
        assert!(rendered.contains("mount_allowed=false"));
        assert!(!rendered.contains("UiStoryStageReport"));
        assert!(!rendered.contains("stage "));
    }

    #[test]
    fn gallery_resource_consumes_story_reports_before_visual_output() {
        let atlas = UiFontAtlasResource::default();
        let gallery =
            runenwerk_editor::runtime::UiGalleryResource::from_checked_in_stories_for_render_target(
                UiSize::new(720.0, 240.0),
                &ThemeTokens::default(),
                &atlas,
            );

        assert!(gallery.passed());
        assert_eq!(gallery.story_reports().len(), 3);
        assert_eq!(gallery.button_count(), 2);
        let frame = gallery
            .frame()
            .expect("eligible stories should compose a frame");
        assert_eq!(frame.surfaces.len(), 1);
        assert_eq!(frame.surfaces[0].id, UiSurfaceId(0));
        let button_rects = preview_button_rects(frame);
        assert_eq!(button_rects.len(), 2);
        assert!(
            button_rects[0].intersect(button_rects[1]).is_none(),
            "preview button rects should not overlap: {:?}",
            button_rects,
        );

        let passed_reports = gallery
            .story_reports()
            .iter()
            .filter(|report| report.outcome() == UiStoryOutcomeV2::Passed)
            .count();
        assert_eq!(passed_reports, 2);
        let expected_failure_report = gallery
            .story_reports()
            .iter()
            .find(|report| report.story_id.as_str() == "ui.gallery.button.missing_source")
            .expect("checked-in expected-failure story should be present");
        assert_eq!(
            expected_failure_report.outcome(),
            UiStoryOutcomeV2::ExpectedFailureMatched
        );
    }

    #[test]
    fn editor_gallery_v2_basic_button_story_passes() {
        let execution = checked_in_execution("ui.gallery.button.basic");

        assert_eq!(execution.report.outcome(), UiStoryOutcomeV2::Passed);
        assert!(execution.mount_decision.allowed);
        assert_eq!(
            execution.mount_decision.reason,
            UiStoryMountBlockReasonV2::Allowed
        );
        assert!(execution.mounted_frame.is_some());
    }

    #[test]
    fn editor_gallery_v2_selected_button_story_passes() {
        let execution = checked_in_execution("ui.gallery.button.selected");

        assert_eq!(execution.report.outcome(), UiStoryOutcomeV2::Passed);
        assert!(execution.mount_decision.allowed);
        assert_eq!(
            execution.mount_decision.reason,
            UiStoryMountBlockReasonV2::Allowed
        );
        assert!(execution.mounted_frame.is_some());
    }

    #[test]
    fn editor_gallery_v2_missing_source_matches_expected_failure() {
        let execution = checked_in_execution("ui.gallery.button.missing_source");

        assert_eq!(
            execution.report.outcome(),
            UiStoryOutcomeV2::ExpectedFailureMatched
        );
        assert!(execution.report.has_blockers());
        assert!(execution
            .report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == "ui_gallery.story.source.read_failed"));
    }

    #[test]
    fn editor_gallery_v2_expected_failure_is_not_mountable() {
        let execution = checked_in_execution("ui.gallery.button.missing_source");

        assert!(!execution.mount_decision.allowed);
        assert_eq!(
            execution.mount_decision.reason,
            UiStoryMountBlockReasonV2::BlockedExpectedFailure
        );
        assert!(execution.mounted_frame.is_none());
    }

    #[test]
    fn editor_gallery_v2_failed_report_blocks_preview_publication() {
        let valid_execution = runenwerk_editor::runtime::run_checked_in_gallery_stories()
            .into_iter()
            .find(|execution| execution.mounted_frame.is_some())
            .expect("at least one checked-in story should produce a mounted frame");
        let failed_report = failed_report("ui.gallery.button.failed_bypass_attempt");

        let gallery = runenwerk_editor::runtime::UiGalleryResource::from_story_executions(
            vec![runenwerk_editor::runtime::UiGalleryStoryExecution {
                report: failed_report,
                mount_decision: UiStoryMountDecisionV2::blocked(
                    UiStoryMountBlockReasonV2::BlockedFailedOutcome,
                ),
                mount_policy: UiStoryMountPolicyV2::EligibleWhenPassed,
                button_report: valid_execution.button_report,
                mounted_frame: valid_execution.mounted_frame,
            }],
            None,
        );

        assert!(!gallery.passed());
        assert_eq!(gallery.button_count(), 0);
        assert!(gallery.frame().is_none());
    }

    #[test]
    fn editor_gallery_v2_mount_policy_blocks_passed_report_preview_publication() {
        let valid_execution = runenwerk_editor::runtime::run_checked_in_gallery_stories()
            .into_iter()
            .find(|execution| execution.mounted_frame.is_some())
            .expect("at least one checked-in story should produce a mounted frame");

        let gallery = runenwerk_editor::runtime::UiGalleryResource::from_story_executions(
            vec![runenwerk_editor::runtime::UiGalleryStoryExecution {
                report: valid_execution.report,
                mount_decision: UiStoryMountDecisionV2::blocked(
                    UiStoryMountBlockReasonV2::BlockedPolicyGalleryOnly,
                ),
                mount_policy: UiStoryMountPolicyV2::GalleryOnly,
                button_report: valid_execution.button_report,
                mounted_frame: valid_execution.mounted_frame,
            }],
            None,
        );

        assert!(gallery.passed());
        assert!(
            gallery.button_count() > 0,
            "runtime button facts should still be retained for a passed story"
        );
        assert!(
            gallery.frame().is_none(),
            "policy-blocked stories must not publish mounted preview frames"
        );
    }

    #[test]
    fn editor_gallery_v2_missing_actual_frame_blocks_preview_publication() {
        let valid_execution = runenwerk_editor::runtime::run_checked_in_gallery_stories()
            .into_iter()
            .find(|execution| execution.mounted_frame.is_some())
            .expect("at least one checked-in story should produce a mounted frame");

        let gallery = runenwerk_editor::runtime::UiGalleryResource::from_story_executions(
            vec![runenwerk_editor::runtime::UiGalleryStoryExecution {
                report: valid_execution.report,
                mount_decision: UiStoryMountDecisionV2::allowed(),
                mount_policy: UiStoryMountPolicyV2::EligibleWhenPassed,
                button_report: valid_execution.button_report,
                mounted_frame: None,
            }],
            None,
        );

        assert!(!gallery.passed());
        assert!(
            gallery.button_count() > 0,
            "runtime button facts should still be retained for a passed story"
        );
        assert!(gallery.frame().is_none());
        assert!(gallery
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == "ui_gallery.story.preview_frame.missing"));
    }

    #[test]
    fn editor_gallery_v2_missing_required_evidence_blocks_preview_publication() {
        let valid_execution = runenwerk_editor::runtime::run_checked_in_gallery_stories()
            .into_iter()
            .find(|execution| execution.mounted_frame.is_some())
            .expect("at least one checked-in story should produce a mounted frame");
        let missing_required_report = failed_report("ui.gallery.button.missing_required");

        let gallery = runenwerk_editor::runtime::UiGalleryResource::from_story_executions(
            vec![runenwerk_editor::runtime::UiGalleryStoryExecution {
                report: missing_required_report,
                mount_decision: UiStoryMountDecisionV2::blocked(
                    UiStoryMountBlockReasonV2::BlockedFailedOutcome,
                ),
                mount_policy: UiStoryMountPolicyV2::EligibleWhenPassed,
                button_report: valid_execution.button_report,
                mounted_frame: valid_execution.mounted_frame,
            }],
            None,
        );

        assert!(!gallery.passed());
        assert_eq!(gallery.button_count(), 0);
        assert!(gallery.frame().is_none());
    }

    #[test]
    fn editor_gallery_v2_does_not_use_old_stage_report_types_for_new_flow() {
        let report = runenwerk_editor::runtime::inspect_checked_in_gallery_stories();
        let rendered = report.render_text();

        assert!(report.passed());
        assert!(!rendered.contains("UiStoryStageKind"));
        assert!(!rendered.contains("UiStoryStageReport"));
        assert!(!rendered.contains("UiStoryRunReport"));
    }

    fn checked_in_execution(story_id: &str) -> runenwerk_editor::runtime::UiGalleryStoryExecution {
        runenwerk_editor::runtime::run_checked_in_gallery_stories()
            .into_iter()
            .find(|execution| execution.report.story_id.as_str() == story_id)
            .unwrap_or_else(|| panic!("checked-in story `{story_id}` should execute"))
    }

    fn failed_report(story_id: &str) -> UiStoryWorkflowReportV2 {
        let story_id = UiStoryId::new(story_id);
        UiStoryWorkflowReportV2 {
            story_id: story_id.clone(),
            workflow_graph: None,
            node_reports: Vec::new(),
            diagnostics: vec![UiStoryDiagnostic::error(
                "ui_gallery.story.synthetic_failure",
                UiStoryDiagnosticOrigin::Report,
                UiStoryDiagnosticSubject::Story(story_id),
                "failed report must block mounted frame publication",
            )],
            outcome: UiStoryOutcomeV2::Failed,
        }
    }

    fn preview_button_rects(frame: &UiFrame) -> Vec<UiRect> {
        frame
            .surfaces
            .iter()
            .flat_map(|surface| &surface.layers)
            .flat_map(|layer| &layer.primitives)
            .filter_map(|primitive| match primitive {
                UiPrimitive::Rect(rect) if rect.rect.width >= 80.0 && rect.rect.height >= 24.0 => {
                    Some(rect.rect)
                }
                _ => None,
            })
            .collect()
    }
}
