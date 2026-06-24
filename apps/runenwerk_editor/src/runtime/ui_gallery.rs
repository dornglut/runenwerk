//! Standalone artifact-backed UI gallery host.

use std::{fs, path::PathBuf};

use engine::plugins::render::{
    DEFAULT_EDITOR_FONT_ID, UiFontAtlasResource, UiFrameProducerId, UiFrameRoute,
    UiFrameSubmission, UiFrameSubmissionOrder, UiFrameSubmissionRegistryResource,
};
use engine::prelude::*;
use ui_compiler::UiCompiler;
use ui_controls::{ControlPackageRegistry, runenwerk_control_package};
use ui_definition::UiNodeDefinition;
use ui_headless_render_data::UiHeadlessRenderDataReport;
use ui_math::{UiPoint, UiRect, UiSize};
use ui_program_lowering::form_ui_program_report_from_node_with_registry_snapshot;
use ui_render_data::{UiFrame, UiFrameFragment, UiFramePlacement, compose_frame_fragments};
use ui_render_primitives::UiRenderPrimitiveReport;
use ui_runtime_view::{ButtonRuntimeHostData, ButtonRuntimeViewReport, UiRuntimeView};
use ui_static_mount::UiStaticMountReport;
use ui_story::{
    NODE_COMPILER, NODE_PREVIEW_FRAME, NODE_PROGRAM_FORMATION, NODE_RENDER_DATA,
    NODE_RENDER_PRIMITIVES, NODE_RUNTIME_VIEW, NODE_SOURCE_LOAD, NODE_SOURCE_PARSE,
    NODE_STATIC_MOUNT, UiStoryCliReportV2, UiStoryDiagnostic, UiStoryDiagnosticOrigin,
    UiStoryDiagnosticSeverity, UiStoryDiagnosticSubject, UiStoryEvidence, UiStoryEvidenceKey,
    UiStoryEvidenceProducerId, UiStoryEvidenceStatus, UiStoryId, UiStoryManifestV2,
    UiStoryMountBlockReasonV2, UiStoryMountDecisionV2, UiStoryMountPolicyV2, UiStoryOutcomeV2,
    UiStoryRunRequestV2, UiStoryRunnerV2, UiStoryWorkflowNodeId, UiStoryWorkflowReportV2,
    ValidatedUiStoryRegistryV2, WORKFLOW_SOURCE_LOAD_ONLY, checked_in_story_registry_v2,
};
use ui_theme::ThemeTokens;

pub const UI_GALLERY_UI_PRODUCER_ID: UiFrameProducerId = ui_frame_producer_id(5_101);
const GALLERY_PREVIEW_TILE_WIDTH: f32 = 320.0;
const GALLERY_PREVIEW_TILE_HEIGHT: f32 = 128.0;
const GALLERY_PREVIEW_PADDING: f32 = 16.0;
const GALLERY_PREVIEW_GAP: f32 = 12.0;

const PRODUCER_SOURCE_LOADER: &str = "runenwerk_editor.ui_gallery.source_loader";
const PRODUCER_SOURCE_PARSER: &str = "runenwerk_editor.ui_gallery.source_parser";
const PRODUCER_PROGRAM_FORMATION: &str = "runenwerk_editor.ui_gallery.program_formation";
const PRODUCER_COMPILER: &str = "runenwerk_editor.ui_gallery.compiler";
const PRODUCER_RUNTIME_VIEW: &str = "runenwerk_editor.ui_gallery.runtime_view";
const PRODUCER_RENDER_PRIMITIVES: &str = "runenwerk_editor.ui_gallery.render_primitives";
const PRODUCER_RENDER_DATA: &str = "runenwerk_editor.ui_gallery.render_data";
const PRODUCER_STATIC_MOUNT: &str = "runenwerk_editor.ui_gallery.static_mount";
const PRODUCER_PREVIEW_FRAME: &str = "runenwerk_editor.ui_gallery.preview_frame";

const EVIDENCE_SOURCE_LOAD: &str = "ui.gallery.source_load";
const EVIDENCE_SOURCE_PARSE: &str = "ui.gallery.source_parse";
const EVIDENCE_PROGRAM_FORMATION: &str = "ui.gallery.program_formation";
const EVIDENCE_COMPILER: &str = "ui.gallery.compiler";
const EVIDENCE_RUNTIME_VIEW: &str = "ui.gallery.runtime_view";
const EVIDENCE_RENDER_PRIMITIVES: &str = "ui.gallery.render_primitives";
const EVIDENCE_RENDER_DATA: &str = "ui.gallery.render_data";
const EVIDENCE_STATIC_MOUNT: &str = "ui.gallery.static_mount";
const EVIDENCE_PREVIEW_FRAME: &str = "ui.gallery.preview_frame";

const fn ui_frame_producer_id(raw: u64) -> UiFrameProducerId {
    match UiFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("ui frame producer id constants must be non-zero"),
    }
}

pub struct UiGalleryPlugin;

impl Plugin for UiGalleryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiGalleryResource>();
        app.add_systems(Update, submit_ui_gallery_frame_system);
    }
}

#[derive(Clone, Debug, ecs::Component, ecs::Resource)]
pub struct UiGalleryResource {
    button_report: ButtonRuntimeViewReport,
    story_reports: Vec<UiStoryWorkflowReportV2>,
    diagnostics: Vec<UiGalleryDiagnostic>,
    frame: Option<UiFrame>,
    prepared_size: Option<UiSize>,
    diagnostics_logged: bool,
}

impl Default for UiGalleryResource {
    fn default() -> Self {
        Self::from_checked_in_stories()
    }
}

impl UiGalleryResource {
    pub fn from_checked_in_fixtures() -> Self {
        Self::from_checked_in_stories()
    }

    pub fn from_checked_in_stories() -> Self {
        let atlas = UiFontAtlasResource::default();
        Self::from_checked_in_stories_for_render_target(
            default_gallery_proof_size(),
            &ThemeTokens::default(),
            &atlas,
        )
    }

    pub fn from_checked_in_stories_for_render_target(
        size: UiSize,
        theme: &ThemeTokens,
        atlas: &UiFontAtlasResource,
    ) -> Self {
        let executions = run_checked_in_gallery_stories_for_render_target(
            default_gallery_proof_size(),
            theme,
            atlas,
        );
        Self::from_story_executions(executions, Some(size))
    }

    pub fn from_story_executions(
        executions: Vec<UiGalleryStoryExecution>,
        prepared_size: Option<UiSize>,
    ) -> Self {
        let mut diagnostics = Vec::new();
        let mut button_report = ButtonRuntimeViewReport::default();
        let mut mounted_previews = Vec::new();
        let mut story_reports = Vec::new();

        for execution in executions {
            let report_blocks_gallery = !execution.report.outcome().is_green();
            append_story_report_diagnostics(
                &execution.report,
                &mut diagnostics,
                report_blocks_gallery,
            );

            if let Some(report) = execution.button_report {
                if !report_blocks_gallery {
                    button_report.buttons.extend(report.buttons.clone());
                }
                diagnostics.extend(report.diagnostics.into_iter().map(|diagnostic| {
                    UiGalleryDiagnostic {
                        stage: UiGalleryStage::WorkflowNode(NODE_RUNTIME_VIEW.to_owned()),
                        story_id: Some(execution.report.story_id.as_str().to_owned()),
                        code: diagnostic.code,
                        message: diagnostic.message,
                        severity: button_gallery_severity(diagnostic.severity),
                        source_map_index: diagnostic.source_map_index,
                        blocks_gallery: report_blocks_gallery,
                    }
                }));
            }

            if execution.report.outcome() == UiStoryOutcomeV2::Passed
                && execution.mount_decision.allowed
            {
                if let Some(mounted_frame) = execution.mounted_frame {
                    mounted_previews.push(mounted_frame);
                } else {
                    diagnostics.push(UiGalleryDiagnostic {
                        stage: UiGalleryStage::WorkflowNode(NODE_PREVIEW_FRAME.to_owned()),
                        story_id: Some(execution.report.story_id.as_str().to_owned()),
                        code: "ui_gallery.story.preview_frame.missing".to_owned(),
                        message: "passed story report did not carry a mounted preview frame"
                            .to_owned(),
                        severity: UiGalleryDiagnosticSeverity::Error,
                        source_map_index: None,
                        blocks_gallery: true,
                    });
                }
            }

            story_reports.push(execution.report);
        }
        let frame = compose_gallery_preview_frame(
            prepared_size.unwrap_or_else(default_gallery_proof_size),
            &mounted_previews,
        );

        Self {
            button_report,
            story_reports,
            diagnostics,
            frame,
            prepared_size,
            diagnostics_logged: false,
        }
    }

    pub fn passed(&self) -> bool {
        !self.has_blocking_diagnostics()
    }

    pub fn frame(&self) -> Option<&UiFrame> {
        self.frame.as_ref()
    }

    pub fn diagnostics(&self) -> &[UiGalleryDiagnostic] {
        &self.diagnostics
    }

    pub fn story_reports(&self) -> &[UiStoryWorkflowReportV2] {
        &self.story_reports
    }

    pub fn button_count(&self) -> usize {
        self.button_report.buttons.len()
    }

    fn has_blocking_diagnostics(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| {
            diagnostic.blocks_gallery && diagnostic.severity == UiGalleryDiagnosticSeverity::Error
        })
    }

    fn prepare_frame_if_needed(
        &mut self,
        size: UiSize,
        theme: &ThemeTokens,
        atlas: &UiFontAtlasResource,
    ) {
        if self.prepared_size == Some(size) {
            return;
        }

        *self = Self::from_checked_in_stories_for_render_target(size, theme, atlas);
    }

    fn log_diagnostics_once(&mut self) {
        if self.diagnostics_logged {
            return;
        }
        self.diagnostics_logged = true;

        if self.diagnostics.is_empty() {
            eprintln!(
                "[ui_gallery] prepared {} buttons from {} story reports",
                self.button_count(),
                self.story_reports.len()
            );
            return;
        }

        for diagnostic in &self.diagnostics {
            eprintln!(
                "[ui_gallery][{}][{}] {}{}{}",
                diagnostic.stage.as_str(),
                diagnostic.severity.as_str(),
                diagnostic.code,
                diagnostic
                    .story_id
                    .as_ref()
                    .map(|id| format!(" story={id}"))
                    .unwrap_or_default(),
                diagnostic
                    .source_map_index
                    .map(|index| format!(" source_map_index={index}"))
                    .unwrap_or_default()
            );
            eprintln!("[ui_gallery] {}", diagnostic.message);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiGalleryStoryExecution {
    pub report: UiStoryWorkflowReportV2,
    pub mount_decision: UiStoryMountDecisionV2,
    pub mount_policy: UiStoryMountPolicyV2,
    pub button_report: Option<ButtonRuntimeViewReport>,
    pub mounted_frame: Option<UiFrame>,
}

pub fn inspect_checked_in_gallery_stories() -> UiStoryCliReportV2 {
    let executions = run_checked_in_gallery_stories();
    UiStoryCliReportV2::from_reports(
        executions
            .iter()
            .map(|execution| (&execution.report, execution.mount_policy)),
    )
}

pub fn run_checked_in_gallery_stories() -> Vec<UiGalleryStoryExecution> {
    let atlas = UiFontAtlasResource::default();
    run_checked_in_gallery_stories_for_render_target(
        default_gallery_proof_size(),
        &ThemeTokens::default(),
        &atlas,
    )
}

pub fn run_checked_in_gallery_stories_for_render_target(
    size: UiSize,
    theme: &ThemeTokens,
    atlas: &UiFontAtlasResource,
) -> Vec<UiGalleryStoryExecution> {
    let registry = match checked_in_story_registry_v2() {
        Ok(registry) => registry,
        Err(report) => {
            return report
                .diagnostics
                .into_iter()
                .map(registry_failure_execution)
                .collect();
        }
    };

    let runner = UiStoryRunnerV2::new(&registry);
    let control_registry = ControlPackageRegistry::new()
        .with_package(runenwerk_control_package())
        .expect("runenwerk controls package should register");
    let snapshot = control_registry.snapshot();

    registry
        .stories()
        .map(|story| execute_gallery_story(story, &registry, &runner, &snapshot, size, theme, atlas))
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiGalleryDiagnostic {
    pub stage: UiGalleryStage,
    pub story_id: Option<String>,
    pub code: String,
    pub message: String,
    pub severity: UiGalleryDiagnosticSeverity,
    pub source_map_index: Option<u32>,
    pub blocks_gallery: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiGalleryStage {
    Story,
    WorkflowNode(String),
    RenderPrimitives,
    RenderData,
    StaticMount,
}

impl UiGalleryStage {
    fn as_str(&self) -> &str {
        match self {
            Self::Story => "story",
            Self::WorkflowNode(node_id) => node_id.as_str(),
            Self::RenderPrimitives => "render_primitives",
            Self::RenderData => "render_data",
            Self::StaticMount => "static_mount",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGalleryDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl UiGalleryDiagnosticSeverity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

pub fn submit_ui_gallery_frame_system(
    window: Res<WindowState>,
    atlas: Res<UiFontAtlasResource>,
    mut gallery: ResMut<UiGalleryResource>,
    mut submissions: ResMut<UiFrameSubmissionRegistryResource>,
) {
    let size = UiSize::new(window.size_px.0 as f32, window.size_px.1 as f32);
    gallery.prepare_frame_if_needed(size, &ThemeTokens::default(), &atlas);
    gallery.log_diagnostics_once();

    if let Some(frame) = gallery.frame().cloned() {
        submissions.replace(
            UiFrameSubmission::new(UI_GALLERY_UI_PRODUCER_ID)
                .with_route(UiFrameRoute::Screen)
                .with_order(UiFrameSubmissionOrder::new(10, 0))
                .with_frame(frame),
        );
    } else {
        submissions.remove(&UI_GALLERY_UI_PRODUCER_ID);
    }
}

fn execute_gallery_story(
    story: &UiStoryManifestV2,
    _registry: &ValidatedUiStoryRegistryV2,
    runner: &UiStoryRunnerV2<'_>,
    snapshot: &ui_controls::ControlPackageRegistrySnapshot,
    size: UiSize,
    theme: &ThemeTokens,
    atlas: &UiFontAtlasResource,
) -> UiGalleryStoryExecution {
    let request = UiStoryRunRequestV2::new(story.story_id.clone());
    let mut run = match runner.begin(request) {
        Ok(run) => run,
        Err(result) => {
            let report = result.into_report(story.expected_outcome.clone());
            let mount_decision = UiStoryMountDecisionV2::from_report(&report, story.mount_policy);
            return UiGalleryStoryExecution {
                report,
                mount_decision,
                mount_policy: story.mount_policy,
                button_report: None,
                mounted_frame: None,
            };
        }
    };

    let mut button_report = None;
    let mut mounted_frame = None;

    let Some(source) = load_story_source(story, &mut run) else {
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    };

    if story.workflow_profile_id.as_str() == WORKFLOW_SOURCE_LOAD_ONLY {
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    }

    let Some(node) = parse_story_node(story, &source, &mut run) else {
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    };

    let formation_report = form_ui_program_report_from_node_with_registry_snapshot(
        story.program_id.as_str(),
        story.source.source_id.as_str(),
        &node,
        snapshot,
    );
    let formation_diagnostics = formation_report
        .diagnostics
        .iter()
        .map(|diagnostic| {
            story_diagnostic(
                NODE_PROGRAM_FORMATION,
                PRODUCER_PROGRAM_FORMATION,
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                UiStoryDiagnosticSeverity::Error,
            )
        })
        .collect::<Vec<_>>();
    run.record(evidence_from_result(
        NODE_PROGRAM_FORMATION,
        PRODUCER_PROGRAM_FORMATION,
        EVIDENCE_PROGRAM_FORMATION,
        formation_report.passed(),
        formation_diagnostics,
    ));
    if !formation_report.passed() {
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    }

    let compiler_report = UiCompiler.compile_report(&formation_report.program);
    let compiler_diagnostics = compiler_report
        .artifact
        .manifest
        .diagnostics
        .iter()
        .map(|diagnostic| {
            story_diagnostic(
                NODE_COMPILER,
                PRODUCER_COMPILER,
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                runtime_artifact_severity(diagnostic.severity),
            )
        })
        .collect::<Vec<_>>();
    run.record(evidence_from_result(
        NODE_COMPILER,
        PRODUCER_COMPILER,
        EVIDENCE_COMPILER,
        compiler_report.passed(),
        compiler_diagnostics,
    ));
    if !compiler_report.passed() {
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    }

    let runtime_report = UiRuntimeView::from_artifact_report(&compiler_report.artifact);
    if !runtime_report.passed() {
        let runtime_diagnostics = runtime_report
            .view
            .diagnostics
            .iter()
            .map(|diagnostic| {
                story_diagnostic(
                    NODE_RUNTIME_VIEW,
                    PRODUCER_RUNTIME_VIEW,
                    diagnostic.code.clone(),
                    diagnostic.message.clone(),
                    runtime_view_severity(diagnostic.severity),
                )
            })
            .collect::<Vec<_>>();
        run.record(evidence_from_result(
            NODE_RUNTIME_VIEW,
            PRODUCER_RUNTIME_VIEW,
            EVIDENCE_RUNTIME_VIEW,
            false,
            runtime_diagnostics,
        ));
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    }

    let button_runtime_report = ButtonRuntimeViewReport::from_runtime_view_report_with_host_data(
        &runtime_report,
        &ButtonRuntimeHostData::new(),
    );
    let button_diagnostics = button_runtime_report
        .diagnostics
        .iter()
        .map(|diagnostic| {
            story_diagnostic(
                NODE_RUNTIME_VIEW,
                PRODUCER_RUNTIME_VIEW,
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                button_runtime_severity(diagnostic.severity),
            )
        })
        .collect::<Vec<_>>();
    run.record(evidence_from_result(
        NODE_RUNTIME_VIEW,
        PRODUCER_RUNTIME_VIEW,
        EVIDENCE_RUNTIME_VIEW,
        button_runtime_report.passed(),
        button_diagnostics,
    ));
    if button_runtime_report.passed() {
        button_report = Some(button_runtime_report.clone());
    } else {
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    }

    let primitive_report = UiRenderPrimitiveReport::from_button_report(
        button_runtime_report.clone(),
        size,
        theme,
        atlas,
        DEFAULT_EDITOR_FONT_ID,
    );
    run.record(render_primitive_evidence(&primitive_report));
    if !primitive_report.passed() {
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    }

    let render_data_report =
        UiHeadlessRenderDataReport::from_render_primitive_report(&primitive_report);
    run.record(render_data_evidence(&render_data_report));
    if !render_data_report.passed() {
        return finish_gallery_story_execution(story, run, button_report, mounted_frame);
    }

    let mount_report = UiStaticMountReport::from_render_data_report(&render_data_report);
    run.record(static_mount_evidence(&mount_report));
    if let Some(mounted) = mount_report.mounted_frame() {
        mounted_frame = Some(mounted.frame.clone());
        run.record(UiStoryEvidence::passed(
            NODE_PREVIEW_FRAME,
            PRODUCER_PREVIEW_FRAME,
            EVIDENCE_PREVIEW_FRAME,
        ));
    } else {
        run.record(UiStoryEvidence::failed(
            NODE_PREVIEW_FRAME,
            PRODUCER_PREVIEW_FRAME,
            EVIDENCE_PREVIEW_FRAME,
            vec![story_diagnostic(
                NODE_PREVIEW_FRAME,
                PRODUCER_PREVIEW_FRAME,
                "ui_gallery.story.preview_frame.missing",
                "static mount did not produce a mounted preview frame",
                UiStoryDiagnosticSeverity::Error,
            )],
        ));
    }

    finish_gallery_story_execution(story, run, button_report, mounted_frame)
}

fn finish_gallery_story_execution(
    story: &UiStoryManifestV2,
    run: ui_story::UiStoryWorkflowRunV2,
    button_report: Option<ButtonRuntimeViewReport>,
    mounted_frame: Option<UiFrame>,
) -> UiGalleryStoryExecution {
    let report = run.finish().into_report(story.expected_outcome.clone());
    let mount_decision = UiStoryMountDecisionV2::from_report(&report, story.mount_policy);

    UiGalleryStoryExecution {
        report,
        mount_decision,
        mount_policy: story.mount_policy,
        button_report,
        mounted_frame,
    }
}

fn registry_failure_execution(diagnostic: UiStoryDiagnostic) -> UiGalleryStoryExecution {
    let report = UiStoryWorkflowReportV2 {
        story_id: UiStoryId::new("checked_in_story_registry_v2"),
        workflow_graph: None,
        node_reports: Vec::new(),
        diagnostics: vec![diagnostic],
        outcome: UiStoryOutcomeV2::InvalidManifest,
    };
    let mount_decision =
        UiStoryMountDecisionV2::blocked(UiStoryMountBlockReasonV2::BlockedInvalidWorkflow);

    UiGalleryStoryExecution {
        report,
        mount_decision,
        mount_policy: UiStoryMountPolicyV2::Never,
        button_report: None,
        mounted_frame: None,
    }
}

fn default_gallery_proof_size() -> UiSize {
    UiSize::new(GALLERY_PREVIEW_TILE_WIDTH, GALLERY_PREVIEW_TILE_HEIGHT)
}

fn compose_gallery_preview_frame(output_size: UiSize, previews: &[UiFrame]) -> Option<UiFrame> {
    if previews.is_empty() {
        return None;
    }

    let columns = gallery_preview_columns(output_size.width);
    let fragments = previews.iter().enumerate().map(|(index, frame)| {
        let column = index % columns;
        let row = index / columns;
        let x = GALLERY_PREVIEW_PADDING
            + column as f32 * (GALLERY_PREVIEW_TILE_WIDTH + GALLERY_PREVIEW_GAP);
        let y = GALLERY_PREVIEW_PADDING
            + row as f32 * (GALLERY_PREVIEW_TILE_HEIGHT + GALLERY_PREVIEW_GAP);
        let origin = UiPoint::new(x, y);
        let clip = UiRect::new(
            x,
            y,
            GALLERY_PREVIEW_TILE_WIDTH,
            GALLERY_PREVIEW_TILE_HEIGHT,
        );
        UiFrameFragment::new(frame, UiFramePlacement::new(origin, clip, index as u32))
    });
    let frame = compose_frame_fragments(output_size, fragments);
    (!frame.is_empty()).then_some(frame)
}

fn gallery_preview_columns(output_width: f32) -> usize {
    let usable_width = (output_width - GALLERY_PREVIEW_PADDING * 2.0 + GALLERY_PREVIEW_GAP)
        .max(GALLERY_PREVIEW_TILE_WIDTH);
    ((usable_width / (GALLERY_PREVIEW_TILE_WIDTH + GALLERY_PREVIEW_GAP)).floor() as usize).max(1)
}

fn load_story_source(
    story: &UiStoryManifestV2,
    run: &mut ui_story::UiStoryWorkflowRunV2,
) -> Option<String> {
    let path = repo_root().join(&story.source.path);
    match fs::read_to_string(&path) {
        Ok(source) => {
            run.record(UiStoryEvidence::passed(
                NODE_SOURCE_LOAD,
                PRODUCER_SOURCE_LOADER,
                EVIDENCE_SOURCE_LOAD,
            ));
            Some(source)
        }
        Err(error) => {
            run.record(UiStoryEvidence::failed(
                NODE_SOURCE_LOAD,
                PRODUCER_SOURCE_LOADER,
                EVIDENCE_SOURCE_LOAD,
                vec![story_diagnostic(
                    NODE_SOURCE_LOAD,
                    PRODUCER_SOURCE_LOADER,
                    "ui_gallery.story.source.read_failed",
                    format!("failed to read {}: {error}", path.display()),
                    UiStoryDiagnosticSeverity::Error,
                )],
            ));
            None
        }
    }
}

fn parse_story_node(
    story: &UiStoryManifestV2,
    source: &str,
    run: &mut ui_story::UiStoryWorkflowRunV2,
) -> Option<UiNodeDefinition> {
    match ron::from_str(source) {
        Ok(node) => {
            run.record(UiStoryEvidence::passed(
                NODE_SOURCE_PARSE,
                PRODUCER_SOURCE_PARSER,
                EVIDENCE_SOURCE_PARSE,
            ));
            Some(node)
        }
        Err(error) => {
            run.record(UiStoryEvidence::failed(
                NODE_SOURCE_PARSE,
                PRODUCER_SOURCE_PARSER,
                EVIDENCE_SOURCE_PARSE,
                vec![story_diagnostic(
                    NODE_SOURCE_PARSE,
                    PRODUCER_SOURCE_PARSER,
                    "ui_gallery.story.source.parse_failed",
                    format!("failed to parse {}: {error}", story.source.path),
                    UiStoryDiagnosticSeverity::Error,
                )],
            ));
            None
        }
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("runenwerk_editor should live under apps/runenwerk_editor")
        .to_path_buf()
}

fn append_story_report_diagnostics(
    report: &UiStoryWorkflowReportV2,
    diagnostics: &mut Vec<UiGalleryDiagnostic>,
    blocks_gallery: bool,
) {
    diagnostics.extend(report.diagnostics().iter().map(|diagnostic| {
        UiGalleryDiagnostic {
            stage: gallery_stage_for_story_diagnostic(diagnostic),
            story_id: Some(report.story_id.as_str().to_owned()),
            code: diagnostic.code.as_str().to_owned(),
            message: diagnostic.message.clone(),
            severity: story_severity(diagnostic.severity),
            source_map_index: None,
            blocks_gallery,
        }
    }));
}

fn gallery_stage_for_story_diagnostic(diagnostic: &UiStoryDiagnostic) -> UiGalleryStage {
    match &diagnostic.subject {
        UiStoryDiagnosticSubject::WorkflowNode(node_id) => {
            UiGalleryStage::WorkflowNode(node_id.as_str().to_owned())
        }
        _ => UiGalleryStage::Story,
    }
}

fn render_primitive_evidence(report: &UiRenderPrimitiveReport) -> UiStoryEvidence {
    let diagnostics = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            story_diagnostic(
                NODE_RENDER_PRIMITIVES,
                PRODUCER_RENDER_PRIMITIVES,
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                match diagnostic.severity {
                    ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity::Info => {
                        UiStoryDiagnosticSeverity::Info
                    }
                    ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity::Warning => {
                        UiStoryDiagnosticSeverity::Warning
                    }
                    ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity::Error => {
                        UiStoryDiagnosticSeverity::Error
                    }
                },
            )
        })
        .collect::<Vec<_>>();
    evidence_from_result(
        NODE_RENDER_PRIMITIVES,
        PRODUCER_RENDER_PRIMITIVES,
        EVIDENCE_RENDER_PRIMITIVES,
        report.passed(),
        diagnostics,
    )
}

fn render_data_evidence(report: &UiHeadlessRenderDataReport) -> UiStoryEvidence {
    let diagnostics = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            story_diagnostic(
                NODE_RENDER_DATA,
                PRODUCER_RENDER_DATA,
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                match diagnostic.severity {
                    ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity::Info => {
                        UiStoryDiagnosticSeverity::Info
                    }
                    ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity::Warning => {
                        UiStoryDiagnosticSeverity::Warning
                    }
                    ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity::Error => {
                        UiStoryDiagnosticSeverity::Error
                    }
                },
            )
        })
        .collect::<Vec<_>>();
    evidence_from_result(
        NODE_RENDER_DATA,
        PRODUCER_RENDER_DATA,
        EVIDENCE_RENDER_DATA,
        report.passed(),
        diagnostics,
    )
}

fn static_mount_evidence(report: &UiStaticMountReport) -> UiStoryEvidence {
    let diagnostics = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            story_diagnostic(
                NODE_STATIC_MOUNT,
                PRODUCER_STATIC_MOUNT,
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                match diagnostic.severity {
                    ui_static_mount::UiStaticMountDiagnosticSeverity::Info => {
                        UiStoryDiagnosticSeverity::Info
                    }
                    ui_static_mount::UiStaticMountDiagnosticSeverity::Warning => {
                        UiStoryDiagnosticSeverity::Warning
                    }
                    ui_static_mount::UiStaticMountDiagnosticSeverity::Error => {
                        UiStoryDiagnosticSeverity::Error
                    }
                },
            )
        })
        .collect::<Vec<_>>();
    evidence_from_result(
        NODE_STATIC_MOUNT,
        PRODUCER_STATIC_MOUNT,
        EVIDENCE_STATIC_MOUNT,
        report.passed(),
        diagnostics,
    )
}

fn evidence_from_result(
    node_id: &str,
    producer_id: &str,
    evidence_key: &str,
    passed: bool,
    diagnostics: Vec<UiStoryDiagnostic>,
) -> UiStoryEvidence {
    let status = if passed {
        UiStoryEvidenceStatus::Passed
    } else {
        UiStoryEvidenceStatus::Failed
    };
    UiStoryEvidence::new(
        UiStoryWorkflowNodeId::new(node_id),
        UiStoryEvidenceProducerId::new(producer_id),
        UiStoryEvidenceKey::new(evidence_key),
        status,
        diagnostics,
    )
}

fn story_diagnostic(
    node_id: &str,
    producer_id: &str,
    code: impl Into<String>,
    message: impl Into<String>,
    severity: UiStoryDiagnosticSeverity,
) -> UiStoryDiagnostic {
    UiStoryDiagnostic::new(
        code,
        severity,
        UiStoryDiagnosticOrigin::ExternalProducer(UiStoryEvidenceProducerId::new(producer_id)),
        UiStoryDiagnosticSubject::WorkflowNode(UiStoryWorkflowNodeId::new(node_id)),
        message,
    )
    .with_context("producer_id", producer_id)
}

fn runtime_artifact_severity(
    severity: ui_artifacts::UiRuntimeArtifactDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Info => UiStoryDiagnosticSeverity::Info,
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Warning => {
            UiStoryDiagnosticSeverity::Warning
        }
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Error => UiStoryDiagnosticSeverity::Error,
    }
}

fn runtime_view_severity(
    severity: ui_runtime_view::UiRuntimeViewDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        ui_runtime_view::UiRuntimeViewDiagnosticSeverity::Info => UiStoryDiagnosticSeverity::Info,
        ui_runtime_view::UiRuntimeViewDiagnosticSeverity::Warning => {
            UiStoryDiagnosticSeverity::Warning
        }
        ui_runtime_view::UiRuntimeViewDiagnosticSeverity::Error => UiStoryDiagnosticSeverity::Error,
    }
}

fn button_runtime_severity(
    severity: ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Info => UiStoryDiagnosticSeverity::Info,
        ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Warning => {
            UiStoryDiagnosticSeverity::Warning
        }
        ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Error => {
            UiStoryDiagnosticSeverity::Error
        }
    }
}

fn button_gallery_severity(
    severity: ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity,
) -> UiGalleryDiagnosticSeverity {
    match severity {
        ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Info => UiGalleryDiagnosticSeverity::Info,
        ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Warning => {
            UiGalleryDiagnosticSeverity::Warning
        }
        ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Error => UiGalleryDiagnosticSeverity::Error,
    }
}

fn story_severity(severity: UiStoryDiagnosticSeverity) -> UiGalleryDiagnosticSeverity {
    match severity {
        UiStoryDiagnosticSeverity::Info => UiGalleryDiagnosticSeverity::Info,
        UiStoryDiagnosticSeverity::Warning => UiGalleryDiagnosticSeverity::Warning,
        UiStoryDiagnosticSeverity::Error | UiStoryDiagnosticSeverity::Fatal => {
            UiGalleryDiagnosticSeverity::Error
        }
    }
}
