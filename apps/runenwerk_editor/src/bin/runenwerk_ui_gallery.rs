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
    use ui_story::UiStoryMountEligibility;
    use ui_theme::ThemeTokens;

    #[test]
    fn story_inspection_runs_checked_in_gallery_reports() {
        let report = runenwerk_editor::runtime::inspect_checked_in_gallery_stories();
        let rendered = report.render_text();

        assert!(report.passed(), "{rendered}");
        assert!(rendered.contains("ui.gallery.button.basic [passed]"));
        assert!(rendered.contains("ui.gallery.button.selected [passed]"));
        assert!(rendered.contains("ui.gallery.button.missing_source [passed]"));
        assert!(rendered.contains("stage render_primitives: passed"));
        assert!(rendered.contains("stage render_data: passed"));
        assert!(rendered.contains("stage static_mount: passed"));
        assert!(rendered.contains("stage preview_frame: passed"));
        assert!(rendered.contains("stage source_load: failed"));
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
        let eligible_reports = gallery
            .story_reports()
            .iter()
            .filter(|report| UiStoryMountEligibility::from_report(report).eligible)
            .count();
        assert_eq!(eligible_reports, 2);
        let expected_failure_report = gallery
            .story_reports()
            .iter()
            .find(|report| report.story_id.as_str() == "ui.gallery.button.missing_source")
            .expect("checked-in expected-failure story should be present");
        assert!(expected_failure_report.passed());
        assert!(!UiStoryMountEligibility::from_report(expected_failure_report).eligible);
    }

    #[test]
    fn story_gallery_resource_refuses_failed_report_even_with_mounted_frame() {
        let valid_execution = runenwerk_editor::runtime::run_checked_in_gallery_stories()
            .into_iter()
            .find(|execution| execution.mounted_frame.is_some())
            .expect("at least one checked-in story should produce a mounted frame");
        let failed_report = ui_story::UiStoryRunReport::unknown_story(
            ui_story::UiStoryId::new("ui.gallery.button.failed_bypass_attempt"),
            ui_story::UiStoryDiagnostic::error(
                "ui_gallery.story.synthetic_failure",
                "failed report must block mounted frame publication",
                ui_story::UiStoryStageKind::Manifest,
            ),
        );

        let gallery = runenwerk_editor::runtime::UiGalleryResource::from_story_executions(
            vec![runenwerk_editor::runtime::UiGalleryStoryExecution {
                report: failed_report,
                button_report: valid_execution.button_report,
                mounted_frame: valid_execution.mounted_frame,
            }],
            None,
        );

        assert!(!gallery.passed());
        assert_eq!(gallery.button_count(), 0);
        assert!(gallery.frame().is_none());
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
