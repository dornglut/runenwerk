use ui_app_integration::{
    CounterHost, UiAppRouteResolutionDiagnostic, counter_packet, counter_payload_schema,
    counter_route_bridge,
};
use ui_program::{RouteCapability, RouteId, RouteSchemaVersion, UiEventPacket};
use ui_schema::{UiSchemaRef, UiSchemaValue};

#[test]
fn unknown_route_is_rejected_without_mutation() {
    let bridge = counter_route_bridge();
    let mut host = CounterHost::new(0);
    let before = host.snapshot();
    let resolution = bridge.resolve(&counter_packet("counter.unknown", "counter.action.increment"));

    assert!(!resolution.is_resolved());
    assert!(matches!(
        resolution.diagnostics.first(),
        Some(UiAppRouteResolutionDiagnostic::UnknownRoute { .. })
    ));
    assert_eq!(host.snapshot(), before);
}

#[test]
fn wrong_schema_version_is_rejected_without_mutation() {
    let bridge = counter_route_bridge();
    let host = CounterHost::new(0);
    let packet = UiEventPacket::new(
        RouteId::new("counter.increment"),
        RouteSchemaVersion::new(2),
        counter_payload_schema(),
        UiSchemaValue::null(),
    )
    .with_capability(RouteCapability::new("counter.action.increment"));

    let resolution = bridge.resolve(&packet);
    assert!(!resolution.is_resolved());
    assert!(matches!(
        resolution.diagnostics.first(),
        Some(UiAppRouteResolutionDiagnostic::WrongSchemaVersion { .. })
    ));
    assert_eq!(host.count(), 0);
}

#[test]
fn missing_capability_is_rejected_without_mutation() {
    let bridge = counter_route_bridge();
    let host = CounterHost::new(0);
    let packet = UiEventPacket::new(
        RouteId::new("counter.increment"),
        RouteSchemaVersion::new(1),
        counter_payload_schema(),
        UiSchemaValue::null(),
    );

    let resolution = bridge.resolve(&packet);
    assert!(!resolution.is_resolved());
    assert!(matches!(
        resolution.diagnostics.first(),
        Some(UiAppRouteResolutionDiagnostic::MissingCapability { .. })
    ));
    assert_eq!(host.count(), 0);
}

#[test]
fn invalid_payload_schema_is_rejected_without_mutation() {
    let bridge = counter_route_bridge();
    let host = CounterHost::new(0);
    let packet = UiEventPacket::new(
        RouteId::new("counter.increment"),
        RouteSchemaVersion::new(1),
        UiSchemaRef::new("counter.action.wrong_payload", 1),
        UiSchemaValue::null(),
    )
    .with_capability(RouteCapability::new("counter.action.increment"));

    let resolution = bridge.resolve(&packet);
    assert!(!resolution.is_resolved());
    assert!(matches!(
        resolution.diagnostics.first(),
        Some(UiAppRouteResolutionDiagnostic::PayloadSchemaMismatch { .. })
    ));
    assert_eq!(host.count(), 0);
}

#[test]
fn resolved_action_is_the_only_mutation_path() {
    let bridge = counter_route_bridge();
    let mut host = CounterHost::new(0);
    let bad = bridge.resolve(&counter_packet("counter.unknown", "counter.action.increment"));
    assert!(!bad.is_resolved());
    assert_eq!(host.count(), 0);

    let good = bridge.resolve(&counter_packet("counter.increment", "counter.action.increment"));
    assert!(good.is_resolved());
    let action = good.action.expect("resolved action is required");
    host.apply_resolved_action(&action.action_id)
        .expect("known counter action should mutate");
    assert_eq!(host.count(), 1);
}
