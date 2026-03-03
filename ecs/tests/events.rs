use ecs::{
    EventChannelConfig, EventLifetime, EventTracingPolicy, ObserverTrigger, OverflowPolicy, World,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct SceneUiEvent {
    action: &'static str,
}

#[test]
fn event_channels_auto_create_and_support_read_drain_clear() {
    let mut world = World::new();
    assert!(!world.has_event_channel::<SceneUiEvent>());

    world.emit_event(SceneUiEvent {
        action: "main_menu",
    });
    assert!(world.has_event_channel::<SceneUiEvent>());
    assert_eq!(world.event_count::<SceneUiEvent>(), 1);
    assert_eq!(
        world.read_events::<SceneUiEvent>(),
        &[SceneUiEvent {
            action: "main_menu"
        }]
    );

    let drained = world.drain_events::<SceneUiEvent>();
    assert_eq!(
        drained,
        vec![SceneUiEvent {
            action: "main_menu"
        }]
    );
    assert_eq!(world.event_count::<SceneUiEvent>(), 0);

    world.emit_event(SceneUiEvent { action: "settings" });
    world.emit_event(SceneUiEvent { action: "pause" });
    assert_eq!(world.clear_events::<SceneUiEvent>(), 2);
    assert_eq!(world.event_count::<SceneUiEvent>(), 0);
}

#[test]
fn overflow_drop_oldest_keeps_latest_events() {
    let mut world = World::new();
    world.configure_event_channel::<SceneUiEvent>(EventChannelConfig {
        capacity: Some(2),
        overflow: OverflowPolicy::DropOldest,
        lifetime: EventLifetime::Manual,
        tracing: EventTracingPolicy::Disabled,
    });

    world.emit_event(SceneUiEvent { action: "a" });
    world.emit_event(SceneUiEvent { action: "b" });
    world.emit_event(SceneUiEvent { action: "c" });

    assert_eq!(
        world.read_events::<SceneUiEvent>(),
        &[SceneUiEvent { action: "b" }, SceneUiEvent { action: "c" }]
    );

    let stats = world
        .event_channel_stats::<SceneUiEvent>()
        .expect("stats should exist");
    assert_eq!(stats.emitted, 3);
    assert_eq!(stats.dropped, 1);
    assert_eq!(stats.pending, 2);
}

#[test]
fn overflow_drop_oldest_still_notifies_on_emit_for_latest_event() {
    let mut world = World::new();
    world.configure_event_channel::<SceneUiEvent>(EventChannelConfig {
        capacity: Some(1),
        overflow: OverflowPolicy::DropOldest,
        lifetime: EventLifetime::Manual,
        tracing: EventTracingPolicy::Disabled,
    });
    world.observe_events::<SceneUiEvent>("emit_observer", ObserverTrigger::OnEmit);

    world.emit_event(SceneUiEvent { action: "a" });
    world.emit_event(SceneUiEvent { action: "b" });

    assert_eq!(world.event_observer_invocations("emit_observer"), Some(2));
    assert_eq!(
        world.read_events::<SceneUiEvent>(),
        &[SceneUiEvent { action: "b" }]
    );

    let notifications = world.drain_event_observer_notifications();
    assert_eq!(notifications.len(), 2);
    assert!(notifications.iter().all(|notification| {
        notification.trigger == ObserverTrigger::OnEmit && notification.event_count == 1
    }));
}

#[test]
fn overflow_drop_newest_preserves_existing_events() {
    let mut world = World::new();
    world.configure_event_channel::<SceneUiEvent>(EventChannelConfig {
        capacity: Some(2),
        overflow: OverflowPolicy::DropNewest,
        lifetime: EventLifetime::Manual,
        tracing: EventTracingPolicy::Disabled,
    });

    world.emit_event(SceneUiEvent { action: "a" });
    world.emit_event(SceneUiEvent { action: "b" });
    world.emit_event(SceneUiEvent { action: "c" });

    assert_eq!(
        world.read_events::<SceneUiEvent>(),
        &[SceneUiEvent { action: "a" }, SceneUiEvent { action: "b" }]
    );

    let stats = world
        .event_channel_stats::<SceneUiEvent>()
        .expect("stats should exist");
    assert_eq!(stats.emitted, 3);
    assert_eq!(stats.dropped, 1);
    assert_eq!(stats.pending, 2);
}

#[test]
fn observer_triggers_fire_for_emit_drain_and_end_of_frame() {
    let mut world = World::new();
    world.observe_events::<SceneUiEvent>("emit_observer", ObserverTrigger::OnEmit);
    world.observe_events::<SceneUiEvent>("drain_observer", ObserverTrigger::OnDrain);
    world.observe_events::<SceneUiEvent>("eof_observer", ObserverTrigger::EndOfFrame);

    world.emit_event(SceneUiEvent {
        action: "main_menu",
    });
    world.emit_event(SceneUiEvent { action: "settings" });

    assert_eq!(world.event_observer_invocations("emit_observer"), Some(2));
    let emit_notifications = world.drain_event_observer_notifications();
    assert_eq!(emit_notifications.len(), 2);
    assert!(
        emit_notifications
            .iter()
            .all(|notification| notification.trigger == ObserverTrigger::OnEmit)
    );

    let drained = world.drain_events::<SceneUiEvent>();
    assert_eq!(drained.len(), 2);
    assert_eq!(world.event_observer_invocations("drain_observer"), Some(1));
    let drain_notifications = world.drain_event_observer_notifications();
    assert_eq!(drain_notifications.len(), 1);
    assert_eq!(drain_notifications[0].trigger, ObserverTrigger::OnDrain);
    assert_eq!(drain_notifications[0].event_count, 2);

    world.emit_event(SceneUiEvent { action: "pause" });
    world.finish_event_frame();
    assert_eq!(world.event_observer_invocations("eof_observer"), Some(1));
    let eof_notifications = world.drain_event_observer_notifications();
    assert_eq!(eof_notifications.len(), 2);
    assert!(
        eof_notifications
            .iter()
            .any(|notification| notification.trigger == ObserverTrigger::OnEmit)
    );
    assert!(
        eof_notifications
            .iter()
            .any(|notification| notification.trigger == ObserverTrigger::EndOfFrame)
    );
}

#[test]
fn frame_transient_lifetime_clears_events_on_finish_frame() {
    let mut world = World::new();
    world.configure_event_channel::<SceneUiEvent>(EventChannelConfig {
        capacity: None,
        overflow: OverflowPolicy::DropOldest,
        lifetime: EventLifetime::FrameTransient,
        tracing: EventTracingPolicy::Disabled,
    });

    world.emit_event(SceneUiEvent { action: "tick" });
    assert_eq!(world.event_count::<SceneUiEvent>(), 1);

    world.finish_event_frame();
    assert_eq!(world.event_count::<SceneUiEvent>(), 0);

    let stats = world
        .event_channel_stats::<SceneUiEvent>()
        .expect("stats should exist");
    assert_eq!(stats.emitted, 1);
    assert_eq!(stats.drained, 1);
    assert_eq!(stats.pending, 0);
}

#[test]
fn drain_events_map_and_filter_helpers_work() {
    let mut world = World::new();
    world.emit_event(SceneUiEvent { action: "menu" });
    world.emit_event(SceneUiEvent { action: "settings" });
    world.emit_event(SceneUiEvent { action: "pause" });

    let long_actions = world.drain_events_filter::<SceneUiEvent, _>(|event| event.action.len() > 5);
    assert_eq!(long_actions.len(), 1);
    assert_eq!(long_actions[0].action, "settings");

    world.emit_event(SceneUiEvent { action: "resume" });
    world.emit_event(SceneUiEvent { action: "quit" });
    let labels =
        world.drain_events_map::<SceneUiEvent, String, _>(|event| format!("ui:{}", event.action));
    assert_eq!(labels, vec!["ui:resume".to_string(), "ui:quit".to_string()]);
}
