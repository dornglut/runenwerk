use super::tests::runner_with_frame_pacing;
use super::*;

struct NativeCursorFixture {
    runner: WinitRunner,
    window: NativeWindowId,
    slot: UiRuntimeSlotId,
    source: InputContext,
    backend_device: winit::event::DeviceId,
    displayed_context: runenui_core::SurfaceInputContext,
    button_position: PhysicalPosition<f64>,
    binding: crate::plugins::ui::UiRuntimePresentationBinding,
    mapping: crate::plugins::ui::UiRuntimeNativeMapping,
}

struct NativeCursorProbe;

impl runenui_core::UiApp for NativeCursorProbe {
    type State = usize;
    type Action = ();
    type HostProtocol = runenui_core::NoHostProtocol;

    fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
        runenui_core::button("activation probe").on_activate(|| ())
    }

    fn update(
        state: &mut Self::State,
        _: Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        *state += 1;
    }
}

fn native_cursor_fixture() -> NativeCursorFixture {
    use crate::plugins::render::RenderFrameProducerId;
    use crate::plugins::render::host::{RenderSurfaceId, RenderSurfaceRegistryResource};
    use crate::plugins::ui::{
        NativeInputRecipient, UiRuntimeFontConfiguration, UiRuntimeNativeMapping,
        UiRuntimePresentationAssociationsResource, UiRuntimePresentationBinding, UiTypedScreenId,
    };
    use runenui_core::{LogicalLength, StyleEnvironment};
    use runenui_runtime::{LogicalSize, SurfaceBuildContext};

    let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
    runner.state.world.insert_resource(InputState::new());
    let window = NativeWindowId::primary();
    let backend_device = winit::event::DeviceId::dummy();
    let source = runner
        .native_input_mut()
        .window_context(window, backend_device)
        .expect("original native window source");
    let mut slots = UiRuntimeSlotsResource::default();
    let slot = slots
        .mount_app::<NativeCursorProbe>(
            UiTypedScreenId::new("native.cursor.stale-display"),
            0,
            &UiRuntimeFontConfiguration::default(),
        )
        .slot_id()
        .expect("real pinned RunenUI slot");
    let mut recipients = UiNativeInputRecipientsResource::default();
    recipients
        .bind(
            window,
            NativeInputRecipient::RunenUiSlot(slot),
            runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .expect("native window owner"),
            &mut slots,
        )
        .expect("one explicit native input target");

    let render_surface = RenderSurfaceId::primary();
    let producer = RenderFrameProducerId::try_from_raw(971).expect("valid UI producer identity");
    let style = StyleEnvironment::default();
    let binding = UiRuntimePresentationBinding::new(slot, producer, render_surface, style.clone());
    let mapping = UiRuntimeNativeMapping::new(window, render_surface, (1280, 720), 1.0)
        .expect("valid attached native mapping");
    let mut render_surfaces = RenderSurfaceRegistryResource::default();
    render_surfaces
        .confirm_surface_attachment(render_surface, window, (1280, 720))
        .expect("real attached primary Render surface");
    let mut presentations = UiRuntimePresentationAssociationsResource::default();
    presentations
        .bind(binding.clone())
        .expect("exact slot/producer/surface binding");
    assert!(presentations.update_mapping(slot, mapping));

    let build = SurfaceBuildContext::tight(
        &style,
        LogicalSize::new(LogicalLength::from(1280_u16), LogicalLength::from(720_u16)),
    );
    let publication = crate::plugins::ui::native_transaction::test_selected_slot_operation(
        &mut slots,
        slot,
        |slots| slots.publish_surface_for_slot(slot, &build),
    )
    .expect("real RunenUI surface publication");
    let displayed_context = publication.input_context().clone();
    let button_bounds = publication
        .frame()
        .nodes()
        .first()
        .expect("activation probe paint bounds")
        .bounds();
    let button_position = PhysicalPosition::new(
        f64::from(button_bounds.x() + 1.0),
        f64::from(button_bounds.y() + 1.0),
    );
    let publication_id = presentations
        .stage(
            binding.clone(),
            mapping,
            slots.ingress_epoch(slot).expect("live ingress epoch"),
            publication,
        )
        .expect("aligned publication staged");
    assert!(presentations.associate_accepted_submission(publication_id));
    assert!(
        presentations
            .promote_completed_publication(publication_id)
            .is_some()
    );

    runner.state.world.insert_resource(slots);
    runner.state.world.insert_resource(recipients);
    runner.state.world.insert_resource(presentations);
    runner.state.world.insert_resource(render_surfaces);
    NativeCursorFixture {
        runner,
        window,
        slot,
        source,
        backend_device,
        displayed_context,
        button_position,
        binding,
        mapping,
    }
}

fn retire_native_cursor_context(fixture: &mut NativeCursorFixture) {
    use runenui_core::{LogicalLength, StyleEnvironment};
    use runenui_runtime::{LogicalSize, RuntimeConfig, SurfaceBuildContext};

    let slots = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same mounted slot");
    let retention = RuntimeConfig::default().surface_snapshot_retention();
    let style = StyleEnvironment::default();
    let build = SurfaceBuildContext::tight(
        &style,
        LogicalSize::new(LogicalLength::from(1280_u16), LogicalLength::from(720_u16)),
    );
    for _ in 0..retention {
        crate::plugins::ui::native_transaction::test_selected_slot_operation(
            slots,
            fixture.slot,
            |slots| slots.publish_surface_for_slot(fixture.slot, &build),
        )
        .expect("later publication retires the displayed context at configured retention");
    }
}

fn promote_fresh_native_cursor_display(fixture: &mut NativeCursorFixture) {
    use runenui_core::{LogicalLength, StyleEnvironment};
    use runenui_runtime::{LogicalSize, SurfaceBuildContext};

    let style = StyleEnvironment::default();
    let build = SurfaceBuildContext::tight(
        &style,
        LogicalSize::new(LogicalLength::from(1280_u16), LogicalLength::from(720_u16)),
    );
    let publication = crate::plugins::ui::native_transaction::test_selected_slot_operation(
        fixture
            .runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("same mounted slot"),
        fixture.slot,
        |slots| slots.publish_surface_for_slot(fixture.slot, &build),
    )
    .expect("fresh aligned RunenUI publication");
    let ingress_epoch = fixture
        .runner
        .state
        .world
        .resource::<UiRuntimeSlotsResource>()
        .expect("mounted slot")
        .ingress_epoch(fixture.slot)
        .expect("current ingress epoch");
    let presentations = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimePresentationAssociationsResource>()
        .expect("existing presentation association owner");
    let publication_id = presentations
        .stage(
            fixture.binding.clone(),
            fixture.mapping,
            ingress_epoch,
            publication,
        )
        .expect("fresh aligned publication staged");
    assert!(presentations.associate_accepted_submission(publication_id));
    assert!(
        presentations
            .promote_completed_publication(publication_id)
            .is_some()
    );
}

fn observe_neutral_cursor(fixture: &mut NativeCursorFixture, x: f64, y: f64) {
    fixture
        .runner
        .apply_semantic_native_input(
            fixture.window,
            PlatformEvent::CursorMoved {
                context: fixture.source,
                position: runen_input::Point2::new(
                    x as f32,
                    y as f32,
                    runen_input::CoordinateSpace::WindowPhysicalPixels,
                ),
            },
        )
        .expect("original physical cursor observation remains neutral truth");
}

fn native_binding_epoch(fixture: &mut NativeCursorFixture) -> u64 {
    fixture
        .runner
        .native_input_recipient(fixture.window)
        .expect("same selected native recipient")
        .1
        .expect("selected binding epoch")
}

fn route_cursor(fixture: &mut NativeCursorFixture, position: PhysicalPosition<f64>) {
    observe_neutral_cursor(fixture, position.x, position.y);
    let binding_epoch = native_binding_epoch(fixture);
    fixture
        .runner
        .route_ui_cursor_moved(
            fixture.window,
            fixture.slot,
            binding_epoch,
            fixture.backend_device,
            fixture.source,
            position,
        )
        .expect("original cursor observation is routed or exactly withheld");
}

fn route_button(
    fixture: &mut NativeCursorFixture,
    state: winit::event::ElementState,
    button: winit::event::MouseButton,
) {
    let binding_epoch = native_binding_epoch(fixture);
    fixture
        .runner
        .route_ui_mouse_button(
            fixture.window,
            fixture.slot,
            binding_epoch,
            fixture.source,
            state,
            button,
        )
        .expect("original button observation is routed or exactly withheld");
}

fn admit_and_route_button(
    fixture: &mut NativeCursorFixture,
    state: winit::event::ElementState,
    button: winit::event::MouseButton,
) {
    let (context, disposition) = fixture
        .runner
        .admit_native_mouse_button(fixture.window, fixture.backend_device, state, button)
        .expect("original native mouse callback admission");
    if disposition == NativeMouseButtonDisposition::Original {
        let epoch = native_binding_epoch(fixture);
        fixture
            .runner
            .route_ui_mouse_button(fixture.window, fixture.slot, epoch, context, state, button)
            .expect("admitted native mouse button reaches the pinned translator");
    }
}

fn pointer_ownership(fixture: &mut NativeCursorFixture) -> runenui_runtime::InputOwnershipSnapshot {
    fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("exact M13 ownership snapshot")
}

fn activation_count(fixture: &NativeCursorFixture) -> usize {
    fixture
        .runner
        .state
        .world
        .resource::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .observe_app::<NativeCursorProbe, _>(fixture.slot, |app| *app.state())
        .expect("observe actual activation probe state")
}

#[test]
fn close_intent_cancels_pinned_mouse_lifetime_before_vetoable_close() {
    let mut fixture = native_cursor_fixture();
    let position = fixture.button_position;
    route_cursor(&mut fixture, position);
    admit_and_route_button(
        &mut fixture,
        winit::event::ElementState::Pressed,
        winit::event::MouseButton::Left,
    );
    assert_eq!(activation_count(&fixture), 0);
    assert!(!pointer_ownership(&mut fixture).pointers().is_empty());

    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("OS close intent remains vetoable");
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("a repeated OS close remains an original pending occurrence");

    assert!(pointer_ownership(&mut fixture).pointers().is_empty());
    assert!(
        fixture.runner.native_ui_mice.contains_key(&fixture.window),
        "the same translator must survive cancellation to suppress the later physical Up"
    );
    fixture
        .runner
        .apply_pending_close_window_event(
            fixture.window,
            WindowEvent::MouseInput {
                device_id: fixture.backend_device,
                state: winit::event::ElementState::Released,
                button: winit::event::MouseButton::Left,
            },
        )
        .expect("pending physical Up is neutral-only and nonfatal");
    assert!(pointer_ownership(&mut fixture).pointers().is_empty());

    fixture
        .runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .expect("window state owner")
        .record_mut(fixture.window)
        .expect("same native window")
        .veto_close();
    route_cursor(&mut fixture, position);
    admit_and_route_button(
        &mut fixture,
        winit::event::ElementState::Pressed,
        winit::event::MouseButton::Left,
    );
    admit_and_route_button(
        &mut fixture,
        winit::event::ElementState::Released,
        winit::event::MouseButton::Left,
    );
    assert_eq!(
        activation_count(&fixture),
        1,
        "a fresh post-veto click activates"
    );
}

#[test]
fn rejected_close_preflight_does_not_cancel_pinned_pointer_or_mutate_window() {
    let mut fixture = native_cursor_fixture();
    let position = fixture.button_position;
    route_cursor(&mut fixture, position);
    admit_and_route_button(
        &mut fixture,
        winit::event::ElementState::Pressed,
        winit::event::MouseButton::Left,
    );
    let journal = fixture
        .runner
        .state
        .world
        .resource_mut::<PlatformWindowEventQueueResource>()
        .expect("native event journal");
    while journal.events().len() < 4096 {
        journal
            .publish(PlatformWindowEvent::new(
                fixture.window,
                PlatformEvent::RedrawRequested,
            ))
            .expect("fill the bounded native event journal");
    }
    let before_lifecycle = fixture
        .runner
        .state
        .world
        .resource::<WindowStateRegistryResource>()
        .expect("window state owner")
        .record(fixture.window)
        .expect("created window")
        .lifecycle_state;

    assert!(
        fixture
            .runner
            .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
            .is_err()
    );

    assert!(!pointer_ownership(&mut fixture).pointers().is_empty());
    assert_eq!(
        fixture
            .runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .expect("window state owner")
            .record(fixture.window)
            .expect("same window")
            .lifecycle_state,
        before_lifecycle
    );
}

#[test]
fn pending_mouse_down_stays_suppressed_through_veto_and_duplicate_press() {
    let mut fixture = native_cursor_fixture();
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close intent begins");
    fixture
        .runner
        .apply_pending_close_window_event(
            fixture.window,
            WindowEvent::MouseInput {
                device_id: fixture.backend_device,
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
            },
        )
        .expect("pending Down remains neutral and tracked by the pinned adapter");
    fixture
        .runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .expect("window state owner")
        .record_mut(fixture.window)
        .expect("same native window")
        .veto_close();

    let position = fixture.button_position;
    route_cursor(&mut fixture, position);
    let (_, duplicate) = fixture
        .runner
        .admit_native_mouse_button(
            fixture.window,
            fixture.backend_device,
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        )
        .expect("duplicate Down remains admitted to neutral state");
    assert_eq!(duplicate, NativeMouseButtonDisposition::DuplicateDown);
    admit_and_route_button(
        &mut fixture,
        winit::event::ElementState::Released,
        winit::event::MouseButton::Left,
    );
    assert_eq!(activation_count(&fixture), 0);
    admit_and_route_button(
        &mut fixture,
        winit::event::ElementState::Pressed,
        winit::event::MouseButton::Left,
    );
    admit_and_route_button(
        &mut fixture,
        winit::event::ElementState::Released,
        winit::event::MouseButton::Left,
    );
    assert_eq!(activation_count(&fixture), 1);
}

#[test]
fn pending_button_from_new_input_context_cannot_be_recreated_as_a_fresh_ui_down() {
    use runen_input::{InputContext, InputDeviceId, InputSourceId};

    let mut fixture = native_cursor_fixture();
    let position = fixture.button_position;
    route_cursor(&mut fixture, position);
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close cancels the original cursor lifetime");
    // Exercise the normalized source/device seam because winit::DeviceId has no
    // public constructor for a second backend identity in a headless test.
    let context_b = InputContext::new(InputSourceId::new(9001), Some(InputDeviceId::new(9002)));
    fixture
        .runner
        .apply_pending_close_mouse_button(
            fixture.window,
            context_b,
            fixture.backend_device,
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        )
        .expect("pending B Down remains neutral physical evidence");
    fixture
        .runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .expect("window state owner")
        .record_mut(fixture.window)
        .expect("same native window")
        .veto_close();

    let neutral_position = runen_input::Point2::new(
        position.x as f32,
        position.y as f32,
        runen_input::CoordinateSpace::WindowPhysicalPixels,
    );
    fixture
        .runner
        .apply_event_group_for_recipient(
            fixture.window,
            [PlatformEvent::CursorMoved {
                context: context_b,
                position: neutral_position,
            }],
            None,
        )
        .expect("B cursor is admitted to neutral state");
    let epoch = native_binding_epoch(&mut fixture);
    fixture
        .runner
        .route_ui_cursor_moved(
            fixture.window,
            fixture.slot,
            epoch,
            fixture.backend_device,
            context_b,
            position,
        )
        .expect("B cursor creates its current pinned translator lifetime");
    let (_, duplicate) = fixture
        .runner
        .admit_native_mouse_button_observation(
            fixture.window,
            context_b,
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        )
        .expect("neutral held state classifies repeated B Down");
    assert_eq!(duplicate, NativeMouseButtonDisposition::DuplicateDown);

    let (context, up) = fixture
        .runner
        .admit_native_mouse_button_observation(
            fixture.window,
            context_b,
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        )
        .expect("authentic B Up remains admitted");
    assert_eq!(up, NativeMouseButtonDisposition::Original);
    let epoch = native_binding_epoch(&mut fixture);
    fixture
        .runner
        .route_ui_mouse_button(
            fixture.window,
            fixture.slot,
            epoch,
            context,
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        )
        .expect("unmatched fresh-translator Up is suppressed by the pinned adapter");
    assert_eq!(activation_count(&fixture), 0);

    for state in [
        winit::event::ElementState::Pressed,
        winit::event::ElementState::Released,
    ] {
        let (context, disposition) = fixture
            .runner
            .admit_native_mouse_button_observation(
                fixture.window,
                context_b,
                state,
                winit::event::MouseButton::Left,
            )
            .expect("fresh B click remains original neutral evidence");
        assert_eq!(disposition, NativeMouseButtonDisposition::Original);
        let epoch = native_binding_epoch(&mut fixture);
        fixture
            .runner
            .route_ui_mouse_button(
                fixture.window,
                fixture.slot,
                epoch,
                context,
                state,
                winit::event::MouseButton::Left,
            )
            .expect("fresh B click reaches the same pinned UI adapter");
    }
    assert_eq!(activation_count(&fixture), 1);
}

#[test]
fn pending_geometry_keeps_cancelled_mouse_bookkeeping_for_veto() {
    let mut fixture = native_cursor_fixture();
    let position = fixture.button_position;
    route_cursor(&mut fixture, position);
    route_button(
        &mut fixture,
        winit::event::ElementState::Pressed,
        winit::event::MouseButton::Left,
    );
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close intent begins");
    fixture
        .runner
        .apply_native_geometry_change(
            fixture.window,
            PlatformEvent::Resized {
                width: 1024,
                height: 768,
            },
        )
        .expect("pending resize remains on the native geometry path");
    fixture
        .runner
        .apply_native_geometry_change(
            fixture.window,
            PlatformEvent::ScaleFactorChanged {
                scale_factor: 1.25,
                width: 1280,
                height: 960,
            },
        )
        .expect("pending DPI change remains on the native geometry path");
    fixture
        .runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .expect("window state owner")
        .record_mut(fixture.window)
        .expect("same native window")
        .veto_close();

    route_cursor(&mut fixture, position);
    route_button(
        &mut fixture,
        winit::event::ElementState::Pressed,
        winit::event::MouseButton::Left,
    );
    route_button(
        &mut fixture,
        winit::event::ElementState::Released,
        winit::event::MouseButton::Left,
    );
    assert_eq!(activation_count(&fixture), 0);
}

#[test]
fn pending_focus_loss_updates_neutral_lifecycle_without_routing_ui() {
    use runenui_core::KeyModifiers;

    let mut fixture = native_cursor_fixture();
    fixture
        .runner
        .native_ui_modifiers
        .insert(fixture.window, KeyModifiers::NONE.with_shift());
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close intent begins");
    fixture
        .runner
        .apply_pending_close_window_event(fixture.window, WindowEvent::Focused(false))
        .expect("focus loss remains neutral and nonfatal while close is pending");
    assert!(
        !fixture
            .runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .expect("window state owner")
            .record(fixture.window)
            .expect("same native window")
            .focused
    );
    assert!(
        !fixture
            .runner
            .native_ui_modifiers
            .contains_key(&fixture.window)
    );
}

#[test]
fn rejected_pending_focus_preflight_preserves_modifier_observation() {
    use runenui_core::KeyModifiers;

    let mut fixture = native_cursor_fixture();
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close intent begins");
    let modifiers = KeyModifiers::NONE.with_shift();
    fixture
        .runner
        .native_ui_modifiers
        .insert(fixture.window, modifiers);
    let journal = fixture
        .runner
        .state
        .world
        .resource_mut::<PlatformWindowEventQueueResource>()
        .expect("native event journal");
    while journal.events().len() < 4096 {
        journal
            .publish(PlatformWindowEvent::new(
                fixture.window,
                PlatformEvent::RedrawRequested,
            ))
            .expect("fill the bounded native event journal");
    }

    assert!(
        fixture
            .runner
            .apply_pending_close_window_event(fixture.window, WindowEvent::Focused(false))
            .is_err()
    );
    assert_eq!(
        fixture.runner.native_ui_modifiers.get(&fixture.window),
        Some(&modifiers),
        "failed original focus-loss admission cannot erase modifier truth"
    );
}

#[test]
fn close_intent_revokes_held_game_keyboard_permission_immediately() {
    use crate::plugins::{ActionState, action};
    use runen_input::{
        DigitalState, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey, ObservationOrigin,
        PhysicalKeyIdentity,
    };

    let mut fixture = native_cursor_fixture();
    fixture
        .runner
        .state
        .world
        .insert_resource(ActionState::new());
    fixture
        .runner
        .apply_event_for_native_window(
            fixture.window,
            PlatformEvent::KeyboardInput {
                context: fixture.source,
                input: KeyboardInput {
                    physical_key: PhysicalKeyIdentity::code("KeyW"),
                    logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
                    location: KeyLocation::Standard,
                    state: DigitalState::Pressed,
                    repeat: false,
                    origin: ObservationOrigin::SourceReport,
                },
            },
        )
        .expect("original gameplay key Down is admitted");
    let mut actions = fixture
        .runner
        .state
        .world
        .remove_resource::<ActionState>()
        .expect("ActionState owner");
    actions.project(
        fixture
            .runner
            .state
            .world
            .resource::<InputState>()
            .expect("physical input owner"),
    );
    fixture.runner.state.world.insert_resource(actions);
    assert!(
        fixture
            .runner
            .state
            .world
            .resource::<ActionState>()
            .expect("game permission owner")
            .action_down(action::WORLD_MOVE_UP)
    );

    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close intent revokes held gameplay permission");
    assert!(
        !fixture
            .runner
            .state
            .world
            .resource::<ActionState>()
            .expect("same game permission owner")
            .action_down(action::WORLD_MOVE_UP)
    );
    assert!(
        fixture
            .runner
            .state
            .world
            .resource::<InputState>()
            .expect("neutral input remains authoritative")
            .keyboard_down_generation_in(fixture.source, &PhysicalKeyIdentity::code("KeyW"))
            .is_some()
    );
}

#[test]
fn pending_keyboard_down_and_up_preserve_neutral_truth_without_game_grant() {
    use crate::plugins::{ActionState, action};
    use runen_input::{
        DigitalState, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey, ObservationOrigin,
        PhysicalKeyIdentity,
    };

    let mut fixture = native_cursor_fixture();
    fixture
        .runner
        .state
        .world
        .insert_resource(ActionState::new());
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close intent begins");
    let key = PhysicalKeyIdentity::code("KeyW");
    let make_input = |state| KeyboardInput {
        physical_key: key.clone(),
        logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
        location: KeyLocation::Standard,
        state,
        repeat: false,
        origin: ObservationOrigin::SourceReport,
    };
    fixture
        .runner
        .apply_pending_close_keyboard_observation(
            fixture.window,
            fixture.source,
            make_input(DigitalState::Pressed),
        )
        .expect("pending Down is neutral-only and denied for gameplay");
    let mut actions = fixture
        .runner
        .state
        .world
        .remove_resource::<ActionState>()
        .expect("same game permission owner");
    actions.project(
        fixture
            .runner
            .state
            .world
            .resource::<InputState>()
            .expect("original physical input owner"),
    );
    fixture.runner.state.world.insert_resource(actions);
    assert!(
        fixture
            .runner
            .state
            .world
            .resource::<InputState>()
            .expect("neutral input owner")
            .keyboard_down_generation_in(fixture.source, &key)
            .is_some()
    );
    assert!(
        !fixture
            .runner
            .state
            .world
            .resource::<ActionState>()
            .expect("game permission owner")
            .action_down(action::WORLD_MOVE_UP)
    );
    fixture
        .runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .expect("window state owner")
        .record_mut(fixture.window)
        .expect("same native window")
        .veto_close();
    let mut actions = fixture
        .runner
        .state
        .world
        .remove_resource::<ActionState>()
        .expect("same game permission owner after veto");
    actions.project(
        fixture
            .runner
            .state
            .world
            .resource::<InputState>()
            .expect("original physical input remains held"),
    );
    fixture.runner.state.world.insert_resource(actions);
    assert!(
        !fixture
            .runner
            .state
            .world
            .resource::<ActionState>()
            .expect("veto does not restore old game permission")
            .action_down(action::WORLD_MOVE_UP)
    );
    fixture
        .runner
        .apply_pending_close_keyboard_observation(
            fixture.window,
            fixture.source,
            make_input(DigitalState::Released),
        )
        .expect("pending Up remains original neutral evidence");
    assert!(
        fixture
            .runner
            .state
            .world
            .resource::<InputState>()
            .expect("same neutral input owner")
            .keyboard_down_generation_in(fixture.source, &key)
            .is_none()
    );
}

#[test]
fn approved_close_retires_recipient_then_original_source() {
    let mut fixture = native_cursor_fixture();
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close intent begins");
    fixture
        .runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .expect("window state owner")
        .record_mut(fixture.window)
        .expect("same native window")
        .approve_close();
    assert!(
        fixture
            .runner
            .retire_native_input_recipient(fixture.window)
            .expect("approved close retires the pinned recipient")
    );
    assert!(
        fixture
            .runner
            .retire_window_input_source(fixture.window)
            .expect("approved close retires the original neutral source")
    );
    assert_eq!(
        fixture
            .runner
            .native_input()
            .window_source_context(fixture.window),
        None
    );
}

#[test]
fn approved_close_does_not_match_pending_lane_and_inactive_recipient_stays_strict() {
    let mut fixture = native_cursor_fixture();
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close intent begins");
    fixture
        .runner
        .state
        .world
        .resource_mut::<WindowStateRegistryResource>()
        .expect("window state owner")
        .record_mut(fixture.window)
        .expect("same native window")
        .approve_close();
    assert!(!fixture.runner.has_exact_pending_close(fixture.window));
    let error = fixture
        .runner
        .native_input_recipient(fixture.window)
        .expect_err("approved close recipient lookup remains inactive");
    assert_eq!(
        error.downcast_ref::<NativeInputRecipientError>(),
        Some(&NativeInputRecipientError::InactiveWindow)
    );
}

#[test]
fn vetoable_close_preserves_current_modifiers_until_pending_release_observed() {
    use runenui_core::KeyModifiers;

    let mut fixture = native_cursor_fixture();
    fixture
        .runner
        .native_ui_modifiers
        .insert(fixture.window, KeyModifiers::NONE.with_shift());
    fixture
        .runner
        .apply_event_for_native_window(fixture.window, PlatformEvent::CloseRequested)
        .expect("close intent is vetoable and preserves modifier authority");
    assert_eq!(
        fixture.runner.native_ui_modifiers.get(&fixture.window),
        Some(&KeyModifiers::NONE.with_shift())
    );

    fixture
        .runner
        .apply_pending_close_modifiers(fixture.window, KeyModifiers::NONE);
    assert_eq!(
        fixture.runner.native_ui_modifiers.get(&fixture.window),
        Some(&KeyModifiers::NONE),
        "an observed release updates the existing modifier owner during close policy"
    );
}

#[test]
fn retired_displayed_cursor_context_is_withheld_then_recovers_without_fake_cancel() {
    let mut fixture = native_cursor_fixture();
    let position = PhysicalPosition::new(240.0, 120.0);
    observe_neutral_cursor(&mut fixture, position.x, position.y);
    retire_native_cursor_context(&mut fixture);

    let binding_epoch = fixture
        .runner
        .native_input_recipient(fixture.window)
        .expect("same selected native recipient")
        .1
        .expect("selected binding epoch");
    fixture
        .runner
        .route_ui_cursor_moved(
            fixture.window,
            fixture.slot,
            binding_epoch,
            fixture.backend_device,
            fixture.source,
            position,
        )
        .expect("one exact retired snapshot rejection is a neutral cursor nonclaim");

    let input = fixture
        .runner
        .state
        .world
        .resource::<InputState>()
        .expect("neutral input owner");
    assert_eq!(
        input.absolute_pointer_position_for_source(fixture.source.source),
        Some(runen_input::Point2::new(
            240.0,
            120.0,
            runen_input::CoordinateSpace::WindowPhysicalPixels,
        )),
        "RunenInput retains original physical cursor truth"
    );
    assert_eq!(input.mouse_position, (0.0, 0.0));
    let cursor_journal = fixture
        .runner
        .state
        .world
        .resource::<PlatformWindowEventQueueResource>()
        .expect("original event journal")
        .events()
        .last()
        .expect("original cursor remains journaled");
    assert!(!cursor_journal.authorized_for_legacy_editor());
    let mouse = fixture
        .runner
        .native_ui_mice
        .get(&fixture.window)
        .expect("translator stays available for the next authentic cursor event");
    assert_eq!(
        mouse.mouse.active_device_id(),
        Some(WinitRunner::native_ui_device(fixture.source).unwrap())
    );
    assert_eq!(mouse.mouse.last_native_position(), Some(position));
    assert!(
        fixture
            .runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("same M13 owner")
            .input_ownership(fixture.slot)
            .expect("exact M13 state")
            .pointers()
            .is_empty(),
        "a rejected stale context cannot create UI pointer ownership"
    );

    fixture
        .runner
        .request_redraw_for_native_window(fixture.window);
    assert!(fixture.runner.frame_requested);
    assert!(
        fixture
            .runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .expect("native window")
            .record(fixture.window)
            .expect("created window")
            .redraw_requested
    );

    let before_cancel = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("pre-cleanup state");
    fixture
        .runner
        .cancel_native_ui_mouse(fixture.window)
        .expect("translator-only stream retires without a fabricated M13 Cancel");
    let after_cancel = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("post-cleanup state");
    assert_eq!(before_cancel.revision(), after_cancel.revision());
    assert!(after_cancel.pointers().is_empty());
    assert!(!fixture.runner.native_ui_mice.contains_key(&fixture.window));

    promote_fresh_native_cursor_display(&mut fixture);
    let fresh_position = PhysicalPosition::new(248.0, 124.0);
    observe_neutral_cursor(&mut fixture, fresh_position.x, fresh_position.y);
    fixture
        .runner
        .route_ui_cursor_moved(
            fixture.window,
            fixture.slot,
            binding_epoch,
            fixture.backend_device,
            fixture.source,
            fresh_position,
        )
        .expect("fresh completed-Present context restores exact cursor routing");
    let active = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("fresh pointer settlement");
    assert_eq!(active.pointers().len(), 1);
    assert_eq!(
        active.pointers()[0].device_id(),
        Some(WinitRunner::native_ui_device(fixture.source).unwrap())
    );
    fixture
        .runner
        .cancel_native_ui_mouse(fixture.window)
        .expect("a canonically owned fresh pointer receives its exact Cancel");
    assert!(
        fixture
            .runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("same M13 owner")
            .input_ownership(fixture.slot)
            .expect("cancelled pointer state")
            .pointers()
            .is_empty()
    );
}

#[test]
fn retired_displayed_move_preserves_existing_pointer_until_exact_cancel() {
    let mut fixture = native_cursor_fixture();
    let binding_epoch = fixture
        .runner
        .native_input_recipient(fixture.window)
        .expect("same selected native recipient")
        .1
        .expect("selected binding epoch");
    let initial_position = PhysicalPosition::new(232.0, 116.0);
    observe_neutral_cursor(&mut fixture, initial_position.x, initial_position.y);
    fixture
        .runner
        .route_ui_cursor_moved(
            fixture.window,
            fixture.slot,
            binding_epoch,
            fixture.backend_device,
            fixture.source,
            initial_position,
        )
        .expect("current completed-Present pointer Move commits");
    let pointer_before = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("committed pointer stream")
        .pointers()[0]
        .clone();
    retire_native_cursor_context(&mut fixture);

    let later_position = PhysicalPosition::new(240.0, 120.0);
    observe_neutral_cursor(&mut fixture, later_position.x, later_position.y);
    fixture
        .runner
        .route_ui_cursor_moved(
            fixture.window,
            fixture.slot,
            binding_epoch,
            fixture.backend_device,
            fixture.source,
            later_position,
        )
        .expect("retired context withholds Move without retiring an existing stream");
    let still_active = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("existing pointer remains canonical");
    assert_eq!(
        still_active.pointers(),
        std::slice::from_ref(&pointer_before)
    );
    fixture
        .runner
        .cancel_native_ui_mouse(fixture.window)
        .expect("only a canonically owned pointer receives Cancel");
    let after_cancel = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("exact Cancel settlement");
    assert!(after_cancel.pointers().is_empty());

    // A translator whose device no longer matches the canonical M13
    // pointer must not submit a Cancel to that foreign stream.
    let fresh_context = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("fresh owner")
        .surfaces()[0]
        .latest_retained_context()
        .expect("current retained context")
        .clone();
    let device = WinitRunner::native_ui_device(fixture.source).unwrap();
    let event = runenui_core::PointerEvent::new(
        runenui_core::PointerId::new(1).expect("same original pointer id"),
        runenui_core::PointerDeviceKind::Mouse,
        runenui_core::PointerPhase::Move,
        runenui_core::LogicalPoint::new(120.0, 60.0).expect("finite point"),
        fresh_context,
    )
    .with_device_id(device);
    let before = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("pre-rejection ownership");
    let submission = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .submit_native_pointer(fixture.slot, event)
        .expect("fresh owner accepts a replacement stream");
    let (batch, status) = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .pump_native_input_records(fixture.slot)
        .expect("replacement stream settles");
    assert!(matches!(
        native_pointer_finality(
            before.scope(),
            device,
            runenui_core::PointerId::new(1).unwrap(),
            fixture.displayed_context.surface_id(),
            submission,
            &batch,
            &status,
        ),
        Ok(UiInputFinality::Committed(_))
    ));
    let mut mouse = MouseInputState::default();
    let translated = mouse
        .cursor_moved(
            device,
            TranslatedPointerPoint {
                position: runenui_core::LogicalPoint::new(120.0, 60.0).unwrap(),
                input_context: fixture.displayed_context.clone(),
                modifiers: KeyModifiers::NONE,
            },
        )
        .expect("translator stream");
    let pointer_id = translated.pointer_id();
    let translator = NativeUiMouseTranslator {
        recipient: fixture.slot,
        source: fixture.source.source,
        backend_device: fixture.backend_device,
        device: UiInputDeviceId::new(991).expect("wrong checked device"),
        binding_epoch,
        scope: before.scope().clone(),
        mouse,
    };
    fixture
        .runner
        .native_ui_mice
        .insert(fixture.window, translator);
    let revision_before = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("canonical pointer")
        .revision();
    assert!(
        fixture
            .runner
            .cancel_native_ui_mouse(fixture.window)
            .is_err()
    );
    let unchanged = fixture
        .runner
        .state
        .world
        .resource_mut::<UiRuntimeSlotsResource>()
        .expect("same M13 owner")
        .input_ownership(fixture.slot)
        .expect("foreign Cancel was not submitted");
    assert_eq!(unchanged.revision(), revision_before);
    assert!(
        unchanged
            .pointers()
            .iter()
            .any(|active| active.pointer_id() == pointer_id && active.device_id() == Some(device))
    );
}

#[test]
fn stale_mouse_down_cancels_only_canonical_pointer_and_suppresses_its_up() {
    use winit::event::{ElementState, MouseButton};

    let mut fixture = native_cursor_fixture();
    let button_position = fixture.button_position;
    route_cursor(&mut fixture, button_position);
    assert_eq!(pointer_ownership(&mut fixture).pointers().len(), 1);
    retire_native_cursor_context(&mut fixture);

    let ownership_before_down = pointer_ownership(&mut fixture).revision();
    route_button(&mut fixture, ElementState::Pressed, MouseButton::Left);
    assert_eq!(activation_count(&fixture), 0);
    let after_rejected_down = pointer_ownership(&mut fixture);
    assert!(after_rejected_down.pointers().is_empty());
    assert_ne!(after_rejected_down.revision(), ownership_before_down);
    let translator = fixture
        .runner
        .native_ui_mice
        .get(&fixture.window)
        .expect("physical translator survives point-authority invalidation");
    assert_eq!(translator.mouse.last_native_position(), None);

    promote_fresh_native_cursor_display(&mut fixture);
    route_cursor(&mut fixture, button_position);
    assert_eq!(pointer_ownership(&mut fixture).pointers().len(), 1);
    let fresh_hover_revision = pointer_ownership(&mut fixture).revision();
    assert_eq!(activation_count(&fixture), 0);
    route_button(&mut fixture, ElementState::Released, MouseButton::Left);
    assert_eq!(activation_count(&fixture), 0);
    assert_eq!(
        pointer_ownership(&mut fixture).revision(),
        fresh_hover_revision,
        "the authentic Up consumes the suppressed Down without a UI Up"
    );

    route_button(&mut fixture, ElementState::Pressed, MouseButton::Left);
    assert_eq!(pointer_ownership(&mut fixture).pointers().len(), 1);
    route_button(&mut fixture, ElementState::Released, MouseButton::Left);
    assert!(pointer_ownership(&mut fixture).pointers().is_empty());
    assert_eq!(activation_count(&fixture), 1);
}

#[test]
fn stale_mouse_down_without_framework_pointer_does_not_fabricate_cancel() {
    use winit::event::{ElementState, MouseButton};

    let mut fixture = native_cursor_fixture();
    let button_position = fixture.button_position;
    observe_neutral_cursor(&mut fixture, button_position.x, button_position.y);
    retire_native_cursor_context(&mut fixture);
    route_cursor(&mut fixture, button_position);
    assert!(pointer_ownership(&mut fixture).pointers().is_empty());
    let before_down = pointer_ownership(&mut fixture).revision();

    route_button(&mut fixture, ElementState::Pressed, MouseButton::Left);
    assert_eq!(activation_count(&fixture), 0);
    assert_eq!(pointer_ownership(&mut fixture).revision(), before_down);
    assert!(pointer_ownership(&mut fixture).pointers().is_empty());
    route_button(&mut fixture, ElementState::Released, MouseButton::Left);
    assert_eq!(activation_count(&fixture), 0);
    assert_eq!(pointer_ownership(&mut fixture).revision(), before_down);
    assert!(pointer_ownership(&mut fixture).pointers().is_empty());
}

#[test]
fn stale_wheel_withholds_without_changing_existing_pointer_ownership() {
    use winit::event::{ElementState, MouseButton, MouseScrollDelta};

    let mut fixture = native_cursor_fixture();
    route_cursor(&mut fixture, PhysicalPosition::new(240.0, 120.0));
    route_button(&mut fixture, ElementState::Pressed, MouseButton::Left);
    retire_native_cursor_context(&mut fixture);
    let before = pointer_ownership(&mut fixture);
    assert_eq!(before.pointers().len(), 1);

    let binding_epoch = native_binding_epoch(&mut fixture);
    fixture
        .runner
        .route_ui_mouse_wheel(
            fixture.window,
            fixture.slot,
            binding_epoch,
            fixture.backend_device,
            fixture.source,
            &MouseScrollDelta::LineDelta(0.0, 1.0),
        )
        .expect("one exact stale displayed-snapshot wheel rejection is withheld");
    let after = pointer_ownership(&mut fixture);
    assert_eq!(after.status(), before.status());
    assert_eq!(after.keyboard(), before.keyboard());
    assert_eq!(after.surfaces(), before.surfaces());
    assert_eq!(after.pointers(), before.pointers());
    let translator = fixture
        .runner
        .native_ui_mice
        .get(&fixture.window)
        .expect("wheel rejection preserves native mouse owner");
    assert_eq!(
        translator.mouse.active_device_id(),
        Some(WinitRunner::native_ui_device(fixture.source).unwrap())
    );
    assert_eq!(
        translator.mouse.last_native_position(),
        Some(PhysicalPosition::new(240.0, 120.0))
    );
}

#[test]
fn stale_terminal_context_still_commits_partial_and_final_mouse_ups() {
    use winit::event::{ElementState, MouseButton};

    let mut fixture = native_cursor_fixture();
    route_cursor(&mut fixture, PhysicalPosition::new(240.0, 120.0));
    route_button(&mut fixture, ElementState::Pressed, MouseButton::Left);
    route_button(&mut fixture, ElementState::Pressed, MouseButton::Right);
    assert_eq!(pointer_ownership(&mut fixture).pointers().len(), 1);
    retire_native_cursor_context(&mut fixture);

    route_button(&mut fixture, ElementState::Released, MouseButton::Right);
    assert_eq!(pointer_ownership(&mut fixture).pointers().len(), 1);
    route_button(&mut fixture, ElementState::Released, MouseButton::Left);
    assert!(pointer_ownership(&mut fixture).pointers().is_empty());
}
