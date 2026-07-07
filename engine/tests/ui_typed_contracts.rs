use std::collections::BTreeMap;

use engine::plugins::ui::{
    IntoUi, UiAction, UiActionHandler, UiHostMutationIntent, UiRuntimeDiagnostic,
    UiRuntimeDiagnosticCode, UiScreen, UiTypedActionDescriptor, UiTypedActionId,
    UiTypedContractFailureReason, UiTypedContractKind, UiTypedIdentityError, UiTypedScreenId,
    UiTypedSource,
};
use ui_app_integration::{UiAppActionId, UiAppSourceBuilder, counter_payload_schema};
use ui_controls::{BUTTON_CONTROL_KIND_ID, ControlPackageRegistry, runenwerk_control_package};
use ui_definition::{
    AuthoredControlAccessibilityDefinition, AuthoredControlKindId, AuthoredControlValue,
    AuthoredId, AuthoredRouteId, UiNodeDefinition, UiValueBinding,
};
use ui_hosts::{DomainCommand, HostCommand, HostKind, HostRouteMapVersion};
use ui_program::{RouteCapability, RouteId, RouteSchemaVersion, UiProgramSourceId};
use ui_schema::UiSchemaRef;

#[test]
fn ui_typed_screen_lowers_to_definition_source_and_program_facts() {
    let screen = CounterScreen { count: 2 };
    let screen_id = screen.screen_id();
    let source = screen.into_ui_source();

    assert_eq!(source.screen_id(), &screen_id);
    assert_eq!(source.source_id().as_str(), "counter.screen.source");

    let registry = ControlPackageRegistry::new()
        .with_package(runenwerk_control_package())
        .expect("runenwerk controls package should register");

    let report = source.lower_with_registry_snapshot(&registry.snapshot());

    assert!(report.passed(), "{:?}", report.formation().diagnostics);
    assert_eq!(report.screen_id().as_str(), "counter.screen");
    assert_eq!(report.source_id().as_str(), "counter.screen.source");
    assert_eq!(report.program().id.as_str(), "counter.screen.program");
    assert!(report.sources().iter().any(|source_record| {
        source_record.source_id.as_str() == "counter.screen.source"
            && source_record.description == "authored UI definition"
    }));
    assert!(
        report
            .source_map_entries()
            .iter()
            .any(|entry| entry.source_id == "counter.screen.source")
    );
    assert!(report.has_route(&RouteId::new("counter.increment")));
    assert_eq!(
        report.route_ids()[0].as_str(),
        RouteId::new("counter.increment").as_str()
    );
}

#[test]
fn ui_typed_action_handler_emits_host_owned_intent_without_mutation() {
    let action = CounterIncrementAction;
    let handler = CounterIncrementHandler;

    let intent = handler.host_intent(&action);

    assert_eq!(intent.action().action_id().as_str(), "counter.increment");
    assert_eq!(intent.action().route().as_str(), "counter.increment");
    assert_eq!(
        intent.required_capabilities()[0].as_str(),
        "counter.action.increment"
    );
    assert_eq!(intent.host_command().host, HostKind::Headless);
    assert_eq!(intent.host_command().command_id, "counter.increment");
    assert_eq!(
        intent
            .domain_command()
            .expect("domain command should be recorded")
            .command_id,
        "increment"
    );

    let mapping = intent.to_host_route_mapping(HostRouteMapVersion::new(1));

    assert_eq!(mapping.route_id.as_str(), "counter.increment");
    assert_eq!(mapping.schema_version.value(), 1);
    assert_eq!(mapping.host_command.host, HostKind::Headless);
    assert_eq!(
        mapping.required_capabilities[0].as_str(),
        "counter.action.increment"
    );
    assert_eq!(
        mapping
            .domain_command
            .expect("domain command should be mapped")
            .domain_id,
        "counter"
    );
}

#[test]
fn ui_typed_action_identity_matches_ui_app_integration_proof_evidence() {
    let typed = CounterIncrementAction.action_descriptor();
    let proof_source = UiAppSourceBuilder::counter_screen(0);
    let proof_action_id = UiAppActionId::new("counter.increment");
    let proof_payload_schema = counter_payload_schema();

    assert_eq!(typed.action_id().as_str(), proof_action_id.as_str());
    assert_eq!(
        typed.action_id().as_str(),
        proof_source.routes[0].route.as_str()
    );
    assert_eq!(
        typed.route().as_str(),
        proof_source.routes[0].route.as_str()
    );
    assert_eq!(typed.schema_version(), RouteSchemaVersion::new(1));
    assert_eq!(typed.payload_schema(), &proof_payload_schema);
    assert_eq!(typed.capability().as_str(), "counter.action.increment");
}

#[test]
fn ui_typed_contract_diagnostics_report_stable_identity_failure() {
    let failure =
        UiTypedActionId::try_new("counter.increment action").expect_err("space is invalid");
    let diagnostic = UiRuntimeDiagnostic::typed_contract_rejected(
        UiTypedContractKind::Action,
        "counter.increment action",
        UiTypedContractFailureReason::InvalidIdentity(failure.clone()),
    );

    assert_eq!(
        diagnostic.code,
        UiRuntimeDiagnosticCode::TypedContractRejected
    );
    assert_eq!(diagnostic.message, "typed UI contract identity is invalid");

    let typed = diagnostic
        .typed_contract
        .expect("typed contract diagnostic should be recorded");
    assert_eq!(typed.contract, UiTypedContractKind::Action);
    assert_eq!(typed.identity, "counter.increment action");
    assert_eq!(
        typed.failure_reason,
        UiTypedContractFailureReason::InvalidIdentity(UiTypedIdentityError::InvalidIdCharacter {
            kind: "action",
            value: "counter.increment action".to_owned(),
        })
    );
}

struct CounterScreen {
    count: u32,
}

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
                        id: AuthoredId::new("counter.count_label"),
                        label: UiValueBinding::static_text(format!("Clicked {} / 5", self.count)),
                        availability: None,
                    },
                    button_control("counter.increment_button", "Click me", "counter.increment"),
                ],
            },
        )
    }
}

struct CounterIncrementAction;

impl UiAction for CounterIncrementAction {
    fn action_descriptor(&self) -> UiTypedActionDescriptor {
        UiTypedActionDescriptor::new(
            UiTypedActionId::new("counter.increment"),
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

fn button_control(id: &str, label: &str, route: &str) -> UiNodeDefinition {
    let mut properties = BTreeMap::new();
    properties.insert(
        "label".to_owned(),
        AuthoredControlValue::String(label.to_owned()),
    );

    UiNodeDefinition::Control {
        id: AuthoredId::new(id),
        kind: AuthoredControlKindId::new(BUTTON_CONTROL_KIND_ID),
        properties,
        bindings: BTreeMap::new(),
        route: Some(AuthoredRouteId::new(route)),
        accessibility: Some(AuthoredControlAccessibilityDefinition {
            role: "button".to_owned(),
            label: Some(label.to_owned()),
        }),
        children: Vec::new(),
    }
}
