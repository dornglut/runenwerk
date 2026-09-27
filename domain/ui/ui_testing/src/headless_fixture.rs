//! Headless host fixture execution.

use runenui_core::{
    ElementId, FontFamilyName, GenericFontFamily, NoHostProtocol, SemanticRole, UiApp, View, text,
};
use runenui_testing::{SemanticQuery, TestHarness};
use serde::{Deserialize, Serialize};
use ui_artifacts::{RuntimeTableKind, UiRuntimeArtifact};
use ui_binding::HostDataSnapshot;
use ui_compiler::{UiCompiler, UiCompilerReport};
use ui_evaluator::{UiEvaluationContext, UiEvaluator, UiOutput};
use ui_hosts::{
    DomainCommand, HeadlessHost, HostCommand, HostKind, HostRouteMapVersion, HostRouteMapping,
    HostSurfaceFacts,
};
use ui_program::{AccessibilityRole, RouteCapability, RouteId, RouteSchemaVersion, UiProgram};
use ui_schema::UiSchemaValue;
use ui_state::UiStateModel;

use crate::program_fixture::headless_program;
use crate::{DiagnosticAssertion, ReproducibilityAssertion, SourceMapAssertion};

const LABEL_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.label";
const CONTROLLED_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/fonts/JetBrainsMono-Regular.ttf"
));

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeadlessFixture {
    pub fixture_id: String,
    pub program: UiProgram,
    pub host: HeadlessHost,
    pub surface_facts: HostSurfaceFacts,
    #[serde(default)]
    pub host_data: Vec<HostDataSnapshot>,
}

impl HeadlessFixture {
    pub fn label_inspector(fixture_id: impl Into<String>) -> Self {
        let fixture_id = fixture_id.into();
        let route_map_version = HostRouteMapVersion::new(1);
        let preview_capability = RouteCapability::new("headless.fixture.preview");
        let host = HeadlessHost::new(route_map_version).with_mapping(
            HostRouteMapping::new(
                RouteId::new("headless.fixture.preview"),
                RouteSchemaVersion::new(1),
                route_map_version,
                HostCommand::new(HostKind::Headless, "headless.fixture.preview"),
            )
            .with_capability(preview_capability.clone())
            .with_domain_command(DomainCommand::new("domain.ui", "domain.ui.fixture.preview")),
        );

        Self {
            fixture_id,
            program: headless_program(preview_capability),
            host,
            surface_facts: HostSurfaceFacts::headless("surface.fixture.headless"),
            host_data: vec![HostDataSnapshot::new(
                "host.fixture.title",
                UiSchemaValue::string("Inspector"),
                1,
            )],
        }
    }

    pub fn compile_report(&self) -> UiCompilerReport {
        UiCompiler.compile_report(&self.program)
    }

    pub fn compile(&self) -> UiRuntimeArtifact {
        self.compile_report().artifact
    }

    pub fn run(&self) -> HeadlessFixtureRun {
        let artifact = self.compile();
        let mut state = UiStateModel::default();
        let mut context = UiEvaluationContext::default();
        for host_data in self.host_data.iter().cloned() {
            context = context.with_host_data(host_data);
        }
        let output = UiEvaluator.evaluate_with_context(&artifact, &mut state, context);
        let runenui_publication = prove_runenui_minimal_label(&artifact, &state);
        let source_map_assertion = SourceMapAssertion::target_in_table(
            "definition.fixture.title",
            "program.fixture.control.title",
            RuntimeTableKind::Control,
        );
        let diagnostic_assertion =
            DiagnosticAssertion::code_absent("ui.compiler.capability.missing_control_declaration");
        let reproducibility_assertion = ReproducibilityAssertion::from_fixture(self);

        HeadlessFixtureRun {
            artifact,
            output,
            state,
            runenui_publication,
            source_map_assertion,
            diagnostic_assertion,
            reproducibility_assertion,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeadlessFixtureRun {
    pub artifact: UiRuntimeArtifact,
    pub output: UiOutput,
    pub state: UiStateModel,
    pub(crate) runenui_publication: Result<(), RunenUiHeadlessProofError>,
    pub source_map_assertion: SourceMapAssertion,
    pub diagnostic_assertion: DiagnosticAssertion,
    pub reproducibility_assertion: ReproducibilityAssertion,
}

impl HeadlessFixtureRun {
    pub fn passed(&self) -> bool {
        self.source_map_assertion
            .assert_artifact(&self.artifact)
            .is_ok()
            && self
                .diagnostic_assertion
                .assert_artifact(&self.artifact)
                .is_ok()
            && self.reproducibility_assertion.passed()
            && self.runenui_publication.is_ok()
            && self.output.diagnostics.is_empty()
    }
}

#[derive(Clone)]
struct MinimalLabelAppState {
    authored_id: ElementId,
    text: String,
}

struct MinimalLabelApp;

impl UiApp for MinimalLabelApp {
    type State = MinimalLabelAppState;
    type Action = ();
    type HostProtocol = NoHostProtocol;

    fn root(state: &Self::State) -> impl View<Self::Action> {
        text(state.text.clone()).id(state.authored_id.clone())
    }

    fn update(
        _: &mut Self::State,
        (): Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StableRunenUiFacts {
    authored_id: ElementId,
    text: String,
    bounds_bits: [u32; 4],
    glyph_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RunenUiHeadlessProofError {
    UnexpectedControlShape,
    UnsupportedControlKind,
    UnexpectedHierarchy,
    UnexpectedStateShape,
    MissingTextState,
    UnexpectedAccessibilityShape,
    UnsupportedAccessibilityRole,
    InvalidAuthoredId,
    FontRegistration,
    FontFamily,
    FontMapping,
    Publication,
    SemanticDiagnostics,
    MissingPublishedNode,
    MissingLayoutNode,
    EmptyLayoutBounds,
    MissingSemanticNode,
    MissingShapedText,
    NonDeterministic,
}

pub(crate) fn prove_runenui_minimal_label(
    artifact: &UiRuntimeArtifact,
    state: &UiStateModel,
) -> Result<(), RunenUiHeadlessProofError> {
    let projection = project_minimal_label(artifact, state)?;
    let first = observe_minimal_label(projection.clone())?;
    let second = observe_minimal_label(projection)?;

    if first != second {
        return Err(RunenUiHeadlessProofError::NonDeterministic);
    }

    Ok(())
}

fn project_minimal_label(
    artifact: &UiRuntimeArtifact,
    state: &UiStateModel,
) -> Result<MinimalLabelAppState, RunenUiHeadlessProofError> {
    let [control_row] = artifact.tables.controls.rows.as_slice() else {
        return Err(RunenUiHeadlessProofError::UnexpectedControlShape);
    };
    let control = &control_row.node;

    if control.control_kind.as_str() != LABEL_CONTROL_KIND_ID {
        return Err(RunenUiHeadlessProofError::UnsupportedControlKind);
    }
    if control.parent.is_some() || !control.children.is_empty() {
        return Err(RunenUiHeadlessProofError::UnexpectedHierarchy);
    }

    let [state_requirement] = control.local_state_requirements.as_slice() else {
        return Err(RunenUiHeadlessProofError::UnexpectedStateShape);
    };
    let text = state
        .value(state_requirement.as_str())
        .and_then(UiSchemaValue::as_str)
        .ok_or(RunenUiHeadlessProofError::MissingTextState)?;

    let [accessibility_row] = artifact.tables.accessibility.rows.as_slice() else {
        return Err(RunenUiHeadlessProofError::UnexpectedAccessibilityShape);
    };
    if accessibility_row.node.control_id != control.node_id {
        return Err(RunenUiHeadlessProofError::UnexpectedAccessibilityShape);
    }
    if accessibility_row.node.role != AccessibilityRole::Label {
        return Err(RunenUiHeadlessProofError::UnsupportedAccessibilityRole);
    }

    let authored_id = ElementId::new(control.node_id.as_str())
        .map_err(|_| RunenUiHeadlessProofError::InvalidAuthoredId)?;

    Ok(MinimalLabelAppState {
        authored_id,
        text: text.to_owned(),
    })
}

fn observe_minimal_label(
    projection: MinimalLabelAppState,
) -> Result<StableRunenUiFacts, RunenUiHeadlessProofError> {
    let authored_id = projection.authored_id.clone();
    let text = projection.text.clone();
    let mut harness = TestHarness::<MinimalLabelApp>::mount(projection);

    let registered = harness
        .register_text_font_bytes(CONTROLLED_FONT_BYTES.to_vec())
        .map_err(|_| RunenUiHeadlessProofError::FontRegistration)?;
    if registered == 0 {
        return Err(RunenUiHeadlessProofError::FontRegistration);
    }

    let family =
        FontFamilyName::new("JetBrains Mono").map_err(|_| RunenUiHeadlessProofError::FontFamily)?;
    let mapped = harness
        .set_text_generic_family_mapping(GenericFontFamily::SansSerif, &[family])
        .map_err(|_| RunenUiHeadlessProofError::FontMapping)?;
    if !mapped {
        return Err(RunenUiHeadlessProofError::FontMapping);
    }

    let (bounds_bits, glyph_count) = {
        let publication = harness
            .publish()
            .map_err(|_| RunenUiHeadlessProofError::Publication)?;
        if !publication.semantic_diagnostics().is_empty() {
            return Err(RunenUiHeadlessProofError::SemanticDiagnostics);
        }

        let node = publication
            .frame()
            .nodes()
            .iter()
            .find(|node| node.authored_id() == Some(&authored_id))
            .ok_or(RunenUiHeadlessProofError::MissingPublishedNode)?;
        let bounds = node.bounds();
        if bounds.width() <= 0.0 || bounds.height() <= 0.0 {
            return Err(RunenUiHeadlessProofError::EmptyLayoutBounds);
        }
        if publication.layout_report().node(node.id()).is_none() {
            return Err(RunenUiHeadlessProofError::MissingLayoutNode);
        }

        let shaped_run = publication
            .paint_scene()
            .items()
            .iter()
            .find_map(|item| item.primitive().as_shaped_text_run())
            .ok_or(RunenUiHeadlessProofError::MissingShapedText)?;
        let shaped = publication
            .paint_scene()
            .shaped_text_resource(shaped_run.resource_ref())
            .ok_or(RunenUiHeadlessProofError::MissingShapedText)?;
        let glyph_count = shaped.glyphs().len();
        if glyph_count == 0 {
            return Err(RunenUiHeadlessProofError::MissingShapedText);
        }

        (
            [
                bounds.x().to_bits(),
                bounds.y().to_bits(),
                bounds.width().to_bits(),
                bounds.height().to_bits(),
            ],
            glyph_count,
        )
    };

    let query = SemanticQuery::new()
        .with_role(SemanticRole::Text)
        .with_name(text.clone());
    harness
        .unique_semantic_target(&query)
        .map_err(|_| RunenUiHeadlessProofError::MissingSemanticNode)?;

    Ok(StableRunenUiFacts {
        authored_id,
        text,
        bounds_bits,
        glyph_count,
    })
}
