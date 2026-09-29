use runenui_core::{
    Effects, Element, ElementId, HostProtocol, SemanticCommand, UiApp, View, button, column, text,
};
#[cfg(test)]
use runenui_core::{
    FontFamilyName, GenericFontFamily, LogicalLength, LogicalRect, PaintPrimitive, StyleEnvironment,
};
use runenui_runtime::{AppRuntime, HostRequestToken, PumpBudget};
#[cfg(test)]
use runenui_runtime::{LogicalSize, SurfaceBuildContext};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiRuntimeSlotMountFailure {
    ControlRegistry,
    ProgramFormation,
    UnsupportedProjection,
}

impl UiRuntimeSlotMountFailure {
    pub const fn message(self) -> &'static str {
        match self {
            Self::ControlRegistry => "RunenUI slot mount could not form the control registry",
            Self::ProgramFormation => "RunenUI slot mount could not form a valid UiProgram",
            Self::UnsupportedProjection => "RunenUI slot mount rejected an unsupported projection",
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
    PublicationRejected,
    InvalidAuthoredId,
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

struct UiRuntimeSlot {
    id: UiRuntimeSlotId,
    screen_id: UiTypedScreenId,
    runtime: AppRuntime<RunenwerkProgramUiApp>,
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

        let runtime = AppRuntime::<RunenwerkProgramUiApp>::mount(RunenwerkProgramUiState {
            source,
            artifact,
            resolved_state,
            registry: registry.clone(),
        });

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
            runtime,
        });
        UiRuntimeSlotMountReport::accepted(screen_id, slot_id)
    }

    #[must_use]
    pub fn contains(&self, slot_id: UiRuntimeSlotId) -> bool {
        self.slots.iter().any(|slot| slot.id == slot_id)
    }

    pub fn slot_ids(&self) -> impl Iterator<Item = UiRuntimeSlotId> + '_ {
        self.slots.iter().map(|slot| slot.id)
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
        let artifact = slot.runtime.state().artifact.clone();
        let mut resolved_state = slot.runtime.state().resolved_state.clone();
        let output = UiEvaluator.evaluate_with_context(&artifact, &mut resolved_state, context);
        if !output.diagnostics.is_empty() {
            return Err(UiRuntimeSlotOperationFailure::EvaluationDiagnostics);
        }
        validate_projection(
            &slot.runtime.state().source,
            &artifact,
            &resolved_state,
            &slot.runtime.state().registry,
        )
        .map_err(|_| UiRuntimeSlotOperationFailure::UnsupportedProjection)?;
        slot.runtime
            .submit_action(RunenwerkProgramUiAction::ReplaceResolvedState(
                resolved_state,
            ))
            .map_err(|_| UiRuntimeSlotOperationFailure::ActionRejected)?;
        settle(&mut slot.runtime)?;
        Ok(())
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
        slot.runtime
            .submit_automation_command(authored_id, SemanticCommand::Activate)
            .map_err(|_| UiRuntimeSlotOperationFailure::AutomationRejected)?;
        settle(&mut slot.runtime)?;
        Ok(())
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
        slot.runtime
            .complete_host_request(&request.token, response)
            .map_err(|_| UiRuntimeSlotOperationFailure::HostResponseRejected)?;
        settle(&mut slot.runtime)
    }

    #[cfg(test)]
    fn resolved_state_value(
        &self,
        slot_id: UiRuntimeSlotId,
        requirement_id: &str,
    ) -> Option<&UiSchemaValue> {
        self.slot(slot_id)?
            .runtime
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
        register_controlled_test_font(&mut slot.runtime)?;
        publication_facts_for_runtime(&mut slot.runtime)
    }

    pub fn unmount(&mut self, slot_id: UiRuntimeSlotId) -> bool {
        let Some(index) = self.slots.iter().position(|slot| slot.id == slot_id) else {
            return false;
        };
        let mut slot = self.slots.remove(index);
        let _ = slot.runtime.shutdown();
        true
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

fn settle(
    runtime: &mut AppRuntime<RunenwerkProgramUiApp>,
) -> Result<(), UiRuntimeSlotOperationFailure> {
    let report = runtime.pump(PumpBudget::new(
        RUNTIME_PUMP_WORK_LIMIT,
        RUNTIME_PUMP_WORK_LIMIT,
        RUNTIME_PUMP_WORK_LIMIT,
        RUNTIME_PUMP_WORK_LIMIT,
    ));
    if report.is_quiescent() {
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
            publication_facts_for_runtime(&mut slot.runtime)
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
