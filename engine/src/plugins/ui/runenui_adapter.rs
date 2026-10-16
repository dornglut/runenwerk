use runenui_core::{
    CommittedTextEvent, CompositionGeneration, CompositionRange, Effects, Element, ElementId,
    FontFamilyName, GenericFontFamily, HostProtocol, InputDeviceId, KeyboardEvent, KeyboardPhase,
    PointerEvent, PointerId, SemanticCommand, SurfaceId, UiApp, View, button, column, text,
};
#[cfg(test)]
use runenui_core::{LogicalLength, LogicalRect, PaintPrimitive, StyleEnvironment};
#[cfg(test)]
use runenui_runtime::LogicalSize;
use runenui_runtime::{
    AppRuntime, CompositionStartSubmission, CompositionSubmission, HostRequestToken,
    InputArbitrationRecord, InputArbitrationScope, InputObservationError, InputOwnershipSnapshot,
    InputPumpBatch, InputShutdownBatch, KeyboardSubmission, PointerSubmission, PumpBudget,
    SubmitCompositionError, SubmitCompositionStartError, SubmitKeyboardError,
    SubmitKeyboardErrorKind, SubmitPointerError, SubmitTextError, SurfaceBuildContext,
    SurfacePublication, TextSubmission, UiInputFamily, UiInputFinality,
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
use crate::runtime::NativeWakeTransport;
use crate::runtime::native_host_wake::NativeHostClock;
use runenui_runtime::MonotonicInstant;

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
    RuntimeInitialization,
    SlotIdentityExhausted,
    SlotCapacityExhausted,
    IncompatibleNativeClock,
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
            Self::RuntimeInitialization => {
                "RunenUI slot mount could not admit initial application work"
            }
            Self::SlotIdentityExhausted => "RunenUI slot mount identity space is exhausted",
            Self::SlotCapacityExhausted => "RunenUI slot mount storage could not be reserved",
            Self::IncompatibleNativeClock => {
                "RunenUI native clock must be configured before mounting; an existing time domain cannot be replaced"
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
    NativeHostClosed,
    RuntimeStopped(runenui_runtime::PumpOutcome),
}

/// An exact native keyboard observation is an immutable M13 fact, not a
/// synthesized Engine gameplay grant. Missing or ambiguous evidence fails closed.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UiRuntimeNativeKeyboardEvidenceFailure {
    Unsettled,
    WrongScope,
    NotProcessed,
    MissingReceipt,
    DuplicateReceipt,
    WrongFamily,
    WrongDevice,
    InvalidRevision,
}

/// Decode only the one framework-authored keyboard settlement identified by
/// the accepted submission and actual mounted runtime scope. A native receipt
/// MUST carry the checked device identity; device-less headless settlements
/// never satisfy an original hardware occurrence. Never infer a positive
/// result from a quiescent pump without the exact ordered receipt.
///
/// This function owns no queue, focus, or permission state. Returning an M13
/// committed nonexclusive conflict does not itself authorize a game action.
pub fn native_keyboard_finality(
    scope: &InputArbitrationScope,
    expected_device: InputDeviceId,
    submission: KeyboardSubmission,
    batch: &InputPumpBatch,
    status: &Result<(), UiRuntimeSlotOperationFailure>,
) -> Result<UiInputFinality, UiRuntimeNativeKeyboardEvidenceFailure> {
    use UiRuntimeNativeKeyboardEvidenceFailure as Failure;
    if status.is_err()
        || batch.pause_reason().is_some()
        || !batch.report().is_quiescent()
        || batch.final_ownership().status() != runenui_runtime::RuntimeStatus::Running
    {
        return Err(Failure::Unsettled);
    }
    if batch.final_ownership().scope() != scope {
        return Err(Failure::WrongScope);
    }
    if !batch
        .processed_through()
        .is_some_and(|through| through >= submission.sequence())
    {
        return Err(Failure::NotProcessed);
    }
    let mut match_for_sequence = None;
    for record in batch.ordered_records() {
        let InputArbitrationRecord::InputSettled(settlement) = record else {
            continue;
        };
        if settlement.sequence() != submission.sequence() {
            continue;
        }
        if match_for_sequence.is_some() {
            return Err(Failure::DuplicateReceipt);
        }
        match_for_sequence = Some(settlement);
    }
    let settlement = match_for_sequence.ok_or(Failure::MissingReceipt)?;
    if settlement.scope() != scope {
        return Err(Failure::WrongScope);
    }
    if settlement.family() != UiInputFamily::Keyboard {
        return Err(Failure::WrongFamily);
    }
    if settlement.device_id() != Some(expected_device) {
        return Err(Failure::WrongDevice);
    }
    if settlement.ownership_revision() > batch.final_ownership().revision() {
        return Err(Failure::InvalidRevision);
    }
    Ok(settlement.finality().clone())
}

/// Native text rejection keeps the exact original committed payload.
#[derive(Debug)]
pub enum UiRuntimeNativeTextAdmissionError {
    Slot {
        failure: UiRuntimeSlotOperationFailure,
        event: CommittedTextEvent,
    },
    Framework(SubmitTextError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRuntimeNativeTextEvidenceFailure {
    Unsettled,
    WrongScope,
    NotProcessed,
    MissingReceipt,
    DuplicateReceipt,
    WrongFamily,
    WrongDevice,
    InvalidRevision,
}

/// Correlates the one M13 source-owned text receipt by scope, FIFO sequence
/// and original device identity. This is not a UI-consumed flag.
pub(crate) fn native_text_finality(
    scope: &InputArbitrationScope,
    expected_device: Option<InputDeviceId>,
    submission: TextSubmission,
    batch: &InputPumpBatch,
    status: &Result<(), UiRuntimeSlotOperationFailure>,
) -> Result<UiInputFinality, UiRuntimeNativeTextEvidenceFailure> {
    use UiRuntimeNativeTextEvidenceFailure as Failure;
    if status.is_err()
        || batch.pause_reason().is_some()
        || !batch.report().is_quiescent()
        || batch.final_ownership().status() != runenui_runtime::RuntimeStatus::Running
    {
        return Err(Failure::Unsettled);
    }
    if batch.final_ownership().scope() != scope {
        return Err(Failure::WrongScope);
    }
    if !batch
        .processed_through()
        .is_some_and(|through| through >= submission.sequence())
    {
        return Err(Failure::NotProcessed);
    }
    let mut found = None;
    for record in batch.ordered_records() {
        let InputArbitrationRecord::InputSettled(settlement) = record else {
            continue;
        };
        if settlement.sequence() != submission.sequence() {
            continue;
        }
        if found.is_some() {
            return Err(Failure::DuplicateReceipt);
        }
        found = Some(settlement);
    }
    let settlement = found.ok_or(Failure::MissingReceipt)?;
    if settlement.scope() != scope {
        return Err(Failure::WrongScope);
    }
    if settlement.family() != UiInputFamily::CommittedText {
        return Err(Failure::WrongFamily);
    }
    if settlement.device_id() != expected_device {
        return Err(Failure::WrongDevice);
    }
    if settlement.ownership_revision() > batch.final_ownership().revision() {
        return Err(Failure::InvalidRevision);
    }
    Ok(settlement.finality().clone())
}

/// Structured pinned composition admission: slot authority, original
/// start request and framework composition event failures stay distinguishable.
#[derive(Debug)]
pub(crate) enum UiRuntimeNativeCompositionAdmissionError {
    Slot(UiRuntimeSlotOperationFailure),
    Start(SubmitCompositionStartError),
    Event(SubmitCompositionError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiRuntimeNativeCompositionEvidenceFailure {
    Unsettled,
    WrongScope,
    NotProcessed,
    MissingReceipt,
    DuplicateReceipt,
    WrongFamily,
    WrongDevice,
    InvalidRevision,
}

/// Only a reached, unique composition-family M13 settlement in the original
/// framework scope may qualify an OS preedit/end/cancel observation. No
/// ownership, focus, generation or text payload is mirrored by this checker.
pub(crate) fn native_composition_finality(
    scope: &InputArbitrationScope,
    sequence: runenui_runtime::WorkSequence,
    batch: &InputPumpBatch,
    status: &Result<(), UiRuntimeSlotOperationFailure>,
) -> Result<UiInputFinality, UiRuntimeNativeCompositionEvidenceFailure> {
    use UiRuntimeNativeCompositionEvidenceFailure as Failure;
    if status.is_err()
        || batch.pause_reason().is_some()
        || !batch.report().is_quiescent()
        || batch.final_ownership().status() != runenui_runtime::RuntimeStatus::Running
    {
        return Err(Failure::Unsettled);
    }
    if batch.final_ownership().scope() != scope {
        return Err(Failure::WrongScope);
    }
    if !batch.processed_through().is_some_and(|end| end >= sequence) {
        return Err(Failure::NotProcessed);
    }
    let mut found = None;
    for record in batch.ordered_records() {
        let InputArbitrationRecord::InputSettled(settlement) = record else {
            continue;
        };
        if settlement.sequence() != sequence {
            continue;
        }
        if found.is_some() {
            return Err(Failure::DuplicateReceipt);
        }
        found = Some(settlement);
    }
    let settlement = found.ok_or(Failure::MissingReceipt)?;
    if settlement.scope() != scope {
        return Err(Failure::WrongScope);
    }
    if settlement.family() != UiInputFamily::Composition {
        return Err(Failure::WrongFamily);
    }
    if settlement.device_id().is_some() {
        return Err(Failure::WrongDevice);
    }
    if settlement.ownership_revision() > batch.final_ownership().revision() {
        return Err(Failure::InvalidRevision);
    }
    Ok(settlement.finality().clone())
}

/// A native pointer rejection retains the exact unaccepted event.
#[derive(Debug)]
pub enum UiRuntimeNativePointerAdmissionError {
    Slot {
        failure: UiRuntimeSlotOperationFailure,
        event: Box<PointerEvent>,
    },
    Framework(SubmitPointerError),
}

/// Host-side verification of the one exact RunenUI M13 pointer settlement;
/// never a game-permission grant without original host/source/lifetime proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRuntimeNativePointerEvidenceFailure {
    Unsettled,
    WrongScope,
    NotProcessed,
    MissingReceipt,
    DuplicateReceipt,
    WrongFamily,
    WrongDevice,
    WrongPointer,
    WrongSurface,
    InvalidRevision,
}

/// Check original native stream identity against the exact framework receipt,
/// not a coalesced UI-consumed flag or a generic quiescent pump.
pub fn native_pointer_finality(
    scope: &InputArbitrationScope,
    expected_device: InputDeviceId,
    expected_pointer: PointerId,
    expected_surface: &SurfaceId,
    submission: PointerSubmission,
    batch: &InputPumpBatch,
    status: &Result<(), UiRuntimeSlotOperationFailure>,
) -> Result<UiInputFinality, UiRuntimeNativePointerEvidenceFailure> {
    use UiRuntimeNativePointerEvidenceFailure as Failure;
    if status.is_err()
        || batch.pause_reason().is_some()
        || !batch.report().is_quiescent()
        || batch.final_ownership().status() != runenui_runtime::RuntimeStatus::Running
    {
        return Err(Failure::Unsettled);
    }
    if batch.final_ownership().scope() != scope {
        return Err(Failure::WrongScope);
    }
    if !batch
        .processed_through()
        .is_some_and(|through| through >= submission.sequence())
    {
        return Err(Failure::NotProcessed);
    }
    let mut found = None;
    for record in batch.ordered_records() {
        let InputArbitrationRecord::InputSettled(settlement) = record else {
            continue;
        };
        if settlement.sequence() != submission.sequence() {
            continue;
        }
        if found.is_some() {
            return Err(Failure::DuplicateReceipt);
        }
        found = Some(settlement);
    }
    let settlement = found.ok_or(Failure::MissingReceipt)?;
    if settlement.scope() != scope {
        return Err(Failure::WrongScope);
    }
    if settlement.family() != UiInputFamily::Pointer {
        return Err(Failure::WrongFamily);
    }
    if settlement.device_id() != Some(expected_device) {
        return Err(Failure::WrongDevice);
    }
    if settlement.pointer_id() != Some(expected_pointer) {
        return Err(Failure::WrongPointer);
    }
    if settlement.surface_id() != Some(expected_surface) {
        return Err(Failure::WrongSurface);
    }
    if settlement.ownership_revision() > batch.final_ownership().revision() {
        return Err(Failure::InvalidRevision);
    }
    Ok(settlement.finality().clone())
}

/// A rejected host keyboard occurrence retains the exact unaccepted event.
#[derive(Debug)]
pub enum UiRuntimeNativeKeyboardAdmissionError {
    Slot {
        failure: UiRuntimeSlotOperationFailure,
        event: KeyboardEvent,
    },
    Framework(SubmitKeyboardError),
}

/// One canonical existing M13 no-focus observation predicate. The caller
/// selects the exact *original* keyboard phase and its separate game policy.
fn native_unfocused_keyboard_rejection_witness(
    expected_device: InputDeviceId,
    expected_phase: KeyboardPhase,
    before: &InputOwnershipSnapshot,
    after: &InputOwnershipSnapshot,
    error: &UiRuntimeNativeKeyboardAdmissionError,
) -> bool {
    let UiRuntimeNativeKeyboardAdmissionError::Framework(framework) = error else {
        return false;
    };
    if framework.kind() != SubmitKeyboardErrorKind::NoFocusedTarget
        || framework.event().phase() != expected_phase
        || framework.event().device_id() != Some(expected_device)
        || before != after
        || before.status() != runenui_runtime::RuntimeStatus::Running
    {
        return false;
    }
    let keyboard = before.keyboard();
    keyboard.focused_node().is_none()
        && keyboard.composition_generation().is_none()
        && keyboard.composition_device_id().is_none()
        && keyboard.space_activation_owner().is_none()
        && keyboard.space_activation_device_id().is_none()
        && before
            .surfaces()
            .iter()
            .all(|surface| surface.modal_blocker().is_none())
        && before
            .pointers()
            .iter()
            .all(|pointer| pointer.capture_owner().is_none() && pointer.pressed_owner().is_none())
}

/// The exact M13 pre-sequence `NoFocusedTarget` original-Down evidence.
/// Gameplay permission still requires source-specific ActionState eligibility.
pub fn native_no_focused_keyboard_claim_witness(
    expected_device: InputDeviceId,
    before: &InputOwnershipSnapshot,
    after: &InputOwnershipSnapshot,
    error: &UiRuntimeNativeKeyboardAdmissionError,
) -> bool {
    native_unfocused_keyboard_rejection_witness(
        expected_device,
        KeyboardPhase::Down,
        before,
        after,
        error,
    )
}

/// Only an authentic original release may retire neutral physical state
/// without M13 admission. This is NEVER positive gameplay press permission.
pub(crate) fn native_original_keyboard_release_nonclaim(
    expected_device: InputDeviceId,
    before: &InputOwnershipSnapshot,
    after: &InputOwnershipSnapshot,
    error: &UiRuntimeNativeKeyboardAdmissionError,
) -> bool {
    native_unfocused_keyboard_rejection_witness(
        expected_device,
        KeyboardPhase::Up,
        before,
        after,
        error,
    ) && matches!(
        error,
        UiRuntimeNativeKeyboardAdmissionError::Framework(framework)
            if !framework.event().is_repeat()
    )
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

    fn pump(&mut self, budget: PumpBudget) -> Result<InputPumpBatch, InputObservationError> {
        match self {
            Self::Counter(runtime) => runtime.pump(budget),
            Self::Typed(runtime) => runtime.pump(budget),
        }
    }

    fn input_ownership(&mut self) -> Result<InputOwnershipSnapshot, InputObservationError> {
        match self {
            Self::Counter(runtime) => runtime.input_ownership(),
            Self::Typed(runtime) => runtime.input_ownership(),
        }
    }

    fn submit_keyboard(
        &mut self,
        event: KeyboardEvent,
    ) -> Result<KeyboardSubmission, SubmitKeyboardError> {
        match self {
            Self::Counter(runtime) => runtime.submit_keyboard(event),
            Self::Typed(runtime) => runtime.submit_keyboard(event),
        }
    }

    fn submit_pointer(
        &mut self,
        event: PointerEvent,
    ) -> Result<PointerSubmission, SubmitPointerError> {
        match self {
            Self::Counter(runtime) => runtime.submit_pointer(event),
            Self::Typed(runtime) => runtime.submit_pointer(event),
        }
    }

    fn submit_text(
        &mut self,
        event: CommittedTextEvent,
    ) -> Result<TextSubmission, SubmitTextError> {
        match self {
            Self::Counter(runtime) => runtime.submit_text(event),
            Self::Typed(runtime) => runtime.submit_text(event),
        }
    }

    fn start_composition(
        &mut self,
        device: Option<InputDeviceId>,
    ) -> Result<CompositionStartSubmission, SubmitCompositionStartError> {
        match self {
            Self::Counter(runtime) => runtime.start_composition(device),
            Self::Typed(runtime) => runtime.start_composition(device),
        }
    }

    fn update_composition(
        &mut self,
        generation: CompositionGeneration,
        preedit: String,
        range: Option<CompositionRange>,
    ) -> Result<CompositionSubmission, SubmitCompositionError> {
        match self {
            Self::Counter(runtime) => runtime.submit_composition_update(generation, preedit, range),
            Self::Typed(runtime) => runtime.update_composition(generation, preedit, range),
        }
    }

    fn end_composition(
        &mut self,
        generation: CompositionGeneration,
    ) -> Result<CompositionSubmission, SubmitCompositionError> {
        match self {
            Self::Counter(runtime) => runtime.submit_composition_end(generation),
            Self::Typed(runtime) => runtime.end_composition(generation),
        }
    }

    fn cancel_composition(
        &mut self,
        generation: CompositionGeneration,
    ) -> Result<CompositionSubmission, SubmitCompositionError> {
        match self {
            Self::Counter(runtime) => runtime.cancel_composition(generation),
            Self::Typed(runtime) => runtime.cancel_composition(generation),
        }
    }

    fn set_native_wake_transport(&self, transport: NativeWakeTransport) {
        match self {
            Self::Counter(runtime) => runtime.set_wake_transport(transport),
            Self::Typed(runtime) => runtime.set_native_wake_transport(transport),
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
    /// Derived from the last actual framework observation, replaced after each pump.
    next_deadline: Option<MonotonicInstant>,
    native_clock: Option<NativeHostClock>,
    /// A deadline already covered by a quiescent checkpoint needs publication,
    /// not an endless sequence of identical pumps when no surface is bound.
    deadline_observed_at: Option<MonotonicInstant>,
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
        // Poll framework-owned local work as well as previously admitted host
        // commands. A later timer/subscription completion must not require a
        // second Engine action merely to advance the sole RunenUI runtime.
        self.begin_readiness_checkpoint();
        let batch = self.runtime.pump(runtime_pump_budget());
        self.observe_batch(batch)
    }

    fn begin_readiness_checkpoint(&mut self) {
        use runenui_runtime::MonotonicClock;
        self.deadline_observed_at = self.native_clock.as_ref().map(MonotonicClock::now);
    }

    fn observe_batch(
        &mut self,
        batch: Result<InputPumpBatch, InputObservationError>,
    ) -> Result<(), UiRuntimeSlotOperationFailure> {
        self.requires_settlement = true;
        let batch = batch.map_err(UiRuntimeSlotOperationFailure::InputObservation)?;
        self.observe_batch_report(&batch)
    }

    /// Update the existing runtime readiness state without consuming its
    /// ordered M13 records. Native callers own the returned batch directly.
    fn observe_batch_report(
        &mut self,
        batch: &InputPumpBatch,
    ) -> Result<(), UiRuntimeSlotOperationFailure> {
        self.requires_settlement = true;
        if let Err(error @ UiRuntimeSlotOperationFailure::RuntimeStopped(_)) =
            settlement_result(batch)
        {
            self.next_deadline = None;
            return Err(error);
        }
        self.next_deadline = batch.report().next_deadline();
        self.requires_settlement = batch.pause_reason().is_some() || !batch.report().is_quiescent();
        if self.requires_settlement {
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        } else {
            Ok(())
        }
    }
}

#[derive(runen_ecs::Resource)]
pub struct UiRuntimeSlotsResource {
    slots: Vec<UiRuntimeSlot>,
    next_slot_id: u64,
    native_clock: Option<NativeHostClock>,
    native_transport: Option<NativeWakeTransport>,
    native_closed: bool,
}

impl Default for UiRuntimeSlotsResource {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            next_slot_id: 1,
            native_clock: None,
            native_transport: None,
            native_closed: false,
        }
    }
}

impl UiRuntimeSlotsResource {
    pub(crate) fn configure_host_clock(
        &mut self,
        clock: Option<NativeHostClock>,
    ) -> Result<(), UiRuntimeSlotMountFailure> {
        if self.native_closed {
            return Err(UiRuntimeSlotMountFailure::IncompatibleNativeClock);
        }
        match (&self.native_clock, clock) {
            (None, None) => Ok(()),
            (Some(current), Some(clock)) if current.same_epoch(&clock) => Ok(()),
            (None, Some(clock)) if self.slots.is_empty() => {
                self.native_clock = Some(clock);
                Ok(())
            }
            _ => Err(UiRuntimeSlotMountFailure::IncompatibleNativeClock),
        }
    }

    pub(crate) fn attach_native_transport(&mut self, transport: NativeWakeTransport) {
        if self.native_clock.is_none() || self.native_closed {
            return;
        }
        for slot in &self.slots {
            slot.runtime.set_native_wake_transport(transport.clone());
        }
        self.native_transport = Some(transport);
        // The runner always imports initial readiness. A settled mount may have
        // acknowledged its wake already; installation alone is not the trigger.
    }

    pub(crate) fn native_readiness(&self) -> anyhow::Result<(bool, Option<std::time::Instant>)> {
        let Some(clock) = &self.native_clock else {
            anyhow::ensure!(
                self.slots.is_empty(),
                "native UI registry was mounted before native clock qualification"
            );
            return Ok((false, None));
        };
        if self.native_closed {
            return Ok((false, None));
        }
        clock.ensure_valid()?;
        let deadline = self
            .slots
            .iter()
            .filter_map(|slot| {
                slot.next_deadline.filter(|deadline| {
                    slot.requires_settlement
                        || slot
                            .deadline_observed_at
                            .is_none_or(|observed| *deadline > observed)
                })
            })
            .min()
            .map(|deadline| clock.deadline(deadline))
            .transpose()?;
        Ok((
            self.slots.iter().any(|slot| slot.requires_settlement),
            deadline,
        ))
    }

    pub(crate) fn settle_native_slots(&mut self) -> anyhow::Result<()> {
        self.native_readiness()?;
        if self.native_clock.is_none() || self.native_closed {
            return Ok(());
        }
        for slot in &mut self.slots {
            match slot.settle_pending() {
                Ok(()) | Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce) => {}
                Err(error) => {
                    return Err(anyhow::anyhow!(
                        "native RunenUI slot {:?} settlement failed: {error:?}",
                        slot.id
                    ));
                }
            }
        }
        self.native_readiness()?;
        Ok(())
    }

    pub(crate) fn settle_for_publication(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<(), UiRuntimeSlotOperationFailure> {
        if self.native_closed {
            return Err(UiRuntimeSlotOperationFailure::NativeHostClosed);
        }
        if self.native_clock.is_none() {
            return self.settle_slot(slot_id);
        }
        let slot = self
            .slot(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        if slot.requires_settlement {
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        } else {
            Ok(())
        }
    }

    pub(crate) fn shutdown_native_slots(&mut self) -> anyhow::Result<()> {
        self.native_closed = true;
        if let Some(transport) = &self.native_transport {
            transport.close();
        }
        let mut errors = Vec::new();
        for index in (0..self.slots.len()).rev() {
            let id = self.slots[index].id;
            match self.unmount_with_retirement(id) {
                Ok(_) => {}
                Err(error) => errors.push(format!("{id:?}: {error:?}")),
            }
        }
        anyhow::ensure!(
            errors.is_empty(),
            "native RunenUI shutdown failed: {}",
            errors.join("; ")
        );
        Ok(())
    }

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
        if self.native_closed {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::RuntimeInitialization,
            );
        }
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

        let Some(next_slot_id) = self.next_slot_id.checked_add(1) else {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::SlotIdentityExhausted,
            );
        };
        if self.slots.try_reserve(1).is_err() {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::SlotCapacityExhausted,
            );
        }

        let mut runtime = AppRuntime::<RunenwerkProgramUiApp>::mount(RunenwerkProgramUiState {
            source,
            artifact,
            resolved_state,
            registry: registry.clone(),
        });
        if let Some(clock) = &self.native_clock {
            runtime.set_monotonic_clock(clock.clone());
        }
        if let Some(transport) = &self.native_transport {
            runtime.set_wake_transport(transport.clone());
        }
        if let Err(failure) = provision_text_fonts(&mut runtime, fonts) {
            let _ = runtime.shutdown();
            return UiRuntimeSlotMountReport::rejected(screen_id, failure);
        }

        let slot_id = UiRuntimeSlotId(self.next_slot_id);
        let mut slot = UiRuntimeSlot {
            id: slot_id,
            screen_id: screen_id.clone(),
            runtime: UiRuntimeSlotKind::Counter(Box::new(runtime)),
            ingress_epoch: 1,
            requires_settlement: false,
            next_deadline: None,
            native_clock: self.native_clock.clone(),
            deadline_observed_at: None,
        };
        if self.native_clock.is_some() {
            match slot.settle_pending() {
                Ok(()) | Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce) => {}
                Err(_) => {
                    let _ = slot.runtime.shutdown();
                    return UiRuntimeSlotMountReport::rejected(
                        screen_id,
                        UiRuntimeSlotMountFailure::RuntimeInitialization,
                    );
                }
            }
        }
        self.next_slot_id = next_slot_id;
        self.slots.push(slot);
        UiRuntimeSlotMountReport::accepted(screen_id, slot_id)
    }

    /// Mounts one ordinary concrete UiApp in the *existing* Engine slot registry.
    /// This does not duplicate the normalized input or GPU presentation owners.
    ///
    /// # Errors
    /// Font admission, initial observation errors or exhausted slot identity
    /// reject before publication. Capacity-paused initial work remains mounted
    /// but is explicitly unsettled until the existing Engine host resumes it.
    pub fn mount_app<A>(
        &mut self,
        screen_id: UiTypedScreenId,
        state: A::State,
        fonts: &UiRuntimeFontConfiguration,
    ) -> UiRuntimeSlotMountReport
    where
        A: UiApp + 'static,
    {
        if self.native_closed {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::RuntimeInitialization,
            );
        }
        let Some(next_slot_id) = self.next_slot_id.checked_add(1) else {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::SlotIdentityExhausted,
            );
        };
        if self.slots.try_reserve(1).is_err() {
            return UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::SlotCapacityExhausted,
            );
        }
        let mut runtime = AppRuntime::<A>::mount(state);
        if let Some(clock) = &self.native_clock {
            runtime.set_monotonic_clock(clock.clone());
        }
        if let Some(transport) = &self.native_transport {
            runtime.set_wake_transport(transport.clone());
        }
        if let Err(error) = provision_text_fonts(&mut runtime, fonts) {
            let _ = runtime.shutdown();
            return UiRuntimeSlotMountReport::rejected(screen_id, error);
        }
        // RunenUI mounts may emit initial actions/host requests before any host
        // command or first frame. Drive the same bounded canonical pump once
        // here; a capacity pause remains resumable, not a false successful flush.
        use runenui_runtime::MonotonicClock;
        let deadline_observed_at = self.native_clock.as_ref().map(MonotonicClock::now);
        let initial = runtime.pump(runtime_pump_budget());
        let next_deadline = initial
            .as_ref()
            .ok()
            .and_then(|batch| batch.report().next_deadline());
        let requires_settlement = match settle_batch(initial) {
            Ok(()) => false,
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce) => true,
            Err(_) => {
                let _ = runtime.shutdown();
                return UiRuntimeSlotMountReport::rejected(
                    screen_id,
                    UiRuntimeSlotMountFailure::RuntimeInitialization,
                );
            }
        };
        let id = UiRuntimeSlotId(self.next_slot_id);
        self.next_slot_id = next_slot_id;
        self.slots.push(UiRuntimeSlot {
            id,
            screen_id: screen_id.clone(),
            runtime: UiRuntimeSlotKind::Typed(Box::new(TypedRuntime { runtime })),
            ingress_epoch: 1,
            requires_settlement,
            next_deadline,
            native_clock: self.native_clock.clone(),
            deadline_observed_at,
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
        let slot = self.slot_mut(slot_id)?;
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
    /// exposing runtime host mutators that could replace native clock/wake wiring.
    pub fn observe_app<A, R>(
        &self,
        slot_id: UiRuntimeSlotId,
        observe: impl FnOnce(super::UiAppObservation<'_, A>) -> R,
    ) -> Result<R, UiRuntimeSlotOperationFailure>
    where
        A: UiApp + 'static,
    {
        if self.native_closed {
            return Err(UiRuntimeSlotOperationFailure::NativeHostClosed);
        }
        let slot = self
            .slot(slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)?;
        let runtime = slot
            .runtime
            .typed::<A>()
            .ok_or(UiRuntimeSlotOperationFailure::WrongAppKind)?;
        Ok(observe(super::UiAppObservation {
            state: runtime.state(),
            pending_host_requests: runtime.pending_host_requests(),
        }))
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
        let slot = self.slot_mut(slot_id)?;
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

    /// One synchronous M13 observation at the owning mounted slot. Pending
    /// framework work is never interpreted as valid gameplay permission.
    pub fn input_ownership(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<InputOwnershipSnapshot, UiRuntimeSlotOperationFailure> {
        let slot = self.slot_mut(slot_id)?;
        if slot.requires_settlement {
            return Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce);
        }
        slot.runtime
            .input_ownership()
            .map_err(UiRuntimeSlotOperationFailure::InputObservation)
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

    /// Polls bounded RunenUI application work on the existing invoker thread,
    /// including resumed partial ingress and framework-owned wakeups.
    /// Publication stays blocked until the pending work is quiescent.
    pub fn settle_slot(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<(), UiRuntimeSlotOperationFailure> {
        self.slot_mut(slot_id)?.settle_pending()
    }

    /// Admit an exact native-normalized keyboard occurrence without inventing
    /// a UI or gameplay disposition. Rejection returns the original event and
    /// framework kind; acceptance retains the framework's work sequence.
    ///
    /// The host MUST pump/correlate the exact M13 record before allowing a
    /// competing gameplay transition.
    pub fn submit_native_keyboard(
        &mut self,
        slot_id: UiRuntimeSlotId,
        event: KeyboardEvent,
    ) -> Result<KeyboardSubmission, UiRuntimeNativeKeyboardAdmissionError> {
        let slot = match self.slot_mut(slot_id) {
            Ok(slot) => slot,
            Err(failure) => {
                return Err(UiRuntimeNativeKeyboardAdmissionError::Slot { failure, event });
            }
        };
        let next = match slot.next_ingress_epoch() {
            Ok(next) => next,
            Err(failure) => {
                return Err(UiRuntimeNativeKeyboardAdmissionError::Slot { failure, event });
            }
        };
        let receipt = slot
            .runtime
            .submit_keyboard(event)
            .map_err(UiRuntimeNativeKeyboardAdmissionError::Framework)?;
        slot.ingress_epoch = next;
        slot.requires_settlement = true;
        Ok(receipt)
    }

    /// Admit exactly one native pointer observation to the pinned runtime.
    /// Callers supply the original completed-Present input context; admission
    /// alone never authorizes gameplay or claims a displayed surface.
    pub(crate) fn submit_native_pointer(
        &mut self,
        slot_id: UiRuntimeSlotId,
        event: PointerEvent,
    ) -> Result<PointerSubmission, UiRuntimeNativePointerAdmissionError> {
        let slot = match self.slot_mut(slot_id) {
            Ok(slot) => slot,
            Err(failure) => {
                return Err(UiRuntimeNativePointerAdmissionError::Slot {
                    failure,
                    event: Box::new(event),
                });
            }
        };
        let next = match slot.next_ingress_epoch() {
            Ok(next) => next,
            Err(failure) => {
                return Err(UiRuntimeNativePointerAdmissionError::Slot {
                    failure,
                    event: Box::new(event),
                });
            }
        };
        let receipt = slot
            .runtime
            .submit_pointer(event)
            .map_err(UiRuntimeNativePointerAdmissionError::Framework)?;
        slot.ingress_epoch = next;
        slot.requires_settlement = true;
        Ok(receipt)
    }

    /// Original native committed text uses the same pinned M13 input FIFO and
    /// ingress generation as keyboard and pointer; it is never gameplay input.
    pub(crate) fn submit_native_text(
        &mut self,
        slot_id: UiRuntimeSlotId,
        event: CommittedTextEvent,
    ) -> Result<TextSubmission, UiRuntimeNativeTextAdmissionError> {
        let slot = match self.slot_mut(slot_id) {
            Ok(slot) => slot,
            Err(failure) => {
                return Err(UiRuntimeNativeTextAdmissionError::Slot { failure, event });
            }
        };
        let next = match slot.next_ingress_epoch() {
            Ok(next) => next,
            Err(failure) => {
                return Err(UiRuntimeNativeTextAdmissionError::Slot { failure, event });
            }
        };
        let receipt = slot
            .runtime
            .submit_text(event)
            .map_err(UiRuntimeNativeTextAdmissionError::Framework)?;
        slot.ingress_epoch = next;
        slot.requires_settlement = true;
        Ok(receipt)
    }

    /// Canonical native composition admission shares one ingress epoch and
    /// one pinned RunenUI queue with keyboard, pointer and committed text.
    /// Neither native preedit nor composition generation is owned by Engine.
    fn admit_native_composition<T>(
        &mut self,
        slot_id: UiRuntimeSlotId,
        submit: impl FnOnce(
            &mut UiRuntimeSlotKind,
        ) -> Result<T, UiRuntimeNativeCompositionAdmissionError>,
    ) -> Result<T, UiRuntimeNativeCompositionAdmissionError> {
        let slot = self
            .slot_mut(slot_id)
            .map_err(UiRuntimeNativeCompositionAdmissionError::Slot)?;
        let next = slot
            .next_ingress_epoch()
            .map_err(UiRuntimeNativeCompositionAdmissionError::Slot)?;
        let receipt = submit(&mut slot.runtime)?;
        slot.ingress_epoch = next;
        slot.requires_settlement = true;
        Ok(receipt)
    }

    pub(crate) fn start_native_composition(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<CompositionStartSubmission, UiRuntimeNativeCompositionAdmissionError> {
        self.admit_native_composition(slot_id, |runtime| {
            runtime
                .start_composition(None)
                .map_err(UiRuntimeNativeCompositionAdmissionError::Start)
        })
    }

    pub(crate) fn update_native_composition(
        &mut self,
        slot_id: UiRuntimeSlotId,
        generation: CompositionGeneration,
        preedit: String,
        range: Option<CompositionRange>,
    ) -> Result<CompositionSubmission, UiRuntimeNativeCompositionAdmissionError> {
        self.admit_native_composition(slot_id, |runtime| {
            runtime
                .update_composition(generation, preedit, range)
                .map_err(UiRuntimeNativeCompositionAdmissionError::Event)
        })
    }

    pub(crate) fn end_native_composition(
        &mut self,
        slot_id: UiRuntimeSlotId,
        generation: CompositionGeneration,
    ) -> Result<CompositionSubmission, UiRuntimeNativeCompositionAdmissionError> {
        self.admit_native_composition(slot_id, |runtime| {
            runtime
                .end_composition(generation)
                .map_err(UiRuntimeNativeCompositionAdmissionError::Event)
        })
    }

    pub(crate) fn cancel_native_composition(
        &mut self,
        slot_id: UiRuntimeSlotId,
        generation: CompositionGeneration,
    ) -> Result<CompositionSubmission, UiRuntimeNativeCompositionAdmissionError> {
        self.admit_native_composition(slot_id, |runtime| {
            runtime
                .cancel_composition(generation)
                .map_err(UiRuntimeNativeCompositionAdmissionError::Event)
        })
    }

    /// Return one actual canonical pump result with ALL ordered M13 records.
    /// Paused and terminal batches remain inspectable, never silently equated
    /// to successful input settlement. There is no additional input queue.
    pub fn pump_native_input_records(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<
        (InputPumpBatch, Result<(), UiRuntimeSlotOperationFailure>),
        UiRuntimeSlotOperationFailure,
    > {
        let slot = self.slot_mut(slot_id)?;
        slot.begin_readiness_checkpoint();
        slot.requires_settlement = true;
        let batch = slot
            .runtime
            .pump(runtime_pump_budget())
            .map_err(UiRuntimeSlotOperationFailure::InputObservation)?;
        let settlement = slot.observe_batch_report(&batch);
        Ok((batch, settlement))
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
        let slot = self.slot_mut(slot_id)?;
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
        let native = self.native_clock.is_some();
        let slot = self.slot_mut(slot_id)?;
        if slot.requires_settlement {
            return Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce);
        }
        let publication = slot.runtime.publish_surface(context)?;
        if native {
            // Publication can create motion deadlines after the preceding pump.
            // Observe through the public API without executing application work.
            slot.begin_readiness_checkpoint();
            let batch = slot.runtime.pump(PumpBudget::new(0, 0, 0, 0));
            match slot.observe_batch(batch) {
                Ok(()) | Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(publication)
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
        let slot = self.slot_mut(slot_id)?;
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
        let slot = self.slot_mut(slot_id)?;
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
        let slot = self.slot_mut(slot_id)?;
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
        let slot = self.slot_mut(slot_id)?;
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

    fn slot_mut(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<&mut UiRuntimeSlot, UiRuntimeSlotOperationFailure> {
        if self.native_closed {
            return Err(UiRuntimeSlotOperationFailure::NativeHostClosed);
        }
        self.slots
            .iter_mut()
            .find(|slot| slot.id == slot_id)
            .ok_or(UiRuntimeSlotOperationFailure::MissingSlot)
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

fn settle_batch(
    batch: Result<InputPumpBatch, InputObservationError>,
) -> Result<(), UiRuntimeSlotOperationFailure> {
    let batch = batch.map_err(UiRuntimeSlotOperationFailure::InputObservation)?;
    settlement_result(&batch)
}

fn settlement_result(batch: &InputPumpBatch) -> Result<(), UiRuntimeSlotOperationFailure> {
    match batch.report().outcome() {
        runenui_runtime::PumpOutcome::Quiescent | runenui_runtime::PumpOutcome::BudgetExhausted => {
        }
        outcome => return Err(UiRuntimeSlotOperationFailure::RuntimeStopped(outcome)),
    }
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
    use std::time::Duration;

    use ui_binding::HostDataSnapshot;
    use ui_controls::{ControlPackageRegistry, runenwerk_control_package};
    use ui_definition::{
        AuthoredBindingRef, AuthoredControlAccessibilityDefinition, AuthoredControlKindId,
        AuthoredControlValue, AuthoredId, AuthoredRouteId, UiValueBinding,
    };
    use ui_program::UiProgramSourceId;

    use crate::plugins::ui::UiTypedActionDescriptor;

    use super::*;

    struct InitialTimerApp;

    impl UiApp for InitialTimerApp {
        type State = usize;
        type Action = bool;
        type HostProtocol = runenui_core::NoHostProtocol;

        fn root(_: &usize) -> impl View<bool> {
            text("timer")
        }

        fn initial_effects(_: &usize) -> impl runenui_core::IntoEffects<bool, Self::HostProtocol> {
            Effects::timer(
                runenui_core::TimerEffect::once(Duration::from_secs(3600), || true)
                    .keyed(runenui_core::WorkKey::new("initial-timer").unwrap()),
            )
        }

        fn update(
            state: &mut usize,
            fired: bool,
        ) -> impl runenui_core::IntoUpdateOutput<bool, Self::HostProtocol> {
            if fired {
                *state += 1;
                Effects::none()
            } else {
                Effects::cancel(
                    runenui_core::WorkFamily::Timer,
                    runenui_core::WorkKey::new("initial-timer").unwrap(),
                )
            }
        }
    }

    #[test]
    fn native_committed_text_rejects_unmounted_target_without_losing_original_utf8() {
        let mut slots = UiRuntimeSlotsResource::default();
        let text = CommittedTextEvent::new("漢字é", Some(InputDeviceId::new(11).unwrap()))
            .expect("original nonempty Unicode text");
        let error = slots
            .submit_native_text(UiRuntimeSlotId(u64::MAX), text)
            .expect_err("a nonexistent native M13 recipient cannot accept text");
        match error {
            UiRuntimeNativeTextAdmissionError::Slot {
                failure: UiRuntimeSlotOperationFailure::MissingSlot,
                event,
            } => {
                assert_eq!(event.text(), "漢字é");
                assert_eq!(event.device_id(), InputDeviceId::new(11));
            }
            other => panic!("unexpected M13 native text admission result: {other:?}"),
        }
    }

    #[test]
    fn native_committed_text_from_focused_typed_field_has_exact_m13_receipt() {
        use runenui_core::{
            CommandOrigin, EditIntent, EditResolution, IntoUpdateOutput, NoHostProtocol,
            SemanticCommand, SemanticEditableMode, TextAffinity, TextDocumentId,
            TextDocumentRevision, TextDocumentSnapshot, TextPosition, TextSelection, UpdateOutput,
        };

        #[derive(Clone)]
        struct TextState {
            text: String,
            revision: u64,
        }
        enum TextAction {
            Edit(Box<EditIntent>),
        }
        struct NativeTextApp;
        impl UiApp for NativeTextApp {
            type State = TextState;
            type Action = TextAction;
            type HostProtocol = NoHostProtocol;

            fn root(state: &Self::State) -> impl View<Self::Action> {
                let snapshot = TextDocumentSnapshot::new(
                    TextDocumentId::new(1276),
                    TextDocumentRevision::new(state.revision),
                );
                let position = TextPosition::new(
                    snapshot,
                    &state.text,
                    state.text.len(),
                    TextAffinity::Downstream,
                )
                .expect("the complete UTF-8 source ends at a scalar boundary");
                runenui_core::text_field(
                    snapshot,
                    state.text.clone(),
                    TextSelection::collapsed(position),
                    SemanticEditableMode::SingleLine,
                    |intent| TextAction::Edit(Box::new(intent)),
                )
                .expect("controlled native text source is valid")
                .id("native.text.field")
            }

            fn update(
                state: &mut Self::State,
                action: Self::Action,
            ) -> impl IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                let TextAction::Edit(intent) = action;
                let replacement = intent.replacement();
                state.text.replace_range(
                    replacement.start()..replacement.end(),
                    intent.replacement_text(),
                );
                state.revision += 1;
                let snapshot = TextDocumentSnapshot::new(
                    TextDocumentId::new(1276),
                    TextDocumentRevision::new(state.revision),
                );
                UpdateOutput::edit(EditResolution::accepted(intent.request().clone(), snapshot))
            }
        }

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<NativeTextApp>(
                UiTypedScreenId::new("native.text.focused"),
                TextState {
                    text: String::new(),
                    revision: 0,
                },
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("real typed text widget mounted");
        let owner = {
            let runtime = slots
                .slot_mut(slot)
                .expect("mounted slot")
                .runtime
                .typed_mut::<NativeTextApp>()
                .expect("typed app runtime");
            runtime.index().nodes()[0].id().clone()
        };
        slots
            .slot_mut(slot)
            .expect("mounted slot")
            .runtime
            .typed_mut::<NativeTextApp>()
            .expect("typed runtime")
            .submit_command(
                owner,
                SemanticCommand::RequestFocus,
                CommandOrigin::programmatic(),
            )
            .expect("real widget accepts focus request");
        slots.settle_slot(slot).expect("focus request settles");
        let before = slots.input_ownership(slot).expect("actual M13 owner");
        assert!(
            before.keyboard().focused_node().is_some(),
            "text ingress requires actual focused RunenUI ownership"
        );

        let original = CommittedTextEvent::new("漢字é", None)
            .expect("original native Unicode commit is nonempty");
        let receipt = slots
            .submit_native_text(slot, original)
            .expect("real focused field accepts native commit");
        let (batch, status) = slots
            .pump_native_input_records(slot)
            .expect("canonical M13 pump produces real ordered receipts");
        assert_eq!(status, Ok(()));
        assert!(
            matches!(
                native_text_finality(before.scope(), None, receipt, &batch, &status),
                Ok(UiInputFinality::Committed(_))
            ),
            "only an exact reached M13 text settlement qualifies this commit"
        );
        assert_eq!(
            native_text_finality(
                before.scope(),
                Some(InputDeviceId::new(11).expect("checked device")),
                receipt,
                &batch,
                &status,
            ),
            Err(UiRuntimeNativeTextEvidenceFailure::WrongDevice),
            "winit IME provides no keyboard device; it must not inherit one"
        );
        let later = slots
            .submit_native_text(
                slot,
                CommittedTextEvent::new("Ω", None).expect("second Unicode commit"),
            )
            .expect("second independent native text sequence");
        assert_eq!(
            native_text_finality(before.scope(), None, later, &batch, &status),
            Err(UiRuntimeNativeTextEvidenceFailure::NotProcessed),
            "an earlier pump must never authorize an unprocessed later commit"
        );
        let edited = &slots
            .slot_mut(slot)
            .expect("text runtime still mounted")
            .runtime
            .typed_mut::<NativeTextApp>()
            .expect("typed text owner")
            .state()
            .text;
        assert_eq!(
            edited, "漢字é",
            "real native text reached the actual M10 editor without duplication"
        );

        // Complete the independently sequenced later Unicode commit before
        // starting a new composition lifecycle on the same real focused field.
        let (_, finish_text) = slots
            .pump_native_input_records(slot)
            .expect("all earlier committed text settles first");
        assert_eq!(finish_text, Ok(()));

        let before_start = slots
            .input_ownership(slot)
            .expect("the same focused typed owner remains live");
        let start = slots
            .start_native_composition(slot)
            .expect("real focused TextField accepts original OS composition start");
        let (start_batch, start_status) = slots
            .pump_native_input_records(slot)
            .expect("M13 composition start returns ordered evidence");
        assert_eq!(start_status, Ok(()));
        assert!(matches!(
            native_composition_finality(
                before_start.scope(),
                start.sequence(),
                &start_batch,
                &start_status
            ),
            Ok(UiInputFinality::Committed(_))
        ));
        let started = slots
            .input_ownership(slot)
            .expect("M13 composition generation is observable");
        assert_eq!(
            started.keyboard().composition_generation(),
            Some(start.generation()),
            "RunenUI, never Engine, owns the active generation"
        );

        let preedit = "aßz";
        let range =
            CompositionRange::new(preedit, 1, 3).expect("UTF-8 range aligns with the ß scalar");
        let update = slots
            .update_native_composition(
                slot,
                start.generation().clone(),
                preedit.to_owned(),
                Some(range),
            )
            .expect("accepted original Unicode preedit");
        let (update_batch, update_status) = slots
            .pump_native_input_records(slot)
            .expect("preedit remains in the same canonical M13 input queue");
        assert_eq!(update_status, Ok(()));
        assert!(matches!(
            native_composition_finality(
                started.scope(),
                update.sequence(),
                &update_batch,
                &update_status
            ),
            Ok(UiInputFinality::Committed(_))
        ));

        let before_end = slots
            .input_ownership(slot)
            .expect("original composition still owns the focused field");
        let end = slots
            .end_native_composition(slot, start.generation().clone())
            .expect("original native IME commit ends its generation");
        let (end_batch, end_status) = slots
            .pump_native_input_records(slot)
            .expect("composition end must settle before durable commit");
        assert_eq!(end_status, Ok(()));
        assert!(matches!(
            native_composition_finality(
                before_end.scope(),
                end.sequence(),
                &end_batch,
                &end_status
            ),
            Ok(UiInputFinality::Committed(_))
        ));
        assert!(
            slots
                .input_ownership(slot)
                .expect("composition end settled")
                .keyboard()
                .composition_generation()
                .is_none()
        );

        let renewed = slots
            .start_native_composition(slot)
            .expect("new OS preedit receives a distinct canonical generation");
        let (_, renewed_status) = slots
            .pump_native_input_records(slot)
            .expect("second composition start settles");
        assert_eq!(renewed_status, Ok(()));
        let before_cancel = slots
            .input_ownership(slot)
            .expect("new composition has one M13 owner");
        assert_eq!(
            before_cancel.keyboard().composition_generation(),
            Some(renewed.generation())
        );
        let cancel = slots
            .cancel_native_composition(slot, renewed.generation().clone())
            .expect("OS IME disabled cancels the current generation");
        let (cancel_batch, cancel_status) = slots
            .pump_native_input_records(slot)
            .expect("composition cancel has exact ordered settlement");
        assert_eq!(cancel_status, Ok(()));
        assert!(matches!(
            native_composition_finality(
                before_cancel.scope(),
                cancel.sequence(),
                &cancel_batch,
                &cancel_status
            ),
            Ok(UiInputFinality::Committed(_))
        ));
        assert!(
            slots
                .input_ownership(slot)
                .expect("canonical cancellation settled")
                .keyboard()
                .composition_generation()
                .is_none()
        );
    }

    #[test]
    fn native_pointer_admission_keeps_exact_device_stream_surface_and_m13_sequence() {
        use runenui_core::{LogicalPoint, NoHostProtocol, PointerDeviceKind, PointerPhase};
        use runenui_runtime::{
            InputArbitrationRecord, UiInputFamily, UiInputFinality, UiInputProcessingRejection,
        };

        struct NativePointerApp;
        impl UiApp for NativePointerApp {
            type State = ();
            type Action = ();
            type HostProtocol = NoHostProtocol;

            fn root(_: &Self::State) -> impl View<Self::Action> {
                text("pointer settlement")
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let fonts = UiRuntimeFontConfiguration::new(vec![CONTROLLED_FONT_BYTES.to_vec()])
            .with_generic_mapping(
                GenericFontFamily::SansSerif,
                vec![FontFamilyName::new("JetBrains Mono").expect("test font name")],
            );
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<NativePointerApp>(UiTypedScreenId::new("native.pointer.probe"), (), &fonts)
            .slot_id()
            .expect("one real pinned RunenUI runtime");
        let style = StyleEnvironment::default();
        let build = SurfaceBuildContext::tight(
            &style,
            LogicalSize::new(
                LogicalLength::from(RUNTIME_SURFACE_WIDTH),
                LogicalLength::from(RUNTIME_SURFACE_HEIGHT),
            ),
        );
        let published = slots
            .publish_surface_for_slot(slot, &build)
            .expect("actual runtime publishes hit context");
        let device = InputDeviceId::new(7).expect("checked native device");
        let other_device = InputDeviceId::new(8).expect("other native device");
        let pointer = PointerId::new(3).expect("nonzero pointer stream");
        let other_pointer = PointerId::new(5).expect("different pointer stream");
        let event = || {
            PointerEvent::new(
                pointer,
                PointerDeviceKind::Mouse,
                PointerPhase::Move,
                LogicalPoint::new(12.0, 8.0).expect("finite"),
                published.input_context().clone(),
            )
            .with_device_id(device)
        };
        let missing = slots
            .submit_native_pointer(UiRuntimeSlotId(u64::MAX), event())
            .expect_err("unmounted recipient cannot accept work");
        assert!(matches!(
            missing,
            UiRuntimeNativePointerAdmissionError::Slot {
                failure: UiRuntimeSlotOperationFailure::MissingSlot,
                ref event,
            } if event.pointer_id() == pointer && event.device_id() == Some(device)
        ));
        let before = slots
            .input_ownership(slot)
            .expect("valid pre-input M13 scope");
        let admitted = slots
            .submit_native_pointer(slot, event())
            .expect("real typed runtime accepts one native pointer");
        let (batch, status) = slots
            .pump_native_input_records(slot)
            .expect("one owner returns exact ordered M13 records");
        assert_eq!(status, Ok(()));
        let settlement = batch
            .ordered_records()
            .iter()
            .find_map(|record| match record {
                InputArbitrationRecord::InputSettled(value)
                    if value.sequence() == admitted.sequence() =>
                {
                    Some(value)
                }
                _ => None,
            })
            .expect("one reached pointer receipt");
        assert_eq!(settlement.family(), UiInputFamily::Pointer);
        assert_eq!(settlement.device_id(), Some(device));
        assert_eq!(settlement.pointer_id(), Some(pointer));
        assert_eq!(
            settlement.surface_id(),
            Some(published.input_context().surface_id())
        );
        assert_eq!(
            native_pointer_finality(
                before.scope(),
                device,
                pointer,
                published.input_context().surface_id(),
                admitted,
                &batch,
                &status,
            ),
            Ok(settlement.finality().clone()),
            "exact pointer finality must come from the framework's receipt"
        );
        assert_eq!(
            native_pointer_finality(
                before.scope(),
                device,
                pointer,
                published.input_context().surface_id(),
                admitted,
                &batch,
                &Err(UiRuntimeSlotOperationFailure::MissingSlot),
            ),
            Err(UiRuntimeNativePointerEvidenceFailure::Unsettled),
            "a failed canonical M13 pump never qualifies an earlier matching pointer receipt"
        );
        assert_eq!(
            native_pointer_finality(
                before.scope(),
                other_device,
                pointer,
                published.input_context().surface_id(),
                admitted,
                &batch,
                &status,
            ),
            Err(UiRuntimeNativePointerEvidenceFailure::WrongDevice),
        );
        assert_eq!(
            native_pointer_finality(
                before.scope(),
                device,
                other_pointer,
                published.input_context().surface_id(),
                admitted,
                &batch,
                &status,
            ),
            Err(UiRuntimeNativePointerEvidenceFailure::WrongPointer),
        );
        // A correctly scoped, sequenced, and device-qualified original
        // pointer can still reach an exact ProcessingRejected receipt. That
        // is NOT a committed native pointer stream or gameplay permission.
        let wrong_stream_device = PointerEvent::new(
            pointer,
            PointerDeviceKind::Mouse,
            PointerPhase::Move,
            LogicalPoint::new(12.0, 8.0).expect("finite"),
            published.input_context().clone(),
        )
        .with_device_id(other_device)
        .with_buttons(runenui_core::PointerButtons::new([
            runenui_core::PointerButton::Primary,
        ]));
        let rejected_submission = slots
            .submit_native_pointer(slot, wrong_stream_device)
            .expect("framework queue admits original foreign-device pointer for processing");
        let (rejected_batch, rejected_status) = slots
            .pump_native_input_records(slot)
            .expect("foreign-device pointer obtains canonical terminal M13 receipt");
        assert_eq!(rejected_status, Ok(()));
        let refused = native_pointer_finality(
            before.scope(),
            other_device,
            pointer,
            published.input_context().surface_id(),
            rejected_submission,
            &rejected_batch,
            &rejected_status,
        );
        assert!(
            matches!(
                refused,
                Ok(UiInputFinality::ProcessingRejected(
                    UiInputProcessingRejection::InvalidPointerStream
                ))
            ),
            "a reached invalid foreign-device stream must be expressly rejected, not committed: {refused:?}"
        );

        let later = slots
            .submit_native_pointer(slot, event())
            .expect("second independently sequenced original pointer event");
        assert_eq!(
            native_pointer_finality(
                before.scope(),
                device,
                pointer,
                published.input_context().surface_id(),
                later,
                &batch,
                &status,
            ),
            Err(UiRuntimeNativePointerEvidenceFailure::NotProcessed),
        );
        let foreign = slots
            .mount_app::<NativePointerApp>(
                UiTypedScreenId::new("native.pointer.foreign"),
                (),
                &fonts,
            )
            .slot_id()
            .expect("distinct framework scope");
        let foreign_ownership = slots.input_ownership(foreign).expect("foreign scope");
        assert_eq!(
            native_pointer_finality(
                foreign_ownership.scope(),
                device,
                pointer,
                published.input_context().surface_id(),
                admitted,
                &batch,
                &status,
            ),
            Err(UiRuntimeNativePointerEvidenceFailure::WrongScope),
        );
    }

    #[test]
    fn native_keyboard_admission_preserves_exact_rejection_and_ordered_m13_settlement() {
        use runenui_core::{
            CommandOrigin, KeyLocation, KeyModifiers, KeyboardCompositionState, KeyboardPhase,
            LogicalKey, NoHostProtocol, PhysicalKey,
        };
        use runenui_runtime::{InputArbitrationRecord, SubmitKeyboardErrorKind, UiInputFinality};

        struct NativeKeyboardApp;
        impl UiApp for NativeKeyboardApp {
            type State = ();
            type Action = ();
            type HostProtocol = NoHostProtocol;

            fn root(_: &Self::State) -> impl View<Self::Action> {
                button("Native focus target")
                    .id("native.keyboard.focus")
                    .key("native.keyboard.focus")
                    .on_activate(|| ())
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<NativeKeyboardApp>(
                UiTypedScreenId::new("native.keyboard.probe"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("actual RunenUI typed runtime mounts");
        let native_device = InputDeviceId::new(7).expect("checked native device identity");
        let keyboard = |device: Option<InputDeviceId>| {
            KeyboardEvent::new(
                KeyboardPhase::Down,
                PhysicalKey::Code(String::from("KeyW")),
                LogicalKey::Character(String::from("w")),
                KeyModifiers::NONE,
                false,
                KeyLocation::Standard,
                KeyboardCompositionState::Inactive,
                device,
            )
        };

        let epoch = slots.ingress_epoch(slot).expect("mount generation");
        let before_unfocused = slots
            .input_ownership(slot)
            .expect("fresh M13 ownership before rejected input");
        let missing = slots
            .submit_native_keyboard(UiRuntimeSlotId(u64::MAX), keyboard(Some(native_device)))
            .expect_err("unknown slot cannot admit native input");
        assert!(matches!(
            missing,
            UiRuntimeNativeKeyboardAdmissionError::Slot {
                failure: UiRuntimeSlotOperationFailure::MissingSlot,
                ref event,
            } if event.physical_key() == &PhysicalKey::Code(String::from("KeyW"))
        ));
        let rejected = slots
            .submit_native_keyboard(slot, keyboard(Some(native_device)))
            .expect_err("missing focus rejects before M13 sequence reservation");
        assert!(matches!(
            rejected,
            UiRuntimeNativeKeyboardAdmissionError::Framework(ref error)
                if error.kind() == SubmitKeyboardErrorKind::NoFocusedTarget
                    && error.event().physical_key()
                        == &PhysicalKey::Code(String::from("KeyW"))
        ));
        // The framework may reject device-less headless keyboard input for
        // lack of focus, but this is not an originating native-device receipt.
        let missing_native_device = slots
            .submit_native_keyboard(slot, keyboard(None))
            .expect_err("missing focus rejects device-less keyboard");
        assert_eq!(slots.ingress_epoch(slot), Some(epoch));
        let after_unfocused = slots
            .input_ownership(slot)
            .expect("fresh M13 ownership after rejected input");
        assert!(native_no_focused_keyboard_claim_witness(
            native_device,
            &before_unfocused,
            &after_unfocused,
            &rejected,
        ));
        assert!(!native_no_focused_keyboard_claim_witness(
            native_device,
            &before_unfocused,
            &after_unfocused,
            &missing,
        ));
        assert!(!native_no_focused_keyboard_claim_witness(
            native_device,
            &before_unfocused,
            &after_unfocused,
            &missing_native_device,
        ));
        let another_device = InputDeviceId::new(17).expect("valid distinct native device");
        assert!(!native_no_focused_keyboard_claim_witness(
            another_device,
            &before_unfocused,
            &after_unfocused,
            &rejected,
        ));
        // The authentic original Up closes physical held truth even when
        // the focused framework target is absent. It must never be mistaken
        // for a new positively permitted gameplay Down.
        let original_up = slots
            .submit_native_keyboard(
                slot,
                KeyboardEvent::new(
                    KeyboardPhase::Up,
                    PhysicalKey::Code(String::from("KeyW")),
                    LogicalKey::Character(String::from("w")),
                    KeyModifiers::NONE,
                    false,
                    KeyLocation::Standard,
                    KeyboardCompositionState::Inactive,
                    Some(native_device),
                ),
            )
            .expect_err("unfocused original Up has no framework target");
        let after_original_up = slots
            .input_ownership(slot)
            .expect("no-focus physical Up does not mutate M13 ownership");
        assert!(native_original_keyboard_release_nonclaim(
            native_device,
            &after_unfocused,
            &after_original_up,
            &original_up,
        ));
        assert!(!native_no_focused_keyboard_claim_witness(
            native_device,
            &after_unfocused,
            &after_original_up,
            &original_up,
        ));
        assert!(!native_original_keyboard_release_nonclaim(
            another_device,
            &after_unfocused,
            &after_original_up,
            &original_up,
        ));
        // A synthesized cancellation is not the original keyboard Down
        // occurrence for which the owner authorized NoFocusedTarget nonclaim.
        let cancel = slots
            .submit_native_keyboard(
                slot,
                KeyboardEvent::new(
                    KeyboardPhase::Cancel,
                    PhysicalKey::Code(String::from("KeyW")),
                    LogicalKey::Character(String::from("w")),
                    KeyModifiers::NONE,
                    false,
                    KeyLocation::Standard,
                    KeyboardCompositionState::Inactive,
                    Some(native_device),
                ),
            )
            .expect_err("no focused RunenUI target for Cancel");
        let after_cancel = slots
            .input_ownership(slot)
            .expect("synthetic rejection cannot mutate M13 ownership");
        assert!(!native_no_focused_keyboard_claim_witness(
            native_device,
            &after_unfocused,
            &after_cancel,
            &cancel,
        ));
        assert!(!native_original_keyboard_release_nonclaim(
            native_device,
            &after_unfocused,
            &after_cancel,
            &cancel,
        ));
        assert_eq!(slots.ingress_epoch(slot), Some(epoch));

        let focus_id = ElementId::new("native.keyboard.focus").expect("valid authored target");
        let runtime = slots
            .slot_mut(slot)
            .expect("mounted")
            .runtime
            .typed_mut::<NativeKeyboardApp>()
            .expect("exact UiApp");
        let target = runtime
            .index()
            .nodes()
            .iter()
            .find(|node| node.authored_id() == Some(&focus_id))
            .expect("mounted focusable target")
            .id()
            .clone();
        let focus_id_from_runtime = target.clone();
        runtime
            .submit_command(
                target,
                SemanticCommand::RequestFocus,
                CommandOrigin::programmatic(),
            )
            .expect("semantic focus admitted");
        slots.settle_slot(slot).expect("focus settles");
        let focused = slots
            .input_ownership(slot)
            .expect("fresh canonical M13 ownership after focus");
        assert_eq!(
            focused.keyboard().focused_node(),
            Some(&focus_id_from_runtime),
            "the semantic focus request must establish a real keyboard target"
        );
        assert!(!native_no_focused_keyboard_claim_witness(
            native_device,
            &before_unfocused,
            &focused,
            &rejected,
        ));

        let receipt = slots
            .submit_native_keyboard(slot, keyboard(Some(native_device)))
            .expect("focused keyboard event accepted");
        assert_eq!(slots.ingress_epoch(slot), Some(epoch + 1));
        let (batch, status) = slots
            .pump_native_input_records(slot)
            .expect("bounded input observation succeeds");
        assert_eq!(status, Ok(()));
        assert_eq!(batch.processed_through(), Some(receipt.sequence()));
        let settled = batch
            .ordered_records()
            .iter()
            .find_map(|record| match record {
                InputArbitrationRecord::InputSettled(value)
                    if value.sequence() == receipt.sequence() =>
                {
                    Some(value)
                }
                _ => None,
            })
            .expect("exact reached M13 input");
        assert_eq!(settled.scope(), batch.final_ownership().scope());
        assert!(matches!(settled.finality(), UiInputFinality::Committed(_)));
        assert_eq!(
            native_keyboard_finality(settled.scope(), native_device, receipt, &batch, &status),
            Ok(settled.finality().clone())
        );

        assert_eq!(
            native_keyboard_finality(
                settled.scope(),
                native_device,
                receipt,
                &batch,
                &Err(UiRuntimeSlotOperationFailure::MissingSlot),
            ),
            Err(UiRuntimeNativeKeyboardEvidenceFailure::Unsettled),
            "even a matching accepted keyboard receipt is invalid after a failed M13 pump"
        );
        assert_eq!(
            native_keyboard_finality(settled.scope(), another_device, receipt, &batch, &status,),
            Err(UiRuntimeNativeKeyboardEvidenceFailure::WrongDevice),
            "an unrelated device cannot borrow this exact M13 receipt"
        );

        let missing_scope_slot = slots
            .mount_app::<NativeKeyboardApp>(
                UiTypedScreenId::new("native.keyboard.foreign"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("independent mounted scope");
        let foreign_scope = slots
            .input_ownership(missing_scope_slot)
            .expect("foreign scope is observable");
        assert_eq!(
            native_keyboard_finality(
                foreign_scope.scope(),
                native_device,
                receipt,
                &batch,
                &status
            ),
            Err(UiRuntimeNativeKeyboardEvidenceFailure::WrongScope)
        );

        let next = slots
            .submit_native_keyboard(slot, keyboard(Some(native_device)))
            .expect("second distinct native keyboard submission");
        assert_eq!(
            native_keyboard_finality(settled.scope(), native_device, next, &batch, &status),
            Err(UiRuntimeNativeKeyboardEvidenceFailure::NotProcessed)
        );

        // Another owner pumping before host correlation must not leave an
        // inferred grant. The first batch cannot stand in for later work.
        slots.settle_slot(slot).expect("canonical work settles");
        let (later, later_status) = slots
            .pump_native_input_records(slot)
            .expect("later empty canonical pump");
        let lost_observation =
            native_keyboard_finality(settled.scope(), native_device, next, &later, &later_status);
        assert!(lost_observation.is_err());

        // RunenUI can process an observation without a hardware device,
        // but that M13 settlement is not valid native game-permission proof.
        let device_less = slots
            .submit_native_keyboard(slot, keyboard(None))
            .expect("non-native keyboard observation can be submitted");
        let (device_less_batch, device_less_status) = slots
            .pump_native_input_records(slot)
            .expect("real M13 batch is observable");
        assert_eq!(device_less_status, Ok(()));
        assert_eq!(
            native_keyboard_finality(
                settled.scope(),
                native_device,
                device_less,
                &device_less_batch,
                &device_less_status,
            ),
            Err(UiRuntimeNativeKeyboardEvidenceFailure::WrongDevice)
        );
    }

    #[test]
    fn app_configuration_installs_one_native_epoch_before_initial_timer_settlement() {
        use crate::app::App;
        use crate::plugins::ui::AppUiExt;
        use runenui_runtime::MonotonicClock;
        let mut app = App::new();
        let first = app
            .ui()
            .mount_app::<InitialTimerApp>(
                UiTypedScreenId::new("native.first"),
                0,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        let clock = app.world().resource::<NativeHostClock>().unwrap().clone();
        let before = clock.now();
        let second = app
            .ui()
            .mount_app::<InitialTimerApp>(
                UiTypedScreenId::new("native.second"),
                0,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
        assert!(slots.native_clock.as_ref().unwrap().same_epoch(&clock));
        assert!(
            slots.slot(first).unwrap().next_deadline.unwrap().as_nanos()
                > Duration::from_secs(3600).as_nanos() as u64
        );
        assert!(
            slots.slot(second).unwrap().next_deadline.unwrap()
                >= before.checked_add(Duration::from_secs(3600)).unwrap()
        );
        assert!(
            slots.native_transport.is_none(),
            "initial timers precede proxy creation"
        );
        assert!(slots.native_readiness().unwrap().1.is_some());
    }

    #[test]
    fn headless_initial_timer_remains_manual_and_late_native_binding_is_rejected() {
        use crate::app::App;
        use crate::plugins::ui::AppUiExt;
        let mut app = App::headless();
        let id = app
            .ui()
            .mount_app::<InitialTimerApp>(
                UiTypedScreenId::new("headless.timer"),
                0,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        assert!(!app.world().has_resource::<NativeHostClock>());
        let slots = app
            .world_mut()
            .resource_mut::<UiRuntimeSlotsResource>()
            .unwrap();
        let deadline = MonotonicInstant::ZERO
            .checked_add(Duration::from_secs(3600))
            .unwrap();
        assert_eq!(slots.slot(id).unwrap().next_deadline, Some(deadline));
        assert_eq!(
            slots.configure_host_clock(Some(NativeHostClock::new())),
            Err(UiRuntimeSlotMountFailure::IncompatibleNativeClock)
        );
        assert!(slots.native_clock.is_none());
        let runtime = slots
            .slot_mut(id)
            .unwrap()
            .runtime
            .typed_mut::<InitialTimerApp>()
            .unwrap();
        runtime.advance_time(Duration::from_secs(3600)).unwrap();
        slots.settle_slot(id).unwrap();
        assert_eq!(
            slots.observe_app::<InitialTimerApp, _>(id, |runtime| *runtime.state()),
            Ok(1)
        );
        assert!(slots.slot(id).unwrap().next_deadline.is_none());
    }

    #[test]
    fn native_host_preparation_rejects_direct_manual_slot_registration_without_rebasing() {
        let mut slots = UiRuntimeSlotsResource::default();
        let id = slots
            .mount_app::<InitialTimerApp>(
                UiTypedScreenId::new("direct.manual"),
                0,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        let deadline = slots.slot(id).unwrap().next_deadline;
        let mut app = crate::app::App::new();
        app.insert_resource(slots);
        assert!(app.prepare_native_ui_host().is_err());
        let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
        assert!(slots.native_clock.is_none());
        assert_eq!(slots.slot(id).unwrap().next_deadline, deadline);
        let slots = app
            .world_mut()
            .resource_mut::<UiRuntimeSlotsResource>()
            .unwrap();
        assert!(
            slots.native_readiness().is_err(),
            "late native enrollment is fail-closed too"
        );
        assert!(slots.settle_native_slots().is_err());
        assert_eq!(slots.slot(id).unwrap().next_deadline, deadline);
        slots.shutdown_native_slots().unwrap();
        assert!(
            slots.is_empty(),
            "even rejected native enrollment has observed retirement"
        );
    }

    #[test]
    fn native_counter_font_admission_obeys_pre_mount_clock_and_rejects_rebinding() {
        let mut slots = UiRuntimeSlotsResource::default();
        let clock = NativeHostClock::new();
        slots.configure_host_clock(Some(clock.clone())).unwrap();
        let (source, _, _, registry) = counter_projection_fixture();
        let id = slots
            .mount_with_fonts(
                source,
                &registry,
                &UiRuntimeFontConfiguration::new(vec![CONTROLLED_FONT_BYTES.to_vec()]),
            )
            .slot_id()
            .unwrap();
        assert!(slots.slot(id).unwrap().runtime.counter().is_ok());
        assert!(slots.native_clock.as_ref().unwrap().same_epoch(&clock));
        assert_eq!(
            slots.configure_host_clock(Some(NativeHostClock::new())),
            Err(UiRuntimeSlotMountFailure::IncompatibleNativeClock)
        );
        assert!(slots.native_clock.as_ref().unwrap().same_epoch(&clock));
        slots.shutdown_native_slots().unwrap();
        assert!(slots.is_empty());
        assert!(slots.native_readiness().unwrap().1.is_none());
        assert!(
            slots
                .mount_app::<InitialTimerApp>(
                    UiTypedScreenId::new("native.closed"),
                    0,
                    &UiRuntimeFontConfiguration::default()
                )
                .failure()
                .is_some()
        );
    }

    #[test]
    fn cancellation_removes_native_deadline_and_closed_runtime_is_not_retriable_work() {
        let mut slots = UiRuntimeSlotsResource::default();
        slots
            .configure_host_clock(Some(NativeHostClock::new()))
            .unwrap();
        let id = slots
            .mount_app::<InitialTimerApp>(
                UiTypedScreenId::new("native.cancel"),
                0,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        assert!(slots.native_readiness().unwrap().1.is_some());
        slots
            .submit_app_action::<InitialTimerApp>(id, false)
            .unwrap();
        assert!(slots.native_readiness().unwrap().1.is_none());
        slots.slot_mut(id).unwrap().runtime.shutdown().unwrap();
        assert!(matches!(
            slots.settle_slot(id),
            Err(UiRuntimeSlotOperationFailure::RuntimeStopped(
                runenui_runtime::PumpOutcome::Closed
            ))
        ));
        assert!(
            slots.settle_native_slots().is_err(),
            "terminal runtime must fail the host, not self-wake forever"
        );
    }

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
        assert!(
            associations
                .promote_completed_publication(fresh_id)
                .is_some()
        );
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

        use crate::plugins::render::RenderFrameProducerId;
        use crate::plugins::render::host::RenderSurfaceId;
        use crate::plugins::ui::presentation::{
            UiRuntimeNativeMapping, UiRuntimePresentationAssociationsResource,
            UiRuntimePresentationBinding,
        };
        use crate::runtime::NativeWindowId;

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
        assert_eq!(
            slots.complete_app_host_request::<ApprovalApp>(
                increment,
                &token,
                ApprovalResponse::Accepted,
            ),
            Err(UiRuntimeSlotOperationFailure::WrongAppKind),
            "a host request cannot be completed in a different typed application"
        );
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
        let increment_publication = slots.publish_surface_for_slot(increment, &surface).unwrap();
        let approval_publication = slots.publish_surface_for_slot(approval, &surface).unwrap();
        assert!(!increment_publication.paint_scene().items().is_empty());
        assert!(!approval_publication.paint_scene().items().is_empty());

        // Both unlike apps use the SAME Engine association ledger with
        // independent slot/producer identity. Synthetic terminal transitions
        // below test correlation only; native Vulkan proves real GPU execution.
        let render_surface = RenderSurfaceId::primary();
        let mapping =
            UiRuntimeNativeMapping::new(NativeWindowId::primary(), render_surface, (320, 160), 1.0)
                .expect("native mapping");
        let increment_binding = UiRuntimePresentationBinding::new(
            increment,
            RenderFrameProducerId::try_from_raw(91).expect("producer identity"),
            render_surface,
            style.clone(),
        );
        let approval_binding = UiRuntimePresentationBinding::new(
            approval,
            RenderFrameProducerId::try_from_raw(92).expect("producer identity"),
            render_surface,
            style,
        );
        let mut associations = UiRuntimePresentationAssociationsResource::default();
        associations.bind(increment_binding.clone()).unwrap();
        associations.bind(approval_binding.clone()).unwrap();
        assert!(associations.update_mapping(increment, mapping));
        assert!(associations.update_mapping(approval, mapping));
        let first_id = associations
            .stage(
                increment_binding.clone(),
                mapping,
                slots.ingress_epoch(increment).unwrap(),
                increment_publication.clone(),
            )
            .unwrap();
        let second_id = associations
            .stage(
                approval_binding,
                mapping,
                slots.ingress_epoch(approval).unwrap(),
                approval_publication,
            )
            .unwrap();
        assert!(associations.associate_accepted_submission(first_id));
        assert!(associations.associate_accepted_submission(second_id));
        assert!(
            associations
                .displayed_for_mapping(increment, mapping)
                .is_none()
        );
        assert!(
            associations
                .displayed_for_mapping(approval, mapping)
                .is_none()
        );
        assert!(associations.reject_terminal_publication(first_id));
        assert!(
            associations
                .promote_completed_publication(second_id)
                .is_some()
        );
        assert!(
            associations
                .displayed_for_mapping(increment, mapping)
                .is_none()
        );
        assert!(
            associations
                .displayed_for_mapping(approval, mapping)
                .is_some()
        );

        let retry_id = associations
            .stage(
                increment_binding,
                mapping,
                slots.ingress_epoch(increment).unwrap(),
                increment_publication,
            )
            .unwrap();
        assert!(associations.associate_accepted_submission(retry_id));
        assert!(
            associations
                .promote_completed_publication(retry_id)
                .is_some()
        );
        assert!(
            associations
                .displayed_for_mapping(increment, mapping)
                .is_some()
        );
        assert!(
            associations
                .displayed_for_mapping(approval, mapping)
                .is_some()
        );

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
    fn typed_mount_settles_initial_app_effects_before_first_publication() {
        use runenui_core::NoHostProtocol;

        struct InitialApp;
        impl UiApp for InitialApp {
            type State = usize;
            type Action = ();
            type HostProtocol = NoHostProtocol;

            fn root(state: &Self::State) -> impl View<Self::Action> {
                text(format!("Initialized {state}"))
            }

            fn initial_effects(
                _: &Self::State,
            ) -> impl runenui_core::IntoEffects<Self::Action, Self::HostProtocol> {
                Effects::action(())
            }

            fn update(
                state: &mut Self::State,
                (): Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                *state += 1;
                Effects::none()
            }
        }

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<InitialApp>(
                UiTypedScreenId::new("ordinary.initial-effects"),
                0,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("typed app initialization is admitted");
        assert_eq!(
            slots.observe_app::<InitialApp, _>(slot, |runtime| *runtime.state()),
            Ok(1),
            "the typed app's initial effect must not be stranded without later host ingress"
        );
        assert_eq!(slots.ingress_epoch(slot), Some(1));
        assert!(slots.has_pending_redraw(slot).is_ok());
    }

    #[test]
    fn typed_mount_capacity_pause_is_resumable_without_new_engine_action() {
        use runenui_core::{IntoEffects, NoHostProtocol};

        struct InitialChainApp;
        impl UiApp for InitialChainApp {
            type State = usize;
            type Action = ();
            type HostProtocol = NoHostProtocol;

            fn root(state: &Self::State) -> impl View<Self::Action> {
                text(format!("Initial steps {state}"))
            }

            fn initial_effects(
                _: &Self::State,
            ) -> impl IntoEffects<Self::Action, Self::HostProtocol> {
                Effects::action(())
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
            .mount_app::<InitialChainApp>(
                UiTypedScreenId::new("ordinary.initial-chain"),
                0,
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("capacity-paused initial work keeps a mounted typed application");
        let partial = slots
            .observe_app::<InitialChainApp, _>(slot, |runtime| *runtime.state())
            .unwrap();
        assert!(partial > 0 && partial < 320);
        assert_eq!(slots.ingress_epoch(slot), Some(1));
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
            slots.observe_app::<InitialChainApp, _>(slot, |runtime| *runtime.state()),
            Ok(320)
        );
        assert!(slots.has_pending_redraw(slot).is_ok());
        assert_eq!(slots.ingress_epoch(slot), Some(1));
    }

    #[test]
    fn failed_typed_shutdown_retains_slot_until_real_retirement_is_observed() {
        use std::any::Any;

        use runenui_core::NoHostProtocol;
        use runenui_runtime::{InputArbitrationRecord, RuntimeStatus};

        struct EmptyApp;
        impl UiApp for EmptyApp {
            type State = ();
            type Action = ();
            type HostProtocol = NoHostProtocol;

            fn root(_: &Self::State) -> impl View<Self::Action> {
                text("shutdown")
            }

            fn update(
                _: &mut Self::State,
                (): Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                Effects::none()
            }
        }

        // Only the failure boundary is injected. Successful retirement below
        // uses the actual pinned RunenUI runtime, including its final ownership
        // snapshot and ordered scope-retirement records.
        struct FailOnceOnShutdown {
            runtime: AppRuntime<EmptyApp>,
            fail_next_shutdown: bool,
        }

        impl TypedRuntimeOps for FailOnceOnShutdown {
            fn set_native_wake_transport(&self, transport: NativeWakeTransport) {
                self.runtime.set_wake_transport(transport);
            }

            fn pump(
                &mut self,
                budget: PumpBudget,
            ) -> Result<InputPumpBatch, InputObservationError> {
                self.runtime.pump(budget)
            }

            fn input_ownership(&mut self) -> Result<InputOwnershipSnapshot, InputObservationError> {
                self.runtime.input_ownership()
            }

            fn submit_keyboard(
                &mut self,
                event: KeyboardEvent,
            ) -> Result<KeyboardSubmission, SubmitKeyboardError> {
                self.runtime.submit_keyboard(event)
            }

            fn submit_pointer(
                &mut self,
                event: PointerEvent,
            ) -> Result<PointerSubmission, SubmitPointerError> {
                self.runtime.submit_pointer(event)
            }

            fn submit_text(
                &mut self,
                event: CommittedTextEvent,
            ) -> Result<TextSubmission, SubmitTextError> {
                self.runtime.submit_text(event)
            }

            fn start_composition(
                &mut self,
                device: Option<InputDeviceId>,
            ) -> Result<CompositionStartSubmission, SubmitCompositionStartError> {
                self.runtime.start_composition(device)
            }

            fn update_composition(
                &mut self,
                generation: CompositionGeneration,
                preedit: String,
                range: Option<CompositionRange>,
            ) -> Result<CompositionSubmission, SubmitCompositionError> {
                self.runtime
                    .submit_composition_update(generation, preedit, range)
            }

            fn end_composition(
                &mut self,
                generation: CompositionGeneration,
            ) -> Result<CompositionSubmission, SubmitCompositionError> {
                self.runtime.submit_composition_end(generation)
            }

            fn cancel_composition(
                &mut self,
                generation: CompositionGeneration,
            ) -> Result<CompositionSubmission, SubmitCompositionError> {
                self.runtime.cancel_composition(generation)
            }

            fn pending_redraw(&mut self) -> bool {
                self.runtime.take_redraw_request().is_some()
            }

            fn publish_surface(
                &mut self,
                context: &SurfaceBuildContext<'_>,
            ) -> Result<SurfacePublication, runenui_runtime::PublishSurfaceError> {
                self.runtime.publish_surface(context)
            }

            fn shutdown(&mut self) -> Result<InputShutdownBatch, InputObservationError> {
                if std::mem::take(&mut self.fail_next_shutdown) {
                    Err(InputObservationError::Capacity)
                } else {
                    self.runtime.shutdown()
                }
            }

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }

        let id = UiRuntimeSlotId(1);
        let mut slots = UiRuntimeSlotsResource::default();
        slots.slots.push(UiRuntimeSlot {
            id,
            screen_id: UiTypedScreenId::new("ordinary.shutdown-preflight"),
            runtime: UiRuntimeSlotKind::Typed(Box::new(FailOnceOnShutdown {
                runtime: AppRuntime::<EmptyApp>::mount(()),
                fail_next_shutdown: true,
            })),
            ingress_epoch: 1,
            requires_settlement: false,
            next_deadline: None,
            native_clock: None,
            deadline_observed_at: None,
        });
        slots.next_slot_id = 2;

        assert!(matches!(
            slots.unmount_with_retirement(id),
            Err(UiRuntimeSlotOperationFailure::InputObservation(
                InputObservationError::Capacity
            ))
        ));
        assert!(
            slots.contains(id),
            "failed shutdown cannot destroy its slot"
        );
        assert_eq!(slots.ingress_epoch(id), Some(1));

        // A native shutdown failure retains the runtime for retirement retry,
        // but neither observations nor ingress may reopen its host authority.
        let UiRuntimeSlotKind::Typed(runtime) = &mut slots.slots[0].runtime else {
            unreachable!("typed shutdown fixture");
        };
        runtime
            .as_any_mut()
            .downcast_mut::<FailOnceOnShutdown>()
            .unwrap()
            .fail_next_shutdown = true;
        slots.native_clock = Some(NativeHostClock::new());
        assert!(slots.shutdown_native_slots().is_err());
        assert!(slots.contains(id));
        assert_eq!(
            slots.observe_app::<EmptyApp, _>(id, |_| ()),
            Err(UiRuntimeSlotOperationFailure::NativeHostClosed)
        );
        assert_eq!(
            slots.submit_app_action::<EmptyApp>(id, ()),
            Err(UiRuntimeSlotOperationFailure::NativeHostClosed)
        );
        assert_eq!(
            slots.settle_slot(id),
            Err(UiRuntimeSlotOperationFailure::NativeHostClosed)
        );

        let retirement = slots
            .unmount_with_retirement(id)
            .expect("retry succeeds")
            .expect("accepted shutdown returns actual retirement");
        assert_eq!(retirement.final_ownership().status(), RuntimeStatus::Closed);
        assert!(
            retirement
                .ordered_records()
                .iter()
                .any(|record| matches!(record, InputArbitrationRecord::ScopeRetired(_)))
        );
        assert!(!slots.contains(id));
        assert_eq!(
            slots
                .unmount_with_retirement(id)
                .map(|value| value.is_none()),
            Ok(true)
        );
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
            Some(UiRuntimeSlotMountFailure::SlotIdentityExhausted)
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

    #[test]
    fn publication_imports_real_motion_deadline_and_quiescent_due_observation_does_not_spin() {
        use runenui_core::{
            AnimationId, ExplicitTimeline, MotionEasing, MotionKeyframe, MotionRepeat, MotionValue,
            ReducedMotionStrategy, SceneOpacity, TimelineSpec, UnitInterval,
        };
        struct MotionApp;
        impl UiApp for MotionApp {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;
            fn root(_: &()) -> impl View<()> {
                text("motion").key("root").timeline(ExplicitTimeline::new(
                    AnimationId::from_static("native-delayed").unwrap(),
                    TimelineSpec::new(
                        vec![
                            MotionKeyframe::new(
                                UnitInterval::ZERO,
                                MotionValue::Opacity(SceneOpacity::TRANSPARENT),
                            ),
                            MotionKeyframe::new(
                                UnitInterval::ONE,
                                MotionValue::Opacity(SceneOpacity::OPAQUE),
                            ),
                        ],
                        vec![MotionEasing::Linear],
                        Duration::from_millis(100),
                        Duration::from_millis(50),
                        MotionRepeat::ONCE,
                        Some(ReducedMotionStrategy::PreserveEssential),
                    )
                    .unwrap(),
                ))
            }
            fn update(
                _: &mut (),
                _: (),
            ) -> impl runenui_core::IntoUpdateOutput<(), Self::HostProtocol> {
            }
        }
        let mut slots = UiRuntimeSlotsResource::default();
        slots
            .configure_host_clock(Some(NativeHostClock::new()))
            .unwrap();
        let id = slots
            .mount_app::<MotionApp>(
                UiTypedScreenId::new("native.motion"),
                (),
                &UiRuntimeFontConfiguration::new(vec![CONTROLLED_FONT_BYTES.to_vec()]),
            )
            .slot_id()
            .unwrap();
        assert!(slots.slot(id).unwrap().next_deadline.is_none());
        let style = StyleEnvironment::default();
        slots
            .publish_surface_for_slot(
                id,
                &SurfaceBuildContext::tight(
                    &style,
                    LogicalSize::new(320_u16.into(), 160_u16.into()),
                ),
            )
            .unwrap();
        assert!(
            slots.native_readiness().unwrap().1.is_some(),
            "publication-created motion must enter host scheduling"
        );
        std::thread::sleep(Duration::from_millis(60));
        slots.settle_native_slots().unwrap();
        assert!(
            slots.slot(id).unwrap().next_deadline.is_some(),
            "framework motion deadline is preserved"
        );
        assert_eq!(
            slots.native_readiness().unwrap(),
            (false, None),
            "already observed quiescent deadline cannot busy-loop without a bound publisher"
        );
    }

    #[test]
    fn native_type_ahead_deadline_is_imported_and_expires_without_application_timer() {
        use runenui_core::{
            CommandOrigin, Element, ElementId, FocusGroup, FocusGroupActivationPolicy,
            FocusGroupTypeAhead, KeyLocation, KeyModifiers, KeyboardCompositionState,
            KeyboardEvent, KeyboardPhase, LogicalKey, PhysicalKey, SemanticCommand, button, column,
        };
        struct SearchApp;
        impl UiApp for SearchApp {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;
            fn root(_: &()) -> impl View<()> {
                let member = |id: &'static str| -> Element<()> {
                    button(id)
                        .id(id)
                        .key(id)
                        .on_activate(|| ())
                        .into_element()
                        .focus_group_search_text(id)
                };
                column(vec![member("alpha"), member("beta")])
                    .key("group")
                    .into_element()
                    .focus_group(
                        FocusGroup::new()
                            .with_activation(FocusGroupActivationPolicy::Manual)
                            .with_type_ahead(
                                FocusGroupTypeAhead::new(Duration::from_millis(30)).unwrap(),
                            ),
                    )
            }
            fn update(
                _: &mut (),
                _: (),
            ) -> impl runenui_core::IntoUpdateOutput<(), Self::HostProtocol> {
            }
        }
        let mut slots = UiRuntimeSlotsResource::default();
        slots
            .configure_host_clock(Some(NativeHostClock::new()))
            .unwrap();
        let id = slots
            .mount_app::<SearchApp>(
                UiTypedScreenId::new("native.search"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        let runtime = slots
            .slot_mut(id)
            .unwrap()
            .runtime
            .typed_mut::<SearchApp>()
            .unwrap();
        let beta = ElementId::new("beta").unwrap();
        let target = runtime
            .index()
            .nodes()
            .iter()
            .find(|node| node.authored_id() == Some(&beta))
            .unwrap()
            .id()
            .clone();
        runtime
            .submit_command(
                target,
                SemanticCommand::RequestFocus,
                CommandOrigin::programmatic(),
            )
            .unwrap();
        slots.settle_slot(id).unwrap();
        slots
            .slot_mut(id)
            .unwrap()
            .runtime
            .typed_mut::<SearchApp>()
            .unwrap()
            .submit_keyboard(KeyboardEvent::new(
                KeyboardPhase::Down,
                PhysicalKey::Code("KeyA".into()),
                LogicalKey::Character("a".into()),
                KeyModifiers::NONE,
                false,
                KeyLocation::Standard,
                KeyboardCompositionState::Inactive,
                None,
            ))
            .unwrap();
        slots.settle_slot(id).unwrap();
        assert!(
            slots.native_readiness().unwrap().1.is_some(),
            "type-ahead timeout is native readiness"
        );
        std::thread::sleep(Duration::from_millis(40));
        slots.settle_native_slots().unwrap();
        assert!(
            slots.slot(id).unwrap().next_deadline.is_none(),
            "framework timeout actually retired"
        );
        assert_eq!(slots.native_readiness().unwrap(), (false, None));
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
