use runenui_core::{
    Effects, Element, ElementId, FontFamilyName, GenericFontFamily, HostProtocol, SemanticCommand,
    UiApp, View, button, column, text,
};
#[cfg(test)]
use runenui_core::{LogicalLength, LogicalRect, PaintPrimitive, StyleEnvironment};
#[cfg(test)]
use runenui_runtime::LogicalSize;
use runenui_runtime::{
    AppRuntime, HostRequestToken, InputObservationError, InputPumpBatch, InputShutdownBatch,
    PumpBudget, SurfaceBuildContext, SurfacePublication,
};
use ui_artifacts::UiRuntimeArtifact;
use ui_controls::{BUTTON_CONTROL_KIND_ID, ControlPackageRegistrySnapshot, ControlSchemaRole};
use ui_definition::{AuthoredControlValue, UiNodeDefinition, UiValue, UiValueBinding};
use ui_evaluator::{UiEvaluationContext, UiEvaluator};
use ui_program::{
    AccessibilityRole, BindingEndpoint, InteractionTrigger, StateRequirementLifecycle,
    UiEventPacket, UiEventSourceControlId,
};
use ui_schema::UiSchemaValue;
use ui_state::UiStateModel;

use super::typed_app_host::{TypedRuntime, TypedRuntimeOps};
use super::{UiTypedScreenId, UiTypedSource};

#[cfg(test)]
const CONTROLLED_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/fonts/JetBrainsMono-Regular.ttf"
));
const RUNTIME_PUMP_WORK_LIMIT: usize = 128;
#[cfg(test)]
const RUNTIME_SURFACE_WIDTH: u16 = 320;
#[cfg(test)]
const RUNTIME_SURFACE_HEIGHT: u16 = 160;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiRuntimeSlotId(u64);

impl UiRuntimeSlotId {
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Explicit font inputs supplied by a host; no test font is a production default.
#[derive(Clone, Debug, Default)]
pub struct UiRuntimeFontConfiguration {
    font_bytes: Vec<Vec<u8>>,
    generic_mappings: Vec<(GenericFontFamily, Vec<FontFamilyName>)>,
}

impl UiRuntimeFontConfiguration {
    /// Creates a font profile from host-owned immutable font-file data.
    #[must_use]
    pub fn new(font_bytes: Vec<Vec<u8>>) -> Self {
        Self {
            font_bytes,
            generic_mappings: Vec::new(),
        }
    }

    /// Adds an explicit generic family mapping resolved by RunenUI's font source.
    #[must_use]
    pub fn with_generic_mapping(
        mut self,
        generic: GenericFontFamily,
        families: Vec<FontFamilyName>,
    ) -> Self {
        self.generic_mappings.push((generic, families));
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiRuntimeSlotMountFailure {
    ControlRegistry,
    ProgramFormation,
    UnsupportedProjection,
    FontProvisioning,
}

impl UiRuntimeSlotMountFailure {
    pub const fn message(self) -> &'static str {
        match self {
            Self::ControlRegistry => "RunenUI slot mount could not form the control registry",
            Self::ProgramFormation => "RunenUI slot mount could not form a valid UiProgram",
            Self::UnsupportedProjection => "RunenUI slot mount rejected an unsupported projection",
            Self::FontProvisioning => {
                "RunenUI slot mount could not apply caller-provided font sources"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiRuntimeSlotOperationFailure {
    MissingSlot,
    EvaluationDiagnostics,
    UnsupportedProjection,
    ActionRejected,
    AutomationRejected,
    HostResponseRejected,
    RuntimeDidNotQuiesce,
    InputObservation(InputObservationError),
    PublicationRevisionExhausted,
    PublicationRejected,
    InvalidAuthoredId,
    WrongAppKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiRuntimeSlotMountReport {
    screen_id: UiTypedScreenId,
    slot_id: Option<UiRuntimeSlotId>,
    failure: Option<UiRuntimeSlotMountFailure>,
}

impl UiRuntimeSlotMountReport {
    fn accepted(screen_id: UiTypedScreenId, slot_id: UiRuntimeSlotId) -> Self {
        Self {
            screen_id,
            slot_id: Some(slot_id),
            failure: None,
        }
    }

    pub(crate) fn rejected(screen_id: UiTypedScreenId, failure: UiRuntimeSlotMountFailure) -> Self {
        Self {
            screen_id,
            slot_id: None,
            failure: Some(failure),
        }
    }

    #[must_use]
    pub fn screen_id(&self) -> &UiTypedScreenId {
        &self.screen_id
    }

    #[must_use]
    pub const fn slot_id(&self) -> Option<UiRuntimeSlotId> {
        self.slot_id
    }

    #[must_use]
    pub const fn failure(&self) -> Option<UiRuntimeSlotMountFailure> {
        self.failure
    }

    #[must_use]
    pub const fn accepted_mount(&self) -> bool {
        self.slot_id.is_some() && self.failure.is_none()
    }
}

pub struct UiRuntimePendingEventRequest {
    token: HostRequestToken,
    packet: UiEventPacket,
}

impl UiRuntimePendingEventRequest {
    #[must_use]
    pub fn packet(&self) -> &UiEventPacket {
        &self.packet
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiRuntimeHostRequestDisposition {
    Accepted,
    Rejected,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
struct UiRuntimeSlotPublicationFacts {
    authored_ids: Vec<String>,
    semantic_names: Vec<String>,
    layout_bounds: Vec<LogicalRect>,
    shaped_text_run_count: usize,
    all_bounds_non_zero: bool,
}

#[cfg(test)]
impl UiRuntimeSlotPublicationFacts {
    fn authored_ids(&self) -> &[String] {
        &self.authored_ids
    }

    fn semantic_names(&self) -> &[String] {
        &self.semantic_names
    }

    const fn shaped_text_run_count(&self) -> usize {
        self.shaped_text_run_count
    }

    const fn all_bounds_non_zero(&self) -> bool {
        self.all_bounds_non_zero
    }
}

#[derive(Clone)]
struct RunenwerkProgramUiState {
    source: UiTypedSource,
    artifact: UiRuntimeArtifact,
    resolved_state: UiStateModel,
    registry: ControlPackageRegistrySnapshot,
}

#[derive(Clone, Debug)]
enum RunenwerkProgramUiAction {
    ReplaceResolvedState(UiStateModel),
    ActivateRoute(UiEventPacket),
    HostRequestCompleted,
}

#[derive(Clone, Debug, PartialEq)]
enum RunenwerkUiHostCommand {
    ActivateRoute(UiEventPacket),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunenwerkUiHostResponse {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunenwerkUiHostResponseKind {
    Decision,
}

struct RunenwerkUiHostProtocol;

impl HostProtocol for RunenwerkUiHostProtocol {
    type Command = RunenwerkUiHostCommand;
    type Response = RunenwerkUiHostResponse;
    type ResponseKind = RunenwerkUiHostResponseKind;

    fn expected_response(_: &Self::Command) -> Self::ResponseKind {
        RunenwerkUiHostResponseKind::Decision
    }

    fn response_kind(_: &Self::Response) -> Self::ResponseKind {
        RunenwerkUiHostResponseKind::Decision
    }
}

struct RunenwerkProgramUiApp;

impl UiApp for RunenwerkProgramUiApp {
    type State = RunenwerkProgramUiState;
    type Action = RunenwerkProgramUiAction;
    type HostProtocol = RunenwerkUiHostProtocol;

    fn root(state: &Self::State) -> impl View<Self::Action> {
        project_node(
            state.source.root(),
            &state.source,
            &state.artifact,
            &state.resolved_state,
            &state.registry,
        )
        .unwrap_or_else(|error| {
            unreachable!("validated Runenwerk projection became invalid in RunenUI root: {error:?}")
        })
    }

    fn update(
        state: &mut Self::State,
        action: Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        match action {
            RunenwerkProgramUiAction::ReplaceResolvedState(resolved_state) => {
                state.resolved_state = resolved_state;
                Effects::none()
            }
            RunenwerkProgramUiAction::ActivateRoute(packet) => {
                Effects::host_request(None, RunenwerkUiHostCommand::ActivateRoute(packet), |_| {
                    RunenwerkProgramUiAction::HostRequestCompleted
                })
            }
            RunenwerkProgramUiAction::HostRequestCompleted => Effects::none(),
        }
    }
}

/// Counter-only authored-program behavior is kept separate from common typed
/// host operations. All variants live in the *same* Engine slot namespace.
enum UiRuntimeSlotKind {
    Counter(Box<AppRuntime<RunenwerkProgramUiApp>>),
    Typed(Box<dyn TypedRuntimeOps>),
}

impl UiRuntimeSlotKind {
    fn counter(&self) -> Result<&AppRuntime<RunenwerkProgramUiApp>, UiRuntimeSlotOperationFailure> {
        match self {
            Self::Counter(runtime) => Ok(runtime),
            Self::Typed(_) => Err(UiRuntimeSlotOperationFailure::WrongAppKind),
        }
    }

    fn counter_mut(
        &mut self,
    ) -> Result<&mut AppRuntime<RunenwerkProgramUiApp>, UiRuntimeSlotOperationFailure> {
        match self {
            Self::Counter(runtime) => Ok(runtime),
            Self::Typed(_) => Err(UiRuntimeSlotOperationFailure::WrongAppKind),
        }
    }

    fn typed<A: UiApp + 'static>(&self) -> Option<&AppRuntime<A>> {
        match self {
            Self::Typed(runtime) => runtime
                .as_any()
                .downcast_ref::<TypedRuntime<A>>()
                .map(|runtime| &runtime.runtime),
            Self::Counter(_) => None,
        }
    }

    fn typed_mut<A: UiApp + 'static>(&mut self) -> Option<&mut AppRuntime<A>> {
        match self {
            Self::Typed(runtime) => runtime
                .as_any_mut()
                .downcast_mut::<TypedRuntime<A>>()
                .map(|runtime| &mut runtime.runtime),
            Self::Counter(_) => None,
        }
    }

    fn pending_redraw(&mut self) -> bool {
        match self {
            Self::Counter(runtime) => runtime.take_redraw_request().is_some(),
            Self::Typed(runtime) => runtime.pending_redraw(),
        }
    }

    fn publish_surface(
        &mut self,
        context: &SurfaceBuildContext<'_>,
    ) -> Result<SurfacePublication, UiRuntimeSlotOperationFailure> {
        match self {
            Self::Counter(runtime) => runtime.publish_surface(context),
            Self::Typed(runtime) => runtime.publish_surface(context),
        }
        .map_err(|_| UiRuntimeSlotOperationFailure::PublicationRejected)
    }

    fn settle(&mut self) -> Result<(), UiRuntimeSlotOperationFailure> {
        match self {
            Self::Counter(runtime) => settle(runtime),
            Self::Typed(runtime) => settle_batch(runtime.pump(runtime_pump_budget())),
        }
    }

    fn shutdown(&mut self) -> Result<InputShutdownBatch, UiRuntimeSlotOperationFailure> {
        match self {
            Self::Counter(runtime) => runtime.shutdown(),
            Self::Typed(runtime) => runtime.shutdown(),
        }
        .map_err(UiRuntimeSlotOperationFailure::InputObservation)
    }
}

struct UiRuntimeSlot {
    id: UiRuntimeSlotId,
    screen_id: UiTypedScreenId,
    runtime: UiRuntimeSlotKind,
    /// Every accepted action/response changes this generation, even on pump failure.
    ingress_epoch: u64,
    /// Incomplete transactions cannot be published or superseded by new ingress.
    requires_settlement: bool,
}

impl UiRuntimeSlot {
    fn next_ingress_epoch(&self) -> Result<u64, UiRuntimeSlotOperationFailure> {
        if self.requires_settlement {
            return Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce);
        }
        self.ingress_epoch
            .checked_add(1)
            .ok_or(UiRuntimeSlotOperationFailure::PublicationRevisionExhausted)
    }

    fn accepted_ingress(&mut self, next: u64) -> Result<(), UiRuntimeSlotOperationFailure> {
        self.ingress_epoch = next;
        self.requires_settlement = true;
        self.settle_pending()
    }

    fn settle_pending(&mut self) -> Result<(), UiRuntimeSlotOperationFailure> {
        if self.requires_settlement {
            self.runtime.settle()?;
            self.requires_settlement = false;
        }
        Ok(())
    }
}

#[derive(runen_ecs::Resource)]
pub struct UiRuntimeSlotsResource {
    slots: Vec<UiRuntimeSlot>,
    next_slot_id: u64,
}

impl Default for UiRuntimeSlotsResource {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            next_slot_id: 1,
        }
    }
}

impl UiRuntimeSlotsResource {
    pub(crate) fn mount(
        &mut self,
        source: UiTypedSource,
        registry: &ControlPackageRegistrySnapshot,
    ) -> UiRuntimeSlotMountReport {
        self.mount_with_fonts(source, registry, &UiRuntimeFontConfiguration::default())
    }

    pub(crate) fn mount_with_fonts(
        &mut self,
        source: UiTypedSource,
        registry: &ControlPackageRegistrySnapshot,
        fonts: &UiRuntimeFontConfiguration,
    ) -> UiRuntimeSlotMountReport {
        let screen_id = source.screen_id().clone();
        let lowering = source.lower_with_registry_snapshot(registry);
        if !lowering.passed() {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::ProgramFormation,
            );
        }

        let artifact = UiRuntimeArtifact::from_program(lowering.program());
        let resolved_state = UiStateModel::default();
        if validate_projection(&source, &artifact, &resolved_state, registry).is_err() {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::UnsupportedProjection,
            );
        }

        let mut runtime = AppRuntime::<RunenwerkProgramUiApp>::mount(RunenwerkProgramUiState {
            source,
            artifact,
            resolved_state,
            registry: registry.clone(),
        });
        if let Err(failure) = provision_text_fonts(&mut runtime, fonts) {
            let _ = runtime.shutdown();
            return UiRuntimeSlotMountReport::rejected(screen_id, failure);
        }

        let Some(next_slot_id) = self.next_slot_id.checked_add(1) else {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::UnsupportedProjection,
            );
        };
        let slot_id = UiRuntimeSlotId(self.next_slot_id);
        self.next_slot_id = next_slot_id;
        self.slots.push(UiRuntimeSlot {
            id: slot_id,
            screen_id: screen_id.clone(),
            runtime: UiRuntimeSlotKind::Counter(Box::new(runtime)),
            ingress_epoch: 1,
            requires_settlement: false,
        });
        UiRuntimeSlotMountReport::accepted(screen_id, slot_id)
    }

    /// Mounts one ordinary concrete UiApp in the *existing* Engine slot registry.
    /// This does not duplicate the normalized input or GPU presentation owners.
    ///
    /// # Errors
    /// Font admission or exhausted slot identity rejects before publication.
    pub fn mount_app<A>(
        &mut self,
        screen_id: UiTypedScreenId,
        state: A::State,
        fonts: &UiRuntimeFontConfiguration,
    ) -> UiRuntimeSlotMountReport
    where
        A: UiApp + 'static,
    {
        let Some(next_slot_id) = self.next_slot_id.checked_add(1) else {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::UnsupportedProjection,
            );
        };
        let mut runtime = AppRuntime::<A>::mount(state);
        if let Err(error) = provision_text_fonts(&mut runtime, fonts) {
            let _ = runtime.shutdown();
            return UiRuntimeSlotMountReport::rejected(screen_id, error);
        }
        let id = UiRuntimeSlotId(self.next_slot_id);
        self.next_slot_id = next_slot_id;
        self.slots.push(UiRuntimeSlot {
            id,
            screen_id: screen_id.clone(),
            runtime: UiRuntimeSlotKind::Typed(Box::new(TypedRuntime { runtime })),
            ingress_epoch: 1,
            requires_settlement: false,
        });
        UiRuntimeSlotMountReport::accepted(screen_id, id)
    }

    /// Typed action ingress increments the Engine epoch immediately upon RunenUI
    /// admission. Incomplete fallible pumps retain the new generation and block
    /// publication until explicitly settled; they never imply rollback.
    pub fn submit_app_action<A>(
        &mut self,
        slot_id: UiRuntimeSlotId,
        action: A::Action,
    ) -> Result<(), UiRuntimeSlotOperationFailure>
    where
        A: UiApp + 'static,
    {
        let slot = self
            .slot_mut(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        let next = slot.next_ingress_epoch()?;
        let runtime = slot
            .runtime
            .typed_mut::<A>()
            .ok_or(UiRuntimeSlotOperationFailure::WrongAppKind)?;
        runtime
            .submit_action(action)
            .map_err(|_| UiRuntimeSlotOperationFailure::ActionRejected)?;
        slot.accepted_ingress(next)
    }

    /// Observe a concrete application's typed state and host requests without
    /// exposing a mutable runtime that could bypass the Engine publication epoch.
    pub fn observe_app<A, R>(
        &self,
        slot_id: UiRuntimeSlotId,
        observe: impl FnOnce(&AppRuntime<A>) -> R,
    ) -> Result<R, UiRuntimeSlotOperationFailure>
    where
        A: UiApp + 'static,
    {
        let slot = self
            .slot(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        let runtime = slot
            .runtime
            .typed::<A>()
            .ok_or(UiRuntimeSlotOperationFailure::WrongAppKind)?;
        Ok(observe(runtime))
    }

    /// Completes only this concrete application's own typed host response.
    pub fn complete_app_host_request<A>(
        &mut self,
        slot_id: UiRuntimeSlotId,
        token: &HostRequestToken,
        response: <A::HostProtocol as HostProtocol>::Response,
    ) -> Result<(), UiRuntimeSlotOperationFailure>
    where
        A: UiApp + 'static,
    {
        let slot = self
            .slot_mut(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        let next = slot.next_ingress_epoch()?;
        let runtime = slot
            .runtime
            .typed_mut::<A>()
            .ok_or(UiRuntimeSlotOperationFailure::WrongAppKind)?;
        runtime
            .complete_host_request(token, response)
            .map_err(|_| UiRuntimeSlotOperationFailure::HostResponseRejected)?;
        slot.accepted_ingress(next)
    }

    #[must_use]
    pub fn contains(&self, slot_id: UiRuntimeSlotId) -> bool {
        self.slots.iter().any(|slot| slot.id == slot_id)
    }

    pub fn slot_ids(&self) -> impl Iterator<Item = UiRuntimeSlotId> + '_ {
        self.slots.iter().map(|slot| slot.id)
    }

    /// Returns the Engine ingress generation that might require new RunenUI publication.
    /// This does not mirror or replace RunenUI's own redraw revision.
    #[must_use]
    pub fn ingress_epoch(&self, slot_id: UiRuntimeSlotId) -> Option<u64> {
        self.slot(slot_id).map(|slot| slot.ingress_epoch)
    }

    /// Resumes a partially processed ingress on the same RunenUI runtime.
    /// Publication stays blocked until the pending work is quiescent.
    pub fn settle_slot(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<(), UiRuntimeSlotOperationFailure> {
        self.slot_mut(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?
            .settle_pending()
    }

    /// Checks RunenUI's own redraw authority; no Engine-owned dirty mirror is used.
    ///
    /// RunenUI's take_redraw_request returns a revision witness without consuming
    /// the dirty revision. A successful publish_surface acknowledges that same
    /// runtime-owned revision transactionally.
    pub fn has_pending_redraw(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<bool, UiRuntimeSlotOperationFailure> {
        let slot = self
            .slot_mut(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        if slot.requires_settlement {
            return Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce);
        }
        Ok(slot.runtime.pending_redraw())
    }

    /// Publishes a slot's complete aligned paint, layout, semantics, and input snapshot.
    /// RunenUI acknowledges its redraw request transactionally on successful publication.
    pub fn publish_surface_for_slot(
        &mut self,
        slot_id: UiRuntimeSlotId,
        context: &SurfaceBuildContext<'_>,
    ) -> Result<SurfacePublication, UiRuntimeSlotOperationFailure> {
        let slot = self
            .slot_mut(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        if slot.requires_settlement {
            return Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce);
        }
        slot.runtime.publish_surface(context)
    }

    #[must_use]
    pub fn screen_id(&self, slot_id: UiRuntimeSlotId) -> Option<&UiTypedScreenId> {
        self.slot(slot_id).map(|slot| &slot.screen_id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn update_state(
        &mut self,
        slot_id: UiRuntimeSlotId,
        context: UiEvaluationContext,
    ) -> Result<(), UiRuntimeSlotOperationFailure> {
        let slot = self
            .slot_mut(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        let next = slot.next_ingress_epoch()?;
        let runtime = slot.runtime.counter_mut()?;
        let artifact = runtime.state().artifact.clone();
        let mut resolved_state = runtime.state().resolved_state.clone();
        let output = UiEvaluator.evaluate_with_context(&artifact, &mut resolved_state, context);
        if !output.diagnostics.is_empty() {
            return Err(UiRuntimeSlotOperationFailure::EvaluationDiagnostics);
        }
        validate_projection(
            &runtime.state().source,
            &artifact,
            &resolved_state,
            &runtime.state().registry,
        )
        .map_err(|_| UiRuntimeSlotOperationFailure::UnsupportedProjection)?;
        runtime
            .submit_action(RunenwerkProgramUiAction::ReplaceResolvedState(
                resolved_state,
            ))
            .map_err(|_| UiRuntimeSlotOperationFailure::ActionRejected)?;
        slot.accepted_ingress(next)
    }

    pub fn activate_authored(
        &mut self,
        slot_id: UiRuntimeSlotId,
        authored_id: &str,
    ) -> Result<(), UiRuntimeSlotOperationFailure> {
        let slot = self
            .slot_mut(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        let authored_id = ElementId::new(authored_id)
            .map_err(|_| UiRuntimeSlotOperationFailure::InvalidAuthoredId)?;
        let next = slot.next_ingress_epoch()?;
        let runtime = slot.runtime.counter_mut()?;
        runtime
            .submit_automation_command(authored_id, SemanticCommand::Activate)
            .map_err(|_| UiRuntimeSlotOperationFailure::AutomationRejected)?;
        slot.accepted_ingress(next)
    }

    /// Returns currently pending RunenUI route-intent requests without completing them.
    ///
    /// Runenwerk route/capability/domain authorization remains authoritative. The caller must
    /// complete each returned request only after that authorization/mutation decision is known.
    pub fn pending_event_requests(
        &self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<Vec<UiRuntimePendingEventRequest>, UiRuntimeSlotOperationFailure> {
        let slot = self
            .slot(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        Ok(slot
            .runtime
            .counter()?
            .pending_host_requests()
            .into_iter()
            .map(|request| UiRuntimePendingEventRequest {
                token: request.token(),
                packet: match request.command() {
                    RunenwerkUiHostCommand::ActivateRoute(packet) => packet.clone(),
                },
            })
            .collect())
    }

    pub fn complete_event_request(
        &mut self,
        slot_id: UiRuntimeSlotId,
        request: UiRuntimePendingEventRequest,
        disposition: UiRuntimeHostRequestDisposition,
    ) -> Result<(), UiRuntimeSlotOperationFailure> {
        let slot = self
            .slot_mut(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        let response = match disposition {
            UiRuntimeHostRequestDisposition::Accepted => RunenwerkUiHostResponse::Accepted,
            UiRuntimeHostRequestDisposition::Rejected => RunenwerkUiHostResponse::Rejected,
        };
        let next = slot.next_ingress_epoch()?;
        let runtime = slot.runtime.counter_mut()?;
        runtime
            .complete_host_request(&request.token, response)
            .map_err(|_| UiRuntimeSlotOperationFailure::HostResponseRejected)?;
        slot.accepted_ingress(next)
    }

    #[cfg(test)]
    fn resolved_state_value(
        &self,
        slot_id: UiRuntimeSlotId,
        requirement_id: &str,
    ) -> Option<&UiSchemaValue> {
        self.slot(slot_id)?
            .runtime
            .counter()
            .ok()?
            .state()
            .resolved_state
            .value(requirement_id)
    }

    #[cfg(test)]
    fn publication_facts(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<UiRuntimeSlotPublicationFacts, UiRuntimeSlotOperationFailure> {
        let slot = self
            .slot_mut(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        let runtime = slot.runtime.counter_mut()?;
        register_controlled_test_font(runtime)?;
        publication_facts_for_runtime(runtime)
    }

    /// Returns the actual RunenUI scope-retirement facts to the owning native
    /// caller, rather than interpreting shutdown success as native-input clearance.
    /// No slot state is removed if shutdown fails.
    ///
    /// # Errors
    /// Returns the exact M13 input-observation error without retiring the slot.
    pub fn unmount_with_retirement(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<Option<InputShutdownBatch>, UiRuntimeSlotOperationFailure> {
        let Some(index) = self.slots.iter().position(|slot| slot.id == slot_id) else {
            return Ok(None);
        };
        let retirement = self.slots[index].runtime.shutdown()?;
        self.slots.remove(index);
        Ok(Some(retirement))
    }

    /// Counter-compatible unmount result. This is **not** native occurrence
    /// retirement; U9-C must use the full shutdown batch and exact slot scope.
    ///
    /// # Errors
    /// A failed RunenUI shutdown leaves the slot mounted and identifiable.
    pub fn unmount(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<bool, UiRuntimeSlotOperationFailure> {
        self.unmount_with_retirement(slot_id)
            .map(|retirement| retirement.is_some())
    }

    fn slot(&self, slot_id: UiRuntimeSlotId) -> Option<&UiRuntimeSlot> {
        self.slots.iter().find(|slot| slot.id == slot_id)
    }

    fn slot_mut(&mut self, slot_id: UiRuntimeSlotId) -> Option<&mut UiRuntimeSlot> {
        self.slots.iter_mut().find(|slot| slot.id == slot_id)
    }
}

#[cfg(test)]
fn publication_facts_for_runtime(
    runtime: &mut AppRuntime<RunenwerkProgramUiApp>,
) -> Result<UiRuntimeSlotPublicationFacts, UiRuntimeSlotOperationFailure> {
    let environment = StyleEnvironment::default();
    let context = SurfaceBuildContext::tight(
        &environment,
        LogicalSize::new(
            LogicalLength::from(RUNTIME_SURFACE_WIDTH),
            LogicalLength::from(RUNTIME_SURFACE_HEIGHT),
        ),
    );
    let publication = runtime
        .publish_surface(&context)
        .map_err(|_| UiRuntimeSlotOperationFailure::PublicationRejected)?;

    let authored_ids = publication
        .frame()
        .nodes()
        .iter()
        .filter_map(|node| node.authored_id().map(|id| id.as_str().to_owned()))
        .collect();
    let layout_bounds = publication
        .frame()
        .nodes()
        .iter()
        .map(|node| node.bounds())
        .collect::<Vec<_>>();
    let all_bounds_non_zero = layout_bounds
        .iter()
        .all(|bounds| bounds.width() > 0.0 && bounds.height() > 0.0);
    let semantic_names = publication
        .semantic_publication()
        .snapshot()
        .nodes()
        .iter()
        .filter_map(|node| node.name().map(str::to_owned))
        .collect();
    let shaped_text_run_count = publication
        .paint_scene()
        .items()
        .iter()
        .filter(|item| matches!(item.primitive(), PaintPrimitive::ShapedTextRun(_)))
        .count();

    Ok(UiRuntimeSlotPublicationFacts {
        authored_ids,
        semantic_names,
        layout_bounds,
        shaped_text_run_count,
        all_bounds_non_zero,
    })
}

/// Applies host-owned immutable font sources through RunenUI's text authority.
/// The slot is not published if this provisioning transaction fails.
fn provision_text_fonts<A: UiApp>(
    runtime: &mut AppRuntime<A>,
    configuration: &UiRuntimeFontConfiguration,
) -> Result<(), UiRuntimeSlotMountFailure> {
    for bytes in &configuration.font_bytes {
        let registered = runtime
            .register_text_font_bytes(bytes.clone())
            .map_err(|_| UiRuntimeSlotMountFailure::FontProvisioning)?;
        if registered == 0 {
            return Err(UiRuntimeSlotMountFailure::FontProvisioning);
        }
    }
    for (generic, families) in &configuration.generic_mappings {
        let mapped = runtime
            .set_text_generic_family_mapping(*generic, families)
            .map_err(|_| UiRuntimeSlotMountFailure::FontProvisioning)?;
        if !mapped {
            return Err(UiRuntimeSlotMountFailure::FontProvisioning);
        }
    }
    Ok(())
}

#[cfg(test)]
fn register_controlled_test_font(
    runtime: &mut AppRuntime<RunenwerkProgramUiApp>,
) -> Result<(), UiRuntimeSlotOperationFailure> {
    let registered = runtime
        .register_text_font_bytes(CONTROLLED_FONT_BYTES.to_vec())
        .map_err(|_| UiRuntimeSlotOperationFailure::PublicationRejected)?;
    if registered == 0 {
        return Err(UiRuntimeSlotOperationFailure::PublicationRejected);
    }

    let family = FontFamilyName::new("JetBrains Mono")
        .map_err(|_| UiRuntimeSlotOperationFailure::PublicationRejected)?;
    let mapped = runtime
        .set_text_generic_family_mapping(GenericFontFamily::SansSerif, &[family])
        .map_err(|_| UiRuntimeSlotOperationFailure::PublicationRejected)?;
    if !mapped {
        return Err(UiRuntimeSlotOperationFailure::PublicationRejected);
    }
    Ok(())
}

fn runtime_pump_budget() -> PumpBudget {
    PumpBudget::new(
        RUNTIME_PUMP_WORK_LIMIT,
        RUNTIME_PUMP_WORK_LIMIT,
        RUNTIME_PUMP_WORK_LIMIT,
        RUNTIME_PUMP_WORK_LIMIT,
    )
}

fn settle<A: UiApp>(runtime: &mut AppRuntime<A>) -> Result<(), UiRuntimeSlotOperationFailure> {
    settle_batch(runtime.pump(runtime_pump_budget()))
}

fn settle_batch(
    batch: Result<InputPumpBatch, InputObservationError>,
) -> Result<(), UiRuntimeSlotOperationFailure> {
    let batch = batch.map_err(UiRuntimeSlotOperationFailure::InputObservation)?;
    // Capacity-paused or partially processed work cannot authorize a new publication.
    if batch.pause_reason().is_none() && batch.report().is_quiescent() {
        Ok(())
    } else {
        Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectionError {
    UnsupportedNode,
    UnsupportedValue,
    MissingControl,
    DuplicateControl,
    MismatchedControl,
    MissingProperties,
    DuplicateProperties,
    MissingBinding,
    DuplicateBinding,
    MismatchedBinding,
    MissingState,
    DuplicateState,
    MismatchedState,
    MissingAccessibility,
    DuplicateAccessibility,
    MismatchedAccessibility,
    MissingRoute,
    DuplicateRoute,
    MismatchedRoute,
    MissingCapability,
    MismatchedCapability,
    MissingActionDescriptor,
    DuplicateActionDescriptor,
    MismatchedActionDescriptor,
    MissingRouteContract,
    DuplicateRouteContract,
    MismatchedRouteContract,
    MissingPayloadSchema,
    DuplicatePayloadSchema,
    InvalidPayload,
}

fn validate_projection(
    source: &UiTypedSource,
    artifact: &UiRuntimeArtifact,
    state: &UiStateModel,
    registry: &ControlPackageRegistrySnapshot,
) -> Result<(), ProjectionError> {
    project_node(source.root(), source, artifact, state, registry).map(|_| ())
}

fn project_node(
    node: &UiNodeDefinition,
    source: &UiTypedSource,
    artifact: &UiRuntimeArtifact,
    state: &UiStateModel,
    registry: &ControlPackageRegistrySnapshot,
) -> Result<Element<RunenwerkProgramUiAction>, ProjectionError> {
    match node {
        UiNodeDefinition::Column { id, children } => {
            let projected = children
                .iter()
                .map(|child| project_node(child, source, artifact, state, registry))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(column(projected).id(id.as_str()).into_element())
        }
        UiNodeDefinition::Label {
            id,
            label,
            availability,
        } => {
            if availability.is_some() {
                return Err(ProjectionError::UnsupportedNode);
            }
            let UiValueBinding::Static(UiValue::Text(label)) = label else {
                return Err(ProjectionError::UnsupportedValue);
            };
            Ok(text(label.clone()).id(id.as_str()).into_element())
        }
        UiNodeDefinition::Control {
            id,
            kind,
            properties,
            bindings,
            route,
            accessibility,
            children,
        } => {
            if kind.as_str() != BUTTON_CONTROL_KIND_ID || !children.is_empty() {
                return Err(ProjectionError::UnsupportedNode);
            }
            if properties.len() != 1 || !properties.contains_key("label") {
                return Err(ProjectionError::UnsupportedValue);
            }

            let expected_control_id = format!("control.{}", id.as_str());
            let mut controls = artifact
                .tables
                .controls
                .rows
                .iter()
                .filter(|row| row.node.node_id.as_str() == expected_control_id);
            let control = controls.next().ok_or(ProjectionError::MissingControl)?;
            if controls.next().is_some() {
                return Err(ProjectionError::DuplicateControl);
            }
            if control.node.control_kind.as_str() != BUTTON_CONTROL_KIND_ID {
                return Err(ProjectionError::MismatchedControl);
            }

            let mut property_rows = artifact
                .tables
                .properties
                .rows
                .iter()
                .filter(|row| row.snapshot.owner_control == control.node.node_id);
            let property_row = property_rows
                .next()
                .ok_or(ProjectionError::MissingProperties)?;
            if property_rows.next().is_some() {
                return Err(ProjectionError::DuplicateProperties);
            }

            let source_label = properties
                .get("label")
                .and_then(|value| match value {
                    AuthoredControlValue::String(value) => Some(value.as_str()),
                    _ => None,
                })
                .ok_or(ProjectionError::UnsupportedValue)?;
            let compiled_label = property_row
                .snapshot
                .get("label")
                .and_then(UiSchemaValue::as_str)
                .ok_or(ProjectionError::MissingProperties)?;
            if source_label != compiled_label {
                return Err(ProjectionError::MismatchedControl);
            }

            let label = compiled_label.to_owned();

            let Some(authored_binding) = bindings.get("selected") else {
                return Err(ProjectionError::MissingBinding);
            };
            if bindings.len() != 1 {
                return Err(ProjectionError::MismatchedBinding);
            }
            let mut binding_rows = artifact.tables.binding_snapshots.rows.iter().filter(|row| {
                matches!(
                    &row.binding.source,
                    BindingEndpoint::HostData { endpoint_id }
                        if endpoint_id.as_str() == authored_binding.as_str()
                )
            });
            let binding_row = binding_rows.next().ok_or(ProjectionError::MissingBinding)?;
            if binding_rows.next().is_some() {
                return Err(ProjectionError::DuplicateBinding);
            }
            let BindingEndpoint::UiState {
                requirement_id,
                endpoint_id,
            } = &binding_row.binding.target
            else {
                return Err(ProjectionError::MismatchedBinding);
            };
            if requirement_id.as_str() != endpoint_id.as_str() {
                return Err(ProjectionError::MismatchedBinding);
            }

            let mut state_rows = artifact
                .tables
                .state
                .rows
                .iter()
                .filter(|row| row.requirement.requirement_id == *requirement_id);
            let state_row = state_rows.next().ok_or(ProjectionError::MissingState)?;
            if state_rows.next().is_some() {
                return Err(ProjectionError::DuplicateState);
            }
            if state_row.requirement.owner_control != control.node.node_id
                || state_row.requirement.lifecycle != StateRequirementLifecycle::HostFed
                || state_row.requirement.schema != binding_row.binding.value_schema
            {
                return Err(ProjectionError::MismatchedState);
            }

            if state
                .value(requirement_id.as_str())
                .is_some_and(|value| value.as_bool().is_none())
            {
                return Err(ProjectionError::UnsupportedValue);
            }

            let authored_accessibility = accessibility
                .as_ref()
                .ok_or(ProjectionError::MissingAccessibility)?;
            let authored_accessibility_label = authored_accessibility
                .label
                .as_deref()
                .ok_or(ProjectionError::MissingAccessibility)?;
            if authored_accessibility.role != "button"
                || authored_accessibility_label != compiled_label
            {
                return Err(ProjectionError::MismatchedAccessibility);
            }
            let mut accessibility_rows = artifact
                .tables
                .accessibility
                .rows
                .iter()
                .filter(|row| row.node.control_id == control.node.node_id);
            let accessibility_row = accessibility_rows
                .next()
                .ok_or(ProjectionError::MissingAccessibility)?;
            if accessibility_rows.next().is_some() {
                return Err(ProjectionError::DuplicateAccessibility);
            }
            if accessibility_row.node.role != AccessibilityRole::Button
                || accessibility_row.node.label.as_deref() != Some(authored_accessibility_label)
            {
                return Err(ProjectionError::MismatchedAccessibility);
            }

            let authored_route = route.as_ref().ok_or(ProjectionError::MissingRoute)?;
            let mut handlers = artifact
                .tables
                .interaction
                .rows
                .iter()
                .filter(|row| row.handler.control_id == control.node.node_id);
            let handler = handlers.next().ok_or(ProjectionError::MissingRoute)?;
            if handlers.next().is_some() {
                return Err(ProjectionError::DuplicateRoute);
            }
            if handler.handler.route.as_str() != authored_route.as_str()
                || handler.handler.trigger != InteractionTrigger::Press
            {
                return Err(ProjectionError::MismatchedRoute);
            }

            if control.node.required_capabilities.is_empty() {
                return Err(ProjectionError::MissingCapability);
            }
            if handler.handler.required_capabilities != control.node.required_capabilities {
                return Err(ProjectionError::MismatchedCapability);
            }
            for capability in &control.node.required_capabilities {
                let record = artifact
                    .manifest
                    .capabilities
                    .iter()
                    .find(|record| record.capability_id == capability.as_str())
                    .ok_or(ProjectionError::MissingCapability)?;
                if !record
                    .declared_by_controls
                    .iter()
                    .any(|owner| owner == control.node.node_id.as_str())
                    || !record
                        .required_by_interactions
                        .iter()
                        .any(|owner| owner == handler.handler.handler_id.as_str())
                {
                    return Err(ProjectionError::MismatchedCapability);
                }
            }

            let mut control_kinds = registry
                .control_kinds
                .iter()
                .filter(|candidate| candidate.control_kind_id.as_str() == kind.as_str());
            let control_kind = control_kinds
                .next()
                .ok_or(ProjectionError::MissingRouteContract)?;
            if control_kinds.next().is_some() {
                return Err(ProjectionError::DuplicateRouteContract);
            }

            let mut route_contracts = control_kind.route_requirements.iter();
            let route_contract = route_contracts
                .next()
                .ok_or(ProjectionError::MissingRouteContract)?;
            if route_contracts.next().is_some() {
                return Err(ProjectionError::DuplicateRouteContract);
            }
            if route_contract.capabilities != handler.handler.required_capabilities {
                return Err(ProjectionError::MismatchedRouteContract);
            }

            let mut descriptors = source
                .action_descriptors()
                .iter()
                .filter(|descriptor| descriptor.route() == &handler.handler.route);
            let descriptor = descriptors
                .next()
                .ok_or(ProjectionError::MissingActionDescriptor)?;
            if descriptors.next().is_some() {
                return Err(ProjectionError::DuplicateActionDescriptor);
            }
            if descriptor.payload_schema() != &handler.handler.payload_schema
                || descriptor.payload_schema() != &control_kind.event_payload_schema
                || descriptor.schema_version() != route_contract.schema_version
            {
                return Err(ProjectionError::MismatchedActionDescriptor);
            }

            let mut payload_schemas = registry.schemas.iter().filter(|candidate| {
                candidate.role == ControlSchemaRole::EventPayload
                    && candidate.schema_ref() == descriptor.payload_schema()
            });
            let payload_schema = payload_schemas
                .next()
                .ok_or(ProjectionError::MissingPayloadSchema)?;
            if payload_schemas.next().is_some() {
                return Err(ProjectionError::DuplicatePayloadSchema);
            }

            let mut packet = UiEventPacket::new(
                handler.handler.route.clone(),
                descriptor.schema_version(),
                descriptor.payload_schema().clone(),
                UiSchemaValue::object([
                    (
                        "route",
                        UiSchemaValue::route_ref(handler.handler.route.as_str()),
                    ),
                    ("activated", UiSchemaValue::bool(true)),
                ]),
            )
            .with_source_control(UiEventSourceControlId::new(control.node.node_id.as_str()))
            .with_payload_validation(&payload_schema.schema);
            if !packet.payload.diagnostics.is_empty() {
                return Err(ProjectionError::InvalidPayload);
            }
            for capability in &handler.handler.required_capabilities {
                packet = packet.with_capability(capability.clone());
            }
            if !packet.requires_capability(descriptor.capability()) {
                packet = packet.with_capability(descriptor.capability().clone());
            }

            Ok(button(label)
                .id(id.as_str())
                .on_activate(move || RunenwerkProgramUiAction::ActivateRoute(packet.clone()))
                .into_element())
        }
        _ => Err(ProjectionError::UnsupportedNode),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ui_binding::HostDataSnapshot;
    use ui_controls::{ControlPackageRegistry, runenwerk_control_package};
    use ui_definition::{
        AuthoredBindingRef, AuthoredControlAccessibilityDefinition, AuthoredControlKindId,
        AuthoredControlValue, AuthoredId, AuthoredRouteId, UiValueBinding,
    };
    use ui_program::UiProgramSourceId;

    use crate::plugins::ui::UiTypedActionDescriptor;

    use super::*;

    #[test]
    fn mounted_slot_publishes_runenui_semantics_and_layout_after_state_ingress() {
        let mut slots = UiRuntimeSlotsResource::default();
        let slot_id = mount_counter(&mut slots);
        slots
            .update_state(slot_id, counter_context(true, 1))
            .expect(
                "host-backed selected state should enter RunenUI through public action ingress",
            );
        assert_eq!(
            slots.resolved_state_value(slot_id, "state.counter.output.selected"),
            Some(&UiSchemaValue::bool(true))
        );

        let facts = slots
            .publication_facts(slot_id)
            .expect("RunenUI publication should succeed");
        for authored_id in ["counter.root", "counter.title", "counter.output"] {
            assert!(
                facts.authored_ids().iter().any(|id| id == authored_id),
                "missing authored id {authored_id}: {:?}",
                facts.authored_ids()
            );
        }
        assert!(
            facts.semantic_names().iter().any(|name| name == "Counter"),
            "{:?}",
            facts.semantic_names()
        );
        assert!(
            facts
                .semantic_names()
                .iter()
                .any(|name| name == "Counter output"),
            "{:?}",
            facts.semantic_names()
        );
        assert!(
            facts.shaped_text_run_count() > 0,
            "controlled publication should contain shaped text"
        );
        assert!(facts.all_bounds_non_zero());
    }

    #[test]
    fn exact_renderer_receipt_promotes_only_matching_pending_runenui_input() {
        use crate::plugins::render::host::RenderSurfaceId;
        use crate::plugins::render::{RenderFrameProducerId, SurfaceFrameSubmissionOrder};
        use crate::plugins::ui::presentation::{
            UiRuntimeNativeMapping, UiRuntimePresentationAssociationsResource,
            UiRuntimePresentationBinding, UiRuntimePresentedReceipt,
        };
        use crate::runtime::NativeWindowId;

        let mut slots = UiRuntimeSlotsResource::default();
        let (source, _, _, registry) = counter_projection_fixture();
        let fonts = UiRuntimeFontConfiguration::new(vec![CONTROLLED_FONT_BYTES.to_vec()])
            .with_generic_mapping(
                GenericFontFamily::SansSerif,
                vec![FontFamilyName::new("JetBrains Mono").expect("valid test family")],
            );
        let slot = slots
            .mount_with_fonts(source, &registry, &fonts)
            .slot_id()
            .expect("mounted Counter");
        let render_surface = RenderSurfaceId::primary();
        let producer = RenderFrameProducerId::try_from_raw(42).expect("valid producer");
        let style = StyleEnvironment::default();
        let binding =
            UiRuntimePresentationBinding::new(slot, producer, render_surface, style.clone())
                .with_order(SurfaceFrameSubmissionOrder::new(20, 0));
        let mapping =
            UiRuntimeNativeMapping::new(NativeWindowId::primary(), render_surface, (320, 160), 1.0)
                .expect("positive native extent and scale");
        let build = SurfaceBuildContext::tight(
            &style,
            LogicalSize::new(
                LogicalLength::from(RUNTIME_SURFACE_WIDTH),
                LogicalLength::from(RUNTIME_SURFACE_HEIGHT),
            ),
        );
        let first = slots
            .publish_surface_for_slot(slot, &build)
            .expect("published source");
        let mut associations = UiRuntimePresentationAssociationsResource::default();
        associations
            .bind(binding.clone())
            .expect("unique producer binding");
        assert!(associations.update_mapping(slot, mapping));
        assert!(!associations.update_mapping(slot, mapping));
        let publication_id = associations
            .stage(binding.clone(), mapping, 1, first.clone())
            .expect("pending paint");

        assert!(associations.associate_accepted_submission(publication_id));
        assert_eq!(associations.in_flight_count(), 1);

        let receipt = UiRuntimePresentedReceipt {
            slot_id: slot,
            producer_id: producer,
            render_surface_id: render_surface,
            surface_id: first.paint_publication().surface_id().clone(),
            paint_revision: first.paint_publication().revision(),
            publication_id,
        };
        let wrong_producer = UiRuntimePresentedReceipt {
            producer_id: RenderFrameProducerId::try_from_raw(43).expect("valid producer"),
            ..receipt.clone()
        };
        assert!(associations.promote(&wrong_producer).is_none());
        assert_eq!(associations.in_flight_count(), 1);

        // The old accepted frame must remain live while a newer equal-paint
        // input generation is published, and only the actual completed frame
        // can become displayed. Its earlier completion is not falsely stale.
        let second_id = associations
            .stage(binding.clone(), mapping, 1, first.clone())
            .expect("new independent presentation of equal painted content");
        assert_ne!(publication_id, second_id);
        assert_eq!(associations.in_flight_count(), 1);
        let first_displayed = associations
            .promote(&receipt)
            .expect("first completed frame retains exact earlier input snapshot");
        assert_eq!(first_displayed.publication_id, publication_id);
        assert_eq!(
            associations.pending(slot).map(|p| p.publication_id()),
            Some(second_id)
        );
        assert!(associations.associate_accepted_submission(second_id));

        let current = UiRuntimePresentedReceipt {
            publication_id: second_id,
            ..receipt
        };
        let displayed = associations
            .promote(&current)
            .expect("second exact GPU completion");
        assert_eq!(&displayed.input_context, first.input_context());
        assert_eq!(displayed.publication_id, second_id);
        assert!(associations.pending(slot).is_none());
        assert!(associations.displayed_for_mapping(slot, mapping).is_some());

        let retry_id = associations
            .stage(binding.clone(), mapping, 1, first.clone())
            .expect("new pending GPU submission");
        assert!(!associations.reject_terminal_publication(second_id));
        assert!(associations.associate_accepted_submission(retry_id));
        assert_eq!(associations.in_flight_count(), 1);
        assert!(associations.reject_terminal_publication(retry_id));
        assert_eq!(associations.in_flight_count(), 0);
        assert!(associations.pending(slot).is_none());
        assert!(
            associations.needs_publication(slot, mapping, 1),
            "failed submission must permit a fresh source publication",
        );
        assert!(
            associations.displayed_for_mapping(slot, mapping).is_some(),
            "failed new frame must not erase older displayed input",
        );

        let accepted_id = associations
            .stage(binding.clone(), mapping, 1, first.clone())
            .expect("retry source publication");
        assert!(associations.associate_accepted_submission(accepted_id));
        let accepted = associations
            .promote_completed_publication(accepted_id)
            .expect("only the renderer-authorized terminal publication is promoted");
        assert_eq!(accepted.publication_id, accepted_id);
        assert!(
            associations
                .promote_completed_publication(retry_id)
                .is_none()
        );

        // Static UI paint may be submitted more than once before any GPU
        // attempt completes. One rejected attempt must leave the identical
        // source generation retained until another accepted attempt completes.
        let repeated_id = associations
            .stage(binding.clone(), mapping, 1, first.clone())
            .expect("one unchanged source generation");
        assert!(associations.associate_accepted_submission(repeated_id));
        assert!(associations.associate_accepted_submission(repeated_id));
        assert_eq!(associations.in_flight_count(), 1);
        assert!(associations.reject_terminal_publication(repeated_id));
        assert_eq!(
            associations.in_flight_count(),
            1,
            "first failed GPU attempt cannot retire a second accepted attempt"
        );
        let recovered = associations
            .promote_completed_publication(repeated_id)
            .expect("second accepted attempt can still display the source generation");
        assert_eq!(recovered.publication_id, repeated_id);
        assert_eq!(associations.in_flight_count(), 0);
        assert!(!associations.reject_terminal_publication(repeated_id));

        // Native GPUs may complete accepted submissions out of source
        // publication order. Newest *successfully displayed* input wins,
        // and a late older completion must not rewind its hit-test context.
        let older_id = associations
            .stage(binding.clone(), mapping, 2, first.clone())
            .expect("older pending in-flight source");
        assert!(associations.associate_accepted_submission(older_id));
        let newer_id = associations
            .stage(binding.clone(), mapping, 3, first.clone())
            .expect("newer pending in-flight source");
        assert!(associations.associate_accepted_submission(newer_id));
        assert_eq!(associations.in_flight_count(), 2);
        let newer_displayed = associations
            .promote_completed_publication(newer_id)
            .expect("newer GPU frame is displayed first");
        assert_eq!(newer_displayed.publication_id, newer_id);
        assert!(
            associations
                .promote_completed_publication(older_id)
                .is_none(),
            "older delayed completion cannot regress displayed input"
        );
        assert_eq!(associations.in_flight_count(), 0);
        assert_eq!(
            associations
                .displayed_for_mapping(slot, mapping)
                .map(|displayed| displayed.publication_id),
            Some(newer_id)
        );
        assert!(!associations.reject_terminal_publication(older_id));

        let different_mapping =
            UiRuntimeNativeMapping::new(NativeWindowId::primary(), render_surface, (640, 320), 2.0)
                .expect("valid resize mapping");
        assert!(associations.update_mapping(slot, different_mapping));
        assert!(
            associations
                .displayed_for_mapping(slot, different_mapping)
                .is_none()
        );
        assert!(associations.displayed_for_mapping(slot, mapping).is_none());
        assert!(associations.update_mapping(slot, mapping));
        assert!(
            associations.displayed_for_mapping(slot, mapping).is_none(),
            "returning to an identical native mapping cannot resurrect an old Present"
        );
        let fresh_id = associations
            .stage(binding.clone(), mapping, 4, first.clone())
            .expect("new generation required after native remap");
        assert!(associations.associate_accepted_submission(fresh_id));
        assert!(associations.promote_completed_publication(fresh_id).is_some());
        assert!(associations.displayed_for_mapping(slot, mapping).is_some());
        associations.invalidate_mapping(slot);
        assert!(associations.displayed_for_mapping(slot, mapping).is_none());
        assert!(associations.update_mapping(slot, mapping));
        assert!(
            associations.displayed_for_mapping(slot, mapping).is_none(),
            "detached surface cannot regain stale displayed input on reattach"
        );
        associations.unbind(slot);
        assert!(associations.binding(slot).is_none());
    }

    #[test]
    fn host_presentation_mapping_rejects_invalid_scale_and_extent() {
        use crate::plugins::render::host::RenderSurfaceId;
        use crate::plugins::ui::{UiRuntimeMappingError, UiRuntimeNativeMapping};
        use crate::runtime::NativeWindowId;

        let window = NativeWindowId::primary();
        let surface = RenderSurfaceId::primary();
        assert_eq!(
            UiRuntimeNativeMapping::new(window, surface, (0, 160), 1.0),
            Err(UiRuntimeMappingError::EmptySurface),
        );
        for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                UiRuntimeNativeMapping::new(window, surface, (320, 160), scale),
                Err(UiRuntimeMappingError::InvalidScale),
            );
        }
        let valid = UiRuntimeNativeMapping::new(window, surface, (320, 160), 1.5)
            .expect("positive finite native raster scale");
        assert_eq!(valid.native_window_id(), window);
        assert_eq!(valid.render_surface_id(), surface);
        assert_eq!(valid.target_size_px(), (320, 160));
        assert_eq!(valid.native_scale_factor(), 1.5);
    }

    #[test]
    fn unbind_and_rebind_retire_only_owned_paint_targets() {
        use crate::plugins::render::RenderFrameProducerId;
        use crate::plugins::render::host::RenderSurfaceId;
        use crate::plugins::ui::{
            UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding,
        };

        let first = UiRuntimeSlotId(17);
        let second = UiRuntimeSlotId(18);
        let producer_a = RenderFrameProducerId::try_from_raw(71).expect("valid producer");
        let producer_b = RenderFrameProducerId::try_from_raw(72).expect("valid producer");
        let surface = RenderSurfaceId::primary();
        let mut associations = UiRuntimePresentationAssociationsResource::default();
        let initial = UiRuntimePresentationBinding::new(
            first,
            producer_a,
            surface,
            StyleEnvironment::default(),
        );
        associations.bind(initial.clone()).expect("first binding");
        assert!(associations.take_withdrawals().is_empty());

        let collision = UiRuntimePresentationBinding::new(
            second,
            producer_a,
            surface,
            StyleEnvironment::default(),
        );
        assert!(associations.bind(collision).is_err());
        assert!(associations.take_withdrawals().is_empty());

        let replacement = UiRuntimePresentationBinding::new(
            first,
            producer_b,
            surface,
            StyleEnvironment::default(),
        );
        associations.bind(replacement).expect("replacement binding");
        assert_eq!(associations.take_withdrawals(), vec![(producer_a, surface)]);
        assert!(associations.take_withdrawals().is_empty());

        associations.unbind(first);
        assert_eq!(associations.take_withdrawals(), vec![(producer_b, surface)]);
        associations.unbind(first);
        assert!(associations.take_withdrawals().is_empty());
    }

    #[test]
    fn invalid_caller_font_is_rejected_before_slot_mount() {
        let mut slots = UiRuntimeSlotsResource::default();
        let (source, _, _, registry) = counter_projection_fixture();
        let invalid_fonts = UiRuntimeFontConfiguration::new(vec![vec![0, 1, 2, 3]]);
        let report = slots.mount_with_fonts(source, &registry, &invalid_fonts);
        assert_eq!(
            report.failure(),
            Some(UiRuntimeSlotMountFailure::FontProvisioning)
        );
        assert!(slots.is_empty());
    }

    #[test]
    fn mounted_slot_publication_exposes_production_shaped_text_resource() {
        let mut slots = UiRuntimeSlotsResource::default();
        let (source, _, _, registry) = counter_projection_fixture();
        let fonts = UiRuntimeFontConfiguration::new(vec![CONTROLLED_FONT_BYTES.to_vec()])
            .with_generic_mapping(
                GenericFontFamily::SansSerif,
                vec![FontFamilyName::new("JetBrains Mono").expect("valid test font family")],
            );
        let slot_id = slots
            .mount_with_fonts(source, &registry, &fonts)
            .slot_id()
            .expect("caller-provided test fonts allow the Counter slot to mount");
        let environment = StyleEnvironment::default();
        let context = SurfaceBuildContext::tight(
            &environment,
            LogicalSize::new(
                LogicalLength::from(RUNTIME_SURFACE_WIDTH),
                LogicalLength::from(RUNTIME_SURFACE_HEIGHT),
            ),
        );
        assert!(
            slots
                .has_pending_redraw(slot_id)
                .expect("mounted RunenUI slot has redraw authority"),
            "newly mounted RunenUI surface should request its first paint",
        );
        let publication = slots
            .publish_surface_for_slot(slot_id, &context)
            .expect("production slot font provisioning permits publication");
        assert!(
            !slots
                .has_pending_redraw(slot_id)
                .expect("redraw authority remains live after publication"),
            "successful RunenUI publication must acknowledge its dirty revision",
        );
        let paint = publication.paint_scene();
        let runs = paint
            .items()
            .iter()
            .filter_map(|item| item.primitive().as_shaped_text_run())
            .collect::<Vec<_>>();
        assert!(
            !runs.is_empty(),
            "Counter publication must contain shaped text"
        );
        assert!(
            runs.iter()
                .all(|run| paint.shaped_text_resource(run.resource_ref()).is_some())
        );

        let mut resource_ids =
            crate::plugins::render::features::RunenUi2dResourceIdentityMap::default();
        let projected = crate::plugins::render::features::project_runenui_paint_to_2d(
            publication.paint_publication(),
            &mut resource_ids,
        )
        .expect("Counter paint subset must be losslessly supported by RunenRender 2D");
        let repeated = crate::plugins::render::features::project_runenui_paint_to_2d(
            publication.paint_publication(),
            &mut resource_ids,
        )
        .expect("repeat publication must preserve semantic resource identities");
        assert_eq!(projected.composition(), repeated.composition());
        assert_eq!(projected.bindings(), repeated.bindings());
        assert_eq!(
            projected.composition().root_entries().len(),
            paint.root_entries().len()
        );
        for (source_entry, projected_entry) in paint
            .root_entries()
            .iter()
            .zip(projected.composition().root_entries())
        {
            let index = source_entry
                .item_index()
                .expect("Counter roots are direct items");
            let source_item = &paint.items()[index];
            let source_run = source_item
                .primitive()
                .as_shaped_text_run()
                .expect("Counter paints only shaped runs");
            let source = paint
                .shaped_text_resource(source_run.resource_ref())
                .expect("RunenUI retains shaped resource");
            let runen_render::composition_2d::Render2dEntry::Item(item) = projected_entry else {
                panic!("Counter must project to direct RunenRender items");
            };
            let runen_render::composition_2d::Render2dPrimitive::ShapedText(text) =
                item.primitive()
            else {
                panic!("Counter must project to exact shaped text");
            };
            let Some(runen_render::composition_2d::Render2dResourceValue::ShapedText(target)) =
                projected.bindings().get(text.resource_id())
            else {
                panic!("projected shaped text must have a binding");
            };
            assert_eq!(target.font().bytes(), source.font().bytes());
            assert_eq!(target.font().face_index(), source.font().face_index());
            assert_eq!(
                target.font().normalized_coords(),
                source.font().normalized_coords()
            );
            assert_eq!(target.font().faux_bold(), source.font().faux_bold());
            assert_eq!(
                target.font().faux_skew(),
                source.font().faux_skew().map(f64::from)
            );
            assert_eq!(target.font_size(), f64::from(source.font_size()));
            assert_eq!(target.glyphs().len(), source.glyphs().len());
            for (actual, original) in target.glyphs().iter().zip(source.glyphs()) {
                assert_eq!(actual.id(), original.id());
                assert_eq!(actual.x(), f64::from(original.x()));
                assert_eq!(actual.y(), f64::from(original.y()));
                assert_eq!(actual.advance(), f64::from(original.advance()));
            }
        }
    }

    #[test]
    fn mounted_slot_rejects_non_bool_selected_state_before_runenui_update() {
        let mut slots = UiRuntimeSlotsResource::default();
        let slot_id = mount_counter(&mut slots);
        let invalid = UiEvaluationContext::default().with_host_data(HostDataSnapshot::new(
            "counter.output.selected",
            UiSchemaValue::string("not-a-bool"),
            1,
        ));

        assert_eq!(
            slots.update_state(slot_id, invalid),
            Err(UiRuntimeSlotOperationFailure::UnsupportedProjection)
        );
        assert_eq!(
            slots.resolved_state_value(slot_id, "state.counter.output.selected"),
            None
        );
    }

    #[test]
    fn mounted_slot_publication_is_deterministic_for_equal_inputs() {
        let first = counter_publication(true, 2);
        let second = counter_publication(true, 2);

        assert_eq!(first, second);
    }

    #[test]
    fn mounted_slot_repeated_equal_state_update_keeps_publication_stable() {
        let mut slots = UiRuntimeSlotsResource::default();
        let slot_id = mount_counter(&mut slots);

        slots
            .update_state(slot_id, counter_context(true, 1))
            .expect("first resolved state update should reach RunenUI");
        let first = slots
            .publication_facts(slot_id)
            .expect("first controlled publication should succeed");

        slots
            .update_state(slot_id, counter_context(true, 2))
            .expect("repeated equal state update should reach RunenUI");
        let second = {
            let slot = slots
                .slot_mut(slot_id)
                .expect("mounted slot should remain present for repeated publication");
            publication_facts_for_runtime(slot.runtime.counter_mut().unwrap())
                .expect("repeated publication should succeed without changing font policy")
        };

        assert_eq!(first, second);
    }

    #[test]
    fn mounted_slot_keeps_control_and_app_capabilities_distinct_on_egress() {
        let mut slots = UiRuntimeSlotsResource::default();
        let slot_id = mount_counter(&mut slots);
        slots
            .activate_authored(slot_id, "counter.output")
            .expect("RunenUI activation should be admitted");
        let requests = slots
            .pending_event_requests(slot_id)
            .expect("RunenUI host request should be observable");
        assert_eq!(requests.len(), 1);
        assert!(
            requests[0]
                .packet()
                .requires_capability(&ui_program::RouteCapability::new(
                    "runenwerk.ui.controls.activate",
                ))
        );
        assert!(
            requests[0]
                .packet()
                .requires_capability(&ui_program::RouteCapability::new(
                    "counter.action.increment",
                ))
        );
    }

    #[test]
    fn projection_rejects_label_availability_outside_bounded_subset() {
        let (source, artifact, state, snapshot) = counter_projection_fixture();
        let node = UiNodeDefinition::Label {
            id: AuthoredId::new("counter.unsupported.available"),
            label: UiValueBinding::static_text("Unsupported"),
            availability: Some(ui_definition::UiAvailabilityBinding::Static(
                ui_definition::UiAvailability::Disabled {
                    reason: "bounded U3 projection does not own availability".to_owned(),
                },
            )),
        };

        assert!(matches!(
            project_node(&node, &source, &artifact, &state, &snapshot),
            Err(ProjectionError::UnsupportedNode)
        ));
    }

    #[test]
    fn projection_rejects_extra_button_properties() {
        let (source, artifact, state, snapshot) = counter_projection_fixture();
        let mut node = counter_output_control();
        let UiNodeDefinition::Control { properties, .. } = &mut node else {
            unreachable!("counter output fixture is a Control");
        };
        properties.insert("unsupported".to_owned(), AuthoredControlValue::Bool(true));

        assert!(matches!(
            project_node(&node, &source, &artifact, &state, &snapshot),
            Err(ProjectionError::UnsupportedValue)
        ));
    }

    #[test]
    fn projection_rejects_missing_authored_button_semantic_name() {
        let (source, artifact, state, snapshot) = counter_projection_fixture();
        let mut node = counter_output_control();
        let UiNodeDefinition::Control { accessibility, .. } = &mut node else {
            unreachable!("counter output fixture is a Control");
        };
        accessibility
            .as_mut()
            .expect("counter output fixture has accessibility")
            .label = None;

        assert!(matches!(
            project_node(&node, &source, &artifact, &state, &snapshot),
            Err(ProjectionError::MissingAccessibility)
        ));
    }

    #[test]
    fn projection_rejects_payload_schema_disagreement_with_canonical_control_kind() {
        let (source, artifact, state, mut snapshot) = counter_projection_fixture();
        let button_kind = snapshot
            .control_kinds
            .iter_mut()
            .find(|candidate| candidate.control_kind_id.as_str() == BUTTON_CONTROL_KIND_ID)
            .expect("counter fixture should contain the canonical Button descriptor");
        button_kind.event_payload_schema =
            ui_schema::UiSchemaRef::new("runenwerk.ui.controls.button.other_event", 1);
        let node = counter_output_control();

        assert!(matches!(
            project_node(&node, &source, &artifact, &state, &snapshot),
            Err(ProjectionError::MismatchedActionDescriptor)
        ));
    }

    fn counter_publication(selected: bool, revision: u64) -> UiRuntimeSlotPublicationFacts {
        let mut slots = UiRuntimeSlotsResource::default();
        let slot_id = mount_counter(&mut slots);
        slots
            .update_state(slot_id, counter_context(selected, revision))
            .expect("resolved state should update RunenUI");
        slots
            .publication_facts(slot_id)
            .expect("RunenUI publication should succeed")
    }

    #[test]
    fn independent_typed_apps_share_one_slot_namespace_and_counter_publication() {
        use runenui_core::NoHostProtocol;

        struct IncrementApp;
        impl UiApp for IncrementApp {
            type State = i32;
            type Action = i32;
            type HostProtocol = NoHostProtocol;

            fn root(state: &Self::State) -> impl View<Self::Action> {
                text(format!("Value {state}"))
            }

            fn update(
                state: &mut Self::State,
                action: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                *state += action;
                Effects::none()
            }
        }

        #[derive(Clone, Debug)]
        enum ApprovalCommand {
            Check,
        }
        #[derive(Clone, Debug)]
        enum ApprovalResponse {
            Accepted,
        }
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum ApprovalKind {
            Decision,
        }
        struct ApprovalProtocol;
        impl HostProtocol for ApprovalProtocol {
            type Command = ApprovalCommand;
            type Response = ApprovalResponse;
            type ResponseKind = ApprovalKind;
            fn expected_response(_: &Self::Command) -> Self::ResponseKind {
                ApprovalKind::Decision
            }
            fn response_kind(_: &Self::Response) -> Self::ResponseKind {
                ApprovalKind::Decision
            }
        }

        struct ApprovalApp;
        enum ApprovalAction {
            Toggle,
            Completed,
        }
        impl UiApp for ApprovalApp {
            type State = bool;
            type Action = ApprovalAction;
            type HostProtocol = ApprovalProtocol;
            fn root(state: &Self::State) -> impl View<Self::Action> {
                text(if *state { "approved" } else { "pending" })
            }
            fn update(
                state: &mut Self::State,
                action: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                match action {
                    ApprovalAction::Toggle => {
                        *state = !*state;
                        Effects::host_request(None, ApprovalCommand::Check, |_| {
                            ApprovalAction::Completed
                        })
                    }
                    ApprovalAction::Completed => Effects::none(),
                }
            }
        }

        let fonts = UiRuntimeFontConfiguration::new(vec![CONTROLLED_FONT_BYTES.to_vec()])
            .with_generic_mapping(
                GenericFontFamily::SansSerif,
                vec![FontFamilyName::new("JetBrains Mono").unwrap()],
            );
        let mut slots = UiRuntimeSlotsResource::default();
        let counter = mount_counter(&mut slots);
        let increment = slots
            .mount_app::<IncrementApp>(UiTypedScreenId::new("ordinary.increment"), 1, &fonts)
            .slot_id()
            .expect("first ordinary app mounts");
        let approval = slots
            .mount_app::<ApprovalApp>(UiTypedScreenId::new("ordinary.approval"), false, &fonts)
            .slot_id()
            .expect("second distinct ordinary app mounts");
        assert_eq!(slots.len(), 3);
        assert_ne!(counter, increment);
        assert_ne!(increment, approval);
        assert_eq!(
            slots.submit_app_action::<IncrementApp>(approval, 9),
            Err(UiRuntimeSlotOperationFailure::WrongAppKind),
        );
        assert_eq!(
            slots.submit_app_action::<IncrementApp>(counter, 9),
            Err(UiRuntimeSlotOperationFailure::WrongAppKind),
        );
        let before = slots.ingress_epoch(increment).unwrap();
        slots
            .submit_app_action::<IncrementApp>(increment, 4)
            .unwrap();
        assert_eq!(slots.ingress_epoch(increment), Some(before + 1));
        assert_eq!(
            slots.observe_app::<IncrementApp, _>(increment, |runtime| *runtime.state()),
            Ok(5),
        );

        slots
            .submit_app_action::<ApprovalApp>(approval, ApprovalAction::Toggle)
            .unwrap();
        assert_eq!(
            slots.observe_app::<ApprovalApp, _>(approval, |runtime| *runtime.state()),
            Ok(true)
        );
        let token = slots
            .observe_app::<ApprovalApp, _>(approval, |runtime| {
                let pending = runtime.pending_host_requests();
                assert_eq!(pending.len(), 1);
                assert!(matches!(pending[0].command(), ApprovalCommand::Check));
                pending[0].token()
            })
            .unwrap();
        slots
            .complete_app_host_request::<ApprovalApp>(approval, &token, ApprovalResponse::Accepted)
            .unwrap();
        assert!(
            slots
                .observe_app::<ApprovalApp, _>(approval, |runtime| runtime
                    .pending_host_requests()
                    .is_empty())
                .unwrap()
        );

        let style = StyleEnvironment::default();
        let surface = SurfaceBuildContext::tight(
            &style,
            LogicalSize::new(
                LogicalLength::from(RUNTIME_SURFACE_WIDTH),
                LogicalLength::from(RUNTIME_SURFACE_HEIGHT),
            ),
        );
        for slot in [increment, approval] {
            let publication = slots.publish_surface_for_slot(slot, &surface).unwrap();
            assert!(!publication.paint_scene().items().is_empty());
        }
        let retirement = slots.unmount_with_retirement(increment).unwrap();
        assert!(retirement.is_some());
        assert!(!slots.contains(increment));
        assert!(slots.contains(approval));
        assert!(slots.contains(counter));
    }

    #[test]
    fn typed_action_partial_pump_advances_epoch_and_defers_publication_until_resettled() {
        use runenui_core::NoHostProtocol;

        struct ChainedApp;
        impl UiApp for ChainedApp {
            type State = usize;
            type Action = ();
            type HostProtocol = NoHostProtocol;

            fn root(state: &Self::State) -> impl View<Self::Action> {
                text(format!("Processed {state}"))
            }

            fn update(
                state: &mut Self::State,
                (): Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                *state += 1;
                if *state < 320 {
                    Effects::action(())
                } else {
                    Effects::none()
                }
            }
        }

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<ChainedApp>(
                UiTypedScreenId::new("ordinary.chained"),
                0,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("typed chained app mounts");
        assert_eq!(
            slots.submit_app_action::<ChainedApp>(slot, ()),
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        );
        let processed = slots
            .observe_app::<ChainedApp, _>(slot, |runtime| *runtime.state())
            .unwrap();
        assert!(processed > 0 && processed < 320);
        assert_eq!(slots.ingress_epoch(slot), Some(2));
        assert_eq!(
            slots.submit_app_action::<ChainedApp>(slot, ()),
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        );
        assert_eq!(
            slots.has_pending_redraw(slot),
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        );
        let style = StyleEnvironment::default();
        let context = SurfaceBuildContext::tight(
            &style,
            LogicalSize::new(LogicalLength::from(320_u16), LogicalLength::from(160_u16)),
        );
        assert!(matches!(
            slots.publish_surface_for_slot(slot, &context),
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        ));
        for _ in 0..8 {
            if slots.settle_slot(slot).is_ok() {
                break;
            }
        }
        assert_eq!(
            slots.observe_app::<ChainedApp, _>(slot, |runtime| *runtime.state()),
            Ok(320)
        );
        assert!(slots.has_pending_redraw(slot).is_ok());
        assert_eq!(slots.ingress_epoch(slot), Some(2));
    }

    #[test]
    fn typed_host_response_partial_pump_cannot_reuse_prior_ingress_epoch() {
        struct ResponseProtocol;
        impl HostProtocol for ResponseProtocol {
            type Command = ();
            type Response = ();
            type ResponseKind = ();

            fn expected_response(_: &Self::Command) -> Self::ResponseKind {}

            fn response_kind(_: &Self::Response) -> Self::ResponseKind {}
        }

        enum ResponseAction {
            Request,
            Continue,
        }

        struct ResponseApp;
        impl UiApp for ResponseApp {
            type State = usize;
            type Action = ResponseAction;
            type HostProtocol = ResponseProtocol;

            fn root(state: &Self::State) -> impl View<Self::Action> {
                text(format!("Responded {state}"))
            }

            fn update(
                state: &mut Self::State,
                action: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                match action {
                    ResponseAction::Request => {
                        Effects::host_request(None, (), |()| ResponseAction::Continue)
                    }
                    ResponseAction::Continue => {
                        *state += 1;
                        if *state < 320 {
                            Effects::action(ResponseAction::Continue)
                        } else {
                            Effects::none()
                        }
                    }
                }
            }
        }

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<ResponseApp>(
                UiTypedScreenId::new("ordinary.response-loop"),
                0,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("typed host-response app mounts");
        slots
            .submit_app_action::<ResponseApp>(slot, ResponseAction::Request)
            .expect("host request queues");
        let token = slots
            .observe_app::<ResponseApp, _>(slot, |runtime| {
                let requests = runtime.pending_host_requests();
                assert_eq!(requests.len(), 1);
                requests[0].token()
            })
            .unwrap();
        let preceding_epoch = slots.ingress_epoch(slot).unwrap();
        assert_eq!(
            slots.complete_app_host_request::<ResponseApp>(slot, &token, ()),
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        );
        assert_eq!(slots.ingress_epoch(slot), Some(preceding_epoch + 1));
        let progress = slots
            .observe_app::<ResponseApp, _>(slot, |runtime| *runtime.state())
            .unwrap();
        assert!(progress > 0 && progress < 320);
        assert_eq!(
            slots.has_pending_redraw(slot),
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        );
        for _ in 0..8 {
            if slots.settle_slot(slot).is_ok() {
                break;
            }
        }
        assert_eq!(
            slots.observe_app::<ResponseApp, _>(slot, |runtime| *runtime.state()),
            Ok(320)
        );
        assert!(slots.has_pending_redraw(slot).is_ok());
        assert_eq!(slots.ingress_epoch(slot), Some(preceding_epoch + 1));
    }

    #[test]
    fn rejected_typed_font_and_exhausted_identity_preserve_slot_inventory() {
        use runenui_core::NoHostProtocol;

        struct EmptyApp;
        impl UiApp for EmptyApp {
            type State = ();
            type Action = ();
            type HostProtocol = NoHostProtocol;

            fn root(_: &Self::State) -> impl View<Self::Action> {
                text("empty")
            }

            fn update(
                _: &mut Self::State,
                (): Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                Effects::none()
            }
        }

        let mut slots = UiRuntimeSlotsResource::default();
        let invalid_font = UiRuntimeFontConfiguration::new(vec![vec![1, 2, 3]]);
        let invalid = slots.mount_app::<EmptyApp>(
            UiTypedScreenId::new("ordinary.invalid-font"),
            (),
            &invalid_font,
        );
        assert_eq!(
            invalid.failure(),
            Some(UiRuntimeSlotMountFailure::FontProvisioning)
        );
        assert!(slots.is_empty());
        assert_eq!(slots.next_slot_id, 1);

        slots.next_slot_id = u64::MAX;
        let exhausted = slots.mount_app::<EmptyApp>(
            UiTypedScreenId::new("ordinary.exhausted"),
            (),
            &UiRuntimeFontConfiguration::default(),
        );
        assert_eq!(
            exhausted.failure(),
            Some(UiRuntimeSlotMountFailure::UnsupportedProjection)
        );
        assert!(slots.is_empty());
        assert_eq!(slots.next_slot_id, u64::MAX);
    }

    fn mount_counter(slots: &mut UiRuntimeSlotsResource) -> UiRuntimeSlotId {
        let (source, artifact, state, snapshot) = counter_projection_fixture();
        if let Err(error) = validate_projection(&source, &artifact, &state, &snapshot) {
            panic!("counter projection should validate before mount: {error:?}");
        }

        let report = slots.mount(source, &snapshot);
        report.slot_id().unwrap_or_else(|| {
            panic!(
                "counter source should mount after projection validation; rejection: {:?}",
                report.failure()
            )
        })
    }

    fn counter_projection_fixture() -> (
        UiTypedSource,
        UiRuntimeArtifact,
        UiStateModel,
        ControlPackageRegistrySnapshot,
    ) {
        let registry = ControlPackageRegistry::new()
            .with_package(runenwerk_control_package())
            .expect("runenwerk controls package should register");
        let snapshot = registry.snapshot();
        let source = counter_source();
        let lowering = source.lower_with_registry_snapshot(&snapshot);
        assert!(
            lowering.passed(),
            "counter lowering should pass: {:?}",
            lowering.formation().diagnostics
        );
        let artifact = UiRuntimeArtifact::from_program(lowering.program());
        (source, artifact, UiStateModel::default(), snapshot)
    }

    fn counter_context(selected: bool, revision: u64) -> UiEvaluationContext {
        UiEvaluationContext::default().with_host_data(HostDataSnapshot::new(
            "counter.output.selected",
            UiSchemaValue::bool(selected),
            revision,
        ))
    }

    fn counter_source() -> UiTypedSource {
        UiTypedSource::new(
            UiTypedScreenId::new("counter.screen"),
            UiProgramSourceId::new("counter.screen.source"),
            UiNodeDefinition::Column {
                id: AuthoredId::new("counter.root"),
                children: vec![
                    UiNodeDefinition::Label {
                        id: AuthoredId::new("counter.title"),
                        label: UiValueBinding::static_text("Counter"),
                        availability: None,
                    },
                    counter_output_control(),
                ],
            },
        )
        .with_action_descriptor(counter_action_descriptor())
    }

    fn counter_action_descriptor() -> UiTypedActionDescriptor {
        UiTypedActionDescriptor::new(
            crate::plugins::ui::UiTypedActionId::new("counter.increment.action"),
            ui_program::RouteId::new("counter.increment"),
            ui_program::RouteSchemaVersion::new(1),
            ui_schema::UiSchemaRef::new("runenwerk.ui.controls.button.event", 1),
            ui_program::RouteCapability::new("counter.action.increment"),
        )
    }

    fn counter_output_control() -> UiNodeDefinition {
        let mut properties = BTreeMap::new();
        properties.insert(
            "label".to_owned(),
            AuthoredControlValue::String("Counter output".to_owned()),
        );

        let mut bindings = BTreeMap::new();
        bindings.insert(
            "selected".to_owned(),
            AuthoredBindingRef::new("counter.output.selected"),
        );

        UiNodeDefinition::Control {
            id: AuthoredId::new("counter.output"),
            kind: AuthoredControlKindId::new(BUTTON_CONTROL_KIND_ID),
            properties,
            bindings,
            route: Some(AuthoredRouteId::new("counter.increment")),
            accessibility: Some(AuthoredControlAccessibilityDefinition {
                role: "button".to_owned(),
                label: Some("Counter output".to_owned()),
            }),
            children: Vec::new(),
        }
    }
}
