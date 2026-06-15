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
use ui_theme::ThemeTokens;

pub const UI_GALLERY_UI_PRODUCER_ID: UiFrameProducerId = ui_frame_producer_id(5_101);

const UI_GALLERY_FIXTURES: &[UiGalleryFixtureSpec] = &[
    UiGalleryFixtureSpec {
        fixture_id: "ui_gallery.button.basic",
        source_id: "assets.ui_gallery.button.basic",
        path: "assets/ui_gallery/button/basic.ron",
        host_bool: None,
    },
    UiGalleryFixtureSpec {
        fixture_id: "ui_gallery.button.selected",
        source_id: "assets.ui_gallery.button.selected",
        path: "assets/ui_gallery/button/selected.ron",
        host_bool: Some(("ui_gallery.button.selected.active", true)),
    },
];

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
    diagnostics: Vec<UiGalleryDiagnostic>,
    frame: Option<UiFrame>,
    prepared_size: Option<UiSize>,
    diagnostics_logged: bool,
}

impl Default for UiGalleryResource {
    fn default() -> Self {
        Self::from_checked_in_fixtures()
    }
}

impl UiGalleryResource {
    pub fn from_checked_in_fixtures() -> Self {
        let mut diagnostics = Vec::new();
        let registry = ControlPackageRegistry::new()
            .with_package(runenwerk_control_package())
            .expect("runenwerk controls package should register");
        let snapshot = registry.snapshot();
        let mut button_report = ButtonRuntimeViewReport::default();

        for fixture in UI_GALLERY_FIXTURES {
            let Some(report) = compile_fixture_button_report(fixture, &snapshot, &mut diagnostics)
            else {
                continue;
            };
            button_report.buttons.extend(report.buttons);
            diagnostics.extend(report.diagnostics.into_iter().map(|diagnostic| {
                UiGalleryDiagnostic {
                    stage: UiGalleryStage::RuntimeView,
                    fixture_id: Some(fixture.fixture_id.to_owned()),
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
                }
            }));
        }

        Self {
            button_report,
            diagnostics,
            frame: None,
            prepared_size: None,
            diagnostics_logged: false,
        }
    }

    pub fn passed(&self) -> bool {
        !self.has_error_diagnostics()
    }

    pub fn frame(&self) -> Option<&UiFrame> {
        self.frame.as_ref()
    }

    pub fn diagnostics(&self) -> &[UiGalleryDiagnostic] {
        &self.diagnostics
    }

    pub fn button_count(&self) -> usize {
        self.button_report.buttons.len()
    }

    fn has_error_diagnostics(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == UiGalleryDiagnosticSeverity::Error)
    }

    fn prepare_frame_if_needed(
        &mut self,
        size: UiSize,
        theme: &ThemeTokens,
        atlas: &UiFontAtlasResource,
    ) {
        if self.prepared_size == Some(size) && self.frame.is_some() {
            return;
        }

        self.frame = None;
        self.prepared_size = Some(size);
        if self.has_error_diagnostics() {
            return;
        }

        let primitive_report = UiRenderPrimitiveReport::from_button_report(
            self.button_report.clone(),
            size,
            theme,
            atlas,
            DEFAULT_EDITOR_FONT_ID,
        );
        append_primitive_diagnostics(&primitive_report, &mut self.diagnostics);
        if !primitive_report.passed() {
            return;
        }

        let render_data_report =
            UiHeadlessRenderDataReport::from_render_primitive_report(&primitive_report);
        append_render_data_diagnostics(&render_data_report, &mut self.diagnostics);
        if !render_data_report.passed() {
            return;
        }

        let mount_report = UiStaticMountReport::from_render_data_report(&render_data_report);
        append_static_mount_diagnostics(&mount_report, &mut self.diagnostics);
        if !mount_report.passed() {
            return;
        }

        self.frame = mount_report
            .mounted_frame()
            .map(|mounted| mounted.frame.clone());
    }

    fn log_diagnostics_once(&mut self) {
        if self.diagnostics_logged {
            return;
        }
        self.diagnostics_logged = true;

        if self.diagnostics.is_empty() {
            eprintln!(
                "[ui_gallery] prepared {} buttons through artifact-backed primitive path",
                self.button_count()
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
                    .fixture_id
                    .as_ref()
                    .map(|id| format!(" fixture={id}"))
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiGalleryDiagnostic {
    pub stage: UiGalleryStage,
    pub fixture_id: Option<String>,
    pub code: String,
    pub message: String,
    pub severity: UiGalleryDiagnosticSeverity,
    pub source_map_index: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGalleryStage {
    FixtureLoad,
    Formation,
    Compiler,
    RuntimeView,
    RenderPrimitives,
    RenderData,
    StaticMount,
}

impl UiGalleryStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::FixtureLoad => "fixture_load",
            Self::Formation => "formation",
            Self::Compiler => "compiler",
            Self::RuntimeView => "runtime_view",
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

#[derive(Clone, Copy, Debug)]
struct UiGalleryFixtureSpec {
    fixture_id: &'static str,
    source_id: &'static str,
    path: &'static str,
    host_bool: Option<(&'static str, bool)>,
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

fn compile_fixture_button_report(
    fixture: &UiGalleryFixtureSpec,
    snapshot: &ui_controls::ControlPackageRegistrySnapshot,
    diagnostics: &mut Vec<UiGalleryDiagnostic>,
) -> Option<ButtonRuntimeViewReport> {
    let Some(node) = load_fixture_node(fixture, diagnostics) else {
        return None;
    };

    let formation_report = form_ui_program_report_from_node_with_registry_snapshot(
        fixture.fixture_id,
        fixture.source_id,
        &node,
        snapshot,
    );
    if !formation_report.passed() {
        diagnostics.extend(formation_report.diagnostics.into_iter().map(|diagnostic| {
            UiGalleryDiagnostic {
                stage: UiGalleryStage::Formation,
                fixture_id: Some(fixture.fixture_id.to_owned()),
                code: diagnostic.code,
                message: diagnostic.message,
                severity: UiGalleryDiagnosticSeverity::Error,
                source_map_index: None,
            }
        }));
        return None;
    }

    let compiler_report = UiCompiler.compile_report(&formation_report.program);
    if !compiler_report.passed() {
        diagnostics.extend(compiler_report.artifact.manifest.diagnostics.iter().map(
            |diagnostic| UiGalleryDiagnostic {
                stage: UiGalleryStage::Compiler,
                fixture_id: Some(fixture.fixture_id.to_owned()),
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
                severity: match diagnostic.severity {
                    ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Info => {
                        UiGalleryDiagnosticSeverity::Info
                    }
                    ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Warning => {
                        UiGalleryDiagnosticSeverity::Warning
                    }
                    ui_artifacts::UiRuntimeArtifactDiagnosticSeverity::Error => {
                        UiGalleryDiagnosticSeverity::Error
                    }
                },
                source_map_index: None,
            },
        ));
        return None;
    }

    let runtime_report = UiRuntimeView::from_artifact_report(&compiler_report.artifact);
    if !runtime_report.passed() {
        diagnostics.extend(runtime_report.view.diagnostics.iter().map(|diagnostic| {
            UiGalleryDiagnostic {
                stage: UiGalleryStage::RuntimeView,
                fixture_id: Some(fixture.fixture_id.to_owned()),
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
                severity: match diagnostic.severity {
                    ui_runtime_view::UiRuntimeViewDiagnosticSeverity::Info => {
                        UiGalleryDiagnosticSeverity::Info
                    }
                    ui_runtime_view::UiRuntimeViewDiagnosticSeverity::Warning => {
                        UiGalleryDiagnosticSeverity::Warning
                    }
                    ui_runtime_view::UiRuntimeViewDiagnosticSeverity::Error => {
                        UiGalleryDiagnosticSeverity::Error
                    }
                },
                source_map_index: diagnostic.source_map_index,
            }
        }));
        return None;
    }

    let host_data = fixture
        .host_bool
        .map(|(endpoint, value)| ButtonRuntimeHostData::new().with_bool(endpoint, value))
        .unwrap_or_default();
    Some(
        ButtonRuntimeViewReport::from_runtime_view_report_with_host_data(
            &runtime_report,
            &host_data,
        ),
    )
}

fn load_fixture_node(
    fixture: &UiGalleryFixtureSpec,
    diagnostics: &mut Vec<UiGalleryDiagnostic>,
) -> Option<UiNodeDefinition> {
    let path = repo_root().join(fixture.path);
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            diagnostics.push(UiGalleryDiagnostic {
                stage: UiGalleryStage::FixtureLoad,
                fixture_id: Some(fixture.fixture_id.to_owned()),
                code: "ui_gallery.fixture.read_failed".to_owned(),
                message: format!("failed to read {}: {error}", path.display()),
                severity: UiGalleryDiagnosticSeverity::Error,
                source_map_index: None,
            });
            return None;
        }
    };

    match ron::from_str(&source) {
        Ok(node) => Some(node),
        Err(error) => {
            diagnostics.push(UiGalleryDiagnostic {
                stage: UiGalleryStage::FixtureLoad,
                fixture_id: Some(fixture.fixture_id.to_owned()),
                code: "ui_gallery.fixture.parse_failed".to_owned(),
                message: format!("failed to parse {}: {error}", path.display()),
                severity: UiGalleryDiagnosticSeverity::Error,
                source_map_index: None,
            });
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

fn append_primitive_diagnostics(
    report: &UiRenderPrimitiveReport,
    diagnostics: &mut Vec<UiGalleryDiagnostic>,
) {
    diagnostics.extend(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| UiGalleryDiagnostic {
                stage: UiGalleryStage::RenderPrimitives,
                fixture_id: None,
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
                severity: match diagnostic.severity {
                    ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity::Info => {
                        UiGalleryDiagnosticSeverity::Info
                    }
                    ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity::Warning => {
                        UiGalleryDiagnosticSeverity::Warning
                    }
                    ui_render_primitives::UiRenderPrimitiveDiagnosticSeverity::Error => {
                        UiGalleryDiagnosticSeverity::Error
                    }
                },
                source_map_index: diagnostic.source_map_index,
            }),
    );
}

fn append_render_data_diagnostics(
    report: &UiHeadlessRenderDataReport,
    diagnostics: &mut Vec<UiGalleryDiagnostic>,
) {
    diagnostics.extend(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| UiGalleryDiagnostic {
                stage: UiGalleryStage::RenderData,
                fixture_id: None,
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
                severity: match diagnostic.severity {
                    ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity::Info => {
                        UiGalleryDiagnosticSeverity::Info
                    }
                    ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity::Warning => {
                        UiGalleryDiagnosticSeverity::Warning
                    }
                    ui_headless_render_data::UiHeadlessRenderDataDiagnosticSeverity::Error => {
                        UiGalleryDiagnosticSeverity::Error
                    }
                },
                source_map_index: diagnostic.source_map_index,
            }),
    );
}

fn append_static_mount_diagnostics(
    report: &UiStaticMountReport,
    diagnostics: &mut Vec<UiGalleryDiagnostic>,
) {
    diagnostics.extend(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| UiGalleryDiagnostic {
                stage: UiGalleryStage::StaticMount,
                fixture_id: None,
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
                severity: match diagnostic.severity {
                    ui_static_mount::UiStaticMountDiagnosticSeverity::Info => {
                        UiGalleryDiagnosticSeverity::Info
                    }
                    ui_static_mount::UiStaticMountDiagnosticSeverity::Warning => {
                        UiGalleryDiagnosticSeverity::Warning
                    }
                    ui_static_mount::UiStaticMountDiagnosticSeverity::Error => {
                        UiGalleryDiagnosticSeverity::Error
                    }
                },
                source_map_index: None,
            }),
    );
}
