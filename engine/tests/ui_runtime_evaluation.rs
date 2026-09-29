use std::collections::BTreeMap;

use engine::plugins::ui::{
    UiAction, UiActionDispatchOutputs, UiActionDispatchReportsResource, UiActionEvent,
    UiActionHandler, UiHostActionExecutor, UiHostMutationIntent, UiHostMutationReceipt,
    UiHostMutationRejection, UiPlugin, UiRuntimeDiagnosticsResource, UiRuntimeSlotsResource,
    UiRuntimeTraceResource, UiScreen, UiTypedActionDescriptor, UiTypedActionId, UiTypedScreenId,
    UiTypedSource, dispatch_ui_action,
};
use engine::prelude::{App, AppUiExt};
use ui_binding::HostDataSnapshot;
use ui_controls::BUTTON_CONTROL_KIND_ID;
use ui_definition::{
    AuthoredBindingRef, AuthoredControlAccessibilityDefinition, AuthoredControlKindId,
    AuthoredControlValue, AuthoredId, AuthoredRouteId, UiNodeDefinition, UiValueBinding,
};
use ui_evaluator::UiEvaluationContext;
use ui_hosts::{
    DomainCommand, HeadlessHost, HostCommand, HostKind, HostRouteMapVersion, HostRouteMapping,
};
use ui_program::{RouteCapability, RouteId, RouteSchemaVersion, UiProgramSourceId};
use ui_schema::{UiSchemaRef, UiSchemaValue};

#[test]
fn ui_runtime_button_activation_egresses_as_runenwerk_route_intent() {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);
    let report = app.ui().mount(CounterScreen);
    let slot_id = report.slot_id().expect("CounterScreen should mount");

    let slots = app
        .world_mut()
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("UiPlugin should install RunenUI runtime slots");
    slots
        .update_state(slot_id, counter_context(false, 1))
        .expect("resolved Runenwerk state should update the runtime");
    slots
        .activate_authored(slot_id, "counter.output")
        .expect("RunenUI semantic activation should be admitted");

    let mut requests = slots
        .pending_event_requests(slot_id)
        .expect("Engine integration should observe the pending RunenUI host request");
    assert_eq!(requests.len(), 1);
    let request = requests.pop().expect("one pending CounterScreen request");
    let packet = request.packet().clone();
    assert_eq!(packet.route, RouteId::new("counter.increment"));
    assert_eq!(packet.schema_version, RouteSchemaVersion::new(1));
    assert_eq!(
        packet.payload_schema(),
        &UiSchemaRef::new("runenwerk.ui.controls.button.event", 1)
    );
    assert!(packet.requires_capability(&RouteCapability::new("counter.action.increment")));
    assert_eq!(
        packet.source_control.as_ref().map(|id| id.as_str()),
        Some("control.counter.output")
    );
    assert_eq!(
        packet.payload.value,
        UiSchemaValue::object([
            ("route", UiSchemaValue::route_ref("counter.increment")),
            ("activated", UiSchemaValue::bool(true)),
        ])
    );
    let action = CounterIncrementAction;
    let handler = CounterIncrementHandler;
    let intent = handler.host_intent(&action);
    assert_eq!(intent.action().route(), &packet.route);
    assert_eq!(
        intent.required_capabilities()[0].as_str(),
        "counter.action.increment"
    );
    assert_eq!(
        intent
            .domain_command()
            .expect("Runenwerk handler should retain domain authorization intent")
            .command_id,
        "increment"
    );

    let route_map_version = HostRouteMapVersion::new(1);
    let host = HeadlessHost::new(route_map_version)
        .with_mapping(intent.to_host_route_mapping(route_map_version));
    let mut executor = CounterHostExecutor::default();
    let mut reports = UiActionDispatchReportsResource::default();
    let mut trace = UiRuntimeTraceResource::default();
    let mut diagnostics = UiRuntimeDiagnosticsResource::default();
    let dispatch = dispatch_ui_action(
        &action,
        &handler,
        &UiActionEvent::new(packet.clone()),
        &host,
        &mut executor,
        UiActionDispatchOutputs::new(&mut reports, &mut trace, &mut diagnostics),
    );

    assert!(dispatch.is_accepted(), "{dispatch:?}");
    assert_eq!(executor.count, 1);
    assert!(diagnostics.is_empty());
    assert_eq!(
        dispatch
            .domain_command()
            .expect("accepted dispatch should preserve domain mutation intent")
            .command_id,
        "increment"
    );

    slots
        .complete_event_request(
            slot_id,
            request,
            if dispatch.is_accepted() {
                engine::plugins::ui::UiRuntimeHostRequestDisposition::Accepted
            } else {
                engine::plugins::ui::UiRuntimeHostRequestDisposition::Rejected
            },
        )
        .expect("RunenUI host request should complete only after Runenwerk authorization");
    assert!(
        slots
            .pending_event_requests(slot_id)
            .expect("completed requests should remain drained")
            .is_empty()
    );

    assert!(slots.unmount(slot_id));
    assert!(!slots.contains(slot_id));
}

fn counter_context(selected: bool, revision: u64) -> UiEvaluationContext {
    UiEvaluationContext::default().with_host_data(HostDataSnapshot::new(
        "counter.output.selected",
        UiSchemaValue::bool(selected),
        revision,
    ))
}

#[derive(Debug, Copy, Clone)]
struct CounterScreen;

impl UiScreen for CounterScreen {
    fn screen_id(&self) -> UiTypedScreenId {
        UiTypedScreenId::new("counter.screen")
    }

    fn build_source(&self) -> UiTypedSource {
        UiTypedSource::new(
            self.screen_id(),
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
        .with_action_descriptor(CounterIncrementAction.action_descriptor())
    }
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

struct CounterIncrementAction;

impl UiAction for CounterIncrementAction {
    fn action_descriptor(&self) -> UiTypedActionDescriptor {
        UiTypedActionDescriptor::new(
            UiTypedActionId::new("counter.increment.action"),
            RouteId::new("counter.increment"),
            RouteSchemaVersion::new(1),
            UiSchemaRef::new("runenwerk.ui.controls.button.event", 1),
            RouteCapability::new("counter.action.increment"),
        )
    }
}

struct CounterIncrementHandler;

impl UiActionHandler<CounterIncrementAction> for CounterIncrementHandler {
    fn host_intent(&self, action: &CounterIncrementAction) -> UiHostMutationIntent {
        UiHostMutationIntent::new(
            action.action_descriptor(),
            HostCommand::new(HostKind::Headless, "counter.increment"),
        )
        .with_domain_command(DomainCommand::new("counter", "increment"))
    }
}

#[derive(Default)]
struct CounterHostExecutor {
    count: u32,
}

impl UiHostActionExecutor for CounterHostExecutor {
    fn apply(
        &mut self,
        intent: &UiHostMutationIntent,
        _packet: &ui_program::UiEventPacket,
        _mapping: &HostRouteMapping,
    ) -> Result<UiHostMutationReceipt, UiHostMutationRejection> {
        self.count += 1;
        Ok(UiHostMutationReceipt::from_intent(intent))
    }
}
