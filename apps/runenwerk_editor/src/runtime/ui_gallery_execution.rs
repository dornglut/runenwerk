use std::{fs, path::PathBuf};

use engine::plugins::render::{DEFAULT_EDITOR_FONT_ID, UiFontAtlasResource};
use ui_compiler::UiCompiler;
use ui_controls::ControlPackageRegistrySnapshot;
use ui_definition::UiNodeDefinition;
use ui_headless_render_data::UiHeadlessRenderDataReport;
use ui_math::UiSize;
use ui_program_lowering::form_ui_program_report_from_node_with_registry_snapshot;
use ui_render_data::UiFrame;
use ui_render_primitives::UiRenderPrimitiveReport;
use ui_runtime_view::{ButtonRuntimeHostData, ButtonRuntimeViewReport, UiRuntimeView};
use ui_static_mount::UiStaticMountReport;
use ui_story::{UiStoryDiagnostic, UiStoryDiagnosticSeverity, UiStoryId, UiStoryManifestV2, UiStoryMountBlockReasonV2, UiStoryMountDecisionV2, UiStoryMountPolicyV2, UiStoryOutcomeV2, UiStoryRunRequestV2, UiStoryRunnerV2, UiStoryWorkflowReportV2, ValidatedUiStoryRegistryV2, WORKFLOW_SOURCE_LOAD_ONLY};
use ui_theme::ThemeTokens;

use super::ui_gallery_diagnostics::{button_runtime_severity, runtime_artifact_severity, runtime_view_severity};
use super::ui_gallery_story_evidence as story_evidence;

#[derive(Clone, Debug, PartialEq)]
pub struct UiGalleryStoryExecution {
    pub report: UiStoryWorkflowReportV2,
    pub mount_decision: UiStoryMountDecisionV2,
    pub mount_policy: UiStoryMountPolicyV2,
    pub button_report: Option<ButtonRuntimeViewReport>,
    pub mounted_frame: Option<UiFrame>,
}

pub(super) fn execute_gallery_story(story: &UiStoryManifestV2, _registry: &ValidatedUiStoryRegistryV2, runner: &UiStoryRunnerV2<'_>, snapshot: &ControlPackageRegistrySnapshot, size: UiSize, theme: &ThemeTokens, atlas: &UiFontAtlasResource) -> UiGalleryStoryExecution {
    let request = UiStoryRunRequestV2::new(story.story_id.clone());
    let mut run = match runner.begin(request) {
        Ok(run) => run,
        Err(result) => return finish_failed_begin(story, result.into_report(story.expected_outcome.clone())),
    };

    let mut button_report = None;
    let mut mounted_frame = None;
    let Some(source) = load_story_source(story, &mut run) else { return finish_gallery_story_execution(story, run, button_report, mounted_frame); };
    if story.workflow_profile_id.as_str() == WORKFLOW_SOURCE_LOAD_ONLY { return finish_gallery_story_execution(story, run, button_report, mounted_frame); }
    let Some(node) = parse_story_node(story, &source, &mut run) else { return finish_gallery_story_execution(story, run, button_report, mounted_frame); };

    let formation_report = form_ui_program_report_from_node_with_registry_snapshot(story.program_id.as_str(), story.source.source_id.as_str(), &node, snapshot);
    let formation_diagnostics = formation_report.diagnostics.iter().map(|diagnostic| story_evidence::PROGRAM_FORMATION.diagnostic(diagnostic.code.clone(), diagnostic.message.clone(), UiStoryDiagnosticSeverity::Error)).collect::<Vec<_>>();
    run.record(story_evidence::PROGRAM_FORMATION.result(formation_report.passed(), formation_diagnostics));
    if !formation_report.passed() { return finish_gallery_story_execution(story, run, button_report, mounted_frame); }

    let compiler_report = UiCompiler.compile_report(&formation_report.program);
    let compiler_diagnostics = compiler_report.artifact.manifest.diagnostics.iter().map(|diagnostic| story_evidence::COMPILER.diagnostic(diagnostic.code.clone(), diagnostic.message.clone(), runtime_artifact_severity(diagnostic.severity))).collect::<Vec<_>>();
    run.record(story_evidence::COMPILER.result(compiler_report.passed(), compiler_diagnostics));
    if !compiler_report.passed() { return finish_gallery_story_execution(story, run, button_report, mounted_frame); }

    let runtime_report = UiRuntimeView::from_artifact_report(&compiler_report.artifact);
    if !runtime_report.passed() {
        let diagnostics = runtime_report.view.diagnostics.iter().map(|diagnostic| story_evidence::RUNTIME_VIEW.diagnostic(diagnostic.code.clone(), diagnostic.message.clone(), runtime_view_severity(diagnostic.severity))).collect::<Vec<_>>();
        run.record(story_evidence::RUNTIME_VIEW.result(false, diagnostics));
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    }

    let button_runtime_report = ButtonRuntimeViewReport::from_runtime_view_report_with_host_data(&runtime_report, &ButtonRuntimeHostData::new());
    let button_diagnostics = button_runtime_report.diagnostics.iter().map(|diagnostic| story_evidence::RUNTIME_VIEW.diagnostic(diagnostic.code.clone(), diagnostic.message.clone(), button_runtime_severity(diagnostic.severity))).collect::<Vec<_>>();
    run.record(story_evidence::RUNTIME_VIEW.result(button_runtime_report.passed(), button_diagnostics));
    if button_runtime_report.passed() { button_report = Some(button_runtime_report.clone()); } else { return finish_gallery_story_execution(story, run, button_report, mounted_frame); }

    let primitive_report = UiRenderPrimitiveReport::from_button_report(button_runtime_report.clone(), size, theme, atlas, DEFAULT_EDITOR_FONT_ID);
    run.record(story_evidence::render_primitive_report(&primitive_report));
    if !primitive_report.passed() { return finish_gallery_story_execution(story, run, button_report, mounted_frame); }

    let render_data_report = UiHeadlessRenderDataReport::from_render_primitive_report(&primitive_report);
    run.record(story_evidence::render_data_report(&render_data_report));
    if !render_data_report.passed() { return finish_gallery_story_execution(story, run, button_report, mounted_frame); }

    let mount_report = UiStaticMountReport::from_render_data_report(&render_data_report);
    run.record(story_evidence::static_mount_report(&mount_report));
    if let Some(mounted) = mount_report.mounted_frame() {
        mounted_frame = Some(mounted.frame.clone());
        run.record(story_evidence::PREVIEW_FRAME.passed());
    } else {
        run.record(story_evidence::PREVIEW_FRAME.failed(vec![story_evidence::PREVIEW_FRAME.diagnostic("ui_gallery.story.preview_frame.missing", "static mount did not produce a mounted preview frame", UiStoryDiagnosticSeverity::Error)]));
    }

    finish_gallery_story_execution(story, run, button_report, mounted_frame)
}

fn finish_failed_begin(story: &UiStoryManifestV2, report: UiStoryWorkflowReportV2) -> UiGalleryStoryExecution {
    let mount_decision = UiStoryMountDecisionV2::from_report(&report, story.mount_policy);
    UiGalleryStoryExecution { report, mount_decision, mount_policy: story.mount_policy, button_report: None, mounted_frame: None }
}

fn finish_gallery_story_execution(story: &UiStoryManifestV2, run: ui_story::UiStoryWorkflowRunV2, button_report: Option<ButtonRuntimeViewReport>, mounted_frame: Option<UiFrame>) -> UiGalleryStoryExecution {
    let report = run.finish().into_report(story.expected_outcome.clone());
    let mount_decision = UiStoryMountDecisionV2::from_report(&report, story.mount_policy);
    UiGalleryStoryExecution { report, mount_decision, mount_policy: story.mount_policy, button_report, mounted_frame }
}

pub(super) fn registry_failure_execution(diagnostic: UiStoryDiagnostic) -> UiGalleryStoryExecution {
    let report = UiStoryWorkflowReportV2 { story_id: UiStoryId::new("checked_in_story_registry_v2"), workflow_graph: None, node_reports: Vec::new(), diagnostics: vec![diagnostic], outcome: UiStoryOutcomeV2::InvalidManifest };
    let mount_decision = UiStoryMountDecisionV2::blocked(UiStoryMountBlockReasonV2::BlockedInvalidWorkflow);
    UiGalleryStoryExecution { report, mount_decision, mount_policy: UiStoryMountPolicyV2::Never, button_report: None, mounted_frame: None }
}

fn load_story_source(story: &UiStoryManifestV2, run: &mut ui_story::UiStoryWorkflowRunV2) -> Option<String> {
    let path = repo_root().join(&story.source.path);
    match fs::read_to_string(&path) {
        Ok(source) => { run.record(story_evidence::SOURCE_LOAD.passed()); Some(source) }
        Err(error) => { run.record(story_evidence::SOURCE_LOAD.failed(vec![story_evidence::SOURCE_LOAD.diagnostic("ui_gallery.story.source.read_failed", format!("failed to read {}: {error}", path.display()), UiStoryDiagnosticSeverity::Error)])); None }
    }
}

fn parse_story_node(story: &UiStoryManifestV2, source: &str, run: &mut ui_story::UiStoryWorkflowRunV2) -> Option<UiNodeDefinition> {
    match ron::from_str(source) {
        Ok(node) => { run.record(story_evidence::SOURCE_PARSE.passed()); Some(node) }
        Err(error) => { run.record(story_evidence::SOURCE_PARSE.failed(vec![story_evidence::SOURCE_PARSE.diagnostic("ui_gallery.story.source.parse_failed", format!("failed to parse {}: {error}", story.source.path), UiStoryDiagnosticSeverity::Error)])); None }
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().and_then(|path| path.parent()).expect("runenwerk_editor should live under apps/runenwerk_editor").to_path_buf()
}
