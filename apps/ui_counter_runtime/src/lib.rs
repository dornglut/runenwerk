use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use engine::plugins::render::RenderPlugin;
use engine::plugins::ui::{
    IntoUi, UiAction, UiActionDispatchReportsResource, UiActionEvent, UiActionHandler,
    UiHostActionExecutor, UiHostMutationIntent, UiHostMutationReceipt, UiHostMutationRejection,
    UiMountRequest, UiMountRequestsResource, UiPlugin, UiRuntimeDiagnosticsResource,
    UiRuntimeEvaluationInput, UiRuntimeEvaluationResource, UiRuntimeFramePublicationStatus,
    UiRuntimeSet, UiRuntimeTraceEvent, UiRuntimeTraceEventKind, UiRuntimeTraceResource, UiScreen,
    UiTypedActionDescriptor, UiTypedActionId, UiTypedScreenId, UiTypedSource, dispatch_ui_action,
};
use engine::prelude::{
    App, AppUiExt, InputState, Plugin, RenderPrepare, Res, ResMut, SystemConfigExt, Update,
    default_plugins,
};
use serde::{Deserialize, Serialize};
use ui_binding::HostDataSnapshot;
use ui_controls::{BUTTON_CONTROL_KIND_ID, ControlPackageRegistry, runenwerk_control_package};
use ui_definition::{
    AuthoredBindingRef, AuthoredControlAccessibilityDefinition, AuthoredControlKindId,
    AuthoredControlValue, AuthoredId, AuthoredRouteId, UiNodeDefinition, UiValueBinding,
};
use ui_evaluator::UiEvaluationContext;
use ui_hosts::{DomainCommand, GameHost, HostCommand, HostKind, HostRouteMapVersion};
use ui_program::{
    RouteCapability, RouteId, RouteSchemaVersion, UiEventPacket, UiEventSourceControlId,
    UiProgramSourceId,
};
use ui_schema::{UiSchemaRef, UiSchemaValue};
use winit::keyboard::KeyCode;

pub const WINDOW_TITLE: &str = "Runenwerk UI Counter Runtime";
pub const COUNTER_SCREEN_ID: &str = "counter.runtime.screen";
pub const COUNTER_SOURCE_ID: &str = "counter.runtime.screen.source";
pub const COUNTER_VALUE_ENDPOINT: &str = "counter.value.text";
pub const COUNTER_VALUE_STATE_KEY: &str = "state.counter.value.selected";
pub const TRACE_STATUS_ENDPOINT: &str = "counter.trace.status";
pub const TRACE_STATUS_STATE_KEY: &str = "state.counter.trace.status.selected";

const ROUTE_MAP_VERSION: HostRouteMapVersion = HostRouteMapVersion::new(1);
const ACTION_SCHEMA_VERSION: RouteSchemaVersion = RouteSchemaVersion::new(1);
const COUNTER_ACTION_PAYLOAD_SCHEMA_ID: &str = "counter.action.payload";
const COUNTER_ACTION_CAPABILITY_PREFIX: &str = "counter.action";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterActionKind {
    Increment,
    Decrement,
    Reset,
}

impl CounterActionKind {
    pub const fn all() -> [Self; 3] {
        [Self::Increment, Self::Decrement, Self::Reset]
    }

    pub const fn route(self) -> &'static str {
        match self {
            Self::Increment => "counter.increment",
            Self::Decrement => "counter.decrement",
            Self::Reset => "counter.reset",
        }
    }

    pub const fn domain_command(self) -> &'static str {
        match self {
            Self::Increment => "increment",
            Self::Decrement => "decrement",
            Self::Reset => "reset",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Increment => "Increment",
            Self::Decrement => "Decrement",
            Self::Reset => "Reset",
        }
    }

    pub const fn input_action(self) -> &'static str {
        match self {
            Self::Increment => "counter.increment",
            Self::Decrement => "counter.decrement",
            Self::Reset => "counter.reset",
        }
    }

    pub fn capability(self) -> RouteCapability {
        RouteCapability::new(format!(
            "{COUNTER_ACTION_CAPABILITY_PREFIX}.{}",
            self.domain_command()
        ))
    }

    pub fn from_route(route: &str) -> Option<Self> {
        Self::all().into_iter().find(|kind| kind.route() == route)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CounterActionSource {
    HumanKeyboard,
    AgentScript,
}

impl CounterActionSource {
    fn source_control_id(self, kind: CounterActionKind) -> UiEventSourceControlId {
        let source = match self {
            Self::HumanKeyboard => "keyboard",
            Self::AgentScript => "agent-script",
        };
        UiEventSourceControlId::new(format!("{source}.{}", kind.domain_command()))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct Counter {
    value: i64,
    revision: u64,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            value: 0,
            revision: 1,
        }
    }
}

impl Counter {
    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn display_text(&self) -> String {
        format!("Count: {}", self.value)
    }

    fn apply(&mut self, kind: CounterActionKind) -> (i64, i64) {
        let before = self.value;
        match kind {
            CounterActionKind::Increment => {
                self.value = self.value.saturating_add(1);
            }
            CounterActionKind::Decrement => {
                self.value = self.value.saturating_sub(1);
            }
            CounterActionKind::Reset => {
                self.value = 0;
            }
        }
        if self.value != before || matches!(kind, CounterActionKind::Reset) {
            self.revision = self.revision.saturating_add(1);
        }
        (before, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterRuntimeActionRecord {
    source: CounterActionSource,
    action: CounterActionKind,
    before: i64,
    after: i64,
    route: String,
    host_command: String,
    domain_command: String,
}

impl CounterRuntimeActionRecord {
    fn new(
        source: CounterActionSource,
        action: CounterActionKind,
        before: i64,
        after: i64,
        route: impl Into<String>,
        host_command: impl Into<String>,
        domain_command: impl Into<String>,
    ) -> Self {
        Self {
            source,
            action,
            before,
            after,
            route: route.into(),
            host_command: host_command.into(),
            domain_command: domain_command.into(),
        }
    }

    pub fn source(&self) -> CounterActionSource {
        self.source
    }

    pub fn action(&self) -> CounterActionKind {
        self.action
    }

    pub fn before(&self) -> i64 {
        self.before
    }

    pub fn after(&self) -> i64 {
        self.after
    }

    pub fn route(&self) -> &str {
        &self.route
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ecs::Resource)]
pub struct CounterRuntimeState {
    status: String,
    action_history: Vec<CounterRuntimeActionRecord>,
}

impl Default for CounterRuntimeState {
    fn default() -> Self {
        Self {
            status: "Ready for counter input".to_owned(),
            action_history: Vec::new(),
        }
    }
}

impl CounterRuntimeState {
    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn action_history(&self) -> &[CounterRuntimeActionRecord] {
        &self.action_history
    }

    pub fn latest_action(&self) -> Option<&CounterRuntimeActionRecord> {
        self.action_history.last()
    }

    fn record_action(&mut self, record: CounterRuntimeActionRecord) {
        self.status = format!(
            "{} via {:?}: {} -> {}",
            record.action.label(),
            record.source,
            record.before,
            record.after
        );
        self.action_history.push(record);
    }

    fn trace_lines(&self) -> Vec<String> {
        if self.action_history.is_empty() {
            return vec!["No counter actions yet".to_owned()];
        }

        self.action_history
            .iter()
            .rev()
            .take(4)
            .map(|record| {
                format!(
                    "{:?} {} {} -> {}",
                    record.source,
                    record.action.label(),
                    record.before,
                    record.after
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CounterAgentScript {
    #[serde(default)]
    pub actions: Vec<CounterActionKind>,
}

impl CounterAgentScript {
    pub fn new(actions: impl IntoIterator<Item = CounterActionKind>) -> Self {
        Self {
            actions: actions.into_iter().collect(),
        }
    }

    pub fn from_ron_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read counter agent script {}", path.display()))?;
        ron::from_str(&source)
            .with_context(|| format!("failed to parse counter agent script {}", path.display()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ecs::Resource)]
struct CounterAgentScriptResource {
    script: CounterAgentScript,
    next_action: usize,
}

impl CounterAgentScriptResource {
    fn new(script: CounterAgentScript) -> Self {
        Self {
            script,
            next_action: 0,
        }
    }

    fn drain_pending_invocations(&mut self) -> Vec<CounterActionInvocation> {
        let mut invocations = Vec::new();
        while let Some(action) = self.script.actions.get(self.next_action).copied() {
            invocations.push(CounterActionInvocation {
                source: CounterActionSource::AgentScript,
                action,
            });
            self.next_action = self.next_action.saturating_add(1);
        }
        invocations
    }
}

impl Default for CounterAgentScriptResource {
    fn default() -> Self {
        Self::new(CounterAgentScript::default())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct CounterActionInvocation {
    source: CounterActionSource,
    action: CounterActionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterRuntimeOptions {
    pub headless: bool,
    pub agent_script: CounterAgentScript,
}

impl CounterRuntimeOptions {
    pub fn headless() -> Self {
        Self {
            headless: true,
            agent_script: CounterAgentScript::default(),
        }
    }

    pub fn windowed() -> Self {
        Self {
            headless: false,
            agent_script: CounterAgentScript::default(),
        }
    }

    pub fn with_agent_script(mut self, script: CounterAgentScript) -> Self {
        self.agent_script = script;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterRuntimeCliOptions {
    pub runtime: CounterRuntimeOptions,
    pub trace_jsonl_path: Option<PathBuf>,
}

impl CounterRuntimeCliOptions {
    pub fn from_env() -> Result<Self> {
        Self::parse(std::env::args().skip(1))
    }

    pub fn parse<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut headless = false;
        let mut script_path: Option<PathBuf> = None;
        let mut trace_jsonl_path: Option<PathBuf> = None;
        let mut exit_after_script = false;
        let mut args = args.into_iter().map(Into::into).peekable();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--headless" => headless = true,
                "--agent-script" => {
                    let Some(path) = args.next() else {
                        bail!("--agent-script requires a path");
                    };
                    script_path = Some(PathBuf::from(path));
                }
                "--trace-jsonl" => {
                    let Some(path) = args.next() else {
                        bail!("--trace-jsonl requires a path");
                    };
                    trace_jsonl_path = Some(PathBuf::from(path));
                }
                "--exit-after-script" => exit_after_script = true,
                "--help" | "-h" => {
                    println!("{}", counter_runtime_usage());
                    std::process::exit(0);
                }
                unknown => bail!("unknown ui_counter_runtime argument {unknown}"),
            }
        }

        if trace_jsonl_path.is_some() && !headless {
            bail!("--trace-jsonl requires --headless so the completed app state can be inspected");
        }
        if exit_after_script && !headless {
            bail!("--exit-after-script requires --headless");
        }

        let agent_script = match script_path {
            Some(path) => CounterAgentScript::from_ron_file(path)?,
            None => CounterAgentScript::default(),
        };

        Ok(Self {
            runtime: CounterRuntimeOptions {
                headless,
                agent_script,
            },
            trace_jsonl_path,
        })
    }
}

pub fn counter_runtime_usage() -> &'static str {
    "Usage: ui_counter_runtime [--headless] [--agent-script PATH] [--trace-jsonl PATH] [--exit-after-script]"
}

pub fn run_counter_runtime(options: CounterRuntimeCliOptions) -> Result<()> {
    let trace_jsonl_path = options.trace_jsonl_path.clone();
    let app = build_counter_app(options.runtime)?;

    if trace_jsonl_path.is_some() {
        let app = app.run_for_frames(1)?;
        let trace = app
            .world()
            .resource::<UiRuntimeTraceResource>()
            .context("UiRuntimeTraceResource should be installed by UiPlugin")?;
        write_trace_jsonl(trace, trace_jsonl_path.expect("trace path checked above"))?;
        return Ok(());
    }

    if app_is_headless(&app) {
        app.run_for_frames(1)?;
    } else {
        app.run()?;
    }
    Ok(())
}

fn app_is_headless(app: &App) -> bool {
    app.world()
        .resource::<CounterRuntimeModeResource>()
        .map(|mode| mode.headless)
        .unwrap_or(false)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Resource)]
struct CounterRuntimeModeResource {
    headless: bool,
}

pub fn build_counter_app(options: CounterRuntimeOptions) -> Result<App> {
    let mut app = if options.headless {
        App::headless()
    } else {
        App::new()
    };
    app.set_title(WINDOW_TITLE);
    app.insert_resource(CounterRuntimeModeResource {
        headless: options.headless,
    });
    app.add_plugins(default_plugins());
    app.add_plugin(RenderPlugin);
    app.add_plugin(UiPlugin);
    app.add_plugin(CounterPlugin::new(options.agent_script));
    Ok(app)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterPlugin {
    agent_script: CounterAgentScript,
}

impl CounterPlugin {
    pub fn new(agent_script: CounterAgentScript) -> Self {
        Self { agent_script }
    }
}

impl Plugin for CounterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Counter>();
        app.init_resource::<CounterRuntimeState>();
        app.init_resource::<UiActionDispatchReportsResource>();
        app.insert_resource(CounterAgentScriptResource::new(self.agent_script.clone()));
        app.mount_ui(CounterScreen::default());
        app.add_input_bindings([
            (
                CounterActionKind::Increment.input_action(),
                KeyCode::ArrowUp,
            ),
            (
                CounterActionKind::Decrement.input_action(),
                KeyCode::ArrowDown,
            ),
            (CounterActionKind::Reset.input_action(), KeyCode::KeyR),
        ]);
        app.add_systems(Update, dispatch_counter_actions_system);
        app.add_systems(
            RenderPrepare,
            evaluate_counter_screen_system.before(UiRuntimeSet::RenderPublication),
        );
    }
}

fn dispatch_counter_actions_system(
    input: Res<InputState>,
    mut counter: ResMut<Counter>,
    mut runtime_state: ResMut<CounterRuntimeState>,
    mut script: ResMut<CounterAgentScriptResource>,
    mut reports: ResMut<UiActionDispatchReportsResource>,
    mut trace: ResMut<UiRuntimeTraceResource>,
    mut diagnostics: ResMut<UiRuntimeDiagnosticsResource>,
    mounts: Res<UiMountRequestsResource>,
) {
    let mut invocations = script.drain_pending_invocations();
    invocations.extend(
        CounterActionKind::all()
            .into_iter()
            .filter(|kind| input.action_pressed(kind.input_action()))
            .map(|action| CounterActionInvocation {
                source: CounterActionSource::HumanKeyboard,
                action,
            }),
    );

    let surface_instance_id = mounts
        .mounted_sessions()
        .first()
        .map(|session| session.surface_instance_id());
    for invocation in invocations {
        dispatch_counter_invocation(
            invocation,
            surface_instance_id,
            &mut *counter,
            &mut *runtime_state,
            &mut *reports,
            &mut *trace,
            &mut *diagnostics,
        );
    }
}

fn dispatch_counter_invocation(
    invocation: CounterActionInvocation,
    surface_instance_id: Option<ui_surface::SurfaceInstanceId>,
    counter: &mut Counter,
    runtime_state: &mut CounterRuntimeState,
    reports: &mut UiActionDispatchReportsResource,
    trace: &mut UiRuntimeTraceResource,
    diagnostics: &mut UiRuntimeDiagnosticsResource,
) {
    let action = CounterAction {
        kind: invocation.action,
    };
    let handler = CounterActionHandler;
    let host = counter_game_host();
    let mut executor = CounterHostExecutor {
        source: invocation.source,
        counter,
        runtime_state,
    };
    let mut event = UiActionEvent::new(counter_event_packet(invocation));
    if let Some(surface_instance_id) = surface_instance_id {
        event = event.with_surface_instance_id(surface_instance_id);
    }

    dispatch_ui_action(
        &action,
        &handler,
        &event,
        &host,
        &mut executor,
        reports,
        trace,
        diagnostics,
    );
}

fn counter_game_host() -> GameHost {
    CounterActionKind::all()
        .into_iter()
        .fold(GameHost::new(ROUTE_MAP_VERSION), |host, kind| {
            let action = CounterAction { kind };
            let handler = CounterActionHandler;
            let intent = handler.host_intent(&action);
            host.with_mapping(intent.to_host_route_mapping(ROUTE_MAP_VERSION))
        })
}

fn counter_event_packet(invocation: CounterActionInvocation) -> UiEventPacket {
    UiEventPacket::new(
        RouteId::new(invocation.action.route()),
        ACTION_SCHEMA_VERSION,
        counter_payload_schema(),
        UiSchemaValue::object([
            (
                "action",
                UiSchemaValue::string(invocation.action.domain_command()),
            ),
            (
                "source",
                UiSchemaValue::string(match invocation.source {
                    CounterActionSource::HumanKeyboard => "human_keyboard",
                    CounterActionSource::AgentScript => "agent_script",
                }),
            ),
        ]),
    )
    .with_capability(invocation.action.capability())
    .with_source_control(invocation.source.source_control_id(invocation.action))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct CounterAction {
    kind: CounterActionKind,
}

impl UiAction for CounterAction {
    fn action_descriptor(&self) -> UiTypedActionDescriptor {
        UiTypedActionDescriptor::new(
            UiTypedActionId::new(self.kind.route()),
            RouteId::new(self.kind.route()),
            ACTION_SCHEMA_VERSION,
            counter_payload_schema(),
            self.kind.capability(),
        )
    }
}

struct CounterActionHandler;

impl UiActionHandler<CounterAction> for CounterActionHandler {
    fn host_intent(&self, action: &CounterAction) -> UiHostMutationIntent {
        UiHostMutationIntent::new(
            action.action_descriptor(),
            HostCommand::new(HostKind::Game, action.kind.route()),
        )
        .with_domain_command(DomainCommand::new("counter", action.kind.domain_command()))
    }
}

struct CounterHostExecutor<'a> {
    source: CounterActionSource,
    counter: &'a mut Counter,
    runtime_state: &'a mut CounterRuntimeState,
}

impl UiHostActionExecutor for CounterHostExecutor<'_> {
    fn apply(
        &mut self,
        intent: &UiHostMutationIntent,
        packet: &UiEventPacket,
        mapping: &ui_hosts::HostRouteMapping,
    ) -> std::result::Result<UiHostMutationReceipt, UiHostMutationRejection> {
        let Some(kind) = CounterActionKind::from_route(packet.route.as_str()) else {
            return Err(UiHostMutationRejection::rejected_by_host());
        };
        if mapping.host_command.host != HostKind::Game
            || intent.host_command().host != HostKind::Game
        {
            return Err(UiHostMutationRejection::rejected_by_host());
        }

        let (before, after) = self.counter.apply(kind);
        self.runtime_state
            .record_action(CounterRuntimeActionRecord::new(
                self.source,
                kind,
                before,
                after,
                packet.route.as_str(),
                &mapping.host_command.command_id,
                mapping
                    .domain_command
                    .as_ref()
                    .map(|command| command.command_id.as_str())
                    .unwrap_or_default(),
            ));
        Ok(UiHostMutationReceipt::from_intent(intent))
    }
}

fn counter_payload_schema() -> UiSchemaRef {
    UiSchemaRef::new(
        COUNTER_ACTION_PAYLOAD_SCHEMA_ID,
        ACTION_SCHEMA_VERSION.value(),
    )
}

fn evaluate_counter_screen_system(
    counter: Res<Counter>,
    runtime_state: Res<CounterRuntimeState>,
    mounts: Res<UiMountRequestsResource>,
    mut runtime: ResMut<UiRuntimeEvaluationResource>,
    mut trace: ResMut<UiRuntimeTraceResource>,
    mut diagnostics: ResMut<UiRuntimeDiagnosticsResource>,
) {
    let input = counter_evaluation_input(&*counter, &*runtime_state);
    let mounted_session = mounts.mounted_sessions().first();
    runtime.evaluate(
        &input,
        mounted_session,
        counter_evaluation_context(&*counter, &*runtime_state),
        &mut *trace,
        &mut *diagnostics,
    );
}

pub fn counter_evaluation_input(
    counter: &Counter,
    runtime_state: &CounterRuntimeState,
) -> UiRuntimeEvaluationInput {
    let registry = ControlPackageRegistry::new()
        .with_package(runenwerk_control_package())
        .expect("runenwerk controls package should register");
    let source = CounterScreen::from_state(counter, runtime_state).into_ui_source();
    let lowering = source.lower_with_registry_snapshot(&registry.snapshot());

    assert!(lowering.passed(), "{:?}", lowering.formation().diagnostics);
    UiRuntimeEvaluationInput::from_lowering_report(&lowering)
}

fn counter_evaluation_context(
    counter: &Counter,
    runtime_state: &CounterRuntimeState,
) -> UiEvaluationContext {
    UiEvaluationContext::default()
        .with_host_data(HostDataSnapshot::new(
            COUNTER_VALUE_ENDPOINT,
            UiSchemaValue::string(counter.display_text()),
            counter.revision(),
        ))
        .with_host_data(HostDataSnapshot::new(
            TRACE_STATUS_ENDPOINT,
            UiSchemaValue::string(runtime_state.status()),
            counter.revision(),
        ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterScreen {
    value_text: String,
    status: String,
    trace_lines: Vec<String>,
}

impl Default for CounterScreen {
    fn default() -> Self {
        Self {
            value_text: Counter::default().display_text(),
            status: CounterRuntimeState::default().status,
            trace_lines: CounterRuntimeState::default().trace_lines(),
        }
    }
}

impl CounterScreen {
    fn from_state(counter: &Counter, runtime_state: &CounterRuntimeState) -> Self {
        Self {
            value_text: counter.display_text(),
            status: runtime_state.status().to_owned(),
            trace_lines: runtime_state.trace_lines(),
        }
    }
}

impl From<CounterScreen> for UiMountRequest {
    fn from(_screen: CounterScreen) -> Self {
        UiMountRequest::new(COUNTER_SCREEN_ID).with_report_label(WINDOW_TITLE)
    }
}

impl UiScreen for CounterScreen {
    fn screen_id(&self) -> UiTypedScreenId {
        UiTypedScreenId::new(COUNTER_SCREEN_ID)
    }

    fn build_source(&self) -> UiTypedSource {
        let trace_children = self
            .trace_lines
            .iter()
            .enumerate()
            .map(|(index, line)| UiNodeDefinition::Label {
                id: AuthoredId::new(format!("counter.trace.line.{index}")),
                label: UiValueBinding::static_text(line),
                availability: None,
            })
            .collect::<Vec<_>>();

        UiTypedSource::new(
            self.screen_id(),
            UiProgramSourceId::new(COUNTER_SOURCE_ID),
            UiNodeDefinition::Column {
                id: AuthoredId::new("counter.root"),
                children: vec![
                    UiNodeDefinition::Label {
                        id: AuthoredId::new("counter.title"),
                        label: UiValueBinding::static_text(WINDOW_TITLE),
                        availability: None,
                    },
                    UiNodeDefinition::Label {
                        id: AuthoredId::new("counter.value.label"),
                        label: UiValueBinding::static_text(&self.value_text),
                        availability: None,
                    },
                    counter_output_control(),
                    UiNodeDefinition::Row {
                        id: AuthoredId::new("counter.actions"),
                        children: CounterActionKind::all()
                            .into_iter()
                            .map(counter_action_control)
                            .collect(),
                    },
                    UiNodeDefinition::Label {
                        id: AuthoredId::new("counter.status.label"),
                        label: UiValueBinding::static_text(&self.status),
                        availability: None,
                    },
                    counter_status_control(),
                    UiNodeDefinition::Column {
                        id: AuthoredId::new("counter.trace"),
                        children: trace_children,
                    },
                ],
            },
        )
    }
}

fn counter_output_control() -> UiNodeDefinition {
    let mut properties = BTreeMap::new();
    properties.insert(
        "label".to_owned(),
        AuthoredControlValue::String("Counter value".to_owned()),
    );

    let mut bindings = BTreeMap::new();
    bindings.insert(
        "selected".to_owned(),
        AuthoredBindingRef::new(COUNTER_VALUE_ENDPOINT),
    );

    UiNodeDefinition::Control {
        id: AuthoredId::new("counter.value"),
        kind: AuthoredControlKindId::new(BUTTON_CONTROL_KIND_ID),
        properties,
        bindings,
        route: None,
        accessibility: Some(AuthoredControlAccessibilityDefinition {
            role: "button".to_owned(),
            label: Some("Counter value".to_owned()),
        }),
        children: Vec::new(),
    }
}

fn counter_status_control() -> UiNodeDefinition {
    let mut properties = BTreeMap::new();
    properties.insert(
        "label".to_owned(),
        AuthoredControlValue::String("Counter status".to_owned()),
    );

    let mut bindings = BTreeMap::new();
    bindings.insert(
        "selected".to_owned(),
        AuthoredBindingRef::new(TRACE_STATUS_ENDPOINT),
    );

    UiNodeDefinition::Control {
        id: AuthoredId::new("counter.trace.status"),
        kind: AuthoredControlKindId::new(BUTTON_CONTROL_KIND_ID),
        properties,
        bindings,
        route: None,
        accessibility: Some(AuthoredControlAccessibilityDefinition {
            role: "button".to_owned(),
            label: Some("Counter runtime status".to_owned()),
        }),
        children: Vec::new(),
    }
}

fn counter_action_control(kind: CounterActionKind) -> UiNodeDefinition {
    let mut properties = BTreeMap::new();
    properties.insert(
        "label".to_owned(),
        AuthoredControlValue::String(kind.label().to_owned()),
    );

    UiNodeDefinition::Control {
        id: AuthoredId::new(format!("counter.action.{}", kind.domain_command())),
        kind: AuthoredControlKindId::new(BUTTON_CONTROL_KIND_ID),
        properties,
        bindings: BTreeMap::new(),
        route: Some(AuthoredRouteId::new(kind.route())),
        accessibility: Some(AuthoredControlAccessibilityDefinition {
            role: "button".to_owned(),
            label: Some(kind.label().to_owned()),
        }),
        children: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CounterTraceJsonLine {
    sequence: usize,
    kind: &'static str,
    action_id: Option<String>,
    route: Option<String>,
    host: Option<String>,
    surface_instance_id: Option<String>,
    failure_reason: Option<String>,
    diagnostic_code: Option<String>,
    runtime_id: Option<String>,
    source_id: Option<String>,
    program_id: Option<String>,
    dirty_cause: Option<String>,
    render_producer_id: Option<String>,
    render_surface_id: Option<String>,
    frame_revision: Option<u64>,
    frame_publication_status: Option<String>,
}

impl CounterTraceJsonLine {
    fn from_event(sequence: usize, event: &UiRuntimeTraceEvent) -> Self {
        Self {
            sequence,
            kind: trace_kind_name(event.kind()),
            action_id: event.action_id().map(|id| id.as_str().to_owned()),
            route: event.route().map(|route| route.as_str().to_owned()),
            host: event.host().map(|host| format!("{host:?}")),
            surface_instance_id: event
                .surface_instance_id()
                .map(|surface| surface.raw().to_string()),
            failure_reason: event
                .failure_reason()
                .map(|reason| reason.as_str().to_owned()),
            diagnostic_code: event.diagnostic_code().map(|code| format!("{code:?}")),
            runtime_id: event.runtime_id().map(str::to_owned),
            source_id: event.source_id().map(str::to_owned),
            program_id: event.program_id().map(str::to_owned),
            dirty_cause: event.dirty_cause().map(|cause| cause.as_str().to_owned()),
            render_producer_id: event
                .render_producer_id()
                .map(|producer| format!("{producer:?}")),
            render_surface_id: event
                .render_surface_id()
                .map(|surface| format!("{surface:?}")),
            frame_revision: event.frame_revision(),
            frame_publication_status: event
                .frame_publication_status()
                .map(frame_publication_status_name)
                .map(str::to_owned),
        }
    }
}

pub fn trace_events_to_jsonl(trace: &UiRuntimeTraceResource) -> Result<String> {
    let mut jsonl = String::new();
    for (sequence, event) in trace.events().iter().enumerate() {
        let line = CounterTraceJsonLine::from_event(sequence, event);
        jsonl.push_str(&serde_json::to_string(&line)?);
        jsonl.push('\n');
    }
    Ok(jsonl)
}

pub fn write_trace_jsonl(trace: &UiRuntimeTraceResource, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create trace directory {}", parent.display()))?;
    }
    fs::write(path, trace_events_to_jsonl(trace)?)
        .with_context(|| format!("failed to write trace jsonl {}", path.display()))
}

fn trace_kind_name(kind: UiRuntimeTraceEventKind) -> &'static str {
    match kind {
        UiRuntimeTraceEventKind::Mounted => "mounted",
        UiRuntimeTraceEventKind::Input => "input",
        UiRuntimeTraceEventKind::Route => "route",
        UiRuntimeTraceEventKind::Capability => "capability",
        UiRuntimeTraceEventKind::Dispatch => "dispatch",
        UiRuntimeTraceEventKind::Mutation => "mutation",
        UiRuntimeTraceEventKind::Rejection => "rejection",
        UiRuntimeTraceEventKind::Diagnostic => "diagnostic",
        UiRuntimeTraceEventKind::RuntimeEvaluation => "runtime_evaluation",
        UiRuntimeTraceEventKind::StateSnapshot => "state_snapshot",
        UiRuntimeTraceEventKind::Invalidation => "invalidation",
        UiRuntimeTraceEventKind::UiFramePublished => "ui_frame_published",
        UiRuntimeTraceEventKind::UiFramePresented => "ui_frame_presented",
    }
}

fn frame_publication_status_name(status: UiRuntimeFramePublicationStatus) -> &'static str {
    match status {
        UiRuntimeFramePublicationStatus::Published => "published",
        UiRuntimeFramePublicationStatus::MissingRuntimeEvaluation => "missing_runtime_evaluation",
    }
}
