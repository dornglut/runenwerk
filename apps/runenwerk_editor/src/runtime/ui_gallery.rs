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
use ui_math::UiSize;
use ui_program_lowering::form_ui_program_report_from_node_with_registry_snapshot;
use ui_render_data::UiFrame;
use ui_render_primitives::UiRenderPrimitiveReport;
use ui_runtime_view::{ButtonRuntimeHostData, ButtonRuntimeViewReport, UiRuntimeView};
use ui_static_mount::UiStaticMountReport;
use ui_story::{
    UiStoryCliReport, UiStoryDiagnostic, UiStoryDiagnosticSeverity, UiStoryHostInputValue,
    UiStoryId, UiStoryManifest, UiStoryMountEligibility, UiStoryRunReport, UiStoryRunner,
    UiStoryStageKind, UiStoryStageReport, UiStoryStageStatus, checked_in_gallery_registry,
    stage_label,
};
use ui_theme::ThemeTokens;

pub const UI_GALLERY_UI_PRODUCER_ID: UiFrameProducerId = ui_frame_producer_id(5_101);

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
    story_reports: Vec<UiStoryRunReport>,
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
        let executions = run_checked_in_gallery_stories_for_render_target(size, theme, atlas);
        Self::from_story_executions(executions, Some(size))
    }

    pub fn from_story_executions(
        executions: Vec<UiGalleryStoryExecution>,
        prepared_size: Option<UiSize>,
    ) -> Self {
        let mut diagnostics = Vec::new();
        let mut button_report = ButtonRuntimeViewReport::default();
        let mut frame = UiFrame::new();
        let mut story_reports = Vec::new();

        for execution in executions {
            let report_blocks_gallery = !execution.report.passed();
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
                        stage: UiGalleryStage::Story(UiStoryStageKind::RuntimeView),
                        story_id: Some(execution.report.story_id.as_str().to_owned()),
                        code: diagnostic.code,
                        message: diagnostic.message,
                        severity: match diagnostic.severity {
                            ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Info => {
                                UiGalleryDiagnosticSeverity::Info
                            }
                            ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Warning => {
                                UiGalleryDiagnosticSeverity::Warning
                            }
                            ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Error => {
                                UiGalleryDiagnosticSeverity::Error
                            }
                        },
                        source_map_index: diagnostic.source_map_index,
                        blocks_gallery: report_blocks_gallery,
                    }
                }));
            }
            let eligibility = UiStoryMountEligibility::from_report(&execution.report);
            if eligibility.eligible {
                if let Some(mounted_frame) = execution.mounted_frame {
                    for surface in mounted_frame.surfaces {
                        frame.push_surface(surface);
                    }
                } else {
                    diagnostics.push(UiGalleryDiagnostic {
                        stage: UiGalleryStage::Story(UiStoryStageKind::PreviewFrame),
                        story_id: Some(execution.report.story_id.as_str().to_owned()),
                        code: "ui_gallery.story.preview_frame.missing".to_owned(),
                        message: "eligible story report did not carry a mounted preview frame"
                            .to_owned(),
                        severity: UiGalleryDiagnosticSeverity::Error,
                        source_map_index: None,
                        blocks_gallery: true,
                    });
                }
            }
            story_reports.push(execution.report);
        }
        let frame = (!frame.is_empty()).then_some(frame);

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

    pub fn story_reports(&self) -> &[UiStoryRunReport] {
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
    pub report: UiStoryRunReport,
    pub button_report: Option<ButtonRuntimeViewReport>,
    pub mounted_frame: Option<UiFrame>,
}

pub fn inspect_checked_in_gallery_stories() -> UiStoryCliReport {
    let reports = run_checked_in_gallery_stories()
        .into_iter()
        .map(|execution| execution.report)
        .collect::<Vec<_>>();
    UiStoryCliReport::from_reports(reports.iter())
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
    let registry = checked_in_gallery_registry();
    if !registry.is_valid() {
        return registry
            .diagnostics()
            .iter()
            .map(|diagnostic| UiGalleryStoryExecution {
                report: UiStoryRunReport::unknown_story(
                    UiStoryId::new("checked_in_gallery_manifest"),
                    UiStoryDiagnostic::error(
                        diagnostic.code.clone(),
                        diagnostic.message.clone(),
                        UiStoryStageKind::Manifest,
                    ),
                ),
                button_report: None,
                mounted_frame: None,
            })
            .collect();
    }

    let runner = UiStoryRunner::new(&registry);
    let control_registry = ControlPackageRegistry::new()
        .with_package(runenwerk_control_package())
        .expect("runenwerk controls package should register");
    let snapshot = control_registry.snapshot();

    registry
        .stories()
        .map(|story| {
            execute_gallery_story(story, &registry, &runner, &snapshot, size, theme, atlas)
        })
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGalleryStage {
    Story(UiStoryStageKind),
    RenderPrimitives,
    RenderData,
    StaticMount,
}

impl UiGalleryStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Story(stage) => stage_label(stage),
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
    story: &UiStoryManifest,
    registry: &ui_story::UiStoryRegistry,
    runner: &UiStoryRunner<'_>,
    snapshot: &ui_controls::ControlPackageRegistrySnapshot,
    size: UiSize,
    theme: &ThemeTokens,
    atlas: &UiFontAtlasResource,
) -> UiGalleryStoryExecution {
    let request = registry.run_request(story.story_id.as_str());
    let mut stage_reports = Vec::new();
    let mut button_report = None;
    let mut mounted_frame = None;

    let Some(source) = load_story_source(story, &mut stage_reports) else {
        return UiGalleryStoryExecution {
            report: runner.run_story_with_stage_reports(&request, stage_reports),
            button_report,
            mounted_frame,
        };
    };

    let Some(node) = parse_story_node(story, &source, &mut stage_reports) else {
        return UiGalleryStoryExecution {
            report: runner.run_story_with_stage_reports(&request, stage_reports),
            button_report,
            mounted_frame,
        };
    };

    let formation_report = form_ui_program_report_from_node_with_registry_snapshot(
        story.program_id.as_str(),
        story.source.source_id.as_str(),
        &node,
        snapshot,
    );
    if !formation_report.passed() {
        stage_reports.push(UiStoryStageReport::failed(
            UiStoryStageKind::ProgramFormation,
            formation_report
                .diagnostics
                .into_iter()
                .map(|diagnostic| {
                    UiStoryDiagnostic::new(
                        diagnostic.code,
                        diagnostic.message,
                        UiStoryStageKind::ProgramFormation,
                        UiStoryDiagnosticSeverity::Error,
                    )
                })
                .collect(),
        ));
        return UiGalleryStoryExecution {
            report: runner.run_story_with_stage_reports(&request, stage_reports),
            button_report,
            mounted_frame,
        };
    }
    stage_reports.push(UiStoryStageReport::passed(
        UiStoryStageKind::ProgramFormation,
    ));

    let compiler_report = UiCompiler.compile_report(&formation_report.program);
    if !compiler_report.passed() {
        stage_reports.push(UiStoryStageReport::failed(
            UiStoryStageKind::Compiler,
            compiler_report
                .artifact
                .manifest
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    UiStoryDiagnostic::new(
                        diagnostic.code.clone(),
                        diagnostic.message.clone(),
                        UiStoryStageKind::Compiler,
                        runtime_artifact_severity(diagnostic.severity),
                    )
                })
                .collect(),
        ));
        return UiGalleryStoryExecution {
            report: runner.run_story_with_stage_reports(&request, stage_reports),
            button_report,
            mounted_frame,
        };
    }
    stage_reports.push(UiStoryStageReport::passed(UiStoryStageKind::Compiler));

    let runtime_report = UiRuntimeView::from_artifact_report(&compiler_report.artifact);
    if !runtime_report.passed() {
        stage_reports.push(UiStoryStageReport::failed(
            UiStoryStageKind::RuntimeView,
            runtime_report
                .view
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    UiStoryDiagnostic::new(
                        diagnostic.code.clone(),
                        diagnostic.message.clone(),
                        UiStoryStageKind::RuntimeView,
                        runtime_view_severity(diagnostic.severity),
                    )
                })
                .collect(),
        ));
        return UiGalleryStoryExecution {
            report: runner.run_story_with_stage_reports(&request, stage_reports),
            button_report,
            mounted_frame,
        };
    }

    let host_data =
        story
            .host_inputs
            .iter()
            .fold(
                ButtonRuntimeHostData::new(),
                |host_data, input| match input.value {
                    UiStoryHostInputValue::Bool(value) => {
                        host_data.with_bool(input.endpoint.as_str(), value)
                    }
                },
            );
    let button_runtime_report = ButtonRuntimeViewReport::from_runtime_view_report_with_host_data(
        &runtime_report,
        &host_data,
    );
    let button_diagnostics = button_runtime_report
        .diagnostics
        .iter()
        .map(|diagnostic| {
            UiStoryDiagnostic::new(
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                UiStoryStageKind::RuntimeView,
                button_runtime_severity(diagnostic.severity),
            )
        })
        .collect::<Vec<_>>();
    if button_runtime_report.passed() {
        stage_reports.push(UiStoryStageReport {
            stage: UiStoryStageKind::RuntimeView,
            status: UiStoryStageStatus::Passed,
            diagnostics: button_diagnostics,
            elapsed_micros: None,
        });
        button_report = Some(button_runtime_report.clone());
    } else {
        stage_reports.push(UiStoryStageReport::failed(
            UiStoryStageKind::RuntimeView,
            button_diagnostics,
        ));
        return UiGalleryStoryExecution {
            report: runner.run_story_with_stage_reports(&request, stage_reports),
            button_report,
            mounted_frame,
        };
    }

    let primitive_report = UiRenderPrimitiveReport::from_button_report(
        button_runtime_report.clone(),
        size,
        theme,
        atlas,
        DEFAULT_EDITOR_FONT_ID,
    );
    stage_reports.push(render_primitive_stage_report(&primitive_report));
    if !primitive_report.passed() {
        return UiGalleryStoryExecution {
            report: runner.run_story_with_stage_reports(&request, stage_reports),
            button_report,
            mounted_frame,
        };
    }

    let render_data_report =
        UiHeadlessRenderDataReport::from_render_primitive_report(&primitive_report);
    stage_reports.push(render_data_stage_report(&render_data_report));
    if !render_data_report.passed() {
        return UiGalleryStoryExecution {
            report: runner.run_story_with_stage_reports(&request, stage_reports),
            button_report,
            mounted_frame,
        };
    }

    let mount_report = UiStaticMountReport::from_render_data_report(&render_data_report);
    stage_reports.push(static_mount_stage_report(&mount_report));
    if let Some(mounted) = mount_report.mounted_frame() {
        mounted_frame = Some(mounted.frame.clone());
        stage_reports.push(UiStoryStageReport::passed(UiStoryStageKind::PreviewFrame));
    } else {
        stage_reports.push(UiStoryStageReport::failed(
            UiStoryStageKind::PreviewFrame,
            vec![UiStoryDiagnostic::error(
                "ui_gallery.story.preview_frame.missing",
                "static mount did not produce a mounted preview frame",
                UiStoryStageKind::PreviewFrame,
            )],
        ));
    }

    UiGalleryStoryExecution {
        report: runner.run_story_with_stage_reports(&request, stage_reports),
        button_report,
        mounted_frame,
    }
}

fn default_gallery_proof_size() -> UiSize {
    UiSize::new(320.0, 128.0)
}

fn load_story_source(
    story: &UiStoryManifest,
    stage_reports: &mut Vec<UiStoryStageReport>,
) -> Option<String> {
    let path = repo_root().join(&story.source.path);
    match fs::read_to_string(&path) {
        Ok(source) => {
            stage_reports.push(UiStoryStageReport::passed(UiStoryStageKind::SourceLoad));
            Some(source)
        }
        Err(error) => {
            stage_reports.push(UiStoryStageReport::failed(
                UiStoryStageKind::SourceLoad,
                vec![UiStoryDiagnostic::error(
                    "ui_gallery.story.source.read_failed",
                    format!("failed to read {}: {error}", path.display()),
                    UiStoryStageKind::SourceLoad,
                )],
            ));
            None
        }
    }
}

fn parse_story_node(
    story: &UiStoryManifest,
    source: &str,
    stage_reports: &mut Vec<UiStoryStageReport>,
) -> Option<UiNodeDefinition> {
    match ron::from_str(source) {
        Ok(node) => {
            stage_reports.push(UiStoryStageReport::passed(UiStoryStageKind::SourceParse));
            Some(node)
        }
        Err(error) => {
            stage_reports.push(UiStoryStageReport::failed(
                UiStoryStageKind::SourceParse,
                vec![UiStoryDiagnostic::error(
                    "ui_gallery.story.source.parse_failed",
                    format!("failed to parse {}: {error}", story.source.path),
                    UiStoryStageKind::SourceParse,
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
    report: &UiStoryRunReport,
    diagnostics: &mut Vec<UiGalleryDiagnostic>,
    blocks_gallery: bool,
) {
    diagnostics.extend(
        report
            .diagnostics
            .iter()
            .map(|diagnostic| UiGalleryDiagnostic {
                stage: UiGalleryStage::Story(diagnostic.stage),
                story_id: Some(report.story_id.as_str().to_owned()),
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
                severity: story_severity(diagnostic.severity),
                source_map_index: None,
                blocks_gallery,
            }),
    );
}

fn render_primitive_stage_report(report: &UiRenderPrimitiveReport) -> UiStoryStageReport {
    let diagnostics = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            UiStoryDiagnostic::new(
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                UiStoryStageKind::RenderPrimitives,
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
    stage_report_from_result(
        UiStoryStageKind::RenderPrimitives,
        report.passed(),
        diagnostics,
    )
}

fn render_data_stage_report(report: &UiHeadlessRenderDataReport) -> UiStoryStageReport {
    let diagnostics = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            UiStoryDiagnostic::new(
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                UiStoryStageKind::RenderData,
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
    stage_report_from_result(UiStoryStageKind::RenderData, report.passed(), diagnostics)
}

fn static_mount_stage_report(report: &UiStaticMountReport) -> UiStoryStageReport {
    let diagnostics = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            UiStoryDiagnostic::new(
                diagnostic.code.clone(),
                diagnostic.message.clone(),
                UiStoryStageKind::StaticMount,
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
    stage_report_from_result(UiStoryStageKind::StaticMount, report.passed(), diagnostics)
}

fn stage_report_from_result(
    stage: UiStoryStageKind,
    passed: bool,
    diagnostics: Vec<UiStoryDiagnostic>,
) -> UiStoryStageReport {
    UiStoryStageReport {
        stage,
        status: if passed {
            UiStoryStageStatus::Passed
        } else {
            UiStoryStageStatus::Failed
        },
        diagnostics,
        elapsed_micros: None,
    }
}

fn runtime_artifact_severity(
    severity: ui_artifacts::UiRuntimeArtifactDiagnosticSeverity,
) -> UiStoryDiagnosticSeverity {
    match severity {
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Info => UiStoryDiagnosticSeverity::Info,
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Warning => {
            UiStoryDiagnosticSeverity::Warning
        }
        ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Error => {
            UiStoryDiagnosticSeverity::Error
        }
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
        ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Info => {
            UiStoryDiagnosticSeverity::Info
        }
        ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Warning => {
            UiStoryDiagnosticSeverity::Warning
        }
        ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Error => {
            UiStoryDiagnosticSeverity::Error
        }
    }
}

fn story_severity(severity: UiStoryDiagnosticSeverity) -> UiGalleryDiagnosticSeverity {
    match severity {
        UiStoryDiagnosticSeverity::Info => UiGalleryDiagnosticSeverity::Info,
        UiStoryDiagnosticSeverity::Warning => UiGalleryDiagnosticSeverity::Warning,
        UiStoryDiagnosticSeverity::Error => UiGalleryDiagnosticSeverity::Error,
    }
}
