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
    assert_eq!(still_active.pointers(), [pointer_before.clone()]);
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
